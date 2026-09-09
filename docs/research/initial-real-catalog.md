# Initial real catalog candidates

Research checked **2026-09-09**. The user subsequently approved these candidates as the initial selection for implementation, subject to listening and decoding validation. The research below records the evidence available at selection time, not completed playback validation. The accepted direction is instrumental emphasis, calm/gentle energy, real-only default browsing, explicit Demo/testing mode, provenance under **Browse sources**, and bundled metadata/attribution with finite recordings fetched on demand. See [product brief](../product.md).

**Recommendation:** consider Radio Paradise **Serenity** and **Mellow Mix**, plus the six Kevin MacLeod recordings below. The finite recordings have an explicit first-party CC BY 4.0 offer. Radio Paradise documents direct listening through external players; that supports the narrower proposed use of publisher stream links for personal playback. It does not supply a license to redistribute Station audio or reproduce logos. SomaFM is excluded because its current terms specifically reject new third-party clients.

Only public pages, their public metadata, and HTTP **HEAD** responses were read. No audio was downloaded or played, no account was used, and nobody was contacted. The musical descriptions below come from publishers, not a listening review. Codec and uninterrupted-playback checks remain part of the eventual GStreamer package validation on Linux x86_64 and macOS arm64/x86_64.

## Two Station candidates

Both are continuous publisher-programmed **Stations**, with no catalog duration or application-controlled sequence of their constituent songs. They use direct stream locators, so neither requires PLS/M3U resolution. Their paths are publisher-published endpoints, not expiring media tokens; this is not a promise that the URLs never change. [Radio Paradise stream picker][rp-streams]

| Candidate | Proposed locator | Format and observed accessibility | Fit and limits |
| --- | --- | --- | --- |
| Radio Paradise — Serenity | `https://stream.radioparadise.com/serenity` | Publisher lists 64 kbit/s AAC+. Unauthenticated HEAD returned 200, `audio/aac`, 64 kbit/s, 44.1 kHz stereo. | Publisher recommends it for focused work; response identifies relaxation programming. Stronger calm candidate. A completely instrumental schedule was **not** established. [FAQ][rp-faq] |
| Radio Paradise — Mellow Mix | `https://stream.radioparadise.com/mellow-192` | Publisher lists 192 kbit/s MP3. Unauthenticated HEAD returned 200, `audio/mpeg`, 192 kbit/s, 44.1 kHz stereo. | Publisher describes gentle songs suited to background listening. Treat as a mix that **can include vocals**, not an instrumental-only Station. [Channel description][rp-mellow] |

These header observations were made on **2026-09-09 at 20:46 UTC**. The published stream picker generates `http://` links; the corresponding `https://` paths above were checked separately by HEAD. The [page's public backing data][rp-stream-data], updated March 24, 2026, supplies channel IDs 42/1, codec labels, and endpoint suffixes. This data was used for research; no runtime dependency on its undocumented schema is proposed.

Serenity also offers `https://stream.radioparadise.com/serenity-flac`. HEAD returned 200 with `application/ogg`; the publisher labels it FLAC. This is an **Ogg-carried FLAC candidate**, not proof that raw-FLAC handling is sufficient. Keeping the smaller AAC+ stream first avoids choosing lossless bandwidth by accident. The AAC profile and decoding quality still need verification using the actual bundled runtime. [Stream data][rp-stream-data]

**Documented use and attribution:** Radio Paradise explicitly instructs listeners to copy its stream URLs into WiFi radios and other players. Its [legacy first-party FAQ][rp-legacy] expressly says listening does not require registration, and its [current FAQ][rp-faq] says listening is free without a subscription. A catalog entry should show `Radio Paradise — <channel>` and link to the [publisher](https://radioparadise.com/) and channel/stream information. That credit is a proposed accurate provenance display; no special mandatory attribution wording for ordinary stream-link playback was found. No blanket grant to copy artwork, restream, or redistribute the underlying recordings was established. The proposed scope is titles, source links and personal playback directly from the publisher. Do not infer broader rights from external-player compatibility.

**Operational fit:** reconnecting a Station on resume can use the same direct endpoint to rejoin its current program. An ended connection is a Station interruption, not finite Track completion. A new request still needs Songdial's preparation and supersession controls; public availability says nothing about readiness, buffering delays, or outage behavior. These are adapter implications, not publisher guarantees.

## Six finite Track candidates

**Artist and recording licensor: Kevin MacLeod / Incompetech. License: CC BY 4.0 for all six.** The current [first-party catalog JSON][inc-data] supplies exact titles, ISRCs, filenames, durations, instruments, and mood descriptions. Each linked detail page finds its entry by ISRC and renders the Listen/Download URL and CC BY 4.0 attribution beside it. This establishes a recording-specific offer; it is not inferred merely from a soundtrack-use FAQ. The [artist's FAQ][inc-faq] also distinguishes his copyrighted recordings from public-domain compositions.

| Title / first-party detail page | ISRC | Publisher duration | Publisher-based musical fit | Direct MP3 locator |
| --- | --- | --- | --- | --- |
| [Music for Manatees](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1400009) | `USUAN1400009` | 17:36 | Slow, relaxed piano, electric piano, strings and synth | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Music%20for%20Manatees.mp3) |
| [Dream Culture](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1300046) | `USUAN1300046` | 3:34 | Dreamlike piano with percussion; relaxed | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Dream%20Culture.mp3) |
| [Airport Lounge](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1100806) | `USUAN1100806` | 5:08 | Light bass, drums, electric piano and vibes | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Airport%20Lounge.mp3) |
| [Wholesome](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1900022) | `USUAN1900022` | 6:04 | Cello, viola, marimba and drums; relaxed groove | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Wholesome.mp3) |
| [Carefree](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1400037) | `USUAN1400037` | 3:25 | Brighter ukulele, guitar and tuned percussion | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Carefree.mp3) |
| [Water Prelude](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1100017) | `USUAN1100017` | 5:24 | Floating background piece; also tagged eerie/mysterious. Instruments unspecified. | [MP3](https://incompetech.com/music/royalty-free/mp3-royaltyfree/Water%20Prelude.mp3) |

The list totals **41:11** using publisher durations. These are six finite recordings, not an upstream Playlist. Songdial may curate its own small Playlists referencing them. The instrumental emphasis is supported by the listed instrumentation, but the exact taste/order should be reviewed by listening before selection; **Water Prelude** is the first candidate to reconsider if its eerie character conflicts with the intended mood.

All six exact MP3 URLs returned **200 to unauthenticated HEAD at 20:44 UTC on September 9, 2026**, without a redirect. Each advertised byte ranges and a finite content length. All returned `application/octet-stream` with attachment disposition, **not** `audio/mpeg`. Therefore the adapter must identify/decode the actual content rather than reject it solely for the generic MIME type. MP3 is established by the artist's filename and player implementation; no audio-byte or duration verification was performed. They require no playlist resolution, account, cookie, or signed locator in the published flow. Neither HEAD success nor a CC license guarantees indefinite hosting.

### Exact license and credit handling

The [artist's CC option][inc-license] is free with attribution. CC BY 4.0 explicitly permits sharing and redistribution, including commercial use, while requiring appropriate attribution, the license link, retained supplied notices, an indication of modifications where applicable, and no misleading endorsement or added restrictions. Thus the offered rights are broader than synchronization with a video; this research found no conflicting standalone-recording restriction for these six. [CC BY 4.0 deed][cc-deed], [legal code §§2–3][cc-code]

Use the artist's per-track template, substituting the exact title, and retain the linked detail page as the source:

```text
"<Title>" Kevin MacLeod (incompetech.com)
Licensed under Creative Commons: By Attribution 4.0 License
https://creativecommons.org/licenses/by/4.0/
```

The template appears in the [detail-page source](https://incompetech.com/music/royalty-free/index.html?isrc=USUAN1400009), `renderTrack`, beside the media buttons. The HTTPS license link above normalizes its HTTP spelling. Credits must be reasonably discoverable according to the [artist's FAQ][inc-faq]; bundled metadata should therefore expose title, artist, source page and license through Songdial's credits/provenance UI, not only in a developer file. Store whether the recording was changed; streaming the publisher's original bytes does not justify claiming an adaptation.

For reviewable catalog evidence, retain the selected entry's ISRC, exact source/detail URL, media URL, license URL, required credit text and checked date. Record a content hash only when an authorized later download actually occurs; no audio hash is claimed here.

## Exclusions and bounded remaining checks

- **SomaFM:** its terms, explicitly updated **September 4, 2026**, state that it cannot approve third-party clients, even noncommercial ones, and is not adding new exceptions. This directly contradicts a new Songdial integration; its older personal-listening links do not override that restriction. Exclude all its channels from the proposed built-in catalog. [Current terms][soma-terms]
- **Scott Buckley:** the CC BY 4.0 statement appears alongside restrictions requiring synchronization and forbidding isolated resale/redistribution to music platforms. Do not resolve that tension by choosing the broader interpretation; exclude from this initial standalone-recording list. [Using This Music][buckley]
- **Nightride FM:** the official site establishes relevant instrumental/chillsynth programming, but this bounded investigation did not establish a comparably clear first-party external-player integration statement. Unofficial players are not permission evidence. Leave out of this initial recommendation. [Publisher homepage](https://nightride.fm/)
- **WFMU:** its [stream help](https://wfmu.org/audiostream.shtml) recommends external players, and [terms](https://wfmu.org/terms-of-service/) permit linking/listening while reserving proprietary marks. It is technically plausible for narrow personal playback, but the researched freeform programming is less aligned with instrumental emphasis than the two candidates above. No third Station is proposed just to fill a quota.

Before accepting the final catalog, listen to the finite recordings and sample the Stations for musical fit; decode these actual remote formats using the bundled GStreamer package on all three targets; exercise HTTP errors, generic MIME, TLS, redirects, and Station disconnect/reconnect without turning those network checks into deterministic CI dependencies. GStreamer documents [MP3 decoding](https://gstreamer.freedesktop.org/documentation/mpg123/index.html), [AAC decoding](https://gstreamer.freedesktop.org/documentation/libav/avdec_aac.html), and [FLAC decoding](https://gstreamer.freedesktop.org/documentation/flac/flacdec.html), but those pages do not establish that the chosen package contains every necessary parser, demuxer, decoder, HTTP source and TLS dependency.

No catalog choice, runtime test, or permission request was performed by this research.

[rp-streams]: https://radioparadise.com/listen/stream-links
[rp-stream-data]: https://vsh-sdata.radioparadise.com/api/pages?populate=deep&pagination%5BpageSize%5D=1&filters%5Burl%5D%5B%24eq%5D=%2Flisten%2Fstream-links
[rp-faq]: https://radioparadise.com/about/faqs-and-info
[rp-legacy]: https://apps.radioparadise.com/rp3.php?name=Help
[rp-mellow]: https://vsh-sdata.radioparadise.com/api/channels?populate=deep&filters%5Bchan_id%5D%5B%24eq%5D=1
[inc-data]: https://incompetech.com/music/royalty-free/pieces.json
[inc-faq]: https://incompetech.com/music/royalty-free/faq.html
[inc-license]: https://incompetech.com/music/royalty-free/licenses/
[cc-deed]: https://creativecommons.org/licenses/by/4.0/
[cc-code]: https://creativecommons.org/licenses/by/4.0/legalcode.en
[soma-terms]: https://somafm.com/contact/tos.html
[buckley]: https://www.scottbuckley.com.au/library/using-this-music/
