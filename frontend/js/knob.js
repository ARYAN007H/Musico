/* ══════════════════════════════════════════════════════════════════════════════
   MUSICO — Rotary Volume Knob Controller
   Click-drag to rotate, updates SVG arc and volume state
   ══════════════════════════════════════════════════════════════════════════════ */

const Knob = (() => {
  const container = document.getElementById('knob-container');
  const knob = document.getElementById('volume-knob');
  const notch = document.getElementById('knob-notch');
  const arc = document.getElementById('knob-arc');
  const label = document.getElementById('vol-label');
  const volIcon = document.getElementById('vol-icon');

  // Arc constants
  const ARC_TOTAL = 207; // Total dash length for 270° arc
  const MIN_ANGLE = -135; // Start angle (7 o'clock)
  const MAX_ANGLE = 135;  // End angle (5 o'clock)
  const RANGE = MAX_ANGLE - MIN_ANGLE; // 270°

  let isDragging = false;

  function init() {
    if (!knob) return;

    // Initial position
    setVolume(App.state.volume, false);

    // Mouse events
    knob.addEventListener('mousedown', startDrag);
    document.addEventListener('mousemove', onDrag);
    document.addEventListener('mouseup', endDrag);

    // Touch events
    knob.addEventListener('touchstart', startDrag, { passive: false });
    document.addEventListener('touchmove', onDrag, { passive: false });
    document.addEventListener('touchend', endDrag);

    // Scroll wheel
    container.addEventListener('wheel', onWheel, { passive: false });
  }

  function startDrag(e) {
    e.preventDefault();
    isDragging = true;
    knob.style.cursor = 'grabbing';
  }

  function onDrag(e) {
    if (!isDragging) return;
    e.preventDefault();

    const rect = container.getBoundingClientRect();
    const cx = rect.left + rect.width / 2;
    const cy = rect.top + rect.height / 2;

    const clientX = e.touches ? e.touches[0].clientX : e.clientX;
    const clientY = e.touches ? e.touches[0].clientY : e.clientY;

    // Calculate angle from center
    let angle = Math.atan2(clientX - cx, cy - clientY) * (180 / Math.PI);

    // Clamp to range
    angle = Math.max(MIN_ANGLE, Math.min(MAX_ANGLE, angle));

    // Convert angle to 0-1 volume
    const vol = (angle - MIN_ANGLE) / RANGE;
    setVolume(vol);
  }

  function endDrag() {
    isDragging = false;
    if (knob) knob.style.cursor = 'grab';
  }

  function onWheel(e) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -0.03 : 0.03;
    setVolume(Math.max(0, Math.min(1, App.state.volume + delta)));
  }

  function setVolume(vol, notify = true) {
    vol = Math.max(0, Math.min(1, vol));
    App.state.volume = vol;

    // Update knob rotation
    const angle = MIN_ANGLE + vol * RANGE;
    if (knob) knob.style.transform = `rotate(${angle}deg)`;

    // Update SVG arc fill
    if (arc) {
      const offset = ARC_TOTAL - (vol * ARC_TOTAL);
      arc.style.strokeDashoffset = offset;
    }

    // Update label
    if (label) label.textContent = `${Math.round(vol * 100)}%`;

    // Update volume icon
    if (volIcon) {
      const svg = volIcon.querySelector('svg');
      if (vol === 0) {
        svg.innerHTML = '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19"/><line x1="23" y1="9" x2="17" y2="15"/><line x1="17" y1="9" x2="23" y2="15"/>';
      } else if (vol < 0.5) {
        svg.innerHTML = '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19"/><path d="M15.54 8.46a5 5 0 010 7.08"/>';
      } else {
        svg.innerHTML = '<polygon points="11 5 6 9 2 9 2 15 6 15 11 19"/><path d="M19.07 4.93a10 10 0 010 14.14M15.54 8.46a5 5 0 010 7.08"/>';
      }
    }

    if (notify) {
      Bridge.invoke('set_volume', { volume: vol });
    }
  }

  document.addEventListener('DOMContentLoaded', init);

  return { setVolume };
})();
