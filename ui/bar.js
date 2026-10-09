// The compact bar: time left, the clock time it stops, pause and stop.
// `initBar` is also used by the browser preview, inside the main page.
window.initBar = async function (root) {
  const { t, clock, span, num } = I18N;
  root.className = 'bar';
  root.setAttribute('data-tauri-drag-region', '');
  root.innerHTML =
    '<span class="dot"></span>' +
    '<div class="bar-main" data-tauri-drag-region><strong class="bar-clock"></strong><span class="bar-sub"></span></div>' +
    '<span class="bar-count" data-tauri-drag-region></span>' +
    '<button type="button" class="icon bar-pause"></button>' +
    '<button type="button" class="icon bar-stop"><svg viewBox="0 0 20 20" width="16" height="16"><rect x="4" y="4" width="12" height="12" rx="2" fill="currentColor"/></svg></button>' +
    '<button type="button" class="icon bar-expand"><svg viewBox="0 0 20 20" width="16" height="16"><path d="M4 8V4h4M16 12v4h-4M4 4l5 5M16 16l-5-5" fill="none" stroke="currentColor" stroke-width="1.800" stroke-linecap="round" stroke-linejoin="round"/></svg></button>';
  const $ = (s) => root.querySelector(s);
  const PLAY = '<svg viewBox="0 0 20 20" width="16" height="16"><path d="M6 4l10 6-10 6z" fill="currentColor"/></svg>';
  const PAUSE = '<svg viewBox="0 0 20 20" width="16" height="16"><path d="M5 4h3.500v12H5zM11.500 4H15v12h-3.500z" fill="currentColor"/></svg>';
  let status = null, settings = null, wasActive = false;

  function render() {
    if (!status) return;
    const s = status, active = s.phase !== 'idle';
    root.dataset.phase = s.phase;
    const timed = s.remainingMs != null;
    $('.bar-clock').textContent = active ? span(timed ? s.remainingMs : s.activeMs) : t('state.off');
    let sub = '';
    if (s.phase === 'userActive') sub = t('state.user');
    else if (s.phase === 'paused') sub = t('state.paused');
    else if (active) sub = s.stopsAtS != null ? t('state.stops', { time: clock(s.stopsAtS) }) : t('state.forever');
    $('.bar-sub').textContent = sub;
    $('.bar-count').textContent = active ? t(s.mode === 'click' ? 'run.clicks' : 'run.wiggles') + ': ' + num(s.actions) : '';
    const resume = s.phase === 'paused' || s.phase === 'userActive';
    $('.bar-pause').innerHTML = resume ? PLAY : PAUSE;
    $('.bar-pause').title = t(resume ? 'run.resume' : 'run.pause');
    $('.bar-pause').classList.toggle('go', resume);
    $('.bar-stop').title = t('run.stop');
    $('.bar-expand').title = t('bar.expand');
    // session over: fold the bar away without popping the big window up
    if (wasActive && !active) API.invoke('set_compact', { compact: false, quiet: true });
    wasActive = active;
  }

  $('.bar-pause').onclick = () => {
    const resume = status && (status.phase === 'paused' || status.phase === 'userActive');
    API.invoke('control', { action: resume ? 'resume' : 'pause' });
  };
  $('.bar-stop').onclick = async () => {
    wasActive = false;
    await API.invoke('control', { action: 'stop' });
    API.invoke('set_compact', { compact: false });
  };
  $('.bar-expand').onclick = () => API.invoke('set_compact', { compact: false });

  API.on('status', (s) => { status = s; render(); });
  API.on('settings', (s) => { settings = s; I18N.setLanguage(s.language); applyTheme(s.theme); render(); });
  function applyTheme(theme) { document.documentElement.dataset.theme = theme || 'system'; }

  const snap = await API.invoke('get_state');
  status = snap.status; settings = snap.settings;
  wasActive = status.phase !== 'idle';
  if (!document.getElementById('tab-run')) { I18N.setLanguage(settings.language); applyTheme(settings.theme); }
  render();
};
