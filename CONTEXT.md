# Songdial

Songdial helps a listener decide what to play through an intent-first terminal experience while keeping music provenance understandable and secondary to discovery.

## Language

**Listener**:
The person using Songdial to decide what to hear and control the resulting Playback session.
_Avoid_: User, operator

**Service**:
An external music product a listener recognizes and can deliberately browse, such as Spotify or YouTube Music.
_Avoid_: Provider

**Source**:
The provenance attached to music content and displayed when relevant. A source may be a service, Songdial curation, or a radio origin.
_Avoid_: Provider

**Listening intent**:
The state or purpose guiding what a Listener wants to hear, such as Deep Work, Calm, Energy, or Reset.
_Avoid_: Mood, activity, collection

**Track**:
A finite piece of recorded music available from one Source that can be played directly. Equivalent music from another Source is a distinct Track.
_Avoid_: Song, item

**Station**:
A continuous music stream that can be played directly.
_Avoid_: Radio, channel

**Playlist**:
An ordered collection of Tracks assembled as a named listening sequence.
_Avoid_: Collection, mix

**Playable item**:
A Track, Station, or Playlist that a Listener can deliberately start.
_Avoid_: Content, media item

**Playback session**:
The current Playable item together with the Queue established when the Listener deliberately starts it.
_Avoid_: Playback context, listening session

**Queue**:
The ordered Tracks scheduled to play after the current item in a Playback session. Stations cannot be queued.
_Avoid_: Up next, playlist

**Now Playing**:
The persistent representation of the current or most recently ended Playable item and its playback state. Its expanded destination also exposes the Queue.
_Avoid_: Player, playback screen

**Destination**:
A named place in Songdial's browsing flow that serves a specific listening task or view.
_Avoid_: Screen, page, route

**Demo catalog**:
The fixed fictional music catalog used to expose interaction and layout problems in the first milestone. It is a design instrument, not a schema for future Service integrations.
_Avoid_: Seed data, provider format
