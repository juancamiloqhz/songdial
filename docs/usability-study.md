# Explore Songdial Listener study

This protocol validates the first executable milestone with three to five target
Listeners. It tests the resolved product model; it is not a request for new
features during a session. Record outcomes in
[`usability-findings.md`](usability-findings.md).

## Participants and data handling

Recruit people who have not read Songdial's product documents and who plausibly
use keyboard-driven music or developer tools. Assign only anonymous IDs (`P01`
through `P05`) and record a broad fit reason, such as “terminal developer” or
“keyboard-driven music-tool Listener.” Do not record names, contact details,
employers, account names, or other identifying information.

Tell each participant that Songdial uses a fictional catalog and simulated
playback, so it needs no account, network access, credentials, or audio device.
Ask separately for permission to retain brief anonymous quotes and screenshots.
Never capture a screenshot without consent, and exclude shell prompts, paths,
usernames, notifications, and unrelated terminal content.

## Prepare the packaged build

Use one commit for every session. From a clean checkout of the study branch,
choose the supported target that matches the host and package it into a fresh
temporary directory. For example, on Linux x86-64:

```sh
git status --short
git rev-parse HEAD
SONGDIAL_STUDY_TARGET=x86_64-unknown-linux-gnu
SONGDIAL_STUDY_DIR="$(mktemp -d)"
rustup target add "$SONGDIAL_STUDY_TARGET"
./scripts/package.sh "$SONGDIAL_STUDY_TARGET" "$SONGDIAL_STUDY_DIR"
./scripts/verify-artifact.sh \
  "$SONGDIAL_STUDY_DIR/songdial-$SONGDIAL_STUDY_TARGET" \
  "$SONGDIAL_STUDY_TARGET"
sha256sum "$SONGDIAL_STUDY_DIR/songdial-$SONGDIAL_STUDY_TARGET"
```

On macOS, select `aarch64-apple-darwin` or `x86_64-apple-darwin` as appropriate
and use `shasum -a 256` for the checksum. Record the commit, target, and checksum
in the findings. The verifier must pass before the first session.

Use a fresh terminal window with a default color environment. Remove Songdial's
test-only environment variables before launch:

```sh
env -u NO_COLOR \
  -u SONGDIAL_TEST_FAIL_RUNTIME_AFTER_DRAW \
  -u SONGDIAL_TEST_FAIL_STARTUP_AFTER_RAW_MODE \
  "$SONGDIAL_STUDY_DIR/songdial-$SONGDIAL_STUDY_TARGET"
```

The application restores the terminal after exit. Clear unrelated terminal
content before launch. Do not show the README, key list, product documents, or
this protocol to the participant.

Prepare exact `80x24` compact and `120x40` wide window sizes. Verify each size
with `stty size` before launching; it prints rows before columns. Participants
with odd IDs start compact and resize wide. Participants with even IDs start
wide and resize compact. Keep the same process running through the resize so
state continuity is observable.

## Facilitator opening

Read this introduction without demonstrating any controls:

> We are evaluating Songdial, not you. It is a terminal application for
> deciding what to play, using a fictional catalog and silent simulated
> playback. Please think aloud and tell me what you expect before each action.
> Use whatever guidance the application itself provides. I will not tell you
> exact keys unless you become completely blocked. You may stop at any time.

Confirm consent to observe the session and record anonymous notes. Record quote
and screenshot consent separately. Start a fresh Songdial process for every
participant so the Demo catalog and application state reset.

## Tasks

Read one task at a time. Do not reveal later tasks or explain product terms
beyond the wording below.

1. **Choose for focused work without playing.** “Imagine you are beginning a
   focused work block. Starting at Home, use the mood-and-activity route to find
   a suitable option. Inspect a promising Station or Playlist without starting
   it. Tell me when you believe you have only selected or opened it, and why.”
2. **Start a Station deliberately.** “Find a continuous radio option, inspect
   it, and then start it. Move your selection to something else and explain how
   you can tell what is selected and what is playing.”
3. **Start a Playlist and change the Queue.** “Find and start a saved Playlist
   for focused work. Inspect the upcoming Queue, open one queued Track and come
   back, remove one queued Track, then add a different Track or Playlist.
   Describe what changed and whether the playing Track was interrupted.”
4. **Search and identify Source.** “From where you are, search for `Night
   Geometry`. Identify the two Track results and the Source of each. Open the
   Harbor Sound Track without starting it, then return to the results. Tell me
   what Search state was restored.”
5. **Continue after resize.** Resize the same terminal process from its starting
   size to the other prepared size. “Continue comparing the Search results.
   Tell me whether your Destination, query, selection, Playback session, or
   Queue changed, and whether the wider or compact layout changed what you can
   do.”
6. **Return Home and exit.** “Return to Home without restarting the application,
   then exit Songdial. Tell me what you expect the first exit action to do while
   music or a Queue is active.”

After the tasks, ask only these neutral questions:

- “At any point, what did you think would start or replace playback?”
- “What did Source mean to you here?”
- “What, if anything, left you unsure how to get back?”
- “What was the most confusing part?”

## Facilitation and scoring rules

Let the participant read the persistent key guide or open in-application help;
record each use, but treat self-serve guidance as unassisted completion. If a
participant stalls, first ask them to describe the visible choices. Next ask
whether the application offers help. Give an exact key only when the participant
cannot recover; record the key, point in the task, and preceding behavior. A
task completed after exact-key coaching is not unassisted.

For every task record `unassisted`, `completed after coaching`, or `blocked`,
plus notable misinterpretations and a concise path through the interface. Do not
turn a participant's feature suggestion into a defect unless current behavior
breaks the milestone specification.

A critical failure is any of the following:

- the participant becomes trapped and cannot navigate back or recover without
  coaching or restart;
- opening or selecting unexpectedly starts or replaces playback;
- the participant cannot distinguish selection from playback after completing
  the relevant task.

The study passes only when:

- there are zero critical trapped-navigation, accidental-playback, or
  selection-versus-playback failures;
- at least 80% of participants complete every task unassisted;
- every participant returns Home and correctly identifies a Track's Source;
- no essential action requires facilitator coaching in more than one session.

With three or four participants, the 80% threshold requires every participant
to complete each task unassisted. With five, it requires at least four
unassisted completions per task. Every observed failure must become either a
reproducible follow-up issue or an explicit milestone decision before the final
verdict.
