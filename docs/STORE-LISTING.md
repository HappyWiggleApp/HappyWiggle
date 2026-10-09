# Store listing texts

Ready to paste into Partner Center (Microsoft) and App Store Connect (Apple).
Wording is deliberately about keeping the computer awake and automating clicks.
Do not add words like "undetectable" or "fake": both stores require accurate
descriptions and reviewers react badly to them.

## Name

SoftMouseWiggler

## Subtitle / short description (max 30 characters on Apple)

Mouse wiggler and auto clicker

## One-line description (Microsoft "short description")

Free, open-source mouse wiggler and auto clicker. Keeps your computer awake. No ads, no tracking.

## Full description (English)

SoftMouseWiggler keeps your computer awake and your session active, without ads, accounts or tracking.

Two modes in one small app:

WIGGLE ONLY
Moves the mouse in a short, natural-looking drift and brings it back where it was. Or choose the invisible one-pixel nudge. It never clicks.

CLICK AREAS
Pick an exact spot, or draw an area and let it click a random point inside. Up to five targets, at random intervals you set.

IT GETS OUT OF YOUR WAY
Touch the mouse and it waits. It carries on a few seconds after you stop. It never fights you for the pointer.

IT STOPS ITSELF
After a number of clicks, after a number of minutes, or at a time of day.

TIMETABLE
Choose days and hours once. It starts and stops on its own, a few minutes earlier or later each day, with an optional break in the middle.

ALWAYS UNDER CONTROL
One hotkey starts and stops it from anywhere. A slim bar shows the time left. It also lives quietly in the tray.

PRIVATE BY DESIGN
Free and open source. The app never connects to the internet. It does not read your screen, your files or your messages.

Please use it responsibly: check your organisation's rules before using automation on a work device.

## Full description (French)

SoftMouseWiggler garde votre ordinateur éveillé et votre session active, sans publicité, sans compte et sans suivi.

Deux modes dans une petite application :

MOUVEMENT SEUL
Déplace la souris d'un petit mouvement naturel puis la remet à sa place. Ou choisissez le déplacement invisible d'un pixel. Il ne clique jamais.

ZONES DE CLIC
Choisissez un point précis, ou dessinez une zone dans laquelle il cliquera au hasard. Jusqu'à cinq cibles, à des intervalles aléatoires que vous réglez.

IL S'EFFACE
Touchez la souris et il attend. Il reprend quelques secondes après votre dernier geste. Il ne vous dispute jamais le pointeur.

IL S'ARRÊTE SEUL
Après un nombre de clics, après un nombre de minutes, ou à une heure donnée.

HORAIRES
Choisissez les jours et les heures une fois. Il démarre et s'arrête seul, quelques minutes plus tôt ou plus tard chaque jour, avec une pause possible au milieu.

TOUJOURS SOUS CONTRÔLE
Un raccourci clavier le démarre et l'arrête depuis n'importe où. Une barre fine affiche le temps restant. Il reste aussi discrètement dans la zone de notification.

RESPECT DE LA VIE PRIVÉE
Gratuit et open source. L'application ne se connecte jamais à Internet. Elle ne lit ni votre écran, ni vos fichiers, ni vos messages.

À utiliser de façon responsable : vérifiez les règles de votre organisation avant d'utiliser un outil d'automatisation sur un ordinateur professionnel.

## Keywords (Apple: 100 characters, comma separated)

mouse jiggler,auto clicker,keep awake,mouse mover,no sleep,wiggle,caffeine,idle,autoclick,scheduler

## Search terms (Microsoft: up to 7)

mouse jiggler, mouse mover, auto clicker, keep awake, prevent sleep, wiggler, auto click

## Category

Microsoft: Utilities & tools. Apple: Utilities.

## Age rating

No objectionable content of any kind: answer "No" or "None" to every question.

## Privacy answers

- Apple "App Privacy": Data Not Collected.
- Microsoft: "Does this product access, collect or transmit personal information?" No. Still give the privacy policy URL.

## Notes for the reviewer (Apple "App Review Information")

SoftMouseWiggler is a keep-awake and click-automation utility, in the same category as other mouse movers already on the Mac App Store.

How to test: open the app, press Start in "Wiggle only" mode. macOS asks once for permission to control the pointer (Privacy & Security > Accessibility); allow it and press Start again. The pointer makes a short movement every 20 to 50 seconds and returns to where it was. Touch the mouse and the app pauses itself. Press F8 or Stop to end.

Technical notes: the app is sandboxed. It posts mouse events with CGEvent.post and asks for access with CGRequestPostEventAccess. It does not use the Accessibility API (AXUIElement), does not observe or record other apps, has no network entitlement and no file access. Keep-awake uses an IOKit power assertion that is released when the session ends. There are no hidden features and no in-app purchases.

## Notes for certification (Microsoft)

Desktop (full trust) app packaged as MSIX. It sends mouse input with the standard SendInput API only while the user has started a session, and stops with F8 or the Stop button. No account or sign-in is needed to test. The app makes no network connections.

## Screenshots to prepare

At least four, taken from the real app:
1. Run tab, wiggling (dark blue panel).
2. Run tab, Click areas with one target.
3. Schedule tab with the week view.
4. The compact bar over a desktop.

Sizes: Mac App Store accepts 1280x800, 1440x900, 2560x1600 or 2880x1800. Microsoft Store: 1366x768 or larger.
