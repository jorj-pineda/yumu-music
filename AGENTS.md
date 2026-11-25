# Yumu

Yumu is a small, fast, native desktop client for YouTube Music, written in
Rust. It is to YouTube Music what [Spotifast](https://github.com/crmne/spotifast)
is to Spotify: a real app with low memory and CPU use, instead of a browser tab
or an Electron wrapper around the website.

Status: early prototype. Search works; playback is being moved to yt-dlp (see
"Where audio comes from" below). There is no interface yet.

## Read first

- `roadmap.md` at the repository root, if it exists. It is gitignored on
  purpose: it is the maintainer's local plan and the list of open decisions.
  Follow it, tick items off as they land, and add what you learn. It is never
  committed.
- `CONTRIBUTING.md` for the pull request rules that apply to everyone.
- `THIRD_PARTY_NOTICES.md` before adapting code from another project.

## What we never compromise on

1. **Light and native.** No browser engine, no Electron, no WebView, no
   telemetry, no hosted backend. Measure memory and idle CPU when a change
   could affect them. Lower is the point of the project.
2. **Optimistic interface.** A control shows its result the moment it is used.
   A double-clicked song is the playing song; the backend makes it true behind
   it. Nothing the user did may flicker away and come back.
3. **Never on the UI thread.** Network, yt-dlp, decoding and disk work run on
   the async runtime or the player thread. The UI draws state and emits
   actions.
4. **Not a downloader.** Yumu streams for listening. Do not add saving,
   exporting or offline copies of audio. That is what draws takedowns.
5. **Not affiliated with Google.** Never use "YouTube", "YT" or Google's logos
   in the app name, icon or branding. Plain descriptive text ("a client for
   YouTube Music") is fine.

## Where audio comes from

YouTube Music has no public API. Findings so far (November 2026):

- **Metadata** (search, albums, playlists, artists) comes from `rustypipe`,
  an Innertube client. Search works and is fast.
- **Streams from rustypipe are broken.** Its web clients fail to decode
  YouTube's current player script, and its iOS client only serves the first
  ~1 MB (about a minute) without a PO token. rustypipe has had no real code
  changes since mid-2025. Do not try to revive this path without asking.
- **Streams come from `yt-dlp`**, run as a child process with `-o -` so it
  fetches the audio itself and pipes bytes to the player. Reusing its stream
  URLs from another HTTP client returns 403. yt-dlp needs a JavaScript runtime
  (Deno, pulled in by Homebrew's formula). Expect a few seconds of startup per
  track; prefetch the next track to hide it.
- Prefer the AAC stream (`-f 140`, m4a). `rodio` with symphonia decodes it.
  Opus in WebM does not decode yet.

## Architecture

Modelled on Spotifast; read its code when building an equivalent piece.

- `src/ui/` draws views and returns `Action`s. Apply actions after drawing in
  the app layer; never mutate application state from inside a borrowed view.
- A backend module owns network work on a tokio runtime.
- A player module owns the audio engine: yt-dlp child process, decoding,
  output, queue position.
- Keep platform code behind `cfg` blocks or target modules. A fix for one
  platform keeps the others compiling.
- Settings and state files stay readable, backward compatible and atomically
  written. Never log cookies, tokens or account details.
- Prefer existing dependencies. Explain a new crate in a comment next to it in
  `Cargo.toml` when the reason is not obvious.

## Adapting code from Spotifast or Kaset

Both are MIT licensed and credited in `THIRD_PARTY_NOTICES.md`. When a file
adapts their code, put a comment at the top naming the source, for example
`// Adapted from Spotifast (MIT), src/player.rs`. Kaset is Swift, so borrow
its ideas, not its assets: never copy its icon, artwork or screens.

## Working in this repository

- **Never commit to `main`.** Every change goes on a branch and lands through a
  pull request to `main`. Branch names: `feat/...`, `fix/...`, `docs/...`,
  `chore/...`.
- **Small, frequent commits.** One focused change per commit, each building
  and passing the checks on its own.
- **Commit titles** use conventional commits in plain language:
  `feat(player): start playback before the whole track downloads`.
- **No AI attribution in commits.** Never add `Co-Authored-By` lines for an
  agent or model. The maintainer is the sole author in history. Say which
  model and harness helped at the end of the pull request description
  instead.
- **Review with sol-loop before pushing.** Run `/sol-loop` on every change
  and push only after Sol approves. End the pull request description with a
  line such as `Reviewed with sol-loop: 2 rounds with <model>. Sol verdict:
  approved`, followed by the model and harness that wrote it.
- **Ask before big decisions**: architecture, new runtime dependencies,
  licensing, platform scope, anything touching history, or anything in the
  roadmap's "Decisions" list. Small, clearly scoped choices do not need asking.
- Do not broaden a task into adjacent features or a general refactor.

## Checks

Run before pushing:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Do not weaken a lint, delete a test, or add an `allow` just to get green
without explaining why the rule does not apply. Report platform coverage
honestly: say "compiled" when a platform was only compiled, not tested.

## Local files to leave alone

- `cache/` holds rustypipe's state. `.tools/` holds the prototype's
  rustypipe-botguard binary. Both are gitignored.
- Never put build output or large scratch files in `/tmp`.
