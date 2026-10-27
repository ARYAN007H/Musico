/* MUSICO — Side A · page interactions */
import { createScene } from './scene.js';

const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
const finePointer = matchMedia('(pointer: fine)').matches;

/* ── 3D scene (booted lazily — after the preloader lifts) ─────── */
const canvas = document.getElementById('stage');
let scene = null;
function bootScene() {
  if (scene) return;
  try {
    scene = createScene(canvas);
    scene.setProgress(lastProgress);
    scene.start();
  } catch (err) {
    canvas.style.display = 'none';
    console.warn('WebGL unavailable — cassette offline.', err);
  }
}

/* scroll + mouse → scene */
let ticking = false;
let lastProgress = 0;
function readScroll() {
  ticking = false;
  const max = document.documentElement.scrollHeight - innerHeight;
  lastProgress = max > 0 ? scrollY / max : 0;
  if (scene) scene.setProgress(lastProgress);
}
addEventListener('scroll', () => {
  if (!ticking) { ticking = true; requestAnimationFrame(readScroll); }
}, { passive: true });
readScroll();

addEventListener('mousemove', (e) => {
  if (!scene || !finePointer) return;
  scene.setMouse((e.clientX / innerWidth) * 2 - 1, (e.clientY / innerHeight) * 2 - 1);
}, { passive: true });

/* debug/QA hook: ?scroll=<px> jumps to a position after load (instant) */
{
  const sp = new URLSearchParams(location.search).get('scroll');
  if (sp) addEventListener('load', () => scrollTo({ top: +sp, behavior: 'instant' }));
}

/* ── preloader ────────────────────────────────────────────────── */
const loader = document.getElementById('loader');
const pct = document.getElementById('loaderPct');
const fill = document.getElementById('loaderFill');
function finishLoader() {
  loader.classList.add('done');
  document.body.classList.add('loaded');
  document.body.style.overflow = '';
  requestAnimationFrame(() => setTimeout(bootScene, 60));
  setTimeout(() => loader.remove(), 900);
}
if (loader && !reduced) {
  document.body.style.overflow = 'hidden';
  let v = 0, holds = 0;
  const tick = setInterval(() => {
    if (v < 100) {
      v = Math.min(100, v + 4 + Math.random() * 9);
      pct.textContent = Math.floor(v) + '%';
      fill.style.width = v + '%';
      if (v >= 100) holds = 1;
    } else if (++holds === 5) {   // ~275ms hold at 100%, then lift — all in-interval
      clearInterval(tick);
      finishLoader();
    }
  }, 55);
} else if (loader) {
  loader.remove();
  finishLoader();
}

/* ── custom cursor ────────────────────────────────────────────── */
if (finePointer) {
  const dot = document.getElementById('cursorDot');
  const ring = document.getElementById('cursorRing');
  let rx = innerWidth / 2, ry = innerHeight / 2, tx = rx, ty = ry, seen = false;
  addEventListener('mousemove', (e) => {
    tx = e.clientX; ty = e.clientY;
    dot.style.transform = `translate(${tx}px, ${ty}px)`;
    if (!seen) { seen = true; dot.classList.add('active'); ring.classList.add('active'); }
  }, { passive: true });
  (function follow() {
    rx += (tx - rx) * 0.16;
    ry += (ty - ry) * 0.16;
    ring.style.transform = `translate(${rx}px, ${ry}px)${ring.classList.contains('hover') ? ' scale(1.7)' : ''}`;
    requestAnimationFrame(follow);
  })();
  document.addEventListener('mouseover', (e) => {
    ring.classList.toggle('hover', !!e.target.closest('a, button, [data-cursor]'));
  });
}

/* ── play the tape ────────────────────────────────────────────── */
const playBtn = document.getElementById('playBtn');
const playLabel = playBtn.querySelector('.play-label');
playBtn.addEventListener('click', () => {
  const on = playBtn.getAttribute('aria-pressed') !== 'true';
  playBtn.setAttribute('aria-pressed', String(on));
  playLabel.textContent = on ? 'PAUSE THE TAPE' : 'PLAY THE TAPE';
  document.body.classList.toggle('playing', on);
  if (scene) scene.setPlaying(on);
});

/* ── generative tape-hiss soundtrack (WebAudio, user-toggled) ─── */
let audio = null;
const soundToggle = document.getElementById('soundToggle');

function buildAudio() {
  const ctx = new (window.AudioContext || window.webkitAudioContext)();
  const master = ctx.createGain();
  master.gain.value = 0;
  master.connect(ctx.destination);

  // hiss: looped noise → lowpass
  const len = ctx.sampleRate * 2;
  const buf = ctx.createBuffer(1, len, ctx.sampleRate);
  const data = buf.getChannelData(0);
  let last = 0;
  for (let i = 0; i < len; i++) {
    const white = Math.random() * 2 - 1;
    last = 0.98 * last + 0.02 * white;
    data[i] = last * 3.2;
  }
  const noise = ctx.createBufferSource();
  noise.buffer = buf; noise.loop = true;
  const hissFilter = ctx.createBiquadFilter();
  hissFilter.type = 'lowpass'; hissFilter.frequency.value = 3800;
  const hissGain = ctx.createGain(); hissGain.gain.value = 0.05;
  noise.connect(hissFilter).connect(hissGain).connect(master);
  noise.start();

  // crackle: sparse pops
  setInterval(() => {
    if (Math.random() > 0.3) return;
    const pop = ctx.createBufferSource();
    const plen = Math.floor(ctx.sampleRate * 0.02);
    const pbuf = ctx.createBuffer(1, plen, ctx.sampleRate);
    const pd = pbuf.getChannelData(0);
    for (let i = 0; i < plen; i++) pd[i] = (Math.random() * 2 - 1) * (1 - i / plen);
    pop.buffer = pbuf;
    const bp = ctx.createBiquadFilter();
    bp.type = 'bandpass'; bp.frequency.value = 1800 + Math.random() * 1200;
    const g = ctx.createGain(); g.gain.value = 0.02 + Math.random() * 0.02;
    pop.connect(bp).connect(g).connect(master);
    pop.start();
  }, 320);

  // warm chord pad, slow progression
  const chords = [
    [130.81, 196.0, 246.94, 329.63],  // Cmaj7-ish
    [110.0, 164.81, 261.63, 329.63],  // Am-ish
    [87.31, 174.61, 261.63, 349.23],  // F-ish
    [98.0, 146.83, 246.94, 293.66],   // G-ish
  ];
  const padFilter = ctx.createBiquadFilter();
  padFilter.type = 'lowpass'; padFilter.frequency.value = 1200;
  const padGain = ctx.createGain(); padGain.gain.value = 0.06;
  padFilter.connect(padGain).connect(master);
  let ci = 0;
  function chord() {
    const t0 = ctx.currentTime;
    chords[ci % chords.length].forEach((f, i) => {
      const osc = ctx.createOscillator();
      osc.type = 'sine';
      osc.frequency.value = f * (i % 2 ? 1.003 : 0.997);
      const env = ctx.createGain();
      env.gain.setValueAtTime(0, t0);
      env.gain.linearRampToValueAtTime(0.9 / chords.length, t0 + 1.4);
      env.gain.setValueAtTime(0.9 / chords.length, t0 + 3.4);
      env.gain.linearRampToValueAtTime(0, t0 + 5.6);
      osc.connect(env).connect(padFilter);
      osc.start(t0); osc.stop(t0 + 5.8);
    });
    ci++;
  }
  chord();
  setInterval(chord, 4200);

  return { ctx, master };
}

soundToggle.addEventListener('click', () => {
  if (!audio) audio = buildAudio();
  const on = soundToggle.getAttribute('aria-pressed') !== 'true';
  soundToggle.setAttribute('aria-pressed', String(on));
  soundToggle.textContent = on ? 'SOUND: ON' : 'SOUND: OFF';
  if (audio.ctx.state === 'suspended') audio.ctx.resume();
  audio.master.gain.cancelScheduledValues(audio.ctx.currentTime);
  audio.master.gain.linearRampToValueAtTime(on ? 1 : 0, audio.ctx.currentTime + 0.6);
});

/* ── reveal on scroll ─────────────────────────────────────────── */
const revealEls = document.querySelectorAll('.side-head, .row, .spec-grid, .p-line, .p-note, .dl-title, .dl-actions, .dl-cmd');
revealEls.forEach((el, i) => {
  el.classList.add('reveal');
  el.style.setProperty('--d', String(i % 4));
});
if (!reduced && 'IntersectionObserver' in window) {
  const io = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('in');
        io.unobserve(entry.target);
      }
    });
  }, { threshold: 0.15, rootMargin: '0px 0px -60px 0px' });
  revealEls.forEach((el) => io.observe(el));
} else {
  revealEls.forEach((el) => el.classList.add('in'));
}
