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
    // when the reel-intro lands, release the hero text (removes .intro-waiting)
    scene.start(() => document.body.classList.remove('intro-waiting'));
    if (QA_INTRO_K !== null) {
      scene.debugHoldIntro(QA_INTRO_K);
      document.body.classList.remove('intro-waiting');
    }
  } catch (err) {
    canvas.style.display = 'none';
    document.body.classList.remove('intro-waiting');
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

/* debug/QA hooks: ?scroll=<px> jumps instantly; ?qa&introk=<0..1> freezes the intro */
const QA_INTRO_K = (() => {
  const q = new URLSearchParams(location.search);
  const sp = q.get('scroll');
  if (sp) addEventListener('load', () => scrollTo({ top: +sp, behavior: 'instant' }));
  return q.has('qa') ? parseFloat(q.get('introk') ?? '0.35') : null;
})();

/* ── preloader ────────────────────────────────────────────────── */
const loader = document.getElementById('loader');
const pct = document.getElementById('loaderPct');
const fill = document.getElementById('loaderFill');
function finishLoader() {
  loader.classList.add('done');
  document.body.classList.add('loaded');
  if (!reduced) document.body.classList.add('intro-waiting');
  document.body.style.overflow = '';
  requestAnimationFrame(() => setTimeout(bootScene, 60));
  setTimeout(() => loader.remove(), 900);
}
if (loader && !reduced && QA_INTRO_K === null) {
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

/* ── generative lo-fi soundtrack (WebAudio, user-toggled) ───────
   slow beat + warm chords, a faint tape hiss, and a mechanical
   reel buzz that fades in only while the tape is playing        */
let audio = null;
const soundToggle = document.getElementById('soundToggle');

function buildAudio() {
  const ctx = new (window.AudioContext || window.webkitAudioContext)();
  const master = ctx.createGain();
  master.gain.value = 0;
  const glue = ctx.createDynamicsCompressor();
  glue.threshold.value = -20; glue.ratio.value = 4; glue.knee.value = 12;
  const warm = ctx.createBiquadFilter();
  warm.type = 'lowpass'; warm.frequency.value = 3400;
  master.connect(glue); glue.connect(warm); warm.connect(ctx.destination);

  /* shared noise buffer for hiss / drums / crackle */
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
    const g = ctx.createGain(); g.gain.value = 0.014;
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
    buzzGain.gain.linearRampToValueAtTime(on ? 0.02 : 0, ctx.currentTime + 0.9);
  };

  /* sparse crackle */
  setInterval(() => {
    if (Math.random() > 0.25) return;
    noiseHit(ctx.currentTime, 0.025, 'bandpass', 1600 + Math.random() * 1400, 2, 0.012 + Math.random() * 0.012);
  }, 340);

  /* ── the beat: 74 BPM, 4-bar loop, swung 16ths ───────────────── */
  const BPM = 74, STEP = 60 / BPM / 4, BAR = STEP * 16, SWING = STEP * 0.55;
  const CHORDS = [ // Fmaj7 → Em7 → Dm7 → Cmaj7 (warm descent)
    { pad: [174.61, 220.0, 261.63, 329.63], bass: 87.31 },
    { pad: [164.81, 196.0, 246.94, 293.66], bass: 82.41 },
    { pad: [146.83, 174.61, 220.0, 261.63], bass: 73.42 },
    { pad: [130.81, 164.81, 196.0, 246.94], bass: 65.41 },
  ];
  const padBus = ctx.createBiquadFilter();
  padBus.type = 'lowpass'; padBus.frequency.value = 1100;
  const padGain = ctx.createGain(); padGain.gain.value = 0.9;
  padBus.connect(padGain).connect(master);

  const kick = (when) => {
    const o = ctx.createOscillator();
    o.type = 'sine';
    o.frequency.setValueAtTime(110, when);
    o.frequency.exponentialRampToValueAtTime(42, when + 0.11);
    const g = ctx.createGain();
    g.gain.setValueAtTime(0.5, when);
    g.gain.exponentialRampToValueAtTime(0.001, when + 0.3);
    o.connect(g).connect(master);
    o.start(when); o.stop(when + 0.32);
  };
  const rim = (when) => noiseHit(when, 0.09, 'bandpass', 1750, 1.2, 0.09);
  const hat = (when, vel) => noiseHit(when, 0.035, 'highpass', 7600, 0.7, 0.022 * vel);
  const bassNote = (when, freq, len) => {
    const o = ctx.createOscillator();
    o.type = 'triangle'; o.frequency.value = freq;
    const f = ctx.createBiquadFilter();
    f.type = 'lowpass'; f.frequency.value = 320;
    const g = ctx.createGain();
    g.gain.setValueAtTime(0, when);
    g.gain.linearRampToValueAtTime(0.15, when + 0.03);
    g.gain.setValueAtTime(0.15, when + len * 0.7);
    g.gain.linearRampToValueAtTime(0, when + len);
    o.connect(f).connect(g).connect(master);
    o.start(when); o.stop(when + len + 0.05);
  };
  const padChord = (when, notes) => {
    notes.forEach((freq, i) => {
      [0.997, 1.004].forEach((detune, j) => {
        const o = ctx.createOscillator();
        o.type = j ? 'sine' : 'triangle';
        o.frequency.value = freq * detune;
        const g = ctx.createGain();
        const v = 0.05 / notes.length * (j ? 0.7 : 1);
        g.gain.setValueAtTime(0, when);
        g.gain.linearRampToValueAtTime(v, when + 0.5);
        g.gain.setValueAtTime(v, when + BAR - 0.9);
        g.gain.linearRampToValueAtTime(0, when + BAR);
        o.connect(g).connect(padBus);
        o.start(when + i * 0.012); o.stop(when + BAR + 0.1);
      });
    });
  };

  const KICK  = [1,0,0,0, 0,0,0,1, 0,0,0,0, 1,0,0,0];
  const RIM   = [0,0,0,0, 1,0,0,0, 0,0,0,0, 1,0,0,0];
  let step = 0, nextTime = ctx.currentTime + 0.1;
  setInterval(() => {
    if (nextTime < ctx.currentTime - 0.15) nextTime = ctx.currentTime + 0.05; // stall-proof
    while (nextTime < ctx.currentTime + 0.4) {
      const s16 = step % 16, bar = Math.floor(step / 16) % 4;
      const swing = (s16 % 4 === 2) ? SWING : 0;
      const at = nextTime + swing;
      const ch = CHORDS[bar];
      if (s16 === 0) padChord(nextTime, ch.pad);
      if (KICK[s16]) kick(at);
      if (RIM[s16]) rim(at);
      if (s16 % 2 === 0) hat(at, s16 % 4 === 0 ? 1 : 0.6);
      if (s16 === 0) bassNote(nextTime, ch.bass, STEP * 6);
      if (s16 === 10) bassNote(at, ch.bass * 1.5, STEP * 4);
      step++;
      nextTime += STEP;
    }
  }, 110);

  return { ctx, master, setReel };
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
