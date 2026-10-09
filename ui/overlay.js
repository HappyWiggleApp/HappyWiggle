// The dimmed full-screen window used to draw a click area.
(async function () {
  const invoke = window.__TAURI__.core.invoke;
  try {
    const snap = await invoke('get_state');
    I18N.setLanguage(snap.settings.language);
  } catch (_) { /* English is fine */ }
  document.getElementById('help').textContent = I18N.t('overlay.help');
  window.startDraw(document.body, (rect) => {
    invoke('overlay_done', { rect });
  });
})();
