// Draw a rectangle on a dimmed layer. Shared by the real full-screen overlay
// and by the browser preview.
window.startDraw = function (layer, onDone) {
  const box = document.createElement('div');
  box.className = 'draw-box';
  box.hidden = true;
  layer.appendChild(box);
  let start = null;

  function rectOf(e) {
    const x = Math.min(start.x, e.clientX), y = Math.min(start.y, e.clientY);
    return { x, y, w: Math.abs(e.clientX - start.x), h: Math.abs(e.clientY - start.y) };
  }
  function down(e) {
    if (e.button !== 0) return;
    start = { x: e.clientX, y: e.clientY };
    box.hidden = false;
    move(e);
    e.preventDefault();
  }
  function move(e) {
    if (!start) return;
    const r = rectOf(e);
    box.style.cssText = 'left:' + r.x + 'px;top:' + r.y + 'px;width:' + r.w + 'px;height:' + r.h + 'px';
  }
  function up(e) {
    if (!start) return;
    let r = rectOf(e);
    // a click without a drag means "this exact spot"
    if (r.w < 5 && r.h < 5) r = { x: start.x, y: start.y, w: 1, h: 1 };
    finish(r);
  }
  function key(e) { if (e.key === 'Escape') finish(null); }
  function finish(r) {
    layer.removeEventListener('mousedown', down);
    window.removeEventListener('mousemove', move);
    window.removeEventListener('mouseup', up);
    window.removeEventListener('keydown', key);
    box.remove();
    onDone(r);
  }
  layer.addEventListener('mousedown', down);
  window.addEventListener('mousemove', move);
  window.addEventListener('mouseup', up);
  window.addEventListener('keydown', key);
};
