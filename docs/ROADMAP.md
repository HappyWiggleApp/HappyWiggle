# From idea to both stores: the roadmap

This is the full path for SoftMouseWiggler, with what is already done, what
is left, who does each step, what it costs and how long it takes.

"You" means the project owner. "Claude" means a step an AI agent can do for
you in a session like the one that produced this repository.

## Where things stand

| Stage | Status |
|---|---|
| 1. Idea and research | Done |
| 2. Product design | Done |
| 3. Engine, with automated tests | Done: 36 tests pass |
| 4. Desktop app and interface | Written. Compiles and runs on Linux. Windows and macOS builds not yet attempted |
| 5. Build automation | Written, not yet run |
| 6. First real builds on Windows and macOS | **Next step, needs your GitHub account** |
| 7. Testing on real machines | Needs you (30 to 60 minutes) |
| 8. Website and privacy policy | Written, to be switched on in GitHub |
| 9. Microsoft Store | Needs your account |
| 10. Mac App Store | Needs your account and 99 USD a year |
| 11. After launch | Ongoing |

### What has been checked, and what has not

Checked on the machine where the code was written (Linux):

- The engine's 36 automated tests pass. They simulate whole working days and cover clicks landing inside the chosen area, the pause when you touch the mouse, stop rules, sleep and wake, screen edges and the timetable.
- The interface walkthrough passes: every screen, five languages, light and dark, with no script errors.
- The desktop app compiles in its three variants (normal, Microsoft Store, Mac App Store) and, in a live run on a virtual display, the timetable started a session by itself and the real pointer wiggled and came back.

Not checked, because it needs Windows or macOS:

- The two small files that talk to the real mouse on Windows (`src-tauri/src/input/windows.rs`) and macOS (`macos.rs`) have never been compiled. They were written against the libraries' source code, but expect that the first build may need a fix or two.
- The tray, hotkey, compact bar and drawing overlay have not been seen on Windows or macOS.
- The store packaging scripts have never run.

An independent review by a second agent, which read the Windows and macOS code against the libraries' sources, found no compile errors there and twelve other problems; all were fixed, and three got a test of their own.

Stage 6 exists to close that gap.

---

## Stage 1. Idea and research (done)

What was done: read the Green Dotter site and listed every feature; checked both stores' rules; chose the technology.

Decisions and why:

| Decision | Why |
|---|---|
| Tauri 2 (Rust + HTML) | One codebase for Windows and macOS, installers of a few MB, good store support |
| MIT licence | Open source is the main trust argument for an app that controls the mouse; MIT has no conflict with store terms |
| Engine separated from the desktop shell | The risky logic can be tested without a screen |
| No network code at all | Makes "no tracking" a fact anyone can verify rather than a promise |

## Stage 2. Product design (done)

Same functions as Green Dotter: click a spot or area at random intervals, wiggle mode, pause when the mouse is touched, stop rules, F8 kill switch, keep-awake, tray, compact bar, weekly timetable with randomised times, session history.

Where SoftMouseWiggler goes further:

- Open source, and the app never goes online.
- Up to five targets, at random or in turn.
- Clicks only if the pointer really reached the target, then puts the pointer back where it was.
- An invisible one-pixel wiggle as an alternative to the visible drift.
- Also pauses when you type, on systems that report keyboard activity.
- Hotkey can be changed.
- Two ways to choose a target: draw on a dimmed screen, or capture the pointer position after a countdown. The second works on any monitor layout.
- Five languages from day one.

Left out of version 0.1, as ideas for later: a plain unbranded icon, battery-aware pausing, profiles, an automatic update check for the direct-download build.

## Stage 3. Engine (done)

`core/` holds all the decisions: when to act, the curved pointer paths, the stop rules, the timetable. It contains no operating-system code, so tests can hand it a pretend mouse and a pretend clock.

Run the tests yourself: `cargo test -p wiggler-core`

## Stage 4. Desktop app and interface (written)

- `src-tauri/`: windows, tray, hotkey, settings files, the background thread, and one file per operating system for the real mouse.
- `ui/`: the three tabs, the compact bar and the drawing overlay, in plain HTML, CSS and JavaScript.

## Stage 5. Build automation (written)

Two GitHub Actions workflows:

- `ci.yml` runs on every push: tests, then real builds on Windows, macOS and Linux. The installers can be downloaded from the run page.
- `release.yml` runs when you push a version tag such as `v0.1.0`: builds the Windows installer, the macOS disk image and the Microsoft Store package, and prepares a draft release.

---

## Stage 6. First real builds (you: 15 minutes, then Claude)

GitHub builds on real Windows and macOS machines for free on public repositories.

1. Create a free account at github.com if you do not have one.
2. Create a new **public** repository named `softmousewiggler`, empty (no README).
3. Put the code there. Either:
   - unzip the project, and in that folder run
     `git remote add origin https://github.com/YOUR-NAME/softmousewiggler.git` then `git push -u origin main`, or
   - connect GitHub to Claude and ask Claude to push it.
4. Open the repository's **Actions** tab and watch the "CI" run (about 15 minutes the first time).
5. If a build fails, copy the error to Claude, or give Claude access to the repository so it can read the logs and fix the code itself. This is the normal loop: build, read the error, fix, push again.
6. When it is green, download the installers from the run's **Artifacts** section.

Then fill in your own details (Claude can do this once you choose them):

- `src-tauri/links.json`: your repository, issues, privacy page and donation links.
- `docs/index.html`: replace `YOUR-GITHUB-NAME` (five places).
- `LICENSE`: your name, if you want it there.

## Stage 7. Test on real machines (you: 30 to 60 minutes)

Install the build from stage 6 on a Windows PC and, if you have one, a Mac. The unsigned Windows installer shows a SmartScreen warning ("More info", then "Run anyway"); the unsigned Mac build needs right-click, Open. Both warnings disappear for store installs.

Checklist:

- [ ] Wiggle mode: the pointer makes a short movement within a few seconds, then every 20 to 50 seconds, and returns to where it was.
- [ ] Touch the mouse: the panel turns amber and it waits; it resumes about 5 seconds after you stop.
- [ ] Type on the keyboard while it runs: it should also wait.
- [ ] F8 starts and stops it while another window is in front.
- [ ] Click mode, "Draw an area": the screen dims, you draw a box, and "Show me" moves the pointer to the middle of that box. Try it on a second monitor and on a high-resolution screen if you have them. This conversion is the least certain part of the app; if "Show me" lands in the wrong place, use "Capture with the pointer" and report it.
- [ ] Click mode with a harmless target (an empty part of the desktop): clicks land inside the box.
- [ ] Stop rules: "after 3 clicks" stops after three.
- [ ] Compact bar: appears, can be dragged, pause and stop work.
- [ ] Close the window: the app stays in the tray; the tray menu works; Quit really quits.
- [ ] Timetable: add a block that starts two minutes from now and watch it start.
- [ ] Leave it running for 20 minutes: the screen does not sleep, and Teams or Slack stay "active".
- [ ] Mac only: the first Start asks for permission; after allowing it in System Settings, Start works.

Report anything odd to Claude with a description or a screenshot.

## Stage 8. Website and privacy policy (you: 5 minutes)

Both stores require a public privacy policy address.

1. In the repository: **Settings > Pages > Build and deployment**: Source "Deploy from a branch", branch `main`, folder `/docs`. Save.
2. After a minute the site is live at `https://YOUR-NAME.github.io/softmousewiggler/` and the policy at `.../privacy.html`.
3. Optional: a custom domain costs about 10 to 15 EUR a year.

Donations: create a page on GitHub Sponsors, Ko-fi or Buy Me a Coffee and put its address in `src-tauri/links.json` under `donate`. The button then appears in Settings on Windows and in the direct-download Mac build.

## Stage 9. Microsoft Store (you: about 1 hour of forms, then 1 to 3 working days of waiting)

Cost: free for individuals since September 2025.

1. Register at https://storedeveloper.microsoft.com as an individual. Identity is checked with a government ID and a selfie.
2. In Partner Center, create a new app and **reserve the name** "SoftMouseWiggler". If it is taken, choose another and tell Claude so the name can be changed everywhere.
3. Open **Product management > Product identity** and copy three values: Package/Identity/Name, Package/Identity/Publisher, and Publisher display name.
4. In your GitHub repository: **Settings > Secrets and variables > Actions > Variables**, add them as `MSSTORE_IDENTITY_NAME`, `MSSTORE_PUBLISHER` and `MSSTORE_PUBLISHER_DISPLAY_NAME`.
5. Set the version to `1.0.0` for the first store release (a version starting with 0 may be refused by the Store), then make a release: `git tag v1.0.0` and `git push origin v1.0.0` (or ask Claude). When the "Release" workflow finishes, download the `.msix` file from the draft release.
6. Back in Partner Center, start a submission:
   - **Packages**: upload the `.msix`. Microsoft signs it; you need no certificate.
   - **Properties**: category Utilities & tools; privacy policy URL from stage 8.
   - **Age ratings**: answer the questionnaire (no to everything).
   - **Store listings**: texts are ready in `docs/STORE-LISTING.md`; add at least four screenshots of the real app.
   - **Pricing**: Free.
   - **Notes for certification**: paste the Microsoft note from `docs/STORE-LISTING.md`.
7. Submit. Certification usually takes up to three working days.

Things to know:

- The Store package relies on the WebView2 component that ships with Windows 11 and with up-to-date Windows 10. On an old, never-updated Windows 10 the app shows an error asking for it.
- "Open when the computer starts" is hidden in the Store build. Windows offers the same switch under Settings > Apps > Startup. Started that way, the window shows unless "Open quietly in the tray" is on.
- Donations: Microsoft's policy allows voluntary donations in non-game apps through a secure third-party service. Whether a plain link to a donation page satisfies the wording is an interpretation, not something Microsoft has confirmed. If certification objects, set `donate` to empty for the Store build and keep the link on the website.

## Stage 10. Mac App Store (you: 2 to 3 hours the first time, then 1 to 3 days of waiting)

Cost: Apple Developer Program, 99 USD a year. You need access to a Mac for the signing step, or the GitHub macOS runner with your certificates stored as secrets.

Honest risk assessment: comparable apps are on the Mac App Store today (Amphetamine moves the pointer; several mouse movers and auto clickers are listed). But Apple has rejected at least one sandboxed app for posting synthetic input, and review is not consistent. Expect that it may take a second attempt with an explanation, and keep the fallback below in mind.

1. Enrol at https://developer.apple.com/programs/ (identity check, payment).
2. In **Certificates, Identifiers & Profiles**:
   - Register an App ID with the bundle identifier from `src-tauri/tauri.conf.json` (`app.softmousewiggler`, or your own, for example `com.yourname.softmousewiggler`; if you change it, change `APP_ID` in `src-tauri/src/lib.rs` too).
   - Create two certificates: "Apple Distribution" and "Mac Installer Distribution".
   - Create a provisioning profile of type "Mac App Store Connect" for that App ID and download it as `src-tauri/embedded.provisionprofile`.
3. In `src-tauri/Entitlements.plist`, replace `TEAMID` with your Team ID (shown on your Apple developer account page).
4. In **App Store Connect**, create the app (platform macOS, the same bundle identifier), and create an API key under Users and Access > Integrations for uploading.
5. On a Mac, run `packaging/macos/build-appstore.sh` with the variables described at the top of that script. It builds the sandboxed app, signs it, makes the `.pkg` and uploads it.
6. In App Store Connect, fill in the listing from `docs/STORE-LISTING.md`: description, keywords, screenshots (1280x800 or one of the other accepted sizes), privacy "Data Not Collected", age rating, price Free, and the **review notes**, which explain the permission and how to test.
7. Submit for review.

Mac App Store specifics already handled in the code:

- The build is sandboxed and requests no network or file access.
- It uses the event-posting permission that Apple's engineers describe as sandbox-compatible, not the Accessibility API.
- The external donation link and the launch-at-login switch are removed in this build. Apple generally wants tips to go through in-app purchase; adding a "tip jar" purchase is a possible later step.

After granting the permission, macOS may only honour it once the app has been quit and reopened; the app says so.

Still to verify in the sandboxed build before submitting (stage 7 checklist, on the Store build): the hotkey, the tray icon, keep-awake, and the dimmed drawing overlay, which uses a transparent window. If the overlay causes a rejection, "Capture with the pointer" covers the same need and the overlay can be switched off for this build.

**Fallback if Apple says no:** distribute the Mac version yourself as a signed and notarised disk image from your website and GitHub. It needs the same 99 USD membership, a "Developer ID Application" certificate, and the secrets listed in `release.yml`. Users get no security warning, and the donation link is allowed.

## Stage 11. After launch

- **Updates**: change the version in `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `core/Cargo.toml` and `package.json`, add a line to `CHANGELOG.md`, push a new tag, upload the new packages to both stores.
- **Feedback**: GitHub Issues is the support channel; the link is in the app.
- **Being found**: the store listings; a post on Reddit (r/software, r/opensource), Hacker News "Show HN" and Product Hunt; listings on AlternativeTo and on open-source directories; winget and Homebrew packages for the direct-download builds.
- **Translations**: each language is one block in `ui/i18n.js`; contributors can add more.
- **Trust**: keep the builds reproducible from the public workflow, and say so on the site.

## Costs in one place

| Item | Cost |
|---|---|
| GitHub (code, builds, website) | Free |
| Microsoft Store developer account | Free |
| Apple Developer Program | 99 USD a year |
| Custom domain (optional) | About 10 to 15 EUR a year |
| Windows code-signing certificate | Not needed for the Store. Only for a warning-free direct download (free through SignPath for open-source projects, on application) |

## What only you can do

Create the accounts and prove your identity; accept the store agreements; pay Apple; choose the final name, bundle identifier and donation service; test on your own machines; press Submit.

Everything else (fixing build errors, changing features, screenshots, texts, translations, release notes) can be handed to Claude.

---

## How this was built with an agent

Useful if you want to repeat the method on another project.

1. **Research before code.** The first step was reading the reference product and the stores' rules. Two helper agents ran in parallel, one on store policy and one on the exact programming interfaces, and reported back with sources. That shaped the design: for example, the Mac version avoids the Accessibility API because the research showed sandboxed apps cannot use it.
2. **Make the risky part testable.** The agent could not see a Windows or Mac screen, so the design put every decision in a part that needs no screen and wrote tests for it first. Several real bugs were caught this way, such as mistaking a screen edge for the user grabbing the mouse.
3. **Check with tools, not by rereading.** Tests, a compiler, a scripted walkthrough of the interface with screenshots, and a live run on a virtual display. The agent looked at the screenshots and fixed what it saw.
4. **A second pair of eyes.** A separate agent that had not written the code reviewed the parts that could not be compiled.
5. **Say what is not verified.** The list near the top of this file is as important as the code.

How to work with an agent on the next steps:

- Give it the outcome and the constraints, as you did ("same functions, free, no ads, both stores").
- Give it access to what it needs: the repository, and the build logs. An agent that can read a failing build can fix it without you in the middle.
- Ask for evidence: "show me the test that proves it".
- Keep decisions that are yours (names, money, publishing) and hand over the rest.
