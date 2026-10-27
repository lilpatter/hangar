'use strict';

const T = window.__TAURI__;
const invoke = (cmd, args) => T.core.invoke(cmd, args);
const appWindow = T.window.getCurrentWindow();

// SF Symbols-style line icons (24pt grid).
const ICONS = {
  search: '<circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/>',
  home: '<path d="M3 10.5 12 3l9 7.5"/><path d="M5 9.5V20a1 1 0 0 0 1 1h4v-6h4v6h4a1 1 0 0 0 1-1V9.5"/>',
  gamepad: '<path d="M6 11h4M8 9v4"/><path d="M15 12h.01M18 10h.01"/><path d="M17.3 5H6.7a4 4 0 0 0-3.96 3.43l-.9 6.33A2.5 2.5 0 0 0 6.1 16.9L8 15h8l1.9 1.9a2.5 2.5 0 0 0 4.26-2.14l-.9-6.33A4 4 0 0 0 17.3 5z"/>',
  drive: '<rect x="3" y="13" width="18" height="7" rx="2"/><path d="M5.5 13 7.8 5.6A1 1 0 0 1 8.76 5h6.48a1 1 0 0 1 .96.6L18.5 13"/><path d="M17 16.5h.01M14 16.5h.01"/>',
  star: '<path d="m12 3 2.7 5.6 6.1.9-4.4 4.3 1 6.1L12 17l-5.4 2.9 1-6.1-4.4-4.3 6.1-.9z"/>',
  clock: '<circle cx="12" cy="12" r="9"/><path d="M12 7v5l3 2"/>',
  app: '<rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>',
  people: '<circle cx="9" cy="8" r="3.5"/><path d="M2.5 20a6.5 6.5 0 0 1 13 0"/><path d="M16 4.6a3.5 3.5 0 0 1 0 6.8M18.5 14.2A6.5 6.5 0 0 1 21.5 20"/>',
  person: '<circle cx="12" cy="8" r="4"/><path d="M4.5 21a7.5 7.5 0 0 1 15 0"/>',
  gear: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 1 1-2.83 2.83l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 1 1-4 0v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 1 1-2.83-2.83l.06-.06A1.65 1.65 0 0 0 4.68 15a1.65 1.65 0 0 0-1.51-1H3a2 2 0 1 1 0-4h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 1 1 2.83-2.83l.06.06A1.65 1.65 0 0 0 9 4.68a1.65 1.65 0 0 0 1-1.51V3a2 2 0 1 1 4 0v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 1 1 2.83 2.83l-.06.06A1.65 1.65 0 0 0 19.4 9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 1 1 0 4h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
  refresh: '<path d="M21 12a9 9 0 1 1-2.64-6.36L21 8"/><path d="M21 3v5h-5"/>',
  info: '<circle cx="12" cy="12" r="9"/><path d="M12 16v-4M12 8h.01"/>',
  play: '<path d="M7 4.5v15l12.5-7.5z"/>',
  download: '<path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/>',
  switch: '<path d="M4 8h14l-3.5-3.5M20 16H6l3.5 3.5"/>',
  external: '<path d="M15 3h6v6M10 14 21 3"/><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>',
  folder: '<path d="M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>',
  trash: '<path d="M4 7h16M10 11v6M14 11v6M6 7l1 13a1 1 0 0 0 1 1h8a1 1 0 0 0 1-1l1-13M9 7V4h6v3"/>',
  x: '<path d="M6 6l12 12M18 6 6 18"/>',
  check: '<circle cx="12" cy="12" r="9"/><path d="m8 12 3 3 5-6"/>',
  warn: '<path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0z"/><path d="M12 9v4M12 17h.01"/>',
  chevron: '<path d="m9 6 6 6-6 6"/>',
  updown: '<path d="m8 9 4-4 4 4M8 15l4 4 4-4"/>',
  plus: '<path d="M12 5v14M5 12h14"/>',
  grid: '<rect x="4" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="4" width="6.5" height="6.5" rx="1.5"/><rect x="4" y="13.5" width="6.5" height="6.5" rx="1.5"/><rect x="13.5" y="13.5" width="6.5" height="6.5" rx="1.5"/>',
  list: '<path d="M9 6h11M9 12h11M9 18h11"/><path d="M4.5 6h.01M4.5 12h.01M4.5 18h.01"/>',
  key: '<circle cx="8" cy="15" r="4"/><path d="m10.8 12.2 8.7-8.7M16 7l3 3M18 5l2 2"/>',
  bolt: '<path d="M13 2 4 14h7l-1 8 9-12h-7z"/>'
};
const icon = (name) => `<svg class="i" viewBox="0 0 24 24" aria-hidden="true">${ICONS[name] || ''}</svg>`;

const state = {
  page: 'home',
  settings: null,
  lib: null,
  status: { running: false, activeAccountId: 0, autoLoginUser: '' },
  loading: true,
  error: null,
  slide: 0,
  details: {},
  query: '',
  filter: '',
  switching: null
};

const $ = (sel) => document.querySelector(sel);
const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const plural = (n, one, many = `${one}s`) => `${n.toLocaleString('en-US')} ${n === 1 ? one : many}`;

// ---------- Formatting ----------
function fmtPlaytime(min) {
  if (!min) return '';
  if (min < 60) return plural(min, 'minute');
  const h = min / 60;
  return h < 10 ? `${h.toFixed(1).replace(/\.0$/, '')} hours` : plural(Math.round(h), 'hour');
}

function fmtBytes(n) {
  if (!n) return '';
  const u = ['B', 'KB', 'MB', 'GB', 'TB'];
  let i = 0;
  while (n >= 1024 && i < u.length - 1) { n /= 1024; i++; }
  return `${n.toFixed(i >= 3 ? 1 : 0)} ${u[i]}`;
}

function relTime(unix) {
  if (!unix) return '';
  const days = Math.floor((Date.now() / 1000 - unix) / 86400);
  if (days <= 0) return 'Today';
  if (days === 1) return 'Yesterday';
  if (days < 7) return `${days} days ago`;
  if (days < 30) return plural(Math.round(days / 7), 'week') + ' ago';
  if (days < 365) return plural(Math.round(days / 30), 'month') + ' ago';
  return plural(Math.round(days / 365), 'year') + ' ago';
}

const year = (unix) => (unix ? new Date(unix * 1000).getFullYear() : '');
const fmtDate = (unix) => (unix ? new Date(unix * 1000).toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' }) : '');

// ---------- Model ----------
const accounts = () => state.lib?.accounts || [];
const games = () => (state.lib?.games || []).filter((g) => g.kind !== 'software' || state.settings.showSoftware);
const accountById = (id) => accounts().find((a) => a.accountId === id);
const accountByName = (n) => accounts().find((a) => a.accountName === n);
const findGame = (id) => state.lib?.games.find((g) => g.appid === Number(id));
const isFav = (g) => state.settings.favorites.includes(g.appid);
const isInstalled = (g) => g.state !== 'none';
const activeAccount = () => (state.status.running ? accountById(state.status.activeAccountId) : null);

// The account whose library we show: the one signed in, or the one Steam will sign in next.
function viewAccount() {
  return activeAccount() || accountByName(state.status.autoLoginUser) || accounts()[0] || null;
}

const ownership = (g, acc) => (acc ? g.owners.find((o) => o.accountId === acc.accountId) : null);
const lastPlayed = (g, acc) => (acc ? ownership(g, acc)?.lastPlayed || 0 : Math.max(0, ...g.owners.map((o) => o.lastPlayed)));
const playtime = (g, acc) => (acc ? ownership(g, acc)?.playtime || 0 : g.owners.reduce((s, o) => s + o.playtime, 0));
const ownerAccounts = (g) => g.owners.map((o) => accountById(o.accountId)).filter(Boolean);

function scoped(list = games()) {
  if (state.settings.scope === 'all') return list;
  const acc = viewAccount();
  return acc ? list.filter((g) => ownership(g, acc)) : list;
}

/** Games Steam knows about on this PC that local files don't tie to any account. */
const unclaimed = () => games().filter((g) => !g.owners.length);

function sortGames(list, sort = state.settings.sort) {
  const acc = state.settings.scope === 'all' ? null : viewAccount();
  const name = (a, b) => a.name.localeCompare(b.name, 'en', { sensitivity: 'base', numeric: true });
  const by = {
    name,
    recent: (a, b) => lastPlayed(b, acc) - lastPlayed(a, acc) || name(a, b),
    playtime: (a, b) => playtime(b, acc) - playtime(a, acc) || name(a, b),
    size: (a, b) => b.sizeOnDisk - a.sizeOnDisk || name(a, b),
    release: (a, b) => (b.release || 0) - (a.release || 0) || name(a, b)
  };
  return [...list].sort(by[sort] || name);
}

function recentFor(acc, n = 12) {
  return games()
    .filter((g) => g.kind !== 'software' && lastPlayed(g, acc) > 0)
    .sort((a, b) => lastPlayed(b, acc) - lastPlayed(a, acc))
    .slice(0, n);
}

/** What the primary button does for a game, given who's signed in. */
function playPlan(g) {
  const active = activeAccount();
  const owners = ownerAccounts(g);
  const verb = isInstalled(g) ? 'Play' : 'Install';
  const direct = !owners.length || (active && owners.includes(active)) || (!state.status.running && owners.includes(viewAccount()));
  if (direct) return { verb, label: verb, account: null };
  const view = viewAccount();
  const target = owners.includes(view) ? view : [...owners].sort((a, b) => playtime(g, b) - playtime(g, a))[0];
  return { verb, label: `${verb} as ${target.personaName}`, account: target };
}

// ---------- Data ----------
async function saveSettings(patch) {
  state.settings = { ...state.settings, ...patch };
  await invoke('settings_set', { value: state.settings }).catch((e) => banner('Couldn’t save settings', String(e)));
}

async function load({ manual = false } = {}) {
  const btn = $('#refresh-btn');
  btn.classList.add('spinning');
  try {
    state.lib = await invoke('library');
    state.status = state.lib.status;
    state.error = null;
    if (state.lib.apiError && manual) banner('Steam Web API', state.lib.apiError);
    else if (manual) banner('Library reloaded', `${plural(games().length, 'title')} across ${plural(accounts().length, 'account')}.`, 'ok');
    loadDetails(slides().map((g) => g.appid));
  } catch (e) {
    state.error = String(e);
  } finally {
    state.loading = false;
    state.loadedAt = Date.now();
    btn.classList.remove('spinning');
    renderChrome();
    // Background reloads shouldn't move the page under the pointer.
    const top = $('#content').scrollTop;
    const rows = [...document.querySelectorAll('.row')].map((r) => r.scrollLeft);
    renderPage();
    $('#content').scrollTop = top;
    document.querySelectorAll('.row').forEach((r, i) => { r.scrollLeft = rows[i] || 0; });
    updateAllRowNavs();
  }
}

async function loadDetails(ids) {
  const need = ids.filter((id) => !(id in state.details));
  if (!need.length) return;
  need.forEach((id) => { state.details[id] = null; });
  const list = await invoke('details', { ids: need }).catch(() => []);
  for (const d of list) state.details[d.appid] = d;
  if (state.page === 'home') renderHero();
  const open = $('#sheet .sheet.game');
  if (open && need.includes(Number(open.dataset.appid))) openGame(open.dataset.appid, true);
}

// Steam can be signed in or out from outside the app; follow along.
async function pollStatus() {
  const s = await invoke('status').catch(() => null);
  if (!s || state.switching) return;
  const changed = s.running !== state.status.running || s.activeAccountId !== state.status.activeAccountId || s.autoLoginUser !== state.status.autoLoginUser;
  state.status = s;
  if (changed) load();
}

// ---------- Artwork ----------
function avatar(acc, cls = '') {
  if (!acc) return `<span class="avatar ${cls}">${icon('person')}</span>`;
  const initials = acc.personaName.replace(/[^\p{L}\p{N} ]/gu, '').trim().split(/\s+/).map((w) => w[0]).join('').slice(0, 2).toUpperCase() || '?';
  return acc.avatar
    ? `<img class="avatar ${cls}" src="${esc(acc.avatar)}" alt="" onerror="this.outerHTML='<span class=&quot;avatar ${cls}&quot;>${esc(initials)}</span>'">`
    : `<span class="avatar ${cls}">${esc(initials)}</span>`;
}

// Falls back from the local cache to a sibling art kind, then to the name.
window.artFail = (img) => {
  const next = img.dataset.alt;
  if (next) { img.dataset.alt = ''; img.src = next; return; }
  img.classList.add('failed');
};

const artImg = (src, alt) => (src ? `<img src="${esc(src)}" ${alt ? `data-alt="${esc(alt)}"` : ''} alt="" loading="lazy" decoding="async" onerror="artFail(this)">` : '');

function poster(g, { sub } = {}) {
  const plan = playPlan(g);
  const acc = state.settings.scope === 'all' ? null : viewAccount();
  const line = sub ?? (isInstalled(g)
    ? (g.state === 'update' ? 'Update available' : g.state === 'downloading' ? 'Downloading…' : fmtPlaytime(playtime(g, acc)) || 'Installed')
    : fmtPlaytime(playtime(g, acc)) || 'Not installed');
  return `<div class="poster ${isInstalled(g) ? '' : 'dim'}" data-action="game" data-id="${g.appid}" title="${esc(g.name)}">
    <div class="art"><div class="fallback">${esc(g.name)}</div>${artImg(g.cover, g.header)}
      ${favButton(g)}
      ${isInstalled(g) ? '' : `<span class="badge">${icon('download')}${plan.account ? esc(plan.account.personaName) : 'Install'}</span>`}</div>
    <div class="name">${esc(g.name)}</div>
    <div class="sub">${esc(line)}</div></div>`;
}

function landscape(g, acc) {
  const lp = lastPlayed(g, acc);
  return `<div class="tile" data-action="game" data-id="${g.appid}">
    <div class="art"><div class="fallback">${esc(g.name)}</div>${artImg(g.header || g.hero, g.hero)}${favButton(g)}
      ${isInstalled(g) ? '' : `<span class="badge">${icon('download')}Not installed</span>`}</div>
    <div class="name">${esc(g.name)}</div>
    <div class="sub">${esc([lp && relTime(lp), fmtPlaytime(playtime(g, acc))].filter(Boolean).join(' · '))}</div></div>`;
}

const favButton = (g) => `<button class="fav-star ${isFav(g) ? 'on' : ''}" data-action="fav" data-id="${g.appid}" title="${isFav(g) ? 'Remove from Favorites' : 'Add to Favorites'}" aria-label="Favorite">${icon('star')}</button>`;

// ---------- Sheets ----------
function openSheet(html, { cls = '', locked = false } = {}) {
  const el = $('#sheet');
  el.innerHTML = `<div class="sheet ${cls}" role="dialog" aria-modal="true">${html}</div>`;
  el.hidden = false;
  el.dataset.locked = locked ? '1' : '';
  el.querySelector('[autofocus]')?.focus();
  return el.firstElementChild;
}
function closeSheet() {
  const el = $('#sheet');
  if (el.dataset.locked) return;
  el.hidden = true;
  el.innerHTML = '';
}

function openPicker() {
  const active = activeAccount();
  openSheet(`
    <div class="body center" style="padding:28px 22px 18px">
      <h3 style="font:700 22px var(--font-display);margin:0 0 4px">Who’s playing?</h3>
      <div style="color:var(--label-2);margin-bottom:14px">Steam switches in the background, without asking for a password.</div>
      <div class="picker">
        ${accounts().map((a) => `<button class="${a === active ? 'active' : ''}" data-action="switch" data-name="${esc(a.accountName)}">
          ${avatar(a)}<span class="pname">${esc(a.personaName)}</span><span class="aname">${a === active ? 'Signed in' : esc(a.accountName)}</span></button>`).join('')}
        <button data-action="add-account"><span class="plus">${icon('plus')}</span><span class="pname">Add Account</span><span class="aname">Sign in with Steam</span></button>
      </div>
    </div>
    <footer><button class="btn" data-action="close-sheet" autofocus>Done</button></footer>`, { cls: 'small' });
}

const STEPS = [['closing', 'Closing Steam'], ['preparing', 'Choosing account'], ['starting', 'Starting Steam'], ['signing-in', 'Signing in']];

function switchView(acc, stage, gameName) {
  const at = STEPS.findIndex(([k]) => k === stage);
  return `<div class="body center" style="padding:28px 30px 22px">
    ${avatar(acc, '').replace('class="avatar ', 'style="width:72px;height:72px;font-size:24px;margin:0 auto 12px" class="avatar ')}
    <h3 style="font:700 18px var(--font-display);margin:0 0 4px">Switching to ${esc(acc.personaName)}</h3>
    <div style="color:var(--label-2)">${gameName ? `${esc(gameName)} will start once Steam is signed in.` : state.settings.showSteamWindow ? 'Steam will open when it’s ready.' : 'Steam stays in the tray, no windows.'}</div>
    <div class="steps">${STEPS.map(([k, label], i) => `<div class="step ${i < at ? 'done' : i === at ? 'now' : ''}"><span class="dotx"></span>${label}</div>`).join('')}</div>
  </div>`;
}

async function switchTo(name, { launch = null } = {}) {
  const acc = accountByName(name);
  if (!acc) return;
  const game = launch ? findGame(launch) : null;
  if (activeAccount() === acc && !launch) { closeSheet(); return banner(`Already signed in as ${acc.personaName}`, '', 'ok'); }
  state.switching = { acc, game };
  openSheet(switchView(acc, 'closing', game?.name), { cls: 'small', locked: true });
  let result;
  try {
    result = await invoke('switch_account', { accountName: name, launch });
  } catch (e) {
    state.switching = null;
    $('#sheet').dataset.locked = '';
    closeSheet();
    return banner('Couldn’t switch accounts', String(e));
  }
  state.switching = null;
  $('#sheet').dataset.locked = '';
  closeSheet();
  state.status = await invoke('status').catch(() => state.status);
  if (result.signedIn) banner(`Signed in as ${acc.personaName}`, game ? `Starting ${game.name}…` : 'Steam is running in the background.', 'ok');
  else banner('Steam is waiting for you', result.needsPassword ? `${acc.personaName} isn’t remembered on this PC. Sign in once in Steam with “Remember me” checked, and switching will be silent from then on.` : 'Steam hasn’t finished signing in. If it asks for a password, sign in once with “Remember me” checked.');
  load();
}

T.event.listen('switch-progress', ({ payload }) => {
  const s = state.switching;
  if (s) openSheet(switchView(s.acc, payload, s.game?.name), { cls: 'small', locked: true });
});

async function addAccount() {
  closeSheet();
  try {
    await invoke('add_account');
    banner('Sign in with Steam', 'Steam’s sign-in window is open. Check “Remember me” so Hangar can switch to the account silently later.', 'ok');
  } catch (e) {
    banner('Couldn’t open Steam', String(e));
  }
}

function play(id) {
  const g = findGame(id);
  if (!g) return;
  const plan = playPlan(g);
  if (plan.account) return switchTo(plan.account.accountName, { launch: g.appid });
  invoke('open_url', { url: `steam://rungameid/${g.appid}` }).catch((e) => banner('Couldn’t open Steam', String(e)));
}

function openGame(id, refresh = false) {
  const g = findGame(id);
  if (!g) return;
  loadDetails([g.appid]);
  const d = state.details[g.appid];
  const plan = playPlan(g);
  const acc = viewAccount();
  const owners = ownerAccounts(g);
  const meta = [g.kind === 'software' ? 'Software' : g.kind === 'demo' ? 'Demo' : null, ...g.genres.slice(0, 3)].filter(Boolean);
  const fact = (k, v) => `<div class="stat"><div class="k">${k}</div><div class="val">${v || '—'}</div></div>`;
  const html = `
    <div class="body">
      <div class="game-hero">
        <img class="bg" src="${esc(g.hero || g.header || '')}" data-alt="${esc(g.header || '')}" alt="" onerror="artFail(this)">
        ${g.logo ? `<img class="logo" src="${esc(g.logo)}" alt="${esc(g.name)}" onerror="this.outerHTML='<div class=&quot;htitle&quot;>${esc(esc(g.name))}</div>'">` : `<div class="htitle">${esc(g.name)}</div>`}
        <button class="circle" data-action="close-sheet" aria-label="Close">${icon('x')}</button>
      </div>
      <div class="game-body">
        <h3 style="font:700 20px var(--font-display);margin:6px 0 6px">${esc(g.name)}</h3>
        <div class="meta">${meta.map(esc).join('<span class="chev-sep">›</span>')}${g.release ? `<span class="sep">•</span><span>${year(g.release)}</span>` : ''}${g.metacritic ? `<span class="box">Metacritic ${g.metacritic}</span>` : ''}${g.reviewPercent ? `<span class="box">${g.reviewPercent}% positive</span>` : ''}</div>
        <p class="desc">${d === undefined || d === null ? '<span class="skeleton" style="display:block;height:44px"></span>' : esc(d.description || 'No description on the Steam store.')}</p>
        <div class="facts-grid">
          ${fact(acc && state.settings.scope !== 'all' ? `Played by ${esc(acc.personaName)}` : 'Playtime', esc(fmtPlaytime(playtime(g, state.settings.scope === 'all' ? null : acc)) || 'Never'))}
          ${fact('Last played', esc(relTime(lastPlayed(g, state.settings.scope === 'all' ? null : acc)) || 'Never'))}
          ${fact(isInstalled(g) ? 'Size on disk' : 'Status', esc(isInstalled(g) ? fmtBytes(g.sizeOnDisk) || 'Installed' : 'Not installed'))}
          ${fact('Developer', esc(g.developer))}
          ${fact('Publisher', esc(g.publisher))}
          ${fact('Released', esc(fmtDate(g.release)))}
        </div>
        ${d?.screenshots?.length ? `<div class="group-title">Screenshots</div>${scroller(d.screenshots.map((s) => `<img src="${esc(s)}" alt="" loading="lazy">`).join(''), 'screenshots', 'shots')}` : ''}
        <div class="group-title">On This PC</div>
        <div class="group">${owners.length ? owners.map((a) => {
          const o = ownership(g, a);
          return `<div class="cell owner-row">${avatar(a)}<div class="grow"><div class="title">${esc(a.personaName)}</div><div class="note">${esc([fmtPlaytime(o.playtime) || 'No playtime recorded', o.lastPlayed && `last played ${relTime(o.lastPlayed).toLowerCase()}`].filter(Boolean).join(' · '))}</div></div>
            ${a === activeAccount() ? '<span class="tag active">Signed in</span>' : `<button class="btn" data-action="switch" data-name="${esc(a.accountName)}">${icon('switch')}Switch</button>`}</div>`;
        }).join('') : '<div class="cell"><div class="grow note" style="color:var(--label-2)">Steam knows this title on this PC, but hasn’t recorded which account uses it.</div></div>'}
        ${g.installPath ? `<div class="cell"><div class="grow"><div class="title">Location</div><div class="note" style="user-select:text">${esc(g.installPath)}</div></div><button class="icon-btn" data-action="reveal" data-path="${esc(g.installPath)}" title="Show in Explorer" aria-label="Show in Explorer">${icon('folder')}</button></div>` : ''}
        </div>
      </div>
    </div>
    <footer>
      <div class="left">
        <button class="btn" data-action="open" data-url="https://store.steampowered.com/app/${g.appid}/">${icon('external')}Store Page</button>
        ${isInstalled(g) ? `<button class="btn danger" data-action="open" data-url="steam://uninstall/${g.appid}">${icon('trash')}Uninstall…</button>` : ''}
      </div>
      <button class="btn ${isFav(g) ? 'on' : ''}" data-action="fav" data-id="${g.appid}">${icon('star')}${isFav(g) ? 'Favorited' : 'Favorite'}</button>
      <button class="btn blue" data-action="play" data-id="${g.appid}" ${refresh ? '' : 'autofocus'}>${icon(plan.verb === 'Play' ? 'play' : 'download')}${esc(plan.label)}</button>
    </footer>`;
  const el = refresh ? $('#sheet .sheet.game') : null;
  if (el) { const top = el.querySelector('.body').scrollTop; el.innerHTML = html; el.querySelector('.body').scrollTop = top; }
  else openSheet(html, { cls: 'game' }).dataset.appid = g.appid;
  updateAllRowNavs();
}

// ---------- Rendering ----------
function patch(sel, html) {
  const el = $(sel);
  if (el) el.innerHTML = html;
}

function banner(title, note, kind = 'warn') {
  const el = $('#banner');
  el.innerHTML = `<div class="banner ${kind}" role="status">${icon(kind === 'ok' ? 'check' : 'warn')}<div class="grow"><b>${esc(title)}</b>${note ? `<div class="note">${esc(note)}</div>` : ''}</div><button class="icon-btn" data-action="dismiss-banner" aria-label="Dismiss">${icon('x')}</button></div>`;
  clearTimeout(banner.t);
  banner.t = setTimeout(() => { el.innerHTML = ''; }, 7000);
}

function renderChrome() {
  const acc = viewAccount();
  const active = activeAccount();
  $('#me').innerHTML = acc
    ? `${avatar(acc)}<div class="who">${esc(acc.personaName)}<small>${active ? 'Signed in to Steam' : state.status.running ? 'Steam is signed out' : 'Steam isn’t running'}</small></div>${icon('updown').replace('class="i"', 'class="i chev"')}`
    : `${avatar(null)}<div class="who">No accounts<small>Sign in with Steam</small></div>`;
  const all = scoped();
  const count = (id, n) => { const el = $(id); el.textContent = n || ''; };
  count('#c-games', all.filter((g) => g.kind !== 'software').length);
  count('#c-installed', all.filter(isInstalled).length);
  count('#c-favorites', games().filter(isFav).length);
  count('#c-software', all.filter((g) => g.kind === 'software').length);
  count('#c-accounts', accounts().length);
  $('#nav-software').hidden = !state.settings.showSoftware || !games().some((g) => g.kind === 'software');
  document.querySelectorAll('#nav a').forEach((a) => a.classList.toggle('active', a.dataset.page === state.page));
}

function renderPage() {
  const pages = {
    home: homePage,
    search: searchPage,
    games: () => libraryPage('games'),
    installed: () => libraryPage('installed'),
    favorites: () => libraryPage('favorites'),
    recent: () => libraryPage('recent'),
    software: () => libraryPage('software'),
    accounts: accountsPage,
    settings: settingsPage
  };
  $('#page').innerHTML = problemState() || (pages[state.page] || homePage)();
  if (state.page === 'home') { renderHero(); updateAllRowNavs(); }
  if (state.page === 'search') {
    const input = $('#search-input');
    input.focus();
    input.setSelectionRange(input.value.length, input.value.length);
    runSearch();
  }
}

function navigate(page) {
  if (state.page === page) return;
  state.page = page;
  state.filter = '';
  $('#content').scrollTop = 0;
  renderChrome();
  renderPage();
}

function problemState() {
  if (state.loading) return state.page === 'home' ? '' : '<div class="page-pad"><div class="skeleton" style="height:28px;width:220px;margin-bottom:24px"></div><div class="skeleton" style="height:320px"></div></div>';
  if (state.error) return `<div class="page-pad"><div class="empty">${icon('warn')}<h3>Can’t read your Steam library</h3><p>${esc(state.error)}</p><button class="btn blue" data-action="refresh">Try Again</button></div></div>`;
  return null;
}

// Home
function slides() {
  const acc = viewAccount();
  const favs = games().filter(isFav).filter((g) => g.hero);
  const recent = recentFor(acc, 8).filter((g) => g.hero);
  const seen = new Set();
  return [...recent.slice(0, 4), ...favs, ...recent.slice(4)].filter((g) => !seen.has(g.appid) && seen.add(g.appid)).slice(0, 7);
}

function homePage() {
  if (state.loading) {
    return `<section class="hero"><div class="hero-body" style="width:420px">
      <div class="skeleton" style="height:90px;width:300px;margin-bottom:18px"></div>
      <div class="skeleton" style="height:12px;width:160px"></div>
      <div class="skeleton" style="height:12px;width:340px;margin-top:14px"></div>
      <div class="skeleton" style="height:12px;width:280px;margin-top:8px"></div>
      <div class="skeleton" style="height:36px;width:150px;margin-top:22px;border-radius:18px"></div></div></section>`;
  }
  const acc = viewAccount();
  const mine = scoped();
  const recent = recentFor(acc);
  const favs = sortGames(games().filter(isFav), 'recent');
  const installed = sortGames(mine.filter((g) => isInstalled(g) && g.kind !== 'software'), 'recent');
  const ready = sortGames(mine.filter((g) => !isInstalled(g) && g.kind === 'game'), 'playtime').slice(0, 24);
  const shelf = (title, sub, page, items) => (items.length ? `<section class="shelf">
      <h2>${title}${sub ? ` <small>${sub}</small>` : ''}${page ? `<button class="see-all" data-page="${page}">See All${icon('chevron')}</button>` : ''}</h2>
      ${scroller(items, title)}</section>` : '');
  return `
    ${slides().length ? '<section class="hero" id="hero"></section>' : `<div class="page-pad"><h1 class="large-title">Home</h1><p class="subtitle">${acc ? `Nothing played yet on ${esc(acc.personaName)}.` : 'Sign in to Steam to get started.'}</p></div>`}
    ${shelf('Continue Playing', acc ? esc(acc.personaName) : '', 'recent', recent.map((g) => landscape(g, acc)).join(''))}
    ${shelf('Favorites', '', 'favorites', favs.map((g) => poster(g)).join(''))}
    ${shelf('Installed', fmtBytes(installed.reduce((s, g) => s + g.sizeOnDisk, 0)), 'installed', installed.map((g) => poster(g)).join(''))}
    ${shelf('Ready to Install', 'Most played first', 'games', ready.map((g) => poster(g)).join(''))}
    <section class="shelf"><h2>Accounts <small>Switch without a password prompt</small><button class="see-all" data-page="accounts">Manage${icon('chevron')}</button></h2>
      ${scroller(accounts().map((a) => `<button class="${a === activeAccount() ? 'active' : ''}" data-action="switch" data-name="${esc(a.accountName)}">${avatar(a)}<span class="pname">${esc(a.personaName)}</span><span class="aname">${a === activeAccount() ? 'Signed in' : plural(scopedCount(a), 'game')}</span></button>`).join(''), 'accounts', 'picker')}</section>
    <div style="height:28px"></div>`;
}

const scopedCount = (a) => games().filter((g) => g.kind !== 'software' && ownership(g, a)).length;

function renderHero() {
  const el = $('#hero');
  if (!el) return;
  const list = slides();
  if (!list.length) return;
  state.slide = Math.min(state.slide, list.length - 1);
  const g = list[state.slide];
  const acc = viewAccount();
  const d = state.details[g.appid];
  const plan = playPlan(g);
  const owners = ownerAccounts(g);
  const prevImg = el.querySelector('.hero-img');
  const keepImg = prevImg && prevImg.dataset.src === g.hero;
  const genres = g.genres.filter((x) => x !== 'Free to Play').slice(0, 2);
  const meta = g.kind === 'software' ? ['Software', ...genres] : genres.length ? genres : ['Game'];
  el.innerHTML = `
    <img class="hero-img ${keepImg ? 'loaded' : ''}" data-src="${esc(g.hero)}" src="${esc(g.hero)}" alt="" onload="this.classList.add('loaded')" onerror="artFail(this)" ${g.header ? `data-alt="${esc(g.header)}"` : ''}>
    <div class="hero-body">
      ${g.logo ? `<img class="hero-logo" src="${esc(g.logo)}" alt="${esc(g.name)}" onload="this.classList.add('loaded')" onerror="this.outerHTML='<h1 class=&quot;hero-title&quot;>${esc(esc(g.name))}</h1>'">` : `<h1 class="hero-title">${esc(g.name)}</h1>`}
      <div class="meta">
        ${meta.map(esc).join('<span class="chev-sep">›</span>')}
        ${g.release ? `<span class="sep">•</span><span>${year(g.release)}</span>` : ''}
        ${isInstalled(g) ? `<span class="box">${g.state === 'update' ? 'UPDATE' : 'INSTALLED'}</span>` : ''}
        ${g.genres.includes('Free to Play') ? '<span class="box">FREE</span>' : ''}
      </div>
      <p class="desc">${d?.description ? esc(d.description) : d === null || d === undefined ? '&nbsp;' : esc(g.developer ? `By ${g.developer}.` : '')}</p>
      ${owners.length ? `<div class="faces"><div class="stack">${owners.slice(0, 4).map((a) => avatar(a)).join('')}</div>
        <span>${esc(heroLine(g, acc))}</span></div>` : ''}
      <div class="actions">
        <button class="pill primary" data-action="play" data-id="${g.appid}">${icon(plan.verb === 'Play' ? 'play' : 'download')}${esc(plan.label)}</button>
        <button class="circle ${isFav(g) ? 'on' : ''}" data-action="fav" data-id="${g.appid}" title="${isFav(g) ? 'Remove from Favorites' : 'Add to Favorites'}" aria-label="Favorite">${icon('star')}</button>
        <button class="circle" data-action="game" data-id="${g.appid}" title="Details" aria-label="Details">${icon('info')}</button>
      </div>
    </div>
    ${list.length > 1 ? `
      <button class="circle hero-nav prev" data-action="hero-step" data-dir="-1" title="Previous" aria-label="Previous game"><svg class="i" viewBox="0 0 24 24"><path d="m15 6-6 6 6 6"/></svg></button>
      <button class="circle hero-nav next" data-action="hero-step" data-dir="1" title="Next" aria-label="Next game">${icon('chevron')}</button>
      <div class="dots">${list.map((x, i) => `<button class="${i === state.slide ? 'on' : ''}" data-action="slide" data-index="${i}" aria-label="Show ${esc(x.name)}"></button>`).join('')}</div>` : ''}`;
}

function stepHero(dir) {
  const n = slides().length;
  if (n > 1) { state.slide = (state.slide + dir + n) % n; renderHero(); }
}

function heroLine(g, acc) {
  const o = acc && ownership(g, acc);
  const time = o?.playtime ? `${fmtPlaytime(o.playtime)} played` : '';
  const when = o?.lastPlayed ? `last played ${relTime(o.lastPlayed).toLowerCase()}` : '';
  const others = ownerAccounts(g).filter((a) => a !== acc);
  const also = others.length ? `also on ${others.map((a) => a.personaName).join(', ')}` : '';
  return [time, when, also].filter(Boolean).join(' · ') || `On ${ownerAccounts(g).map((a) => a.personaName).join(' and ')}`;
}

// Horizontal shelf with TV-app style arrows; a mouse wheel can't scroll sideways on its own.
function scroller(items, label, cls = '') {
  return `<div class="row-wrap" data-at-start="true">
    <button class="row-nav prev" data-action="row-scroll" data-dir="-1" aria-label="Scroll ${esc(label)} left"><svg class="i" viewBox="0 0 24 24"><path d="m15 6-6 6 6 6"/></svg></button>
    <div class="row ${cls}">${items}</div>
    <button class="row-nav next" data-action="row-scroll" data-dir="1" aria-label="Scroll ${esc(label)} right">${icon('chevron')}</button>
  </div>`;
}

function updateRowNav(row) {
  const wrap = row.closest('.row-wrap');
  if (!wrap) return;
  wrap.dataset.atStart = row.scrollLeft <= 4;
  wrap.dataset.atEnd = row.scrollLeft + row.clientWidth >= row.scrollWidth - 4;
}
function updateAllRowNavs() {
  document.querySelectorAll('.row-wrap .row').forEach(updateRowNav);
}
document.addEventListener('scroll', (e) => { if (e.target.classList?.contains('row')) updateRowNav(e.target); }, true);
window.addEventListener('resize', updateAllRowNavs);

// Library pages
const LIBRARY = {
  games: { title: 'Games', sub: 'Everything in your library, installed or not.', pick: (l) => l.filter((g) => g.kind !== 'software') },
  installed: { title: 'Installed', sub: 'Ready to play on this PC.', pick: (l) => l.filter(isInstalled), flat: true },
  favorites: { title: 'Favorites', sub: 'Saved on this PC only. Star any game to add it.', pick: () => games().filter(isFav), allAccounts: true },
  recent: { title: 'Recently Played', sub: 'Most recent first.', pick: (l) => l.filter((g) => g.kind !== 'software' && lastPlayed(g, state.settings.scope === 'all' ? null : viewAccount()) > 0), sort: 'recent', flat: true },
  software: { title: 'Software', sub: 'Tools and applications from Steam.', pick: (l) => l.filter((g) => g.kind === 'software') }
};

function libraryPage(kind) {
  const cfg = LIBRARY[kind];
  const acc = viewAccount();
  const all = state.settings.scope === 'all';
  const q = state.filter.trim().toLowerCase();
  const match = (g) => !q || g.name.toLowerCase().includes(q) || (g.developer || '').toLowerCase().includes(q) || g.genres.some((x) => x.toLowerCase().includes(q));
  let list = cfg.pick(cfg.allAccounts ? games() : scoped()).filter(match);
  list = sortGames(list, cfg.sort || state.settings.sort);
  const extra = !cfg.allAccounts && !all && kind === 'games' ? sortGames(cfg.pick(unclaimed()).filter(match), cfg.sort || state.settings.sort) : [];
  const view = state.settings.view;
  const groups = cfg.flat ? [[null, list]] : [['Installed', list.filter(isInstalled)], ['Not Installed', list.filter((g) => !isInstalled(g))]];
  if (extra.length) groups.push(['Also on This PC', extra, 'Steam has these on this PC but hasn’t recorded which account uses them.']);
  const body = groups.filter(([, items]) => items.length).map(([title, items, note]) => `
    ${title ? `<div class="section-head"><h2>${title}</h2><small>${plural(items.length, 'title')}${title === 'Installed' ? ` · ${fmtBytes(items.reduce((s, g) => s + g.sizeOnDisk, 0))}` : ''}${note ? ` · ${note}` : ''}</small></div>` : '<div style="height:16px"></div>'}
    ${view === 'list' ? `<div class="group">${items.map(listRow).join('')}</div>` : `<div class="grid">${items.map((g) => poster(g)).join('')}</div>`}`).join('');
  const total = list.length + extra.length;
  return `<div class="page-pad wide">
    <h1 class="large-title">${cfg.title}</h1>
    <p class="subtitle">${cfg.sub} ${cfg.allAccounts ? '' : all ? `All ${plural(accounts().length, 'account')} on this PC.` : acc ? `Showing ${esc(acc.personaName)}.` : ''}</p>
    <div class="toolbar">
      <label class="search-field">${icon('search')}<input id="filter-input" placeholder="Filter ${esc(cfg.title.toLowerCase())}" value="${esc(state.filter)}" spellcheck="false"></label>
      ${cfg.allAccounts ? '' : `<div class="segmented" role="radiogroup">
        <button class="${all ? '' : 'on'}" data-action="scope" data-value="account" role="radio" aria-checked="${!all}">${esc(acc?.personaName || 'This Account')}</button>
        <button class="${all ? 'on' : ''}" data-action="scope" data-value="all" role="radio" aria-checked="${all}">All Accounts</button></div>`}
      <div class="spacer"></div>
      ${cfg.sort ? '' : `<select class="popup" data-setting-select="sort" aria-label="Sort by">
        ${[['name', 'Name'], ['recent', 'Recently Played'], ['playtime', 'Playtime'], ['size', 'Size on Disk'], ['release', 'Release Date']].map(([v, l]) => `<option value="${v}" ${state.settings.sort === v ? 'selected' : ''}>${l}</option>`).join('')}
      </select>`}
      <div class="segmented icons" role="radiogroup" aria-label="View">
        <button class="${view === 'grid' ? 'on' : ''}" data-action="view" data-value="grid" title="Grid" aria-label="Grid">${icon('grid')}</button>
        <button class="${view === 'list' ? 'on' : ''}" data-action="view" data-value="list" title="List" aria-label="List">${icon('list')}</button></div>
    </div>
    ${total ? body : `<div class="empty">${icon(kind === 'favorites' ? 'star' : 'gamepad')}<h3>${q ? 'No Results' : kind === 'favorites' ? 'No Favorites Yet' : `No ${cfg.title}`}</h3><p>${q ? `Nothing matches “${esc(state.filter)}”.` : kind === 'favorites' ? 'Star a game from its artwork or details to keep it here.' : 'Nothing to show for this account.'}</p></div>`}
  </div>`;
}

function listRow(g) {
  const acc = state.settings.scope === 'all' ? null : viewAccount();
  const plan = playPlan(g);
  const owners = ownerAccounts(g);
  const tag = g.state === 'update' ? '<span class="tag update">Update</span>' : g.state === 'downloading' ? '<span class="tag new">Downloading</span>' : '';
  const lp = lastPlayed(g, acc);
  return `<div class="cell clickable game-row" data-action="game" data-id="${g.appid}">
    <div class="thumb">${artImg(g.icon || g.header, g.header)}</div>
    <div style="min-width:0"><div class="title">${esc(g.name)} ${tag}</div>
      <div class="note">${owners.length > 1 || state.settings.scope === 'all' ? `<span class="mini-faces">${owners.map((a) => avatar(a)).join('')}</span>` : ''}${esc([g.developer, year(g.release)].filter(Boolean).join(' · '))}</div></div>
    <div class="col">${esc(fmtPlaytime(playtime(g, acc)) || '—')}<br><span style="color:var(--label-3)">${esc(lp ? relTime(lp) : 'Never played')}</span></div>
    <div class="col">${esc(isInstalled(g) ? fmtBytes(g.sizeOnDisk) : 'Not installed')}</div>
    <div class="row-actions">
      <button class="icon-btn ${isFav(g) ? 'on' : ''}" data-action="fav" data-id="${g.appid}" title="Favorite" aria-label="Favorite">${icon('star')}</button>
      <button class="btn ${isInstalled(g) ? 'blue' : ''}" data-action="play" data-id="${g.appid}" title="${esc(plan.label)}">${icon(plan.verb === 'Play' ? 'play' : 'download')}${plan.account ? `${plan.verb} as…` : plan.verb}</button>
    </div></div>`;
}

// Accounts
function accountsPage() {
  const active = activeAccount();
  return `<div class="page-pad wide">
    <h1 class="large-title">Accounts</h1>
    <p class="subtitle">Accounts Steam remembers on this PC. Switching closes Steam, picks the account and restarts it ${state.settings.showSteamWindow ? 'with its window open' : 'quietly in the tray'}. No password prompt, no account picker.</p>
    <div class="people">
      ${accounts().map((a) => {
        const n = scopedCount(a);
        const hours = games().reduce((s, g) => s + (ownership(g, a)?.playtime || 0), 0);
        return `<div class="person ${a === active ? 'active' : ''}">
          ${avatar(a)}
          <div class="pname">${esc(a.personaName)}</div>
          <div class="aname">${esc(a.accountName)}</div>
          <div class="facts">${plural(n, 'game')}${hours ? ` · ${esc(fmtPlaytime(hours))}` : ''}${a.lastLogin ? `<br>Last signed in ${esc(relTime(a.lastLogin).toLowerCase())}` : ''}</div>
          ${a === active ? '<span class="tag active">Signed In</span>'
            : `<button class="btn ${a.rememberPassword ? 'blue' : ''}" data-action="switch" data-name="${esc(a.accountName)}">${icon('switch')}${a.rememberPassword ? 'Switch' : 'Sign In…'}</button>`}
        </div>`;
      }).join('')}
      <button class="person add" data-action="add-account"><span class="plus">${icon('plus')}</span><div class="pname">Add Account</div><div class="aname">Opens Steam’s sign-in window</div></button>
    </div>
    <p class="footnote" style="margin-top:16px">An account switches silently once it has signed in on this PC with “Remember me” checked. Steam keeps that sign-in token; Hangar never sees or stores passwords.</p>
  </div>`;
}

// Search
function searchPage() {
  return `<div class="page-pad wide">
    <h1 class="large-title">Search</h1>
    <p class="subtitle">Every game on every account on this PC.</p>
    <label class="search-field">${icon('search')}<input id="search-input" placeholder="Games, developers, genres" value="${esc(state.query)}" spellcheck="false"></label>
    <div id="search-results"></div></div>`;
}

function runSearch() {
  const out = $('#search-results');
  if (!out) return;
  const q = state.query.trim().toLowerCase();
  if (!q) {
    const recent = recentFor(null, 12);
    out.innerHTML = recent.length ? `<div class="section-head"><h2>Recently Played</h2><small>All accounts</small></div><div class="grid">${recent.map((g) => poster(g)).join('')}</div>` : '';
    return;
  }
  const hits = games().filter((g) => [g.name, g.developer, g.publisher, ...g.genres, String(g.appid)].join(' ').toLowerCase().includes(q));
  const ranked = hits.sort((a, b) => (b.name.toLowerCase().startsWith(q) - a.name.toLowerCase().startsWith(q)) || a.name.localeCompare(b.name));
  const people = accounts().filter((a) => `${a.personaName} ${a.accountName}`.toLowerCase().includes(q));
  out.innerHTML = (people.length ? `<div class="section-head"><h2>Accounts</h2></div><div class="group">${people.map((a) => `<div class="cell owner-row">${avatar(a)}<div class="grow"><div class="title">${esc(a.personaName)}</div><div class="note">${esc(a.accountName)}</div></div>${a === activeAccount() ? '<span class="tag active">Signed in</span>' : `<button class="btn" data-action="switch" data-name="${esc(a.accountName)}">${icon('switch')}Switch</button>`}</div>`).join('')}</div>` : '')
    + (ranked.length ? `<div class="section-head"><h2>Games</h2><small>${plural(ranked.length, 'result')}</small></div><div class="grid">${ranked.map((g) => poster(g)).join('')}</div>` : '')
    || `<div class="empty">${icon('search')}<h3>No Results</h3><p>Nothing matches “${esc(state.query)}”.</p></div>`;
}

// Settings
function settingsPage() {
  const s = state.settings;
  const sw = (key, label, note) => `<div class="cell"><div class="grow"><div class="title">${label}</div>${note ? `<div class="note">${note}</div>` : ''}</div>
    <label class="switch"><input type="checkbox" data-setting="${key}" ${s[key] ? 'checked' : ''}><span></span></label></div>`;
  const lib = state.lib;
  return `<div class="page-pad" style="max-width:720px">
    <h1 class="large-title">Settings</h1>
    <div class="group-title">Switching</div>
    <div class="group">
      ${sw('showSteamWindow', 'Open Steam’s window after switching', 'Off: Steam starts minimized to the tray with no windows or pop-ups.')}
      ${sw('offlineMode', 'Start Steam in offline mode', 'For accounts that are already signed in on this PC.')}
    </div>
    <div class="group-title">Library</div>
    <div class="group">
      ${sw('showSoftware', 'Show software', 'Include tools and applications such as Wallpaper Engine or Lossless Scaling.')}
      <div class="cell"><div class="grow"><div class="title">Steam Web API key</div><div class="note">Optional. Lists games an account owns but has never used on this PC. Get a key at <span class="link" data-action="open" data-url="https://steamcommunity.com/dev/apikey">steamcommunity.com/dev/apikey</span>.</div></div>
        <div class="api-field"><input id="api-key" type="password" placeholder="Not set" value="${esc(s.webApiKey)}" spellcheck="false" autocomplete="off"><button class="btn" data-action="save-key">Save</button></div></div>
      <div class="cell"><div class="grow"><div class="title">Library source</div><div class="note">${lib?.complete ? 'Complete list from the Steam Web API, plus local playtime.' : 'Steam’s local cache on this PC: every game an account has opened, played or installed here.'}</div></div>
        <div class="val">${lib?.complete ? 'Web API' : 'This PC'}</div></div>
      ${lib?.apiError ? `<div class="cell"><div class="grow note" style="color:var(--orange)">${esc(lib.apiError)}</div></div>` : ''}
    </div>
    <div class="group-title">Favorites</div>
    <div class="group"><div class="cell"><div class="grow"><div class="title">${plural(s.favorites.length, 'favorite')}</div><div class="note">Stored on this PC, shared by every account.</div></div>
      <button class="btn danger" data-action="clear-favs" ${s.favorites.length ? '' : 'disabled'}>Clear All…</button></div></div>
    <div class="group-title">Steam</div>
    <div class="group">
      <div class="cell"><div class="grow">Install folder</div><div class="val" style="user-select:text">${esc(lib?.steamPath || '—')}</div></div>
      ${(lib?.folders || []).slice(1).map((f) => `<div class="cell"><div class="grow">Library folder</div><div class="val" style="user-select:text">${esc(f)}</div></div>`).join('')}
      <div class="cell"><div class="grow">Status</div><div class="val">${state.status.running ? activeAccount() ? `Signed in as ${esc(activeAccount().personaName)}` : 'Running, signed out' : 'Not running'}</div></div>
    </div>
    <div class="group-title">About</div>
    <div class="group">
      <div class="cell"><div class="grow">Hangar</div><div class="val">Version 1.0.0</div></div>
      <div class="cell"><div class="grow">Game data</div><div class="val">Steam client cache · Steam store</div></div>
    </div>
    <p class="footnote">Not affiliated with Valve. Steam and the Steam logo are trademarks of Valve Corporation.</p>
  </div>`;
}

// ---------- Events ----------
async function toggleFav(id) {
  id = Number(id);
  const favs = state.settings.favorites.includes(id) ? state.settings.favorites.filter((x) => x !== id) : [...state.settings.favorites, id];
  await saveSettings({ favorites: favs });
  renderChrome();
  const sheet = $('#sheet .sheet.game');
  if (sheet) openGame(sheet.dataset.appid, true);
  // Keep scroll position and avoid re-rendering the whole page for a star.
  document.querySelectorAll(`[data-action="fav"][data-id="${id}"]`).forEach((b) => {
    const on = favs.includes(id);
    b.classList.toggle('on', on);
    if (b.classList.contains('btn')) b.innerHTML = `${icon('star')}${on ? 'Favorited' : 'Favorite'}`;
  });
  if (state.page === 'favorites' || (state.page === 'home' && !sheet)) {
    const top = $('#content').scrollTop;
    const rows = [...document.querySelectorAll('.row')].map((r) => r.scrollLeft);
    renderPage();
    $('#content').scrollTop = top;
    document.querySelectorAll('.row').forEach((r, i) => { r.scrollLeft = rows[i] || 0; });
    updateAllRowNavs();
  }
}

const actions = {
  'win-close': () => appWindow.close(),
  'win-min': () => appWindow.minimize(),
  'win-max': () => appWindow.toggleMaximize(),
  refresh: () => load({ manual: true }),
  'dismiss-banner': () => { $('#banner').innerHTML = ''; },
  'close-sheet': closeSheet,
  picker: openPicker,
  switch: (el) => switchTo(el.dataset.name),
  'add-account': addAccount,
  slide: (el) => { state.slide = Number(el.dataset.index); renderHero(); },
  'hero-step': (el) => stepHero(Number(el.dataset.dir)),
  'row-scroll': (el) => {
    const row = el.parentElement.querySelector('.row');
    row.scrollBy({ left: Number(el.dataset.dir) * row.clientWidth * 0.85, behavior: 'smooth' });
  },
  game: (el) => openGame(el.dataset.id),
  play: (el) => { closeSheet(); play(el.dataset.id); },
  fav: (el) => toggleFav(el.dataset.id),
  open: (el) => invoke('open_url', { url: el.dataset.url }).catch((e) => banner('Couldn’t open link', String(e))),
  reveal: (el) => invoke('reveal', { path: el.dataset.path }).catch((e) => banner('Couldn’t open folder', String(e))),
  scope: async (el) => { await saveSettings({ scope: el.dataset.value }); renderChrome(); renderPage(); },
  view: async (el) => { await saveSettings({ view: el.dataset.value }); renderPage(); },
  'save-key': async () => {
    await saveSettings({ webApiKey: $('#api-key').value.trim() });
    banner('Key saved', 'Reloading your library from Steam…', 'ok');
    load({ manual: true });
  },
  'clear-favs': () => openSheet(`
    <div class="body" style="padding:24px 22px 8px">
      <h3 style="font:700 17px var(--font-display);margin:0 0 6px">Clear all favorites?</h3>
      <div style="color:var(--label-2)">${plural(state.settings.favorites.length, 'game')} will be removed from Favorites. The games stay in your library.</div>
    </div>
    <footer><button class="btn" data-action="close-sheet" autofocus>Cancel</button><button class="btn blue" style="background:var(--red)" data-action="do-clear-favs">Clear</button></footer>`, { cls: 'small' }),
  'do-clear-favs': async () => { closeSheet(); await saveSettings({ favorites: [] }); renderChrome(); renderPage(); }
};

document.addEventListener('click', (e) => {
  if (e.target.id === 'sheet') { closeSheet(); return; }
  const a = e.target.closest('[data-action]');
  if (a && !a.disabled) { e.stopPropagation(); actions[a.dataset.action]?.(a); return; }
  const nav = e.target.closest('[data-page]');
  if (nav) { closeSheet(); navigate(nav.dataset.page); }
});

document.addEventListener('change', async (e) => {
  const key = e.target.dataset.setting;
  if (key) {
    await saveSettings({ [key]: e.target.checked });
    renderChrome();
    if (key === 'showSoftware') renderPage();
  }
  const sel = e.target.dataset.settingSelect;
  if (sel) { await saveSettings({ [sel]: e.target.value }); renderPage(); }
});

let inputTimer = null;
document.addEventListener('input', (e) => {
  if (e.target.id === 'search-input') {
    state.query = e.target.value;
    clearTimeout(inputTimer);
    inputTimer = setTimeout(runSearch, 120);
  }
  if (e.target.id === 'filter-input') {
    state.filter = e.target.value;
    clearTimeout(inputTimer);
    inputTimer = setTimeout(() => {
      renderPage();
      const input = $('#filter-input');
      input.focus();
      input.setSelectionRange(input.value.length, input.value.length);
    }, 120);
  }
});

document.addEventListener('keydown', (e) => {
  if (e.key === 'Escape') closeSheet();
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'r') { e.preventDefault(); load({ manual: true }); }
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'f') { e.preventDefault(); closeSheet(); navigate('search'); }
  if (state.page === 'home' && $('#sheet').hidden && document.activeElement?.tagName !== 'INPUT' && (e.key === 'ArrowLeft' || e.key === 'ArrowRight')) {
    stepHero(e.key === 'ArrowRight' ? 1 : -1);
  }
});
document.addEventListener('contextmenu', (e) => { if (!e.target.closest('input, .val, .note')) e.preventDefault(); });

// ---------- Boot ----------
(async function boot() {
  document.querySelectorAll('[data-icon]').forEach((el) => { el.outerHTML = icon(el.dataset.icon); });
  try {
    state.settings = await invoke('settings_get');
  } catch {
    state.settings = { favorites: [], showSteamWindow: false, offlineMode: false, webApiKey: '', view: 'grid', sort: 'name', scope: 'account', showSoftware: true };
  }
  renderChrome();
  renderPage();
  await load();
  setInterval(pollStatus, 3000);
  // Advance the hero like the TV app, unless the pointer is resting on it.
  setInterval(() => {
    const hero = $('#hero');
    if (!hero || hero.matches(':hover') || !$('#sheet').hidden || document.hidden) return;
    stepHero(1);
  }, 9000);
  // Pick up installs and uninstalls made in Steam while Hangar was in the background.
  window.addEventListener('focus', () => { if (Date.now() - (state.loadedAt || 0) > 20000) load(); });
})();
