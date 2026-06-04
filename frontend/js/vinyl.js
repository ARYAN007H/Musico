/* ══════════════════════════════════════════════════════════════════════════════
   MUSICO — Vinyl Disc Animation Controller
   Controls spin/pause of main and mini vinyl discs
   ══════════════════════════════════════════════════════════════════════════════ */

const Vinyl = (() => {
  const disc = document.getElementById('vinyl-disc');
  const tonearm = document.getElementById('tonearm');
  const miniVinyl = document.getElementById('mini-vinyl');

  function setPlaying(playing) {
    if (disc) {
      disc.classList.toggle('paused', !playing);
    }
    if (miniVinyl) {
      miniVinyl.classList.toggle('paused', !playing);
    }
    if (tonearm) {
      tonearm.classList.toggle('resting', !playing);
      tonearm.classList.toggle('playing', playing);
    }
  }

  function setAlbumArt(url) {
    const label = document.getElementById('vinyl-label');
    const fallback = document.getElementById('vinyl-fallback');
    const miniArt = document.getElementById('mini-art');

    if (url) {
      // Set main vinyl art
      let img = label.querySelector('img');
      if (!img) {
        img = document.createElement('img');
        label.appendChild(img);
      }
      img.src = url;
      img.alt = 'Album Art';
      if (fallback) fallback.style.display = 'none';

      // Set mini art
      if (miniArt) {
        let miniImg = miniArt.querySelector('img');
        if (!miniImg) {
          miniImg = document.createElement('img');
          miniArt.innerHTML = '';
          miniArt.appendChild(miniImg);
        }
        miniImg.src = url;
        miniImg.alt = 'Album Art';
      }
    } else {
      // Show fallback
      const img = label.querySelector('img');
      if (img) img.remove();
      if (fallback) fallback.style.display = '';
    }
  }

  return { setPlaying, setAlbumArt };
})();
