#!/usr/bin/env python3
"""THROWAWAY: observe GStreamer candidate isolation at a captured PCM output.

No real audio sink, Songdial application, catalog, or production adapter code.
Independent decode pipelines feed a bounded appsink; one serialized controller
chooses PCM for a persistent appsrc/fakesink output. See README.md.
"""
import array
import io
import json
import struct
import threading
import time
import wave
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path

import gi

gi.require_version("Gst", "1.0")
from gi.repository import Gst

Gst.init(None)
RATE = 8000
FRAMES = 160
CAPS = "audio/x-raw,format=S16LE,rate=8000,channels=1,layout=interleaved"
START = time.monotonic()
EVENTS = []


def record(kind, **fields):
    entry = {"at": round(time.monotonic() - START, 4), "event": kind, **fields}
    EVENTS.append(entry)
    print(json.dumps(entry), flush=True)


def wav(value, seconds=10, opening=None):
    count = int(RATE * seconds)
    samples = [value] * count
    if opening is not None:
        samples[:16] = [opening] * 16
    result = io.BytesIO()
    with wave.open(result, "wb") as output:
        output.setnchannels(1)
        output.setsampwidth(2)
        output.setframerate(RATE)
        output.writeframes(struct.pack(f"<{count}h", *samples))
    return result.getvalue()


class HTTP(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.1"

    def log_message(self, *_):
        pass

    def do_GET(self):
        delay = {"/bad": .25, "/late-b": .40, "/c": .03}.get(self.path, 0)
        time.sleep(delay)
        if self.path == "/bad":
            self.send_error(503, "controlled preparation failure")
            return
        data = {
            "/a": wav(1000, 30), "/late-b": wav(2000), "/c": wav(3000),
            "/ready-cancel": wav(5000), "/finite": wav(6000, .4, 6100),
            "/invalid": b"this is not a WAV decoder input",
            "/truncated": wav(7000, 2)[:44 + RATE // 5 * 2],
        }
        endless = self.path == "/endless"
        body = wav(4000, 1) if endless else data.get(self.path)
        if body is None:
            self.send_error(404)
            return
        self.send_response(200)
        self.send_header("Content-Type", "audio/wav")
        self.send_header("Connection", "close")
        self.close_connection = True
        if not endless:
            self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        try:
            if not endless:
                self.wfile.write(body)
            else:
                # A long WAV header followed by an unbounded, bounded-write stream.
                header = bytearray(body[:44])
                struct.pack_into("<I", header, 4, 0x7FFFFF00 + 36)
                struct.pack_into("<I", header, 40, 0x7FFFFF00)
                self.wfile.write(header)
                chunk = struct.pack(f"<{FRAMES}h", *([4000] * FRAMES))
                while True:
                    self.wfile.write(chunk)
                    self.wfile.flush()
                    time.sleep(.02)
        except (BrokenPipeError, ConnectionResetError):
            pass


class Source:
    def __init__(self, name, generation, description):
        self.name, self.generation = name, generation
        self.pipeline = Gst.parse_launch(
            f"{description} ! audioconvert ! {CAPS} ! "
            "appsink name=staging max-buffers=2 max-bytes=65536 "
            "drop=false sync=false enable-last-sample=false"
        )
        self.sink = self.pipeline.get_by_name("staging")
        self.bus = self.pipeline.get_bus()
        self.pending = b""
        self.ready = False
        self.closed = False
        self.eos_reported = False
        self.errors = []
        self.messages = []
        self.max_pending = 0
        self.pause_return = self.pipeline.set_state(Gst.State.PAUSED).value_nick
        record("source-created", name=name, generation=generation,
               pause_return=self.pause_return)
        self.pipeline.set_state(Gst.State.PLAYING)

    def observe_bus(self):
        while True:
            msg = self.bus.pop_filtered(Gst.MessageType.ERROR | Gst.MessageType.EOS)
            if msg is None:
                break
            if msg.type == Gst.MessageType.ERROR:
                error, detail = msg.parse_error()
                self.errors.append(error.message)
                record("source-error", name=self.name, reason=error.message)
                self.messages.append("error")
            else:
                record("source-eos", name=self.name)
                self.messages.append("eos")

    def fill(self):
        if not self.pending:
            sample = self.sink.emit("try-pull-sample", 0)
            if sample is not None:
                buffer = sample.get_buffer()
                self.pending = buffer.extract_dup(0, buffer.get_size())
                self.max_pending = max(self.max_pending, len(self.pending))
                if self.pending and not self.ready:
                    self.ready = True
                    record("decoded-ready", name=self.name,
                           bytes=len(self.pending), first=struct.unpack_from("<h", self.pending)[0])
        return bool(self.pending)

    def chunk(self):
        if not self.fill():
            return None
        chunk, self.pending = self.pending[:FRAMES * 2], self.pending[FRAMES * 2:]
        return chunk

    def close(self):
        if self.closed:
            return
        before = time.monotonic()
        self.pipeline.set_state(Gst.State.NULL)
        self.closed = True
        record("source-closed", name=self.name,
               elapsed_ms=round((time.monotonic() - before) * 1000, 3))


class Controller:
    def __init__(self):
        self.lock = threading.RLock()
        self.generation = 0
        self.current = None
        self.session = {"current": None, "queue": [], "state": "stopped"}
        self.commits = []
        self.eos = []
        self.sources = []
        self.captured = []
        self.pending_output = {}
        self.pushes = 0
        self.position = 0
        self.running = True
        self.output = Gst.parse_launch(
            f"appsrc name=output caps=\"{CAPS}\" is-live=true format=time "
            "block=false max-buffers=2 ! fakesink name=capture sync=true "
            "async=false signal-handoffs=true enable-last-sample=false"
        )
        self.appsrc = self.output.get_by_name("output")
        self.output.get_by_name("capture").connect("handoff", self.capture)
        self.output.set_state(Gst.State.PLAYING)
        self.thread = threading.Thread(target=self.pump, daemon=True)
        self.thread.start()

    def capture(self, _sink, buffer, _pad):
        data = buffer.extract_dup(0, buffer.get_size())
        values = array.array("h", data)
        with self.lock:
            generation = int(buffer.offset)
            self.pending_output[generation] -= 1
            self.captured.append({"at": time.monotonic() - START,
                                  "pts": int(buffer.pts), "frames": len(values),
                                  "generation": generation,
                                  "values": sorted(set(values)), "first": values[0]})

    def submit(self, name, description):
        with self.lock:
            self.generation += 1
            generation = self.generation
        source = Source(name, generation, description)
        self.sources.append(source)
        return source

    def cancel(self, source):
        with self.lock:
            if source.generation == self.generation:
                self.generation += 1
            record("cancel", name=source.name, generation=self.generation)

    def commit(self, source, queue=()):
        with self.lock:
            if source.generation != self.generation or not source.ready or source.errors:
                record("commit-rejected", name=source.name, generation=source.generation)
                return False
            old = self.current
            self.current = source
            self.session = {"current": source.name, "queue": list(queue), "state": "playing"}
            self.commits.append(source.name)
            record("commit", generation=source.generation, session=self.session.copy())
        if old is not None:
            old.close()
        return True

    def stop(self):
        with self.lock:
            self.generation += 1
            old, self.current = self.current, None
            self.session["state"] = "stopped"
            record("explicit-stop", session=self.session.copy())
        if old is not None:
            old.close()

    def pump(self):
        while self.running:
            with self.lock:
                # Bound both queued appsrc work and the downstream sink buffer.
                if self.pushes - len(self.captured) < 2 and self.current is not None:
                    source = self.current
                    chunk = source.chunk()
                    if chunk:
                        buffer = Gst.Buffer.new_allocate(None, len(chunk), None)
                        buffer.fill(0, chunk)
                        running_time = self.output.get_clock().get_time() - self.output.get_base_time()
                        self.position = max(self.position, running_time)
                        buffer.pts = self.position
                        buffer.duration = len(chunk) // 2 * Gst.SECOND // RATE
                        buffer.offset = source.generation
                        self.position += buffer.duration
                        self.pending_output[source.generation] = self.pending_output.get(source.generation, 0) + 1
                        self.appsrc.emit("push-buffer", buffer)
                        self.pushes += 1
                    elif (source.sink.get_property("eos") and not source.eos_reported
                          and self.pending_output.get(source.generation, 0) == 0):
                        source.eos_reported = True
                        self.eos.append(source.name)
                        self.session["state"] = "stopped"
                        record("current-drained", session=self.session.copy())
            time.sleep(.002)

    def wait(self, predicate, timeout=3):
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            for source in self.sources:
                if not source.closed:
                    source.observe_bus()
            if predicate():
                return
            time.sleep(.005)
        raise RuntimeError("probe watchdog expired")

    def ready(self, source):
        self.wait(lambda: source.fill() or source.errors)
        if source.errors:
            raise RuntimeError(f"{source.name} failed instead of becoming ready")

    def seen(self):
        return {value for buffer in self.captured for value in buffer["values"]}

    def close(self):
        self.running = False
        self.thread.join(2)
        for source in self.sources:
            source.close()
        self.output.set_state(Gst.State.NULL)


def main():
    server = ThreadingHTTPServer(("127.0.0.1", 0), HTTP)
    server.daemon_threads = True
    threading.Thread(target=server.serve_forever, daemon=True).start()
    base = f"http://127.0.0.1:{server.server_port}"
    uri = lambda path: f'souphttpsrc location="{base}{path}" timeout=2 ! wavparse'
    controller = Controller()
    checks = {}
    try:
        record("runtime", version=Gst.version_string(), output="fakesink; no audible playback")
        a = controller.submit("A", uri("/a"))
        controller.ready(a)
        controller.commit(a, queue=("queued-track",))
        controller.wait(lambda: len(controller.captured) >= 10)

        before = len(controller.captured)
        bad = controller.submit("HTTP-failure", uri("/bad"))
        controller.wait(lambda: bad.errors)
        bad.close()
        checks["failed_preparation_preserves_A_and_queue"] = (
            controller.session == {"current": "A", "queue": ["queued-track"], "state": "playing"}
            and len(controller.captured) > before + 5
            and controller.seen() == {1000})

        invalid = controller.submit("decode-failure", uri("/invalid"))
        controller.wait(lambda: invalid.errors)
        invalid.close()
        checks["decoder_error_isolated"] = controller.session["current"] == "A"

        b = controller.submit("late-B", uri("/late-b"))
        time.sleep(.03)
        c = controller.submit("C", uri("/c"))
        controller.ready(c)
        controller.commit(c)
        # Deliberately leave obsolete B alive to reproduce uncooperative cancellation.
        controller.ready(b)
        checks["late_B_cannot_commit"] = not controller.commit(b)
        b.close()
        controller.wait(lambda: 3000 in controller.seen())
        checks["late_B_never_reaches_output"] = 2000 not in controller.seen()

        canceled = controller.submit("ready-canceled", uri("/ready-cancel"))
        controller.ready(canceled)
        controller.cancel(canceled)
        checks["ready_candidate_cancelled_before_output"] = not controller.commit(canceled)
        canceled.close()
        checks["canceled_candidate_never_reaches_output"] = 5000 not in controller.seen()

        finite = controller.submit("finite", uri("/finite"))
        controller.ready(finite)
        controller.commit(finite)
        controller.wait(lambda: "finite" in controller.eos)
        controller.wait(lambda: controller.pushes == len(controller.captured))
        finite_output = [row for row in controller.captured if 6000 in row["values"] or 6100 in row["values"]]
        checks["finite_opening_preserved"] = finite_output[0]["first"] == 6100
        checks["finite_duration_preserved"] = sum(row["frames"] for row in finite_output) == 3200
        checks["finite_completion_reported_once_and_retained"] = (
            controller.eos.count("finite") == 1
            and controller.session == {"current": "finite", "queue": [], "state": "stopped"})

        live = controller.submit("true-live", "audiotestsrc is-live=true wave=silence samplesperbuffer=160")
        controller.ready(live)
        controller.commit(live)
        controller.wait(lambda: 0 in controller.seen())
        checks["true_live_no_preroll_decoded_before_commit"] = live.pause_return == "no-preroll"

        endless = controller.submit("endless-http", uri("/endless"))
        controller.ready(endless)
        controller.commit(endless)
        controller.wait(lambda: 4000 in controller.seen())
        time.sleep(.2)
        checks["endless_http_reaches_output_without_EOS"] = (
            controller.session["current"] == "endless-http" and not endless.sink.get_property("eos"))

        controller.stop()
        controller.wait(lambda: controller.pushes == len(controller.captured))
        stopped_at = len(controller.captured)
        time.sleep(.08)
        checks["explicit_stop_halts_output_after_bounded_tail"] = len(controller.captured) == stopped_at

        current_error = controller.submit("current-error", "audiotestsrc is-live=true wave=silence samplesperbuffer=160 ! identity error-after=10")
        controller.ready(current_error)
        controller.commit(current_error)
        controller.wait(lambda: current_error.errors)
        checks["current_error_attributed_to_current_source"] = controller.session["current"] == "current-error"
        record("current-error-observation", session=controller.session.copy(), errors=current_error.errors)
        controller.stop()

        truncated = controller.submit("truncated", uri("/truncated"))
        controller.ready(truncated)
        controller.commit(truncated)
        controller.wait(lambda: "truncated" in controller.eos or truncated.errors)
        record("truncation-observation", messages=truncated.messages,
               errors=truncated.errors, current=controller.session.copy())

        pts = [row["pts"] for row in controller.captured]
        checks["output_timestamps_strictly_increase"] = all(a < b for a, b in zip(pts, pts[1:]))
        checks["exactly_one_commit_per_selected_source"] = controller.commits == [
            "A", "C", "finite", "true-live", "endless-http", "current-error", "truncated"]
        checks["staging_python_buffer_bounded_for_fixtures"] = max(s.max_pending for s in controller.sources) <= 65536
    finally:
        controller.close()
        server.shutdown()
        server.server_close()
    record("checks", checks=checks, all_passed=all(checks.values()))
    result = {"runtime": Gst.version_string(), "checks": checks, "events": EVENTS,
              "output_buffers": controller.captured, "commits": controller.commits,
              "source_max_python_buffer_bytes": {s.name: s.max_pending for s in controller.sources}}
    Path("results.json").write_text(json.dumps(result, indent=2) + "\n")
    return 0 if all(checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
