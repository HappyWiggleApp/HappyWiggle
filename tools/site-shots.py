"""Take the website's screenshots from the real interface (preview build).
Run: python3 tools/make-preview.py && python3 tools/site-shots.py"""
import pathlib
from playwright.sync_api import sync_playwright

root = pathlib.Path(__file__).resolve().parent.parent
out = root / "docs" / "img"; out.mkdir(parents=True, exist_ok=True)
HIDE = "#preview-note{display:none!important} body{overflow:hidden}"

with sync_playwright() as p:
    b = p.chromium.launch()
    def page(scheme="light", h=640):
        ctx = b.new_context(viewport={"width": 440, "height": h}, locale="en-GB", color_scheme=scheme, device_scale_factor=2)
        pg = ctx.new_page(); pg.goto((root / "dist" / "preview.html").as_uri()); pg.wait_for_selector("#btn-start")
        pg.add_style_tag(content=HIDE); return pg
    # wiggle, idle and running
    pg = page(); pg.screenshot(path=str(out / "run-idle.png"))
    pg.click("#btn-start"); pg.wait_for_timeout(5200); pg.screenshot(path=str(out / "run-wiggling.png"))
    pg.evaluate("API.touch()"); pg.wait_for_timeout(500); pg.screenshot(path=str(out / "run-waiting.png"))
    pg.click("#btn-compact"); pg.wait_for_timeout(400)
    pg.locator("#bar").screenshot(path=str(out / "bar-waiting.png"))
    pg.click(".bar-pause"); pg.wait_for_timeout(600)
    pg.locator("#bar").screenshot(path=str(out / "bar-running.png"))
    # click mode with a target and a stop rule
    pg = page(h=700); pg.click('[data-mode="click"]'); pg.click("#btn-draw")
    pg.mouse.move(120, 300); pg.mouse.down(); pg.mouse.move(330, 410, steps=4); pg.mouse.up(); pg.wait_for_timeout(300)
    pg.select_option("#f-stop-kind", "afterMinutes"); pg.fill("#f-stop-minutes", "90"); pg.keyboard.press("Tab"); pg.wait_for_timeout(500)
    pg.click("#btn-start"); pg.wait_for_timeout(4500); pg.screenshot(path=str(out / "run-clicking.png"))
    # timetable
    pg = page(h=760); pg.click('[data-tab="schedule"]'); pg.click("#btn-preset"); pg.wait_for_timeout(700)
    pg.screenshot(path=str(out / "schedule.png"))
    # advanced options, dark
    pg = page("dark", h=760); pg.click("#btn-advanced"); pg.wait_for_timeout(500)
    pg.evaluate("window.scrollTo(0, 250)"); pg.wait_for_timeout(200); pg.screenshot(path=str(out / "advanced-dark.png"))
    pg = page("dark"); pg.click("#btn-start"); pg.wait_for_timeout(4200); pg.screenshot(path=str(out / "run-wiggling-dark.png"))
    b.close()
print("screenshots in", out)
