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

/* ── soundtrack (WebAudio, user-toggled) ────────────────────────
   looped from sound/loop.mp3 — drop any mp3 in there (renamed to
   loop.mp3) and the site plays it. Defaults to "Gymnopédie No. 1"
   by Kevin MacLeod (incompetech.com, CC-BY 4.0), plus a faint tape
   hiss and a mechanical reel buzz that fades in only while the
   tape is playing                                                 */
let audio = null;
const soundToggle = document.getElementById('soundToggle');

function buildAudio() {
  const ctx = new (window.AudioContext || window.webkitAudioContext)();
  const master = ctx.createGain();
  master.gain.value = 0;
  const glue = ctx.createDynamicsCompressor();
  glue.threshold.value = -20; glue.ratio.value = 3.5; glue.knee.value = 14;
  const warm = ctx.createBiquadFilter();
  warm.type = 'lowpass'; warm.frequency.value = 14000;
  master.connect(glue); glue.connect(warm); warm.connect(ctx.destination);

  /* the music itself — swap sound/loop.mp3 to change it */
  const music = new Audio('sound/loop.mp3');
  music.loop = true;
  music.preload = 'auto';
  music.addEventListener('error', () => console.warn('sound/loop.mp3 missing — music off, ambience only.'));
  const musicGain = ctx.createGain();
  musicGain.gain.value = 0.6;
  ctx.createMediaElementSource(music).connect(musicGain).connect(master);

  /* shared noise buffer for hiss / crackle */
  const nlen = ctx.sampleRate * 2;
  const nbuf = ctx.createBuffer(1, nlen, ctx.sampleRate);
  const nd = nbuf.getChannelData(0);
  let last = 0;
  for (let i = 0; i < nlen; i++) {
    const w = Math.random() * 2 - 1;
    last = 0.98 * last + 0.02 * w;
    nd[i] = last * 3.2;
  }
  const noiseHit = (when, dur, type, freq, q, vol) => {
    const src = ctx.createBufferSource();
    src.buffer = nbuf;
    src.loop = true;
    const f = ctx.createBiquadFilter();
    f.type = type; f.frequency.value = freq; f.Q.value = q;
    const g = ctx.createGain();
    g.gain.setValueAtTime(vol, when);
    g.gain.exponentialRampToValueAtTime(0.001, when + dur);
    src.connect(f).connect(g).connect(master);
    src.start(when, Math.random() * 1.5);
    src.stop(when + dur + 0.05);
  };

  /* constant ultra-faint tape hiss */
  {
    const src = ctx.createBufferSource();
    src.buffer = nbuf; src.loop = true;
    const f = ctx.createBiquadFilter();
    f.type = 'lowpass'; f.frequency.value = 2800;
    const g = ctx.createGain(); g.gain.value = 0.012;
    src.connect(f).connect(g).connect(master);
    src.start();
  }

  /* reel buzz — mechanical whir, fades in only while playing */
  const buzzGain = ctx.createGain();
  buzzGain.gain.value = 0;
  {
    const osc = ctx.createOscillator();
    osc.type = 'sawtooth'; osc.frequency.value = 64;
    const am = ctx.createGain(); am.gain.value = 0.65;
    const lfo = ctx.createOscillator(); lfo.frequency.value = 7.2;
    const lfoG = ctx.createGain(); lfoG.gain.value = 0.35;
    lfo.connect(lfoG).connect(am.gain);
    const bp = ctx.createBiquadFilter();
    bp.type = 'bandpass'; bp.frequency.value = 150; bp.Q.value = 1.1;
    osc.connect(am).connect(bp).connect(buzzGain).connect(master);
    osc.start(); lfo.start();
  }
  const setReel = (on) => {
    buzzGain.gain.cancelScheduledValues(ctx.currentTime);
    buzzGain.gain.setValueAtTime(buzzGain.gain.value, ctx.currentTime);
    buzzGain.gain.linearRampToValueAtTime(on ? 0.018 : 0, ctx.currentTime + 0.9);
  };

  /* sparse crackle */
  setInterval(() => {
    if (Math.random() > 0.25) return;
    noiseHit(ctx.currentTime, 0.025, 'bandpass', 1600 + Math.random() * 1400, 2, 0.01 + Math.random() * 0.01);
  }, 340);

  return { ctx, master, setReel, music };
}

soundToggle.addEventListener('click', () => {
  if (!audio) audio = buildAudio();
  const on = soundToggle.getAttribute('aria-pressed') !== 'true';
  soundToggle.setAttribute('aria-pressed', String(on));
  soundToggle.textContent = on ? 'SOUND: ON' : 'SOUND: OFF';
  if (audio.ctx.state === 'suspended') audio.ctx.resume();
  if (on) audio.music.play().catch(() => {});
  else audio.music.pause();
  audio.master.gain.cancelScheduledValues(audio.ctx.currentTime);
  audio.master.gain.linearRampToValueAtTime(on ? 1 : 0, audio.ctx.currentTime + 0.8);
});

/* ── play the tape ────────────────────────────────────────────── */
const playBtn = document.getElementById('playBtn');
const playLabel = playBtn.querySelector('.play-label');
playBtn.addEventListener('click', () => {
  const on = playBtn.getAttribute('aria-pressed') !== 'true';
  playBtn.setAttribute('aria-pressed', String(on));
  playLabel.textContent = on ? 'PAUSE THE TAPE' : 'PLAY THE TAPE';
  document.body.classList.toggle('playing', on);
  if (scene) scene.setPlaying(on);
  if (audio) audio.setReel(on);
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
