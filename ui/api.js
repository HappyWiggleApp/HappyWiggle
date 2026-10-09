// One small door to the native side.
//
// Inside the app, calls go to the Rust commands in src-tauri/src/lib.rs.
// Opened in an ordinary browser (for design work, or the online preview),
// a pretend back end below answers instead, so every screen can be tried
// without touching a real mouse.
(function () {
  const T = window.__TAURI__;
  if (T) {
    window.API = {
      preview: false,
      invoke: (cmd, args) => T.core.invoke(cmd, args),
      on: (name, fn) => T.event.listen(name, (e) => fn(e.payload)),
    };
    return;
  }

  // ------------------------------------------------------------ pretend back end
  const listeners = {};
  const emit = (name, payload) => (listeners[name] || []).forEach((fn) => fn(JSON.parse(JSON.stringify(payload))));
  const jit = (kind, minutes) => ({ kind, minutes });
  const presetBlock = () => ({
    days: [true, true, true, true, true, false, false], startMin: 540, endMin: 1020,
    startJitter: jit('before', 10), endJitter: jit('after', 10),
    break: { startMin: 780, endMin: 840, startJitter: jit('either', 5), endJitter: jit('either', 5) }, mode: null,
  });
  let settings = {
    mode: 'wiggle', areas: [], areaOrder: 'random', button: 'left', clickMinS: 20, clickMaxS: 50,
    smoothMove: true, returnCursor: true, wiggleStyle: 'natural', wiggleMinS: 20, wiggleMaxS: 50, wiggleDistance: 60,
    stop: { kind: 'never', actions: 100, minutes: 60, atMinute: 1020 },
    pauseOnUser: true, resumeAfterS: 5, keepAwake: true, hotkey: 'F8', compactOnStart: false,
    theme: 'system', language: 'auto', autostart: false, startHidden: false, onboarded: false, advanced: false,
    schedule: { enabled: false, blocks: [], skipDay: null },
  };
  const now = () => Date.now();
  const sod = () => { const d = new Date(); return d.getHours() * 3600 + d.getMinutes() * 60 + d.getSeconds(); };
  const dayNo = (d) => Math.floor((d.getTime() - d.getTimezoneOffset() * 60000) / 86400000);
  const weekday = (d) => (d.getDay() + 6) % 7;
  const rand = (a, b) => a + Math.random() * (b - a);

  const eng = { phase: 'idle', mode: 'wiggle', actions: 0, activeMs: 0, nextAt: 0, deadline: null, shifts: false, last: now(), pausedAt: 0, userAt: 0, lastStop: null };
  let history = { sessions: [] };
  let sessionStart = null;

  // deterministic "random" so the week view does not flicker
  function seeded(n) { const x = Math.sin(n * 9301 + 49297) * 233280; return x - Math.floor(x); }
  function window_(j, minute) {
    const b = minute * 60, k = j.minutes * 60;
    return j.kind === 'exact' ? [b, b] : j.kind === 'before' ? [b - k, b] : j.kind === 'after' ? [b, b + k] : [b - k, b + k];
  }
  function resolve(j, minute, seed) { const [lo, hi] = window_(j, minute); return Math.round(lo + (hi - lo) * seeded(seed)); }
  function planFor(day, wd) {
    const out = [];
    settings.schedule.blocks.forEach((b, i) => {
      if (!b.days[wd]) return;
      const s = resolve(b.startJitter, b.startMin, day * 7 + i), e = resolve(b.endJitter, b.endMin, day * 11 + i);
      const lo = window_(b.startJitter, b.startMin)[0], hi = window_(b.endJitter, b.endMin)[1];
      if (b.break) {
        const ps = resolve(b.break.startJitter, b.break.startMin, day * 13 + i), pe = resolve(b.break.endJitter, b.break.endMin, day * 17 + i);
        out.push({ startS: s, endS: ps, earliestS: lo, latestS: window_(b.break.startJitter, b.break.startMin)[1], mode: b.mode });
        out.push({ startS: pe, endS: e, earliestS: window_(b.break.endJitter, b.break.endMin)[0], latestS: hi, mode: b.mode });
      } else out.push({ startS: s, endS: e, earliestS: lo, latestS: hi, mode: b.mode });
    });
    return out.sort((a, b) => a.startS - b.startS);
  }

  function status() {
    const t = now(), active = eng.phase !== 'idle';
    let remaining = eng.deadline != null && active ? Math.max(0, eng.deadline + (eng.phase === 'paused' && eng.shifts ? t - eng.pausedAt : 0) - t) : null;
    const today = settings.schedule.enabled ? planFor(dayNo(new Date()), weekday(new Date())) : [];
    const s = sod(), skipped = settings.schedule.skipDay === dayNo(new Date());
    const seg = today.find((x) => x.startS <= s && s < x.endS), next = today.find((x) => x.startS > s);
    const state = !settings.schedule.enabled ? 'off' : skipped ? 'skipped' : (seg || next) ? 'waiting' : 'done';
    return {
      phase: eng.phase, mode: active ? eng.mode : settings.mode, origin: 'manual', actions: eng.actions, activeMs: eng.activeMs,
      nextInMs: eng.phase === 'running' ? Math.max(0, eng.nextAt - t) : null, remainingMs: remaining,
      stopsAtS: remaining != null ? Math.round(s + remaining / 1000) % 86400 : null,
      actionsLeft: active && settings.stop.kind === 'afterActions' ? Math.max(0, settings.stop.actions - eng.actions) : null,
      schedule: { state, nextStartS: next ? next.startS : null, nextEndS: (seg || next) ? (seg || next).endS : null, today },
      lastStop: eng.lastStop,
    };
  }
  const push = () => emit('status', status());

  function stop(reason) {
    if (eng.phase === 'idle') return;
    eng.phase = 'idle';
    eng.lastStop = { reason, message: null, actions: eng.actions, activeMs: eng.activeMs };
    history.sessions.push({ day: dayNo(new Date()), startS: sessionStart, endS: sod(), mode: eng.mode, actions: eng.actions, bySchedule: false, reason, userPauseS: 0, sleepS: 0 });
    push();
  }
  function start() {
    if (settings.mode === 'click' && !settings.areas.length) throw 'no-target';
    const t = now();
    Object.assign(eng, { phase: 'running', mode: settings.mode, actions: 0, activeMs: 0, last: t, nextAt: t + rand(2000, 4000), deadline: null, shifts: false, lastStop: null });
    if (settings.stop.kind === 'afterMinutes') { eng.deadline = t + settings.stop.minutes * 60000; eng.shifts = true; }
    if (settings.stop.kind === 'atTime') { let d = settings.stop.atMinute * 60 - sod(); if (d <= 0) d += 86400; eng.deadline = t + d * 1000; }
    sessionStart = sod();
    push();
  }
  setInterval(() => {
    const t = now(), gap = t - eng.last; eng.last = t;
    if (eng.phase === 'running') {
      eng.activeMs += gap;
      if (eng.deadline != null && t >= eng.deadline) return stop(eng.shifts ? 'timeLimit' : 'clockTime');
      if (settings.stop.kind === 'afterActions' && eng.actions >= settings.stop.actions) return stop('actionLimit');
      if (t >= eng.nextAt) {
        eng.actions += 1;
        const lo = eng.mode === 'click' ? settings.clickMinS : settings.wiggleMinS, hi = eng.mode === 'click' ? settings.clickMaxS : settings.wiggleMaxS;
        eng.nextAt = t + rand(lo, hi) * 1000;
      }
    } else if (eng.phase === 'userActive' && t - eng.userAt >= settings.resumeAfterS * 1000) {
      eng.phase = 'running'; eng.nextAt = t + rand(1500, 3500);
    }
    push();
  }, 250);

  function sanitize(s) {
    const fix = (a, b) => { let lo = Math.min(3600, Math.max(1, +s[a] || 1)), hi = Math.min(3600, Math.max(1, +s[b] || lo)); if (hi < lo) [lo, hi] = [hi, lo]; s[a] = lo; s[b] = hi; };
    fix('clickMinS', 'clickMaxS'); fix('wiggleMinS', 'wiggleMaxS');
    s.resumeAfterS = Math.min(600, Math.max(1, +s.resumeAfterS || 5));
    s.stop.actions = Math.max(1, +s.stop.actions || 1); s.stop.minutes = Math.min(1440, Math.max(1, +s.stop.minutes || 1));
    s.schedule.blocks.forEach((b) => {
      if (b.endMin <= b.startMin) b.endMin = Math.min(1440, b.startMin + 1);
      if (b.break && (b.break.endMin <= b.break.startMin || b.break.startMin <= b.startMin || b.break.endMin >= b.endMin)) b.break = null;
    });
    return s;
  }

  function pickArea(index) {
    return new Promise((resolve) => {
      const layer = document.createElement('div');
      layer.className = 'overlay overlay-inline';
      layer.innerHTML = '<p class="overlay-help">' + I18N.t('overlay.help') + '</p>';
      document.body.appendChild(layer);
      window.startDraw(layer, (r) => {
        layer.remove();
        if (r) {
          const area = { x: Math.round(r.x) + 600, y: Math.round(r.y) + 200, w: Math.round(r.w), h: Math.round(r.h) };
          if (index != null && index < settings.areas.length) settings.areas[index] = area; else if (settings.areas.length < 5) settings.areas.push(area);
          emit('settings', settings);
        }
        resolve();
      });
    });
  }

  let pointer = { x: 800, y: 450 };
  window.addEventListener('mousemove', (e) => { pointer = { x: e.screenX, y: e.screenY }; });

  const commands = {
    get_state: () => ({
      info: { version: '0.1.0', os: 'preview', autostart: 'switch', donate: true, permission: true, links: { source: 'https://', issues: 'https://', privacy: 'https://', donate: 'https://' } },
      settings, status: status(),
    }),
    save_settings: ({ settings: s }) => { settings = sanitize(JSON.parse(JSON.stringify(s))); emit('settings', settings); push(); return settings; },
    control: ({ action }) => {
      const t = now();
      if (action === 'toggle') action = eng.phase === 'idle' ? 'start' : 'stop';
      if (action === 'start') start();
      else if (action === 'stop') stop('manual');
      else if (action === 'pause' && eng.phase !== 'idle') { eng.phase = 'paused'; eng.pausedAt = t; }
      else if (action === 'resume' && eng.phase === 'paused') { const p = t - eng.pausedAt; if (eng.shifts && eng.deadline) eng.deadline += p; eng.nextAt += p; eng.phase = 'running'; }
      else if (action === 'resume' && eng.phase === 'userActive') { eng.phase = 'running'; eng.nextAt = t + 2000; }
      push();
    },
    week: () => {
      const d = new Date(), today = dayNo(d), monday = today - weekday(d);
      return [0, 1, 2, 3, 4, 5, 6].map((wd) => ({ day: monday + wd, weekday: wd, today: monday + wd === today, segments: planFor(monday + wd, wd) }));
    },
    history: () => history,
    clear_history: () => { history = { sessions: [] }; },
    permission: () => true,
    capture_cursor: () => pointer,
    preview_area: () => {},
    pick_area: ({ index }) => pickArea(index),
    set_compact: ({ compact }) => { document.body.classList.toggle('compact-preview', !!compact); },
    set_tray_labels: () => {},
    open_link: () => {},
    open_startup_settings: () => {},
    quit: () => {},
  };

  window.API = {
    preview: true,
    invoke: (cmd, args) => new Promise((resolve, reject) => {
      try { resolve(JSON.parse(JSON.stringify(commands[cmd](args || {}) ?? null))); } catch (e) { reject(e); }
    }),
    on: (name, fn) => { (listeners[name] = listeners[name] || []).push(fn); },
    /** Preview only: behave as if the user had just moved the mouse. */
    touch: () => { if (eng.phase === 'running' && settings.pauseOnUser) { eng.phase = 'userActive'; eng.userAt = now(); push(); } else if (eng.phase === 'userActive') eng.userAt = now(); },
  };
})();
