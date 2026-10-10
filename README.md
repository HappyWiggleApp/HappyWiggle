# HappyWiggle

A free, open-source mouse wiggler and auto clicker for Windows and macOS.
No ads, no account, no tracking, no network access.

It keeps your computer awake and your session active in one of two ways:

- **Wiggle only**: moves the mouse in a short, natural-looking drift (or an invisible one-pixel nudge). Never clicks.
- **Click areas**: clicks a spot or a random point inside an area you choose, at random intervals.

## What it does

| | |
|---|---|
| Steps aside | Touch the mouse (and, where the system reports it, the keyboard) and it waits, then carries on a few seconds after you stop. |
| Stops itself | After a number of clicks, after a number of minutes, or at a clock time. |
| Timetable | Pick days and hours once. Start, stop and lunch break drift by a few minutes each day. |
| Kill switch | A global hotkey (F8 by default, changeable) starts and stops it from anywhere. |
| Compact bar | A slim always-on-top bar with the time left and the time it will stop. |
| Keeps awake | Screen on and sleep off while a session runs, released as soon as it ends. |
| Tray | Lives in the tray or menu bar; start, pause and stop from there. |
| Careful clicks | Glides to the target like a hand, clicks only if it really arrived, puts the pointer back. |
| Simple first | Three choices to get going; exact timing and movement style sit behind "Advanced options". |
| Starts with the computer | Optional, in every build: direct download, Microsoft Store and Mac App Store. |
| Five languages | English, French, German, Spanish, Portuguese. Light and dark. |

## Use it responsibly

This is a tool you control. Check your employer's and IT policies before using
automation on a work or managed device, and do not use it to misrepresent
whether you are working.

## How it is built

```
core/        The engine, in Rust, with no operating-system code at all.
             Timing, motion paths, stop rules, timetable. 36 automated tests.
src-tauri/   The desktop shell (Tauri 2): windows, tray, hotkey, settings on
             disk, and one small file per OS that talks to the real mouse.
ui/          The interface: plain HTML, CSS and JavaScript. No framework, no build step.
packaging/   Scripts for the Microsoft Store (.msix) and the Mac App Store (.pkg).
docs/        Website and privacy policy (served by GitHub Pages).
tools/       Icon rendering, interface walkthrough test, live test.
```

Why the split: everything that decides *when* to move or click is in `core/`
and is tested by simulating whole working days in milliseconds. The part that
cannot be tested without a real screen is kept as small as possible
(`src-tauri/src/input/`, about 100 lines per OS).

## Build it yourself

You need [Rust](https://rustup.rs), [Node.js](https://nodejs.org) and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your system.

```sh
npm install
npm run dev          # run the app with live reload
npm run build        # make an installer for the system you are on
npm test             # engine tests
```

To look at the interface alone, in a browser, with a pretend back end:

```sh
python3 tools/make-preview.py     # writes dist/preview.html
```

## Before you publish your own copy

1. Put your links in `src-tauri/links.json` (source, issues, privacy, donate). An empty link hides its button.
2. Pick your own `identifier` in `src-tauri/tauri.conf.json` (and `APP_ID` in `src-tauri/src/lib.rs`).
3. Follow `docs/ROADMAP.md`.

## Privacy

The app stores three small files on your computer (your settings, a session
history of at most 35 days, and a random number used to vary the timetable).
Nothing leaves the machine: the app has no network code, and the store builds
do not even request the permission to go online. See [PRIVACY.md](PRIVACY.md).

## Support

HappyWiggle is free and stays free. If it saved you some trouble, you can [buy me a coffee on Ko-fi](https://ko-fi.com/happywiggleapp).

## Author

Created by Geordy, who had the idea and directs the project. The code was written with Claude, Anthropic's AI model.

## Licence

MIT. See [LICENSE](LICENSE).
