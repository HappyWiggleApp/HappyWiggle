// The main window. Reads settings and live status from the native side,
// draws them, and sends back whatever the user changes.
(async function () {
  const { t, clock, span, about, num } = I18N;
  const $ = (id) => document.getElementById(id);
  const invoke = API.invoke;

  let S = null;      // settings
  let ST = null;     // live status
  let INFO = null;   // facts about this build
  let noticeKey = null;

  // ------------------------------------------------------------------ helpers
  const hhmm = (min) => String(Math.floor(min / 60) % 24).padStart(2, '0') + ':' + String(min % 60).padStart(2, '0');
  const minutes = (value) => { const [h, m] = (value || '0:0').split(':').map(Number); return (h || 0) * 60 + (m || 0); };
  const active = () => ST && ST.phase !== 'idle';

  function prettyKey(combo) {
    const mac = INFO && INFO.os === 'macos';
    return combo.split('+').map((p) => {
      if (p === 'Super') return mac ? 'Cmd' : 'Win';
      if (p === 'Alt' && mac) return 'Option';
      return p.replace(/^Key/, '').replace(/^Digit/, '');
    }).join(' + ');
  }

  function notice(key, vars) {
    noticeKey = key;
    $('notice-text').textContent = t(key, vars);
    $('notice').hidden = false;
  }
  function problem(code) {
    const text = String(code || '');
    const key = 'err.' + text.split(':')[0];
    if (I18N.keys.en[key]) notice(key); else notice('err.generic', { msg: text });
  }
  $('notice-close').onclick = () => { $('notice').hidden = true; noticeKey = null; };

  let saveTimer = null;
  async function doSave() {
    saveTimer = null;
    try { S = await invoke('save_settings', { settings: S }); renderForm(); renderSchedule(); }
    catch (e) { problem(e); const snap = await invoke('get_state'); S = snap.settings; renderForm(); }
  }
  /** Send the settings to the native side, shortly after the last change. */
  function save(now) {
    clearTimeout(saveTimer);
    if (now) return doSave();
    saveTimer = setTimeout(doSave, 300);
  }
  /** Make sure a change typed a moment ago has reached the engine. */
  async function flush() {
    if (saveTimer) { clearTimeout(saveTimer); await doSave(); }
  }

  async function control(action) {
    try {
      await flush();
      await invoke('control', { action });
      $('notice').hidden = true;
      if (action === 'start' && S.compactOnStart) invoke('set_compact', { compact: true });
    } catch (e) { problem(e); refreshPermission(); }
  }

  // ------------------------------------------------------------------ tabs
  document.querySelectorAll('.tabs button').forEach((b) => {
    b.onclick = () => {
      document.querySelectorAll('.tabs button').forEach((x) => x.setAttribute('aria-selected', String(x === b)));
      document.querySelectorAll('.tab').forEach((s) => { s.hidden = s.id !== 'tab-' + b.dataset.tab; });
      if (b.dataset.tab === 'schedule') renderSchedule();
      if (b.dataset.tab === 'settings') refreshPermission();
    };
  });

  // ------------------------------------------------------------------ stage
  const pathEl = $('trail-path'), pointerEl = $('trail-pointer');
  const pathLen = pathEl.getTotalLength();
  const calm = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  let lastActions = 0, anim = null;

  function placePointer(f) {
    const p = pathEl.getPointAtLength(pathLen * f);
    pointerEl.setAttribute('transform', 'translate(' + (p.x - 2).toFixed(1) + ' ' + (p.y - 2).toFixed(1) + ')');
  }
  /** One glide along the curve and back: played each time the engine acts. */
  function playWiggle() {
    if (calm || anim) return;
    const t0 = performance.now(), dur = 1500;
    const ease = (x) => x * x * (3 - 2 * x);
    const step = (now) => {
      const k = Math.min(1, (now - t0) / dur);
      placePointer(0.12 + 0.76 * ease(k < 0.5 ? k * 2 : (1 - k) * 2));
      if (k < 1) anim = requestAnimationFrame(step); else { anim = null; }
    };
    anim = requestAnimationFrame(step);
  }
  placePointer(0.12);

  function renderStatus() {
    if (!ST || !S) return;
    const s = ST, on = active();
    const stage = $('stage');
    stage.dataset.phase = s.phase;
    stage.dataset.mode = s.mode;
    const countWord = t(s.mode === 'click' ? 'run.clicks' : 'run.wiggles');

    let state = '', big = '', sub = '';
    if (!on) {
      state = t('state.off');
      if (s.lastStop && s.lastStop.reason !== 'error') {
        sub = t('stopped.' + s.lastStop.reason) + ' ' + t('stopped.summary', { label: countWord, n: num(s.lastStop.actions), time: span(s.lastStop.activeMs) });
      } else if (s.schedule.state === 'waiting' && s.schedule.nextStartS != null) {
        sub = t('state.off.next', { time: clock(s.schedule.nextStartS) });
      } else {
        sub = t('state.off.hint', { key: prettyKey(S.hotkey) });
      }
    } else {
      const timed = s.remainingMs != null;
      big = timed ? t('state.left', { time: span(s.remainingMs) }) : span(s.activeMs);
      if (s.phase === 'userActive') { state = t('state.user'); sub = t('state.user.hint', { n: S.resumeAfterS }); }
      else if (s.phase === 'paused') { state = t('state.paused'); sub = timed && s.origin === 'manual' && S.stop.kind === 'afterMinutes' ? t('state.paused.hint') : ''; }
      else {
        state = t(s.mode === 'click' ? 'state.clicking' : 'state.wiggling');
        sub = s.stopsAtS != null ? t('state.stops', { time: clock(s.stopsAtS) })
          : s.actionsLeft != null ? t('state.leftcount', { n: num(s.actionsLeft) }) : t('state.forever');
      }
    }
    $('st-state').textContent = state;
    $('st-clock').textContent = big;
    $('st-clock').hidden = !on;
    $('st-sub').textContent = sub;
    $('st-stats').hidden = !on;
    $('st-count-label').textContent = countWord;
    $('st-count').textContent = num(s.actions);
    $('st-next').textContent = s.nextInMs != null ? Math.ceil(s.nextInMs / 1000) + ' s' : '–';
    $('st-elapsed').textContent = span(s.activeMs);

    if (on && s.actions > lastActions) playWiggle();
    lastActions = on ? s.actions : 0;

    // a stop caused by a problem is explained once, in the notice
    if (!on && s.lastStop && s.lastStop.reason === 'error' && s.lastStop.message && noticeKey !== 'shown:' + s.lastStop.message) {
      problem(s.lastStop.message);
      noticeKey = 'shown:' + s.lastStop.message;
    }

    $('btn-start').hidden = on;
    $('btn-stop').hidden = !on;
    $('btn-pause').hidden = !on;
    $('btn-compact').hidden = !on;
    const resume = s.phase === 'paused' || s.phase === 'userActive';
    $('btn-pause').textContent = t(resume ? 'run.resume' : 'run.pause');
    $('btn-pause').dataset.action = resume ? 'resume' : 'pause';
    document.body.classList.toggle('session', on);
    document.querySelectorAll('[data-lock]').forEach((el) => { el.disabled = on; });
    renderScheduleStatus();
  }

  $('btn-start').onclick = () => control('start');
  $('btn-stop').onclick = () => control('stop');
  $('btn-pause').onclick = () => control($('btn-pause').dataset.action);
  $('btn-compact').onclick = () => invoke('set_compact', { compact: true });

  // ------------------------------------------------------------------ run form
  /** Wire an input to a settings field. `get`/`set` convert between the two. */
  function bind(id, get, set, lock) {
    const el = $(id);
    if (lock) el.dataset.lock = '1';
    el._fill = () => { if (document.activeElement !== el || el.type === 'checkbox' || el.tagName === 'SELECT') { const v = get(); if (el.type === 'checkbox') el.checked = !!v; else el.value = v; } };
    el.addEventListener(el.type === 'checkbox' || el.tagName === 'SELECT' || el.type === 'range' ? 'input' : 'change', () => {
      set(el.type === 'checkbox' ? el.checked : el.type === 'number' || el.type === 'range' ? Number(el.value) : el.value);
      renderForm();
      save();
    });
    bound.push(el);
  }
  const bound = [];

  bind('f-wiggle-style', () => S.wiggleStyle, (v) => { S.wiggleStyle = v; });
  bind('f-wiggle-min', () => S.wiggleMinS, (v) => { S.wiggleMinS = v; });
  bind('f-wiggle-max', () => S.wiggleMaxS, (v) => { S.wiggleMaxS = v; });
  bind('f-wiggle-distance', () => S.wiggleDistance, (v) => { S.wiggleDistance = v; });
  bind('f-click-min', () => S.clickMinS, (v) => { S.clickMinS = v; });
  bind('f-click-max', () => S.clickMaxS, (v) => { S.clickMaxS = v; });
  bind('f-order', () => S.areaOrder, (v) => { S.areaOrder = v; });
  bind('f-button', () => S.button, (v) => { S.button = v; });
  bind('f-smooth', () => S.smoothMove, (v) => { S.smoothMove = v; });
  bind('f-return', () => S.returnCursor, (v) => { S.returnCursor = v; });
  bind('f-stop-kind', () => S.stop.kind, (v) => { S.stop.kind = v; }, true);
  bind('f-stop-actions', () => S.stop.actions, (v) => { S.stop.actions = v; }, true);
  bind('f-stop-minutes', () => S.stop.minutes, (v) => { S.stop.minutes = v; }, true);
  bind('f-stop-time', () => hhmm(S.stop.atMinute), (v) => { S.stop.atMinute = minutes(v); }, true);
  bind('f-pause-user', () => S.pauseOnUser, (v) => { S.pauseOnUser = v; });
  bind('f-resume', () => S.resumeAfterS, (v) => { S.resumeAfterS = v; });
  bind('f-awake', () => S.keepAwake, (v) => { S.keepAwake = v; });
  bind('f-theme', () => S.theme, (v) => { S.theme = v; applyTheme(); });
  bind('f-language', () => S.language, (v) => { S.language = v; applyLanguage(); });
  bind('f-compact', () => S.compactOnStart, (v) => { S.compactOnStart = v; });
  bind('f-autostart', () => S.autostart, (v) => { S.autostart = v; });
  bind('f-hidden', () => S.startHidden, (v) => { S.startHidden = v; });

  document.querySelectorAll('.segmented button').forEach((b) => {
    b.dataset.lock = '1';
    b.onclick = () => { S.mode = b.dataset.mode; renderForm(); save(); };
  });

  function renderForm() {
    if (!S) return;
    bound.forEach((el) => el._fill());
    document.querySelectorAll('.segmented button').forEach((b) => b.setAttribute('aria-checked', String(b.dataset.mode === S.mode)));
    $('opts-wiggle').hidden = S.mode !== 'wiggle';
    $('opts-click').hidden = S.mode !== 'click';
    $('row-distance').hidden = S.wiggleStyle !== 'natural';
    $('o-wiggle-distance').textContent = t('unit.px', { n: S.wiggleDistance });
    $('row-order').hidden = S.areas.length < 2;
    $('row-resume').hidden = !S.pauseOnUser;
    $('opt-after-actions').textContent = t(S.mode === 'click' ? 'stop.clicks' : 'stop.wiggles');
    $('f-stop-actions').hidden = S.stop.kind !== 'afterActions';
    $('f-stop-minutes').hidden = S.stop.kind !== 'afterMinutes';
    $('f-stop-time').hidden = S.stop.kind !== 'atTime';
    $('hotkey-hint').textContent = prettyKey(S.hotkey);
    if (!listening) $('btn-hotkey').textContent = prettyKey(S.hotkey);
    renderTargets();
    document.querySelectorAll('[data-lock]').forEach((el) => { el.disabled = active(); });
  }

  // ------------------------------------------------------------------ click targets
  function renderTargets() {
    const list = $('targets');
    list.textContent = '';
    $('targets-empty').hidden = S.areas.length > 0;
    S.areas.forEach((a, i) => {
      const spot = a.w <= 1 && a.h <= 1;
      const li = document.createElement('li');
      const label = document.createElement('span');
      const name = document.createElement('b');
      name.textContent = t(spot ? 'click.spot' : 'click.area', { n: i + 1 });
      const where = document.createElement('small');
      where.textContent = spot ? t('click.at', a) : t('click.size', a);
      label.append(name, where);
      li.append(label);
      [['click.show', () => invoke('preview_area', { index: i }).catch(problem)],
       ['click.redraw', () => draw(i)],
       ['common.remove', () => { S.areas.splice(i, 1); renderForm(); save(true); }]].forEach(([key, fn]) => {
        const b = document.createElement('button');
        b.type = 'button'; b.className = 'link'; b.textContent = t(key); b.onclick = fn;
        if (key !== 'click.show') b.dataset.lock = '1';
        li.append(b);
      });
      list.append(li);
    });
    $('btn-draw').dataset.lock = '1';
    $('btn-capture').dataset.lock = '1';
  }

  async function draw(index) {
    if (index == null && S.areas.length >= 5) return notice('err.targets-full');
    try { await invoke('pick_area', { index: index == null ? null : index }); } catch (e) { problem(e); }
  }
  $('btn-draw').onclick = () => draw(null);

  // "Capture with the pointer": works on every screen layout, no overlay needed.
  let captureTimer = null;
  function countdown(text) {
    return new Promise((resolve, reject) => {
      let n = 3;
      $('capture-text').textContent = text;
      $('capture-count').textContent = n;
      $('capture').hidden = false;
      $('capture-cancel').onclick = () => { clearInterval(captureTimer); $('capture').hidden = true; reject('cancel'); };
      captureTimer = setInterval(async () => {
        n -= 1;
        if (n > 0) { $('capture-count').textContent = n; return; }
        clearInterval(captureTimer);
        try { resolve(await invoke('capture_cursor')); } catch (e) { reject(e); }
      }, 1000);
    });
  }
  async function capture(kind) {
    $('capture-choice').hidden = true;
    try {
      let area;
      if (kind === 'spot') {
        const p = await countdown(t('capture.spot.text'));
        area = { x: p.x, y: p.y, w: 1, h: 1 };
      } else {
        const a = await countdown(t('capture.corner1'));
        const b = await countdown(t('capture.corner2'));
        area = { x: Math.min(a.x, b.x), y: Math.min(a.y, b.y), w: Math.max(1, Math.abs(b.x - a.x)), h: Math.max(1, Math.abs(b.y - a.y)) };
      }
      S.areas.push(area);
      renderForm();
      save(true);
    } catch (e) { if (e !== 'cancel') problem(e); }
    $('capture').hidden = true;
  }
  $('btn-capture').onclick = () => { if (S.areas.length >= 5) return notice('err.targets-full'); $('capture-choice').hidden = false; };
  $('capture-spot').onclick = () => capture('spot');
  $('capture-area').onclick = () => capture('area');
  $('capture-choice-cancel').onclick = () => { $('capture-choice').hidden = true; };

  // ------------------------------------------------------------------ schedule
  const JITTERS = ['exact'].concat(...['before', 'after', 'either'].map((k) => [5, 10, 15, 20, 30].map((n) => k + ':' + n)));
  const jitValue = (j) => (j.kind === 'exact' || !j.minutes ? 'exact' : j.kind + ':' + j.minutes);
  const jitParse = (v) => { const [kind, n] = v.split(':'); return { kind, minutes: Number(n) || 0 }; };
  const newBlock = () => ({
    days: [true, true, true, true, true, false, false], startMin: 540, endMin: 1020,
    startJitter: { kind: 'before', minutes: 10 }, endJitter: { kind: 'after', minutes: 10 },
    break: { startMin: 780, endMin: 840, startJitter: { kind: 'either', minutes: 5 }, endJitter: { kind: 'either', minutes: 5 } },
    mode: null,
  });

  function el(tag, props, ...kids) {
    const n = document.createElement(tag);
    Object.assign(n, props || {});
    n.append(...kids);
    return n;
  }
  function jitterSelect(j, onChange) {
    const sel = el('select');
    JITTERS.forEach((v) => {
      const p = jitParse(v);
      sel.append(el('option', { value: v, textContent: t('jit.' + p.kind, { n: p.minutes }) }));
    });
    if (!JITTERS.includes(jitValue(j))) sel.append(el('option', { value: jitValue(j), textContent: t('jit.' + j.kind, { n: j.minutes }) }));
    sel.value = jitValue(j);
    sel.onchange = () => onChange(jitParse(sel.value));
    return sel;
  }
  function timeInput(min, onChange) {
    const i = el('input', { type: 'time', value: hhmm(min) });
    i.onchange = () => onChange(minutes(i.value));
    return i;
  }

  function renderBlocks() {
    const host = $('blocks');
    if (host.contains(document.activeElement) && document.activeElement.type === 'time') return; // do not yank a field being edited
    host.textContent = '';
    const changed = () => { save(true); };
    S.schedule.blocks.forEach((b, i) => {
      const days = el('div', { className: 'days' });
      b.days.forEach((on, d) => {
        const chip = el('button', { type: 'button', textContent: t('day.' + d) });
        chip.setAttribute('aria-pressed', String(on));
        chip.onclick = () => { b.days[d] = !b.days[d]; changed(); };
        days.append(chip);
      });
      const breakOn = el('input', { type: 'checkbox', checked: !!b.break });
      breakOn.onchange = () => {
        const mid = Math.round((b.startMin + b.endMin) / 2 / 30) * 30;
        b.break = breakOn.checked ? { startMin: mid - 30, endMin: mid + 30, startJitter: { kind: 'either', minutes: 5 }, endJitter: { kind: 'either', minutes: 5 } } : null;
        changed();
      };
      const mode = el('select');
      [['', 'sched.mode.same'], ['wiggle', 'mode.wiggle'], ['click', 'mode.click']].forEach(([v, k]) => mode.append(el('option', { value: v, textContent: t(k) })));
      mode.value = b.mode || '';
      mode.onchange = () => { b.mode = mode.value || null; changed(); };
      const remove = el('button', { type: 'button', className: 'link', textContent: t('common.remove') });
      remove.onclick = () => { S.schedule.blocks.splice(i, 1); if (!S.schedule.blocks.length) S.schedule.enabled = false; changed(); };

      const card = el('div', { className: 'block' },
        el('div', { className: 'block-head' }, el('strong', { textContent: t('sched.block', { n: i + 1 }) }), remove),
        days,
        el('div', { className: 'row' }, el('span', { textContent: t('sched.start') }),
          el('span', { className: 'range' }, timeInput(b.startMin, (v) => { b.startMin = v; changed(); }), jitterSelect(b.startJitter, (j) => { b.startJitter = j; changed(); }))),
        el('div', { className: 'row' }, el('span', { textContent: t('sched.end') }),
          el('span', { className: 'range' }, timeInput(b.endMin, (v) => { b.endMin = v; changed(); }), jitterSelect(b.endJitter, (j) => { b.endJitter = j; changed(); }))),
        el('label', { className: 'row switch' }, el('span', {}, el('b', { textContent: t('sched.break') })), breakOn));
      if (b.break) {
        card.append(el('div', { className: 'row' }, el('span', { textContent: t('sched.break.from') }),
          el('span', { className: 'range' },
            timeInput(b.break.startMin, (v) => { b.break.startMin = v; changed(); }),
            el('i', { textContent: t('sched.break.to').toLowerCase() }),
            timeInput(b.break.endMin, (v) => { b.break.endMin = v; changed(); }))));
      }
      card.append(el('label', { className: 'row' }, el('span', { textContent: t('sched.mode') }), mode));
      host.append(card);
    });
  }

  async function renderWeek() {
    const days = await invoke('week');
    const host = $('week');
    let lo = 8 * 3600, hi = 18 * 3600;
    days.forEach((d) => d.segments.forEach((s) => { lo = Math.min(lo, s.earliestS); hi = Math.max(hi, s.latestS); }));
    lo = Math.max(0, Math.floor(lo / 3600 - 1) * 3600);
    hi = Math.min(86400, Math.ceil(hi / 3600 + 1) * 3600);
    const pct = (s) => ((s - lo) / (hi - lo) * 100).toFixed(2) + '%';
    const width = (a, b) => ((b - a) / (hi - lo) * 100).toFixed(2) + '%';
    host.textContent = '';
    const axis = el('div', { className: 'axis' });
    const step = (hi - lo) / 3600 > 12 ? 4 : 2;
    for (let h = lo / 3600; h <= hi / 3600; h += step) {
      const tick = el('span', { textContent: clock(h * 3600 % 86400) });
      tick.style.left = pct(h * 3600);
      axis.append(tick);
    }
    host.append(el('div', { className: 'week-row' }, el('span'), axis));
    const d = new Date(), nowS = d.getHours() * 3600 + d.getMinutes() * 60;
    days.forEach((day) => {
      const track = el('div', { className: 'track' });
      day.segments.forEach((s) => {
        const drift = el('i', { className: 'drift' });
        drift.style.left = pct(s.earliestS); drift.style.width = width(s.earliestS, s.latestS);
        const plan = el('i', { className: 'plan' });
        plan.style.left = pct(s.startS); plan.style.width = width(s.startS, s.endS);
        plan.title = clock(s.startS) + ' – ' + clock(s.endS);
        track.append(drift, plan);
      });
      if (day.today && nowS > lo && nowS < hi) { const n = el('i', { className: 'now' }); n.style.left = pct(nowS); track.append(n); }
      const row = el('div', { className: 'week-row' + (day.today ? ' today' : '') }, el('span', { textContent: t('day.' + day.weekday) }), track);
      if (day.today && S.schedule.skipDay === day.day) row.classList.add('skipped');
      host.append(row);
    });
    todayNo = (days.find((x) => x.today) || {}).day;
    $('btn-skip').textContent = t(S.schedule.skipDay === todayNo ? 'sched.unskip' : 'sched.skip');
  }
  let todayNo = null;

  function renderScheduleStatus() {
    if (!ST || !S) return;
    const sc = ST.schedule;
    const vars = { start: sc.nextStartS != null ? clock(sc.nextStartS) : '', end: sc.nextEndS != null ? clock(sc.nextEndS % 86400) : '' };
    let key = 'sched.' + sc.state;
    // "waiting" while already inside a block (for example a hand-started session is running)
    if (sc.state === 'waiting' && sc.nextStartS == null) key = 'sched.done';
    $('sched-status').textContent = t(key, vars);
    $('sched-status').dataset.state = sc.state;
  }

  async function renderHistory() {
    const h = await invoke('history');
    const list = $('history');
    list.textContent = '';
    const items = h.sessions.slice(-40).reverse();
    $('hist-empty').hidden = items.length > 0;
    $('btn-clear-history').hidden = items.length === 0;
    items.forEach((r) => {
      const when = todayNo != null && r.day === todayNo ? t('hist.today') : todayNo != null && r.day === todayNo - 1 ? t('hist.yesterday')
        : new Intl.DateTimeFormat(I18N.lang, { weekday: 'short', day: 'numeric', month: 'short' }).format(new Date(r.day * 86400000));
      const count = t(r.mode === 'click' ? 'run.clicks' : 'run.wiggles') + ': ' + num(r.actions);
      const notes = [t(r.bySchedule ? 'hist.bytimetable' : 'hist.byhand'), count, t('hist.reason.' + r.reason)];
      if (r.userPauseS >= 60) notes.push(t('hist.userpause', { time: about(r.userPauseS) }));
      if (r.sleepS >= 60) notes.push(t('hist.sleep', { time: about(r.sleepS) }));
      list.append(el('li', {},
        el('b', { textContent: when }),
        el('span', { textContent: t('hist.line', { start: clock(r.startS), end: clock(r.endS) }) }),
        el('small', { textContent: notes.join(', ') })));
    });
  }

  function renderSchedule() {
    if (!S || $('tab-schedule').hidden) return;
    const has = S.schedule.blocks.length > 0;
    $('sched-empty').hidden = has;
    $('sched-body').hidden = !has;
    if (!has) return;
    $('f-sched-on').checked = S.schedule.enabled;
    renderBlocks();
    renderScheduleStatus();
    renderWeek().then(renderHistory);
  }

  $('btn-preset').onclick = () => { S.schedule.blocks = [newBlock()]; S.schedule.enabled = true; renderSchedule(); save(true); };
  $('btn-add-block').onclick = () => {
    if (S.schedule.blocks.length >= 6) return;
    const b = newBlock(); b.break = null; b.days = [false, false, false, false, false, true, false]; b.startMin = 600; b.endMin = 780;
    S.schedule.blocks.push(b); save(true);
  };
  $('f-sched-on').onchange = () => { S.schedule.enabled = $('f-sched-on').checked; save(true); };
  $('btn-skip').onclick = () => { S.schedule.skipDay = S.schedule.skipDay === todayNo ? null : todayNo; save(true); };
  $('btn-clear-history').onclick = async () => { await invoke('clear_history'); renderHistory(); };

  // ------------------------------------------------------------------ settings tab
  let listening = false;
  function hotkeyDown(e) {
    e.preventDefault();
    if (e.key === 'Escape') return stopListening();
    if (['Control', 'Shift', 'Alt', 'Meta'].includes(e.key)) return;
    const mods = [];
    if (e.ctrlKey) mods.push('Ctrl');
    if (e.altKey) mods.push('Alt');
    if (e.shiftKey) mods.push('Shift');
    if (e.metaKey) mods.push('Super');
    const fkey = /^F\d{1,2}$/.test(e.code);
    const strong = e.ctrlKey || e.altKey || e.metaKey;
    if (!fkey && !(strong && /^(Key[A-Z]|Digit\d|Period|Comma|Slash|Semicolon|Space)$/.test(e.code))) {
      stopListening();
      return notice('err.hotkey.combo');
    }
    S.hotkey = mods.concat(e.code).join('+');
    stopListening();
    save(true);
  }
  function stopListening() { listening = false; window.removeEventListener('keydown', hotkeyDown, true); $('btn-hotkey').classList.remove('listening'); renderForm(); }
  $('btn-hotkey').onclick = () => {
    if (listening) return stopListening();
    listening = true;
    $('btn-hotkey').textContent = t('set.hotkey.press');
    $('btn-hotkey').classList.add('listening');
    window.addEventListener('keydown', hotkeyDown, true);
  };

  async function refreshPermission() {
    if (!INFO || INFO.os !== 'macos') return;
    const ok = await invoke('permission', { request: false });
    $('row-permission').hidden = false;
    $('permission-state').textContent = t(ok ? 'set.permission.ok' : 'set.permission.missing');
    $('btn-permission').hidden = ok;
  }
  $('btn-permission').onclick = async () => { await invoke('permission', { request: true }); notice('err.permission'); refreshPermission(); };
  $('btn-quit').onclick = () => invoke('quit');
  document.querySelectorAll('[data-link]').forEach((b) => { b.onclick = () => invoke('open_link', { which: b.dataset.link }).catch(problem); });

  function applyTheme() { document.documentElement.dataset.theme = S.theme || 'system'; }
  function applyLanguage() {
    I18N.setLanguage(S.language);
    $('version').textContent = t('about.version', { v: INFO.version });
    invoke('set_tray_labels', { labels: {
      start: t('run.start'), stop: t('run.stop'), pause: t('run.pause'), resume: t('run.resume'),
      show: t('tray.show'), quit: t('tray.quit'), idle: t('tray.idle'), running: t('tray.running'), paused: t('tray.paused'), waiting: t('tray.waiting'),
    } });
    renderForm(); renderStatus(); renderSchedule();
  }

  // ------------------------------------------------------------------ start-up
  API.on('status', (s) => { ST = s; renderStatus(); });
  API.on('settings', (s) => {
    const lang = S && S.language, theme = S && S.theme;
    S = s;
    if (lang !== s.language) applyLanguage();
    if (theme !== s.theme) applyTheme();
    renderForm(); renderSchedule();
  });
  API.on('problem', problem);
  API.on('hidden-to-tray', async () => {
    if (S && !S.onboarded) { S.onboarded = true; save(true); notice('note.tray'); }
  });
  API.on('session-started', () => { if (S && S.compactOnStart) invoke('set_compact', { compact: true }); });

  const snap = await invoke('get_state');
  S = snap.settings; ST = snap.status; INFO = snap.info;
  $('row-autostart').hidden = !INFO.autostart;
  document.querySelectorAll('[data-link]').forEach((b) => {
    const url = INFO.links[b.dataset.link];
    b.hidden = !url || (b.dataset.link === 'donate' && !INFO.donate);
  });
  applyTheme();
  applyLanguage();
  setInterval(() => { if (!$('tab-schedule').hidden && S.schedule.blocks.length) renderWeek(); }, 60000);

  if (API.preview) {
    $('preview-note').hidden = false;
    $('preview-touch').onclick = () => API.touch();
    const bar = document.createElement('div');
    bar.id = 'bar';
    document.body.append(bar);
    window.initBar(bar);
  }
})();
