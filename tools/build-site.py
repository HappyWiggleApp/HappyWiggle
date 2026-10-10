"""Build the website in docs/ (served by GitHub Pages).

    python3 tools/build-site.py

One shared header and footer; the pages are plain HTML with no scripts.
The privacy page is generated from PRIVACY.md so the two never disagree.
Screenshots come from tools/site-shots.py.
"""
import html
import pathlib
import re

root = pathlib.Path(__file__).resolve().parent.parent
docs = root / "docs"

REPO = "https://github.com/HappyWiggleApp/HappyWiggle"
RELEASES = REPO + "/releases"
KOFI = "https://ko-fi.com/happywiggleapp"


def page(title, description, body, here=""):
    def nav(href, label, cls=""):
        cur = ' aria-current="page"' if href == here else ""
        c = f' class="{cls}"' if cls else ""
        return f'<a href="{href}"{c}{cur}>{label}</a>'

    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<meta name="description" content="{description}">
<meta property="og:title" content="{title}">
<meta property="og:description" content="{description}">
<meta property="og:type" content="website">
<meta property="og:image" content="https://happywiggleapp.github.io/HappyWiggle/img/social.png">
<link rel="icon" href="icon.svg">
<link rel="stylesheet" href="site.css">
</head>
<body>
<div class="wrap">
  <header class="site-head">
    <a class="logo" href="./"><img src="icon.svg" width="32" height="32" alt="">HappyWiggle</a>
    <nav>
      {nav("./#modes", "How it works")}
      {nav("./#features", "Features")}
      {nav("faq.html", "Questions")}
      {nav("privacy.html", "Privacy")}
      <a href="{REPO}">Source code</a>
      <a class="coffee" href="{KOFI}">Buy me a coffee</a>
    </nav>
  </header>
{body}
  <footer>
    <div class="foot-links">
      <a href="./">Home</a><a href="faq.html">Questions</a><a href="privacy.html">Privacy policy</a>
      <a href="{REPO}">Source code</a><a href="{REPO}/issues">Report a problem</a><a href="{KOFI}">Support on Ko-fi</a>
    </div>
    <p>Made by Geordy. Free and open source under the MIT licence.</p>
    <p>HappyWiggle is an independent project. It is not affiliated with, endorsed by or sponsored by Microsoft, Apple, Slack, Zoom or any other company named on this site.</p>
    <p>Use it responsibly. Check your employer's and IT policies before using automation on a work or managed device, and do not use it to misrepresent whether you are working.</p>
  </footer>
</div>
</body>
</html>
"""


DOWNLOADS = f"""<div class="buttons">
        <a class="btn light" href="{RELEASES}">Download for Windows<small>Windows 10 or later, 64-bit</small></a>
        <a class="btn light" href="{RELEASES}">Download for Mac<small>macOS 11 or later, Intel and Apple Silicon</small></a>
        <a class="btn ghost" href="{REPO}">Read the code</a>
      </div>"""

INDEX = f"""
  <main>
  <div class="hero">
    <div class="hero-copy">
      <h1>Keeps your computer awake. Then gets out of your way.</h1>
      <p class="sub">A mouse wiggler and auto clicker in one small app. Free, open source, and it never goes online.</p>
      {DOWNLOADS}
      <p class="note">Windows is tested. The Mac version is an early preview. Downloads are on the project's GitHub releases page.</p>
    </div>
    <div class="shot">
      <div class="bar"><i></i><i></i><i></i><span>HappyWiggle</span></div>
      <img src="img/run-wiggling.png" width="880" height="1280" alt="HappyWiggle running in Wiggle mode: a dark blue panel shows the running time, the number of wiggles and the seconds until the next one.">
    </div>
  </div>

  <section>
    <h2>What is HappyWiggle?</h2>
    <p class="lede">It moves your mouse a little, or clicks a harmless spot, every so often. That keeps the screen on, stops the computer going to sleep, and keeps your session from going idle while you read, watch, present or step away for a moment.</p>
    <div class="facts-strip">
      <div><b>Free</b><span>No ads, no account, no paid tier.</span></div>
      <div><b>Open source</b><span>Every line is public on GitHub.</span></div>
      <div><b>Offline</b><span>It contains no network code at all.</span></div>
      <div><b>Five languages</b><span>English, French, German, Spanish, Portuguese.</span></div>
    </div>
  </section>

  <section id="modes">
    <h2>Two ways to stay active</h2>
    <p class="lede">Pick one on the main screen. You can switch whenever a session is not running.</p>
    <div class="duo">
      <div class="mode">
        <h3>Wiggle only</h3>
        <p>Moves the pointer in a short, curved drift and brings it back exactly where it was. It never clicks, so nothing on screen can be changed by accident.</p>
        <ul>
          <li>Nothing to set up: press Start.</li>
          <li>Prefer not to see it? Choose the invisible one-pixel nudge.</li>
        </ul>
        <img src="img/run-wiggling.png" width="880" height="1280" alt="Wiggle mode running.">
      </div>
      <div class="mode">
        <h3>Click areas</h3>
        <p>Choose an exact spot, or draw a box and let it click a random point inside. Useful for apps that only count a click as activity.</p>
        <ul>
          <li>Up to five targets, at random or in turn.</li>
          <li>It glides there like a hand, then puts the pointer back.</li>
        </ul>
        <img src="img/run-clicking.png" width="880" height="1400" alt="Click mode running with one target area and 1 hour 29 minutes left.">
      </div>
    </div>
  </section>

  <section>
    <h2>Three steps, then forget about it</h2>
    <ol class="steps">
      <li><h3>Choose a mode</h3><p>Wiggle only, or Click areas. For clicks, draw a box over somewhere harmless: an empty part of a window where a click sends, submits and changes nothing.</p></li>
      <li><h3>Say how often and when to stop</h3><p>Often, Normal or Rarely. Stop when you say so, after a number of clicks, after a number of minutes, or at a time of day.</p></li>
      <li><h3>Press Start</h3><p>Or press F8 from anywhere. Touch the mouse and it waits for you. Press F8 again and it stops.</p></li>
    </ol>
  </section>

  <section id="features">
    <h2>What you get</h2>
    <div class="features">
      <div><h3>It steps aside</h3><p>Touch the mouse and it waits, then carries on a few seconds after you stop. It never fights you for the pointer. On Windows it notices typing too.</p></div>
      <div><h3>It stops itself</h3><p>After a number of clicks or wiggles, after a number of minutes, or at a clock time. The screen shows how long is left and when it will stop.</p></div>
      <div><h3>One key, anywhere</h3><p>F8 starts and stops it whatever window is in front. You can change the key.</p></div>
      <div><h3>Random, not robotic</h3><p>Every interval is picked at random inside the range you choose, and every click lands on a different point in your area.</p></div>
      <div><h3>Careful clicks</h3><p>Before clicking it checks the pointer really reached the target. If it did not, it skips the click. It never clicks somewhere you did not choose.</p></div>
      <div><h3>Keeps the computer awake</h3><p>Screen on and sleep off while a session runs. The moment it ends, your normal power settings are back.</p></div>
      <div><h3>Lives in the tray</h3><p>Close the window and it stays in the tray or menu bar. Start, pause and stop from there.</p></div>
      <div><h3>Starts with the computer</h3><p>Optional. Switch it on and the timetable works without you thinking about it.</p></div>
      <div><h3>Simple first</h3><p>Three choices to get going. Exact seconds, movement style and distance sit behind "Advanced options" for those who want them.</p></div>
    </div>
  </section>

  <section id="timetable">
    <div class="split">
      <div>
        <h2>Set a timetable once</h2>
        <p class="lede">Pick the days and hours. HappyWiggle starts and stops itself.</p>
        <ul>
          <li>One click sets up "Weekdays, 9 to 5" with a lunch break.</li>
          <li>Start, stop and break each drift by a few minutes, differently every day.</li>
          <li>"Skip today" for a day off. Stop it by hand and it stays off until the next block.</li>
          <li>A history shows what actually happened: when it ran, and when it waited because you were at the mouse or the computer was asleep.</li>
        </ul>
        <p class="caption">It cannot wake a sleeping computer or get past a locked screen. If that happens mid-day, it carries on when you are back.</p>
      </div>
      <img src="img/schedule.png" width="880" height="1520" alt="The Schedule tab: a week view with blue bars from about 9 to 5 on weekdays, a gap at lunch, and the block editor below.">
    </div>
  </section>

  <section>
    <div class="split flip">
      <div class="stack">
        <img src="img/bar-running.png" width="800" height="128" alt="The compact bar while running: time, Runs until you stop it, wiggle count, pause, stop and expand buttons.">
        <img src="img/bar-waiting.png" width="800" height="128" alt="The compact bar in amber: waiting for you to let go of the mouse.">
      </div>
      <div>
        <h2>A small bar, if you prefer</h2>
        <p class="lede">Shrink the window to a slim bar that stays on top. One glance tells you how long is left and what time it stops.</p>
        <ul>
          <li>Pause, stop and expand are right there.</li>
          <li>It turns amber while it waits for you to let go of the mouse.</li>
          <li>Light and dark themes follow your computer, or choose one.</li>
        </ul>
      </div>
    </div>
  </section>

  <section id="trust">
    <div class="trust">
      <div>
        <h2>Private, and you can check</h2>
        <p class="lede">An app that moves your mouse deserves scrutiny. So everything is in the open.</p>
        <p class="caption">The installers are built by a public script from the public code, on GitHub's machines. Anyone can read the code, and anyone can rebuild it.</p>
      </div>
      <ul class="checks">
        <li>Runs entirely on your computer</li>
        <li>No account, no sign-in</li>
        <li>No analytics, no crash reports, no update pings</li>
        <li>Does not read your screen, files, keystrokes or messages</li>
        <li>Knows only the coordinates you gave it, not what is under them</li>
        <li>On Mac, one permission: to control the pointer</li>
        <li>Stop at any time with F8</li>
      </ul>
    </div>
  </section>

  <section>
    <h2>How it compares</h2>
    <p class="lede">There are several ways to keep a computer active. They do different things.</p>
    <div class="table-scroll">
    <table class="compare">
      <thead><tr><th></th><th>What it does</th><th>Worth knowing</th></tr></thead>
      <tbody>
        <tr><td>HappyWiggle</td><td>Real mouse movement or clicks, on a schedule you control, plus keep-awake.</td><td>Free and open source. Pauses when you use the computer.</td></tr>
        <tr><td>Keep-awake tools</td><td>Stop the screen and computer from sleeping.</td><td>They send no input, so apps that watch for activity still see you as idle.</td></tr>
        <tr><td>USB mouse jigglers</td><td>A gadget that pretends to be a mouse.</td><td>Costs money, has no schedule or stop rule, and cannot tell when you are back.</td></tr>
        <tr><td>Jiggler websites</td><td>A web page that animates something.</td><td>A browser page cannot create real mouse input, so most of them do nothing useful.</td></tr>
      </tbody>
    </table>
    </div>
  </section>

  <section>
    <h2>Common questions</h2>
    <div class="faq">
      <details><summary>Is it really free?</summary><div><p>Yes. No ads, no paid version, no account. If it is useful to you, you can <a href="{KOFI}">buy me a coffee</a>, and that is entirely optional.</p></div></details>
      <details><summary>Windows shows a security warning when I install it. Why?</summary><div><p>Windows warns about any new program it has not seen many times before. Click "More info", then "Run anyway". The warning does not appear for versions installed from the Microsoft Store.</p></div></details>
      <details><summary>Will it keep my status active in Teams or Slack?</summary><div><p>HappyWiggle sends real mouse input, which is what a computer uses to decide you are there. Whether a particular chat app keeps showing you as active also depends on that app and on your organisation's settings, and we have not tested every one. Try Wiggle mode first; if an app only reacts to clicks, use Click areas on a harmless spot.</p></div></details>
      <details><summary>Can it click something by accident?</summary><div><p>Wiggle mode never clicks. Click mode clicks only inside the areas you chose, and only after checking the pointer arrived there. Choose a spot where a click does nothing: an empty margin, not a button or a message list.</p></div></details>
    </div>
    <p class="caption"><a href="faq.html">All questions and answers</a></p>
  </section>

  <section>
    <div class="notice">
      <h2>Use it responsibly</h2>
      <p>HappyWiggle is a tool you control. Check your employer's and IT policies before using any automation on a work or managed device, and do not use it to misrepresent whether you are working.</p>
    </div>
  </section>

  <section>
    <div class="coffee-box">
      <div>
        <h2>Free for everyone, and it stays that way</h2>
        <p>No subscription, no nag screen, no upsell. If HappyWiggle saved you some trouble, a coffee keeps new versions coming.</p>
      </div>
      <div class="buttons" style="margin-top:0">
        <a class="btn light" href="{KOFI}">Buy me a coffee</a>
        <a class="btn ghost" href="{RELEASES}">Download instead</a>
      </div>
    </div>
  </section>
  </main>
"""

FAQ_GROUPS = [
    ("Getting started", [
        ("What is HappyWiggle?", "A small free app for Windows and Mac that keeps your computer awake and your session active. It either moves the mouse a little (Wiggle only) or clicks a spot or area you choose (Click areas), at random intervals."),
        ("Where do I download it?", f'From the <a href="{RELEASES}">releases page on GitHub</a>. Take the file ending in <strong>.exe</strong> for Windows or <strong>.dmg</strong> for Mac.'),
        ("Windows shows a security warning when I install it. Why?", "Windows SmartScreen warns about any new program that has not yet been downloaded by many people. Click \"More info\", then \"Run anyway\". Versions installed from the Microsoft Store do not show the warning."),
        ("My Mac says the app cannot be opened, or is damaged.", "The direct download is not yet signed with an Apple certificate. Right-click the app and choose Open, then confirm. The Mac version is an early preview; please report what you find."),
        ("Why does the Mac version ask for a permission?", "macOS requires your approval before any app may move the pointer or click. HappyWiggle asks once. Allow it under System Settings, Privacy & Security, Accessibility, then press Start again. If it still asks, quit HappyWiggle and open it again."),
        ("Is there a Linux version?", "The code builds on Linux and works under X11, but it is not offered as a supported download. Wayland does not let ordinary apps move the pointer."),
    ]),
    ("Using it", [
        ("Which mode should I use?", "Start with Wiggle only: it needs no setup and cannot click anything. Use Click areas only if something you rely on reacts to clicks and not to movement."),
        ("Where should I put a click area?", "Somewhere a click does nothing: an empty margin of a window, a blank part of the desktop. Not on a button, a link, a message list or a text box."),
        ("Can it click something by accident?", "Wiggle mode never clicks. Click mode clicks only inside your areas, and only after checking that the pointer really arrived there. If the pointer could not reach the target, for example because a monitor was unplugged, the click is skipped."),
        ("What happens when I come back to my desk?", "Touch the mouse and it waits; the panel turns amber. It carries on a few seconds after you stop, and you can change that delay. On Windows it also notices typing."),
        ("How do I stop it quickly?", "Press F8 from anywhere, whatever window is in front. You can also use the Stop button, the compact bar or the tray menu. The key can be changed in Settings."),
        ("What do Often, Normal and Rarely mean?", "Often is every 5 to 15 seconds, Normal every 20 to 50 seconds, Rarely every 1 to 3 minutes. The exact moment is random each time. Under Advanced options you can type your own range."),
        ("Can it run on a schedule?", "Yes. On the Schedule tab pick days and hours, or use the \"Weekdays, 9 to 5\" preset. Start, stop and lunch break drift by a few minutes each day. HappyWiggle has to be running for this, so switch on \"Open when the computer starts\"."),
        ("Does it work when the computer is asleep or the screen is locked?", "No. No app can move the mouse on a sleeping computer or behind a lock screen. While a session runs HappyWiggle keeps the computer awake, and if it does sleep or lock, the session carries on when you are back."),
        ("I closed the window. Is it still running?", "Yes. Closing the window keeps HappyWiggle in the tray (Windows) or menu bar (Mac), where the hotkey and timetable keep working. To close it completely, choose Quit from the tray menu or from Settings."),
    ]),
    ("Chat apps and presence", [
        ("Will it keep my status active in Teams, Slack or similar apps?", "HappyWiggle sends real mouse input, which is what the computer uses to decide you are there, and many apps take their cue from that. Whether a particular app keeps showing you as active also depends on the app and on your organisation's settings. We have not tested every app, so try it and see."),
        ("Movement alone does not keep one of my apps active. What now?", "Some apps only count activity inside their own window. Switch to Click areas and place a small area on a harmless part of that app's window."),
        ("Do keep-awake tools do the same thing?", "No. Tools that only prevent sleep send no input, so an app that watches for activity still sees you as idle. HappyWiggle does both."),
        ("Is it allowed at my workplace?", "That depends on your workplace. Check your employer's and IT policies before using automation on a work or managed device, and do not use it to misrepresent whether you are working."),
    ]),
    ("Safety and privacy", [
        ("Is it safe?", f'It only moves the pointer and clicks where you told it to. It does not read your screen, files, keystrokes or messages. You do not have to take that on trust: <a href="{REPO}">all the code is public</a>, and the installers are built from it by a public script.'),
        ("Does it send any data about me?", 'No. The app contains no network code: no analytics, no crash reports, no update checks. See the <a href="privacy.html">privacy policy</a>.'),
        ("What does it store on my computer?", "Three small files: your settings, a list of past sessions kept for 35 days, and one random number used to vary the timetable. Uninstalling removes them."),
        ("Will my antivirus complain?", "Tools that simulate mouse input are sometimes flagged by cautious security software, and new unsigned programs more so. If yours objects, you can inspect the code, build it yourself, or use the store version when it is available."),
    ]),
    ("The project", [
        ("Is it really free?", f'Yes. No ads, no paid tier, no account. Donations through <a href="{KOFI}">Ko-fi</a> are welcome and entirely optional.'),
        ("Who makes it?", "HappyWiggle is an independent project by Geordy, released under the MIT licence. The code was written with Claude, Anthropic's AI model."),
        ("I found a problem, or I have an idea.", f'Please <a href="{REPO}/issues">open an issue on GitHub</a>. Say which system you use and what you expected to happen.'),
        ("Can I help translate it?", f'Yes. Each language is one block of text in a single file. Open an issue or a pull request on <a href="{REPO}">GitHub</a>.'),
    ]),
]


def faq_page():
    parts = ['  <main>\n  <div class="page-title"><h1>Questions and answers</h1><p class="lede">How HappyWiggle works, what it does and does not do, and what to try when something is unclear.</p></div>']
    for name, items in FAQ_GROUPS:
        rows = "\n".join(f"      <details><summary>{html.escape(q)}</summary><div><p>{a}</p></div></details>" for q, a in items)
        parts.append(f'  <div class="faq-group">\n    <h2>{name}</h2>\n    <div class="faq">\n{rows}\n    </div>\n  </div>')
    parts.append(f"""  <section>
    <div class="coffee-box">
      <div><h2>Still stuck?</h2><p>Tell us what you see. A short description and a screenshot are usually enough.</p></div>
      <div class="buttons" style="margin-top:0"><a class="btn light" href="{REPO}/issues">Report a problem</a><a class="btn ghost" href="{RELEASES}">Download</a></div>
    </div>
  </section>
  </main>""")
    return "\n".join(parts)


def privacy_page():
    out, in_list = [], False

    def fmt(s):
        return re.sub(r"\*\*(.+?)\*\*", r"<strong>\1</strong>", html.escape(s))

    for line in (root / "PRIVACY.md").read_text().strip().split("\n"):
        if line.startswith("- "):
            if not in_list:
                out.append("<ul>"); in_list = True
            out.append("<li>" + fmt(line[2:]) + "</li>"); continue
        if in_list:
            out.append("</ul>"); in_list = False
        if line.startswith("# "):
            out.append('<div class="page-title"><h1>' + fmt(line[2:]) + "</h1></div>")
        elif line.startswith("## "):
            out.append("<h2>" + fmt(line[3:]) + "</h2>")
        elif line.strip():
            out.append("<p>" + fmt(line) + "</p>")
    if in_list:
        out.append("</ul>")
    return '  <main class="prose">\n' + "\n".join(out) + "\n  </main>"


(docs / "index.html").write_text(page(
    "HappyWiggle: free mouse wiggler and auto clicker for Windows and Mac",
    "Keeps your computer awake and your session active. Wiggle the mouse or click a safe area at random intervals. Free, open source, no ads, never goes online.",
    INDEX, "./"))
(docs / "faq.html").write_text(page(
    "Questions and answers: HappyWiggle",
    "How HappyWiggle works, installing on Windows and Mac, chat apps, safety and privacy.",
    faq_page(), "faq.html"))
(docs / "privacy.html").write_text(page(
    "Privacy policy: HappyWiggle",
    "HappyWiggle does not collect, transmit or share any personal data.",
    privacy_page(), "privacy.html"))
print("site built in", docs)
