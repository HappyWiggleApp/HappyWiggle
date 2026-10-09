"""Drive the preview build through its main screens in a headless browser,
fail on any JavaScript error, and save screenshots.
Run: python3 tools/ui-check.py [output-folder]"""
import pathlib, sys
from playwright.sync_api import sync_playwright

root = pathlib.Path(__file__).resolve().parent.parent
out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else root / "dist" / "shots"); out.mkdir(parents=True, exist_ok=True)
errors = []
with sync_playwright() as p:
    b = p.chromium.launch()
    def page(lang="en-US", scheme="light", w=440, h=700):
        ctx = b.new_context(viewport={"width": w, "height": h}, locale=lang, color_scheme=scheme, device_scale_factor=2)
        pg = ctx.new_page()
        pg.on("pageerror", lambda e: errors.append(str(e)))
        pg.on("console", lambda m: errors.append(m.text) if m.type == "error" else None)
        pg.goto((root / "dist" / "preview.html").as_uri())
        pg.wait_for_selector("#btn-start")
        return pg
    pg = page()
    pg.screenshot(path=str(out / "01-idle.png"))
    # simple view: the detailed fields stay out of sight until asked for
    assert not pg.is_visible("#f-int-min") and not pg.is_visible("#f-wiggle-distance")
    assert pg.input_value("#f-often") == "normal"
    pg.select_option("#f-often", "often"); pg.wait_for_timeout(450)
    pg.click("#btn-advanced"); pg.wait_for_timeout(450)
    assert pg.is_visible("#f-int-min") and pg.input_value("#f-int-min") == "5" and pg.input_value("#f-int-max") == "15"
    pg.fill("#f-int-max", "33"); pg.keyboard.press("Tab"); pg.wait_for_timeout(500)
    assert pg.input_value("#f-often") == "custom", "an odd interval shows as Custom"
    assert "33" in pg.inner_text("#opt-often-custom")
    pg.screenshot(path=str(out / "01b-advanced.png"), full_page=True)
    pg.select_option("#f-often", "normal"); pg.wait_for_timeout(450)
    assert pg.input_value("#f-int-max") == "50"
    pg.click("#btn-advanced"); pg.wait_for_timeout(450)
    assert not pg.is_visible("#f-int-min")
    pg.click("#btn-start"); pg.wait_for_timeout(4500)
    assert pg.get_attribute("#stage", "data-phase") == "running"
    pg.screenshot(path=str(out / "02-wiggling.png"))
    pg.click("#preview-touch"); pg.wait_for_timeout(400)
    assert pg.get_attribute("#stage", "data-phase") == "userActive"
    pg.screenshot(path=str(out / "03-user.png"))
    pg.click("#btn-compact"); pg.wait_for_timeout(300)
    pg.screenshot(path=str(out / "04-compact.png"))
    pg.click(".bar-expand"); pg.click("#btn-stop"); pg.wait_for_timeout(300)
    assert pg.get_attribute("#stage", "data-phase") == "idle"
    # click mode: needs a target first
    pg.click('[data-mode="click"]'); pg.click("#btn-start"); pg.wait_for_timeout(200)
    assert pg.is_visible("#notice"), "starting Click mode without a target must explain why"
    pg.screenshot(path=str(out / "05-click-empty.png"))
    pg.click("#btn-draw"); pg.mouse.move(120, 300); pg.mouse.down(); pg.mouse.move(320, 420, steps=5)
    pg.screenshot(path=str(out / "06-drawing.png")); pg.mouse.up(); pg.wait_for_timeout(300)
    assert pg.locator("#targets li").count() == 1
    pg.select_option("#f-stop-kind", "afterMinutes"); pg.fill("#f-stop-minutes", "90"); pg.keyboard.press("Tab")
    pg.wait_for_timeout(500); pg.click("#btn-start"); pg.wait_for_timeout(4000)
    pg.screenshot(path=str(out / "07-clicking.png"), full_page=True)
    assert "1:29" in pg.inner_text("#st-clock")
    pg.click("#btn-pause"); pg.wait_for_timeout(300); assert pg.get_attribute("#stage", "data-phase") == "paused"
    pg.click("#btn-stop")
    # schedule
    pg.click('[data-tab="schedule"]'); pg.screenshot(path=str(out / "08-schedule-empty.png"))
    pg.click("#btn-preset"); pg.wait_for_timeout(600)
    assert pg.locator(".week .plan").count() == 10, pg.locator(".week .plan").count()
    pg.screenshot(path=str(out / "09-schedule.png"), full_page=True)
    assert pg.is_visible("#sched-autostart"), "the timetable suggests opening at startup"
    pg.click('[data-tab="settings"]'); pg.wait_for_timeout(200)
    assert pg.is_visible("#f-autostart") and not pg.is_visible("#f-compact")
    pg.screenshot(path=str(out / "10-settings.png"), full_page=True)
    pg.click("#f-advanced"); pg.wait_for_timeout(450)
    assert pg.is_visible("#f-compact") and pg.is_visible("#f-hidden")
    pg.click("#f-autostart"); pg.wait_for_timeout(450)
    pg.click('[data-tab="schedule"]'); pg.wait_for_timeout(300)
    assert not pg.is_visible("#sched-autostart")
    pg.click('[data-tab="run"]'); pg.wait_for_timeout(200)
    assert pg.is_visible("#f-int-min"), "one switch opens the advanced options everywhere"
    pg.screenshot(path=str(out / "10b-run-advanced-click.png"), full_page=True)
    pg.click('[data-tab="settings"]'); pg.wait_for_timeout(200)
    # hotkey capture
    pg.click("#btn-hotkey"); pg.keyboard.press("Control+Shift+KeyK"); pg.wait_for_timeout(500)
    assert pg.inner_text("#btn-hotkey") == "Ctrl + Shift + K", pg.inner_text("#btn-hotkey")
    # every language renders without missing keys
    for lang in ["fr", "de", "es", "pt"]:
        pg.select_option("#f-language", lang); pg.wait_for_timeout(400)
        for tab in ["run", "schedule", "settings"]:
            pg.click(f'[data-tab="{tab}"]'); pg.wait_for_timeout(250)
            text = pg.inner_text("body")
            import re
            leaks = re.findall(r"\b(?:tab|run|state|mode|click|sched|set|err|hist|stop|behave|about|jit|common|wiggle|often|adv)\.[a-z0-9.]+\b", text)
            assert not leaks, (lang, tab, leaks)
            wide = pg.evaluate("document.documentElement.scrollWidth > document.documentElement.clientWidth")
            assert not wide, f"horizontal overflow in {lang}/{tab}"
            if lang == "fr": pg.screenshot(path=str(out / f"11-fr-{tab}.png"), full_page=True)
    pg = page(scheme="dark"); pg.click("#btn-start"); pg.wait_for_timeout(3500)
    pg.screenshot(path=str(out / "12-dark-running.png"))
    pg.click('[data-tab="schedule"]'); pg.click("#btn-preset"); pg.wait_for_timeout(500)
    pg.screenshot(path=str(out / "13-dark-schedule.png"), full_page=True)
    b.close()
if errors:
    print("JS ERRORS:"); [print(" ", e) for e in errors]; sys.exit(1)
print("ui-check ok")
