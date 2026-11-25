# Contributing

Yumu is a small native client for YouTube Music, built by one maintainer with
a clear idea of what it should be: light, fast and simple. Contributions are
welcome, but review time is the scarce part. Reviewing a pull request usually
takes more effort than writing one, so please read this before you open one.

## What helps most

- **Bug reports.** Open an issue with what you did, what you expected and
  what happened, plus your OS and Yumu version. A crash, a regression and a
  real performance problem are all bugs.
- **Bug fixes.** A focused fix for a real bug is the most likely pull request
  to be merged.
- **Triage.** Point out duplicates, confirm a bug still happens (or no longer
  does) on the latest version, and test open pull requests. A tested +1 on an
  existing pull request is better than a second one.

## Ask before building

Open an issue or discussion first, and wait for a yes, before working on:

- new features
- interface redesigns (moving navigation, panels, menus or layout)
- new settings (a good default, a shortcut or a command is usually better)
- architecture changes or new dependencies
- support for another platform

Pull requests for these without prior agreement will be closed, however good
the code is. That is about keeping the project coherent, not about you.

## Out of scope

Yumu will not accept:

- an embedded browser, Electron or a WebView
- telemetry or a hosted backend
- saving, downloading or exporting audio
- ways around paid features, DRM or region restrictions

## Rules for a pull request

These apply to every pull request, drafts included. A pull request that
follows them can still be closed; a review or merge is never owed.

### One problem per pull request

Fix one problem, and include everything that fix needs: code, tests and docs.
Cleanups, refactors and a second fix go in their own pull requests.

### Show that it works

Say how you found the problem, how you checked your change, and what you saw.
"Tests pass" is not evidence on its own.

Interface changes need before and after screenshots, in both light and dark
themes. Add a short recording when the change is about motion or timing.
Upload these to the pull request; never commit them.

### Pass the checks

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Add a focused test for changed behaviour where one is practical. Do not weaken
a lint or delete a test to get green.

### Say if you used AI

AI-assisted work is welcome. Name the model and tool you used at the end of
the pull request description. You are responsible for every line: read it,
understand it, and test it before asking for review.

### Keep it mergeable

Use a conventional commit title in plain language, for example
`fix(player): skipping no longer replays the previous song`. Keep your branch
up to date with `main`. A pull request that conflicts with `main` and has no
activity for 7 days may be closed.

### Three open pull requests per person

Get one merged or closed before opening a fourth.

## Conduct

Be kind and patient. Repeated pings, arguing over a closed pull request, or
flooding the project with automated pull requests will get you blocked.

## Forks and branding

The MIT license covers the source code. You are welcome to fork it. A fork
should use its own name and icon and make clear it is not the official Yumu.

Yumu is not affiliated with, endorsed by or sponsored by Google or YouTube.
Do not add their names or logos to the app's name, icon or branding.

## Setup

You need Rust (stable) and `yt-dlp` with a JavaScript runtime. On macOS:

```sh
brew install yt-dlp   # also installs Deno
cargo run --release -- "artist song"
```
