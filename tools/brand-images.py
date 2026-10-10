"""Render the banner images: the social preview for the website, and a
profile picture and cover for the Ko-fi page.
Run: python3 tools/brand-images.py  ->  docs/img/social.png, assets/kofi-*.png"""
import base64, pathlib
from playwright.sync_api import sync_playwright
root = pathlib.Path(__file__).resolve().parent.parent
icon = base64.b64encode((root / "assets/icon.svg").read_bytes()).decode()
shot = base64.b64encode((root / "docs/img/run-wiggling.png").read_bytes()).decode()

def banner(w, h, big, show_shot=True):
    return f"""<style>
    html,body{{margin:0}} body{{width:{w}px;height:{h}px;background:#1a2a78;color:#eef1ff;font-family:system-ui,'Segoe UI',Roboto,sans-serif;overflow:hidden;position:relative}}
    .t{{position:absolute;left:{int(w*0.06)}px;top:50%;transform:translateY(-50%);max-width:{int(w*0.56)}px}}
    .l{{display:flex;align-items:center;gap:{int(big*0.3)}px;font-weight:700;font-size:{int(big*0.52)}px;letter-spacing:-0.01em}}
    .l img{{width:{int(big*0.8)}px;height:{int(big*0.8)}px}}
    h1{{font-size:{big}px;line-height:1.04;letter-spacing:-0.035em;margin:{int(big*0.35)}px 0 {int(big*0.3)}px}}
    p{{font-size:{int(big*0.42)}px;color:#b3bdf0;margin:0}}
    .s{{position:absolute;right:{int(w*0.06)}px;top:{int(h*0.12)}px;width:{int(w*0.27)}px;border-radius:14px;box-shadow:0 24px 60px rgba(4,8,30,.55)}}
    svg{{position:absolute;right:0;bottom:0;width:{int(w*0.5)}px;opacity:.35}}
    </style>
    <svg viewBox="0 0 240 60"><path d="M0 44 C 45 4, 80 58, 120 30 S 190 6, 240 34" fill="none" stroke="#b3bdf0" stroke-width="1.2" stroke-dasharray="2 6" stroke-linecap="round"/></svg>
    <div class="t"><div class="l"><img src="data:image/svg+xml;base64,{icon}">HappyWiggle</div>
    <h1>Keeps your computer awake. Then gets out of your way.</h1>
    <p>Free, open-source mouse wiggler and auto clicker.</p></div>
    {'<img class="s" src="data:image/png;base64,' + shot + '">' if show_shot else ''}"""

with sync_playwright() as p:
    b = p.chromium.launch()
    for name, w, h, big, s in [("docs/img/social.png", 1200, 630, 58, True), ("assets/kofi-cover.png", 1500, 500, 54, True)]:
        pg = b.new_page(viewport={"width": w, "height": h}); pg.set_content(banner(w, h, big, s)); pg.wait_for_timeout(300)
        pg.screenshot(path=str(root / name)); pg.close()
    pg = b.new_page(viewport={"width": 512, "height": 512})
    pg.set_content(f"<style>html,body{{margin:0;background:#1a2a78}}img{{display:block;width:512px;height:512px}}</style><img src='data:image/svg+xml;base64,{icon}'>")
    pg.wait_for_timeout(200); pg.screenshot(path=str(root / "assets/kofi-profile.png")); b.close()
print("done")
