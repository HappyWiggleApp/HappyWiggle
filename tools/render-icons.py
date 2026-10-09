"""Render the SVG artwork in assets/ to PNG. Run: python3 tools/render-icons.py
Then: npx tauri icon assets/icon.png   (creates every size the installers need)"""
import pathlib
from playwright.sync_api import sync_playwright

root = pathlib.Path(__file__).resolve().parent.parent
jobs = [("icon.svg", 1024, root / "assets/icon.png"),
        ("tray-on.svg", 64, root / "src-tauri/icons/tray-on.png"),
        ("tray-off.svg", 64, root / "src-tauri/icons/tray-off.png")]
with sync_playwright() as p:
    b = p.chromium.launch()
    for name, size, out in jobs:
        pg = b.new_page(viewport={"width": size, "height": size})
        svg = (root / "assets" / name).read_text()
        pg.set_content(f"<style>html,body{{margin:0;background:transparent}}svg{{display:block}}</style>{svg}")
        pg.screenshot(path=str(out), omit_background=True)
        pg.close()
    b.close()
