/* MUSICO — 3D cassette scene (procedural, three.js ES module) */
import * as THREE from 'three';

const INK = 0x16130e;

function roundedRect(target, w, h, r, cx = 0, cy = 0) {
  const x = cx - w / 2, y = cy - h / 2;
  target.moveTo(x + r, y);
  target.lineTo(x + w - r, y);
  target.absarc(x + w - r, y + r, r, -Math.PI / 2, 0);
  target.lineTo(x + w, y + h - r);
  target.absarc(x + w - r, y + h - r, r, 0, Math.PI / 2);
  target.lineTo(x + r, y + h);
  target.absarc(x + r, y + h - r, r, Math.PI / 2, Math.PI);
  target.lineTo(x, y + r);
  target.absarc(x + r, y + r, r, Math.PI / 2, Math.PI * 1.5);
  target.closePath();
}

/* ── label artwork, drawn on canvas → texture ─────────────────── */
const W = 1024, H = 645, PX = 102.4; // px per world unit
const WIN = { x: 154, y: 159, w: 716, h: 317, r: 40 }; // transparent window (matches 3D hole)

const C = {
  paper: '#f2ede3', ink: '#16130e', accent: '#ff4d00', violet: '#6a5cd8',
};

function labelCanvas() {
  const cv = document.createElement('canvas');
  cv.width = W; cv.height = H;
  return cv;
}

function shellBase(ctx) {
  ctx.clearRect(0, 0, W, H);
  // label paper with rounded corners (shell corners peek through)
  ctx.fillStyle = C.paper;
  roundRectPath(ctx, 3, 3, W - 6, H - 6, 44);
  ctx.fill();
  ctx.strokeStyle = 'rgba(22,19,14,0.28)';
  ctx.lineWidth = 2;
  roundRectPath(ctx, 10, 10, W - 20, H - 20, 38);
  ctx.stroke();

  // corner screws
  [[46, 46], [978, 46], [46, 599], [978, 599]].forEach(([x, y]) => {
    ctx.strokeStyle = 'rgba(22,19,14,0.8)';
    ctx.lineWidth = 3;
    ctx.beginPath(); ctx.arc(x, y, 13, 0, Math.PI * 2); ctx.stroke();
    ctx.beginPath(); ctx.moveTo(x - 7, y - 7); ctx.lineTo(x + 7, y + 7); ctx.stroke();
  });

  // capstan cut-outs, bottom center
  ctx.fillStyle = 'rgba(22,19,14,0.88)';
  trap(ctx, 452, 600, 470, 645); trap(ctx, 574, 600, 592, 645);
  ctx.beginPath(); ctx.arc(512, 622, 9, 0, Math.PI * 2); ctx.fill();

  // window: punch transparent so the 3D reels show through, then print the surround
  ctx.clearRect(WIN.x + 3, WIN.y + 3, WIN.w - 6, WIN.h - 6);
  ctx.strokeStyle = C.ink;
  ctx.lineWidth = 4;
  roundRectPath(ctx, WIN.x, WIN.y, WIN.w, WIN.h, WIN.r);
  ctx.stroke();
  ctx.strokeStyle = 'rgba(22,19,14,0.18)';
  ctx.lineWidth = 2;
  roundRectPath(ctx, WIN.x + 9, WIN.y + 9, WIN.w - 18, WIN.h - 18, WIN.r - 8);
  ctx.stroke();
}

function trap(ctx, x1, y1, x2, y2) {
  ctx.beginPath();
  ctx.moveTo(x1 + 6, y1); ctx.lineTo(x2 - 6, y1);
  ctx.lineTo(x2, y2); ctx.lineTo(x1, y2);
  ctx.closePath(); ctx.fill();
}

function roundRectPath(ctx, x, y, w, h, r) {
  ctx.beginPath();
  ctx.moveTo(x + r, y);
  ctx.arcTo(x + w, y, x + w, y + h, r);
  ctx.arcTo(x + w, y + h, x, y + h, r);
  ctx.arcTo(x, y + h, x, y, r);
  ctx.arcTo(x, y, x + w, y, r);
  ctx.closePath();
}

function sideDot(ctx, x, y, letter, color) {
  ctx.fillStyle = color;
  ctx.beginPath(); ctx.arc(x, y, 54, 0, Math.PI * 2); ctx.fill();
  ctx.fillStyle = C.paper;
  ctx.font = '62px Anton, sans-serif';
  ctx.textAlign = 'center'; ctx.textBaseline = 'middle';
  ctx.fillText(letter, x, y + 4);
  ctx.textAlign = 'left'; ctx.textBaseline = 'alphabetic';
}

function drawFront(cv) {
  const ctx = cv.getContext('2d');
  shellBase(ctx);

  ctx.fillStyle = C.ink;
  ctx.font = '46px Anton, sans-serif';
  ctx.fillText('MUSICO', 66, 80);
  ctx.font = '700 15px "Space Mono", monospace';
  ctx.fillStyle = 'rgba(22,19,14,0.62)';
  ctx.fillText('STEREO CASSETTE · 90 MIN', 68, 112);

  sideDot(ctx, 880, 92, 'A', C.accent);

  ctx.fillStyle = C.ink;
  ctx.font = '600 44px Caveat, cursive';
  ctx.fillText('your mix · your machine', 66, 140);

  // transport glyph print, under window
  ctx.fillStyle = 'rgba(22,19,14,0.4)';
  ctx.font = '700 17px "Space Mono", monospace';
  ctx.fillText('REW ◀◀   PLAY ▶   FF ▶▶   STOP ■   PAUSE ▌▌', 316, 512);

  ctx.fillStyle = C.ink;
  ctx.font = '700 17px "Space Mono", monospace';
  ctx.fillText('STEREO ● 90 MIN ● NORMAL BIAS ● CrO₂-FREE', 66, 552);
  ctx.fillStyle = 'rgba(22,19,14,0.6)';
  ctx.fillText('RECORDED ON-DEVICE — NO CLOUD, NO SERVERS', 66, 580);
  ctx.font = '700 13px "Space Mono", monospace';
  ctx.fillText('℗ 2026 ARYAN TAPES', 66, 622);
  ctx.textAlign = 'right';
  ctx.fillText('MADE WITH RUST + THREE.JS', 958, 622);
  ctx.textAlign = 'left';
}

function drawBack(cv) {
  const ctx = cv.getContext('2d');
  shellBase(ctx);

  ctx.fillStyle = C.ink;
  ctx.font = '46px Anton, sans-serif';
  ctx.fillText('MUSICO', 66, 80);
  ctx.font = '700 15px "Space Mono", monospace';
  ctx.fillStyle = 'rgba(22,19,14,0.62)';
  ctx.fillText('DOLBY-FREE · HONEST SOUND', 68, 112);

  sideDot(ctx, 880, 92, 'B', C.violet);

  ctx.fillStyle = C.ink;
  ctx.font = '600 44px Caveat, cursive';
  ctx.fillText('flip it — the spec sheet', 66, 140);

  // tracklist, two columns under the window — B1–B3 left, B4–B6 right
  const tracks = [
    ['B1', 'SMART RADIO', '3:42'],
    ['B2', 'RUST ENGINE', '4:05'],
    ['B3', 'INSTANT LIBRARY', '5:14'],
    ['B4', 'SYNCED LYRICS', '2:58'],
    ['B5', 'EQ + REPLAYGAIN', '3:21'],
    ['B6', 'MPRIS NATIVE', '4:47'],
  ];
  ctx.font = '700 19px "Space Mono", monospace';
  ctx.fillStyle = C.ink;
  const rows = [524, 560, 596];
  tracks.forEach(([num, name, dur], i) => {
    const col = Math.floor(i / 3), row = rows[i % 3];
    const x = col === 0 ? 66 : 554, right = col === 0 ? 470 : 958;
    ctx.textAlign = 'left';
    ctx.fillText(`${num} ${name}`, x, row);
    ctx.textAlign = 'right';
    ctx.fillText(dur, right, row);
    ctx.strokeStyle = 'rgba(22,19,14,0.3)';
    ctx.lineWidth = 1.5;
    ctx.setLineDash([2, 5]);
    ctx.beginPath(); ctx.moveTo(x + 205, row - 5); ctx.lineTo(right - 52, row - 5); ctx.stroke();
    ctx.setLineDash([]);
  });
  ctx.textAlign = 'center';
  ctx.fillStyle = 'rgba(22,19,14,0.6)';
  ctx.font = '700 13px "Space Mono", monospace';
  ctx.fillText('NO CLOUD ● NO ACCOUNTS ● NO TELEMETRY', 512, 634);
  ctx.textAlign = 'left';
}

/* ── scene ────────────────────────────────────────────────────── */
export function createScene(canvas) {
  const reduced = matchMedia('(prefers-reduced-motion: reduce)').matches;
  const finePointer = matchMedia('(pointer: fine)').matches;

  const renderer = new THREE.WebGLRenderer({ canvas, antialias: true, alpha: true });
  renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
  renderer.toneMapping = THREE.ACESFilmicToneMapping;
  renderer.toneMappingExposure = 1.1;

  const scene = new THREE.Scene();
  const camera = new THREE.PerspectiveCamera(35, 1, 0.1, 80);

  scene.add(new THREE.HemisphereLight(0xfff6e6, 0x453f33, 0.9));
  const key = new THREE.DirectionalLight(0xffffff, 1.7);
  key.position.set(4, 6, 7);
  scene.add(key);
  const rim = new THREE.DirectionalLight(0xffe2c4, 0.9);
  rim.position.set(-6, -3, -5);
  scene.add(rim);
  const warm = new THREE.PointLight(0xff4d00, 10, 24, 2);
  warm.position.set(-3, -3.5, 3.5);
  scene.add(warm);

  /* cassette group */
  const cassette = new THREE.Group();
  scene.add(cassette);

  const capMat = new THREE.MeshStandardMaterial({ color: 0x26242e, roughness: 0.46, metalness: 0.22 });
  const sideMat = new THREE.MeshStandardMaterial({ color: 0x111017, roughness: 0.62, metalness: 0.1 });

  const shell = new THREE.Shape();
  roundedRect(shell, 10, 6.3, 0.42);
  const holePath = new THREE.Path();
  roundedRect(holePath, 7.0, 3.1, 0.3, 0, 0.05);
  shell.holes.push(holePath);

  const bodyGeo = new THREE.ExtrudeGeometry(shell, {
    depth: 0.85, bevelEnabled: true, bevelThickness: 0.07, bevelSize: 0.06, bevelSegments: 2, curveSegments: 14,
  });
  bodyGeo.translate(0, 0, -0.425);
  cassette.add(new THREE.Mesh(bodyGeo, [capMat, sideMat]));

  /* labels */
  const frontCv = labelCanvas(), backCv = labelCanvas();
  drawFront(frontCv); drawBack(backCv);
  const frontTex = new THREE.CanvasTexture(frontCv);
  const backTex = new THREE.CanvasTexture(backCv);
  [frontTex, backTex].forEach((t) => {
    t.colorSpace = THREE.SRGBColorSpace;
    t.anisotropy = renderer.capabilities.getMaxAnisotropy();
  });
  const labelMat = { transparent: true, roughness: 0.85, metalness: 0, emissive: 0xffffff, emissiveIntensity: 0.3 };
  const frontLabel = new THREE.Mesh(
    new THREE.PlaneGeometry(10, 6.3),
    new THREE.MeshStandardMaterial({ map: frontTex, emissiveMap: frontTex, ...labelMat }),
  );
  frontLabel.position.z = 0.505;
  cassette.add(frontLabel);
  const backLabel = new THREE.Mesh(
    new THREE.PlaneGeometry(10, 6.3),
    new THREE.MeshStandardMaterial({ map: backTex, emissiveMap: backTex, ...labelMat }),
  );
  backLabel.position.z = -0.505;
  backLabel.rotation.y = Math.PI;
  cassette.add(backLabel);

  /* interior (DoubleSide — visible through the window from both sides) */
  const cavity = new THREE.Mesh(
    new THREE.PlaneGeometry(7.1, 3.2),
    new THREE.MeshBasicMaterial({ color: 0x14110c, side: THREE.DoubleSide }),
  );
  cavity.position.z = -0.24;
  cassette.add(cavity);

  /* tape spools + hubs */
  const tapeMat = new THREE.MeshStandardMaterial({ color: 0x2b2118, roughness: 0.88 });
  const tapeGeo = new THREE.CylinderGeometry(1, 1, 0.55, 36);
  tapeGeo.rotateX(Math.PI / 2);
  const tapeL = new THREE.Mesh(tapeGeo, tapeMat);
  const tapeR = new THREE.Mesh(tapeGeo, tapeMat);
  tapeL.position.set(-2.2, 0.05, 0);
  tapeR.position.set(2.2, 0.05, 0);
  cassette.add(tapeL, tapeR);

  const hubMat = new THREE.MeshStandardMaterial({ color: 0xe9e3d4, roughness: 0.5 });
  const discGeo = new THREE.CircleGeometry(0.74, 32);
  const toothGeo = new THREE.BoxGeometry(0.13, 0.36, 0.12);
  const hubGeo = new THREE.TorusGeometry(0.62, 0.075, 8, 28);

  function makeReel(x) {
    const g = new THREE.Group();
    g.position.set(x, 0.05, 0);
    const disc = new THREE.Mesh(discGeo, hubMat);
    disc.position.z = 0.24;
    g.add(disc);
    for (let i = 0; i < 6; i++) {
      const tooth = new THREE.Mesh(toothGeo, hubMat);
      const a = (i / 6) * Math.PI * 2;
      tooth.position.set(Math.cos(a) * 0.46, Math.sin(a) * 0.46, 0.3);
      tooth.rotation.z = a;
      g.add(tooth);
    }
    const hub = new THREE.Mesh(hubGeo, hubMat);
    hub.position.z = 0.3;
    g.add(hub);
    cassette.add(g);
    return g;
  }
  const reelL = makeReel(-2.2);
  const reelR = makeReel(2.2);

  /* glass sheen over window */
  const sheenCv = document.createElement('canvas');
  sheenCv.width = 256; sheenCv.height = 128;
  {
    const sctx = sheenCv.getContext('2d');
    const grad = sctx.createLinearGradient(0, 0, 256, 128);
    grad.addColorStop(0, 'rgba(255,255,255,0)');
    grad.addColorStop(0.35, 'rgba(255,255,255,0.16)');
    grad.addColorStop(0.5, 'rgba(255,255,255,0.03)');
    grad.addColorStop(0.72, 'rgba(255,255,255,0.10)');
    grad.addColorStop(1, 'rgba(255,255,255,0)');
    sctx.fillStyle = grad;
    sctx.fillRect(0, 0, 256, 128);
  }
  const sheen = new THREE.Mesh(
    new THREE.PlaneGeometry(7.1, 3.2),
    new THREE.MeshBasicMaterial({ map: new THREE.CanvasTexture(sheenCv), transparent: true, depthWrite: false }),
  );
  sheen.position.z = 0.525;
  cassette.add(sheen);

  /* redraw labels once real fonts arrive */
  if (document.fonts) {
    Promise.all([
      document.fonts.load('46px Anton'),
      document.fonts.load('600 44px Caveat'),
      document.fonts.load('700 17px "Space Mono"'),
    ]).then(() => {
      drawFront(frontCv); drawBack(backCv);
      frontTex.needsUpdate = true;
      backTex.needsUpdate = true;
    }).catch(() => {});
  }

  /* ── scroll choreography ────────────────────────────────────── */
  // p, x, y, rz (facing rotY), rx, s (scale), flip (0 = side A, 1 = side B)
  const KF = [
    { p: 0.00, x: 2.1,  y: -0.15, rz: -0.30, rx: 0.05,  s: 0.78, flip: 0 },
    { p: 0.20, x: 3.3,  y: 0.10,  rz: 0.16,  rx: -0.05, s: 0.70, flip: 0.07 },
    { p: 0.44, x: -3.7, y: 0.05,  rz: -0.42, rx: 0.06,  s: 0.72, flip: 1 },
    { p: 0.60, x: -3.9, y: 0.00,  rz: 0.12,  rx: -0.03, s: 0.66, flip: 1 },
    { p: 0.74, x: 0.0,  y: 0.25,  rz: 0.06,  rx: -0.06, s: 0.62, flip: 1 },
    { p: 0.90, x: 0.0,  y: -0.20, rz: -0.26, rx: 0.05,  s: 0.80, flip: 0 },
    { p: 1.00, x: 0.0,  y: -0.35, rz: -0.30, rx: 0.05,  s: 0.78, flip: 0 },
  ];
  const smooth = (t) => t * t * t * (t * (t * 6 - 15) + 10);
  const lerp = (a, b, t) => a + (b - a) * t;

  function samplePose(p) {
    let i = 0;
    while (i < KF.length - 2 && p > KF[i + 1].p) i++;
    const a = KF[i], b = KF[i + 1];
    const t = smooth(Math.min(1, Math.max(0, (p - a.p) / (b.p - a.p || 1))));
    return {
      x: lerp(a.x, b.x, t), y: lerp(a.y, b.y, t),
      rz: lerp(a.rz, b.rz, t), rx: lerp(a.rx, b.rx, t),
      s: lerp(a.s, b.s, t), flip: lerp(a.flip, b.flip, t),
    };
  }

  /* ── state ──────────────────────────────────────────────────── */
  const state = { progress: 0, mx: 0, my: 0 };
  let playing = false;
  let reelSpeed = 0, tapeK = 0, tapeDir = 1;

  /* drag to rotate */
  const drag = { cur: { x: 0, y: 0 }, target: { x: 0, y: 0 }, down: false, moved: 0, lx: 0, ly: 0 };
  const INTERACTIVE = 'a, button, input, textarea, select, kbd, code, [data-no-drag]';
  if (finePointer && !reduced) {
    addEventListener('pointerdown', (e) => {
      if (e.target.closest(INTERACTIVE)) return;
      drag.down = true; drag.moved = 0;
      drag.lx = e.clientX; drag.ly = e.clientY;
    });
    addEventListener('pointermove', (e) => {
      if (!drag.down) return;
      const dx = e.clientX - drag.lx, dy = e.clientY - drag.ly;
      drag.lx = e.clientX; drag.ly = e.clientY;
      drag.moved += Math.abs(dx) + Math.abs(dy);
      drag.target.y = THREE.MathUtils.clamp(drag.target.y + dx * 0.006, -1.6, 1.6);
      drag.target.x = THREE.MathUtils.clamp(drag.target.x + dy * 0.0035, -0.55, 0.55);
    });
    addEventListener('pointerup', () => { drag.down = false; });
  }

  /* ── sizing ─────────────────────────────────────────────────── */
  let aspect = 1;
  function resize() {
    const w = innerWidth, h = innerHeight;
    aspect = w / h;
    camera.aspect = aspect;
    camera.position.z = Math.min(26, Math.max(13, 13 * 1.5 / aspect));
    camera.updateProjectionMatrix();
    renderer.setSize(w, h, false);
  }
  resize();
  addEventListener('resize', resize);

  /* ── loop ───────────────────────────────────────────────────── */
  const clock = new THREE.Clock();
  let paused = false;
  let started = false;               // rendering waits for the preloader
  document.addEventListener('visibilitychange', () => { paused = document.hidden; });

  // adaptive resolution + fps cap: survive weak/software GPUs
  const basePR = Math.min(devicePixelRatio, 2);
  let curPR = basePR;
  let emaDt = 16, frames = 0, lastRender = 0;

  function frame() {
    requestAnimationFrame(frame);
    if (paused || !started) return;
    const now = clock.getElapsedTime();
    if (now - lastRender < 1 / 32) return;   // ≤32fps is plenty for slow motion
    const dt = Math.min(clock.getDelta(), 0.05);
    lastRender = now;

    emaDt = emaDt * 0.95 + (dt * 1000) * 0.05;
    if (++frames % 90 === 0 && emaDt > 42 && curPR > 0.55) {
      curPR = Math.max(0.55, curPR * 0.8);
      renderer.setPixelRatio(curPR);
      renderer.setSize(innerWidth, innerHeight, false);
    }

    const t = clock.elapsedTime;

    // pose from scroll (+ narrow-screen adjustments)
    const pose = samplePose(state.progress);
    const narrow = aspect < 0.9;
    const px = narrow ? pose.x * 0.22 : pose.x;
    const py = narrow ? pose.y + 0.35 : pose.y;
    const ps = pose.s * (narrow ? 0.5 : 1);

    // drag easing
    if (!drag.down) {
      drag.target.x *= 0.94; drag.target.y *= 0.94;
    }
    drag.cur.x = lerp(drag.cur.x, drag.target.x, 0.09);
    drag.cur.y = lerp(drag.cur.y, drag.target.y, 0.09);

    const sway = reduced ? 0 : Math.sin(t * 0.45) * 0.045;
    const bob = reduced ? 0 : Math.sin(t * 0.85) * 0.09;

    cassette.position.x = lerp(cassette.position.x, px, 0.08);
    cassette.position.y = lerp(cassette.position.y, py + bob, 0.08);
    cassette.scale.setScalar(lerp(cassette.scale.x, ps, 0.08));
    cassette.rotation.y = lerp(cassette.rotation.y, pose.rz + pose.flip * Math.PI + drag.cur.y + sway, 0.09);
    cassette.rotation.x = lerp(cassette.rotation.x, pose.rx + drag.cur.x, 0.09);
    cassette.rotation.z = lerp(cassette.rotation.z, Math.sin(t * 0.35) * 0.012, 0.05);

    // reels + tape transfer
    const targetSpeed = reduced ? 0 : (playing ? 3.6 : 0.02);
    reelSpeed = lerp(reelSpeed, targetSpeed, 0.04);
    reelL.rotation.z -= reelSpeed * dt;
    reelR.rotation.z -= reelSpeed * dt * 1.15;
    tapeK += reelSpeed * dt * 0.006 * tapeDir;
    if (tapeK > 1) { tapeK = 1; tapeDir = -1; }
    if (tapeK < 0) { tapeK = 0; tapeDir = 1; }
    const leftR = 1.42 - 0.57 * tapeK;
    const rightR = 0.85 + 0.57 * tapeK;
    tapeL.scale.set(leftR, leftR, 1);
    tapeR.scale.set(rightR, rightR, 1);

    // play vibration
    if (playing && !reduced) cassette.position.y += Math.sin(t * 42) * 0.004;

    // subtle mouse parallax on camera
    if (!reduced) {
      camera.position.x = lerp(camera.position.x, state.mx * 0.35, 0.05);
      camera.position.y = lerp(camera.position.y, -state.my * 0.25, 0.05);
      camera.lookAt(0, 0, 0);
    }

    renderer.render(scene, camera);
  }
  frame();

  return {
    start() { started = true; },
    setProgress(p) { state.progress = Math.min(1, Math.max(0, p)); },
    setMouse(x, y) { state.mx = x; state.my = y; },
    setPlaying(v) { playing = v; },
  };
}
