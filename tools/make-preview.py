"""Bundle the UI into one HTML file that runs in any browser with the pretend
back end. Run: python3 tools/make-preview.py   ->  dist/preview.html"""
import pathlib, re
root = pathlib.Path(__file__).resolve().parent.parent
ui = root / "ui"
html = (ui / "index.html").read_text()
html = html.replace('<link rel="stylesheet" href="app.css">', "<style>\n" + (ui / "app.css").read_text() + "</style>")
def inline(m):
    name = m.group(1)
    return "<script>\n" + (ui / name).read_text() + "</script>"
html = re.sub(r'<script src="([\w.-]+)"></script>', inline, html)
svg = (ui / "icon.svg").read_text()
import base64
html = html.replace('src="icon.svg"', 'src="data:image/svg+xml;base64,' + base64.b64encode(svg.encode()).decode() + '"')
html = html.replace("<title>HappyWiggle</title>", "<title>HappyWiggle Preview</title>")
out = root / "dist"; out.mkdir(exist_ok=True)
(out / "preview.html").write_text(html)
print("wrote", out / "preview.html", len(html) // 1024, "KB")
