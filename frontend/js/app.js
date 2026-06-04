/* ══════════════════════════════════════════════════════════════════════════════
   MUSICO — Main Application Controller
   View routing, state management, UI population, event handlers
   ══════════════════════════════════════════════════════════════════════════════ */

const App = (() => {
  // ─── State ─────────────────────────────────────────────────────────────────
  const state = {
    currentView: 'now-playing',
    isPlaying: false,
    currentSong: null,
    position: 0,
    duration: 0,
    volume: 0.68,
    shuffleMode: 0, // 0=off, 1=shuffle, 2=smart
    repeatMode: 0,  // 0=off, 1=one, 2=all
    isLiked: false,
    library: [],
    queue: [],
    recommendations: [],
    sortField: 'title',
    sortAscending: true,
    viewMode: 'list',
    searchQuery: '',
    palette: 'nebula',
  };

  // ─── Initialization ────────────────────────────────────────────────────────
  async function init() {
    setupNavigation();
    setupSettingsTabs();
    setupSearch();
    setupSortControls();
    setupViewToggle();
    setupTransportControls();
    setupToggles();
    setupPaletteSwatches();

    // Load library
    state.library = await Bridge.invoke('get_library');
    renderSongList(state.library);
    renderAlbumGrid(state.library);
    generateWaveform();

    // Load recommendations
    state.recommendations = await Bridge.invoke('get_recommendations');
    renderRecommendations();

    // Update library count
    document.getElementById('lib-count').textContent = `${state.library.length} songs`;
  }

  // ─── Navigation ────────────────────────────────────────────────────────────
  function setupNavigation() {
    const navItems = document.querySelectorAll('.nav-item');
    navItems.forEach(item => {
      item.addEventListener('click', () => {
        const view = item.dataset.view;
        navigateTo(view);
      });
    });
  }

  function navigateTo(viewName) {
    state.currentView = viewName;

    // Update views
    document.querySelectorAll('.view').forEach(v => v.classList.remove('active'));
    const target = document.getElementById(`view-${viewName}`);
    if (target) target.classList.add('active');

    // Update nav items
    document.querySelectorAll('.nav-item').forEach(n => n.classList.remove('active'));
    const activeNav = document.querySelector(`.nav-item[data-view="${viewName}"]`);
    if (activeNav) activeNav.classList.add('active');

    // Show/hide mini player
    const miniPlayer = document.getElementById('mini-player');
    if (viewName === 'now-playing') {
      miniPlayer.classList.add('hidden');
    } else if (state.currentSong) {
      miniPlayer.classList.remove('hidden');
    }
  }

  // ─── Settings Tabs ─────────────────────────────────────────────────────────
  function setupSettingsTabs() {
    document.querySelectorAll('.settings-tab').forEach(tab => {
      tab.addEventListener('click', () => {
        document.querySelectorAll('.settings-tab').forEach(t => t.classList.remove('active'));
        tab.classList.add('active');

        document.querySelectorAll('.settings-panel').forEach(p => p.classList.remove('active'));
        const panel = document.getElementById(`panel-${tab.dataset.tab}`);
        if (panel) panel.classList.add('active');
      });
    });
  }

  // ─── Search ────────────────────────────────────────────────────────────────
  function setupSearch() {
    const input = document.getElementById('search-input');
    const bar = document.getElementById('search-bar');
    const clear = document.getElementById('search-clear');

    input.addEventListener('input', () => {
      state.searchQuery = input.value;
      bar.classList.toggle('has-query', input.value.length > 0);
      filterLibrary();
    });

    clear.addEventListener('click', () => {
      input.value = '';
      state.searchQuery = '';
      bar.classList.remove('has-query');
      filterLibrary();
    });
  }

  function filterLibrary() {
    const q = state.searchQuery.toLowerCase();
    const filtered = q
      ? state.library.filter(s =>
          s.title.toLowerCase().includes(q) ||
          s.artist.toLowerCase().includes(q) ||
          s.album.toLowerCase().includes(q)
        )
      : state.library;

    const sorted = sortSongs(filtered);
    renderSongList(sorted);
    renderAlbumGrid(sorted);

    document.getElementById('lib-count').textContent = q
      ? `${filtered.length} results`
      : `${state.library.length} songs`;
  }

  // ─── Sort ──────────────────────────────────────────────────────────────────
  function setupSortControls() {
    document.querySelectorAll('.sort-pill[data-sort]').forEach(pill => {
      pill.addEventListener('click', () => {
        const field = pill.dataset.sort;
        if (state.sortField === field) {
          state.sortAscending = !state.sortAscending;
        } else {
          state.sortField = field;
          state.sortAscending = true;
        }

        // Update pills
        document.querySelectorAll('.sort-pill[data-sort]').forEach(p => {
          p.classList.remove('active');
          p.querySelector('.sort-arrow')?.remove();
        });
        pill.classList.add('active');
        const arrow = document.createElement('span');
        arrow.className = 'sort-arrow';
        arrow.textContent = state.sortAscending ? ' ↑' : ' ↓';
        pill.appendChild(arrow);

        filterLibrary();
      });
    });
  }

  function sortSongs(songs) {
    return [...songs].sort((a, b) => {
      let va = a[state.sortField] || '';
      let vb = b[state.sortField] || '';
      if (state.sortField === 'duration' || state.sortField === 'duration_secs') {
        va = a.duration_secs || 0;
        vb = b.duration_secs || 0;
        return state.sortAscending ? va - vb : vb - va;
      }
      va = String(va).toLowerCase();
      vb = String(vb).toLowerCase();
      const cmp = va.localeCompare(vb);
      return state.sortAscending ? cmp : -cmp;
    });
  }

  // ─── View Toggle ───────────────────────────────────────────────────────────
  function setupViewToggle() {
    document.getElementById('toggle-list').addEventListener('click', () => {
      state.viewMode = 'list';
      document.getElementById('toggle-list').classList.add('active');
      document.getElementById('toggle-grid').classList.remove('active');
      document.getElementById('song-list-view').style.display = '';
      document.getElementById('album-grid-view').style.display = 'none';
    });

    document.getElementById('toggle-grid').addEventListener('click', () => {
      state.viewMode = 'grid';
      document.getElementById('toggle-grid').classList.add('active');
      document.getElementById('toggle-list').classList.remove('active');
      document.getElementById('song-list-view').style.display = 'none';
      document.getElementById('album-grid-view').style.display = '';
    });
  }

  // ─── Transport Controls ────────────────────────────────────────────────────
  function setupTransportControls() {
    document.getElementById('btn-play').addEventListener('click', togglePlay);
    document.getElementById('mini-play').addEventListener('click', togglePlay);
    document.getElementById('btn-prev').addEventListener('click', () => Bridge.invoke('previous'));
    document.getElementById('btn-next').addEventListener('click', () => Bridge.invoke('next'));
    document.getElementById('mini-next').addEventListener('click', () => Bridge.invoke('next'));

    document.getElementById('btn-shuffle').addEventListener('click', () => {
      state.shuffleMode = (state.shuffleMode + 1) % 3;
      const btn = document.getElementById('btn-shuffle');
      btn.classList.toggle('active', state.shuffleMode > 0);
      updateModeIndicator();
    });

    document.getElementById('btn-repeat').addEventListener('click', () => {
      state.repeatMode = (state.repeatMode + 1) % 3;
      const btn = document.getElementById('btn-repeat');
      btn.classList.toggle('active', state.repeatMode > 0);
      updateModeIndicator();
    });

    document.getElementById('btn-like').addEventListener('click', () => {
      state.isLiked = !state.isLiked;
      const btn = document.getElementById('btn-like');
      btn.classList.toggle('liked', state.isLiked);
      if (state.isLiked) {
        btn.querySelector('svg').setAttribute('fill', 'currentColor');
      } else {
        btn.querySelector('svg').setAttribute('fill', 'none');
      }
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
      if (e.target.tagName === 'INPUT') return;
      switch (e.key) {
        case ' ': e.preventDefault(); togglePlay(); break;
        case 'n': case 'N': Bridge.invoke('next'); break;
        case 'p': case 'P': Bridge.invoke('previous'); break;
        case 's': case 'S':
          state.shuffleMode = (state.shuffleMode + 1) % 3;
          document.getElementById('btn-shuffle').classList.toggle('active', state.shuffleMode > 0);
          updateModeIndicator();
          break;
        case 'r': case 'R':
          state.repeatMode = (state.repeatMode + 1) % 3;
          document.getElementById('btn-repeat').classList.toggle('active', state.repeatMode > 0);
          updateModeIndicator();
          break;
        case 'ArrowUp': e.preventDefault(); Knob.setVolume(Math.min(1, state.volume + 0.05)); break;
        case 'ArrowDown': e.preventDefault(); Knob.setVolume(Math.max(0, state.volume - 0.05)); break;
      }
    });
  }

  function togglePlay() {
    if (!state.currentSong && state.library.length > 0) {
      playSong(state.library[0]);
      return;
    }
    state.isPlaying = !state.isPlaying;
    updatePlayState();

    if (state.isPlaying) {
      Bridge.invoke('resume');
      startProgressSimulation();
    } else {
      Bridge.invoke('pause');
      stopProgressSimulation();
    }
  }

  function playSong(song) {
    state.currentSong = song;
    state.isPlaying = true;
    state.position = 0;
    state.duration = song.duration_secs;
    state.isLiked = false;

    // Update Now Playing UI
    document.getElementById('np-title').textContent = song.title;
    document.getElementById('np-artist').textContent = `${song.artist} · ${song.album}`;
    document.getElementById('time-total').textContent = formatTime(song.duration_secs);
    document.getElementById('time-current').textContent = '0:00';

    // Update mini player
    document.getElementById('mini-title').textContent = song.title;
    document.getElementById('mini-artist').textContent = song.artist;

    // Reset like
    const likeBtn = document.getElementById('btn-like');
    likeBtn.classList.remove('liked');
    likeBtn.querySelector('svg').setAttribute('fill', 'none');

    // Generate new waveform for this song
    generateWaveform(song.id);

    updatePlayState();
    updateSongListPlaying();
    startProgressSimulation();

    Bridge.invoke('play_song', { song });
  }

  function updatePlayState() {
    const playing = state.isPlaying;

    // Vinyl
    Vinyl.setPlaying(playing);

    // Play button icon
    const playIcon = document.getElementById('play-icon');
    if (playing) {
      playIcon.innerHTML = '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>';
    } else {
      playIcon.innerHTML = '<polygon points="8,5 20,12 8,19"/>';
    }

    // Mini player
    const miniPlay = document.getElementById('mini-play');
    if (playing) {
      miniPlay.querySelector('svg').innerHTML = '<rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/>';
    } else {
      miniPlay.querySelector('svg').innerHTML = '<polygon points="8,5 20,12 8,19"/>';
    }

    // Mini player visibility
    const miniPlayer = document.getElementById('mini-player');
    if (state.currentSong && state.currentView !== 'now-playing') {
      miniPlayer.classList.remove('hidden');
    }
  }

  function updateModeIndicator() {
    const modes = [];
    if (state.shuffleMode === 1) modes.push('<span class="mode-active">Shuffle</span>');
    if (state.shuffleMode === 2) modes.push('<span class="mode-active">Smart Radio ✦</span>');
    if (state.repeatMode === 1) modes.push('<span class="mode-active">Repeat One</span>');
    if (state.repeatMode === 2) modes.push('<span class="mode-active">Repeat All</span>');
    document.getElementById('mode-indicator').innerHTML = modes.join(' · ');
  }

  // ─── Progress Simulation (standalone mode) ─────────────────────────────────
  let progressInterval = null;

  function startProgressSimulation() {
    stopProgressSimulation();
    progressInterval = setInterval(() => {
      if (state.isPlaying && state.duration > 0) {
        state.position = Math.min(state.position + 0.25, state.duration);
        updateProgress();
        if (state.position >= state.duration) {
          stopProgressSimulation();
          state.isPlaying = false;
          state.position = 0;
          updatePlayState();
          updateProgress();
        }
      }
    }, 250);
  }

  function stopProgressSimulation() {
    if (progressInterval) {
      clearInterval(progressInterval);
      progressInterval = null;
    }
  }

  function updateProgress() {
    const pct = state.duration > 0 ? (state.position / state.duration) * 100 : 0;

    // Seek bar
    document.getElementById('seek-fill').style.width = `${pct}%`;
    document.getElementById('time-current').textContent = formatTime(state.position);

    // Mini player progress
    document.getElementById('mini-player').style.setProperty('--mini-progress', `${pct}%`);

    // Update waveform
    updateWaveformProgress(pct);
  }

  // ─── Toggles ───────────────────────────────────────────────────────────────
  function setupToggles() {
    document.querySelectorAll('.s-toggle').forEach(toggle => {
      toggle.addEventListener('click', () => {
        toggle.classList.toggle('on');
      });
    });
  }

  // ─── Palette Swatches ──────────────────────────────────────────────────────
  function setupPaletteSwatches() {
    document.querySelectorAll('.swatch').forEach(swatch => {
      swatch.addEventListener('click', () => {
        const palette = swatch.dataset.palette;
        state.palette = palette;
        document.documentElement.setAttribute('data-palette', palette);

        document.querySelectorAll('.swatch').forEach(s => s.classList.remove('selected'));
        swatch.classList.add('selected');
      });
    });
  }

  // ─── Render Song List ──────────────────────────────────────────────────────
  function renderSongList(songs) {
    const container = document.getElementById('song-list-view');
    container.innerHTML = '';

    if (songs.length === 0) {
      container.innerHTML = `
        <div class="empty-state">
          <div class="empty-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          </div>
          <div class="empty-title">${state.searchQuery ? 'No results found' : 'Your library is empty'}</div>
          <div class="empty-desc">${state.searchQuery ? `Nothing matching "${state.searchQuery}"` : 'Point Musico at your music collection in Settings'}</div>
        </div>`;
      return;
    }

    songs.forEach((song, i) => {
      const isPlaying = state.currentSong?.id === song.id;
      const row = document.createElement('div');
      row.className = `song-row${isPlaying ? ' playing' : ''}`;
      row.innerHTML = `
        <div class="song-num">${isPlaying
          ? '<div class="eq-bars"><span></span><span></span><span></span></div>'
          : i + 1}</div>
        <div class="song-art">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/></svg>
        </div>
        <div class="song-info">
          <div class="song-name">${song.title}</div>
          <div class="song-detail">${song.artist} · ${song.album}</div>
        </div>
        <div class="song-duration">${formatTime(song.duration_secs)}</div>
        <button class="song-queue-btn" title="Add to queue">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 5v14M5 12h14"/></svg>
        </button>`;

      row.addEventListener('click', (e) => {
        if (e.target.closest('.song-queue-btn')) return;
        playSong(song);
      });

      container.appendChild(row);
    });
  }

  function renderAlbumGrid(songs) {
    const container = document.getElementById('album-grid-view');
    container.innerHTML = '';

    songs.forEach(song => {
      const card = document.createElement('div');
      card.className = 'album-card';
      card.innerHTML = `
        <div class="album-card-art">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/></svg>
        </div>
        <div class="album-card-title">${song.title}</div>
        <div class="album-card-artist">${song.artist}</div>`;

      card.addEventListener('click', () => playSong(song));
      container.appendChild(card);
    });
  }

  function updateSongListPlaying() {
    document.querySelectorAll('.song-row').forEach((row, i) => {
      const song = state.library[i];
      const isPlaying = song && state.currentSong?.id === song.id;
      row.classList.toggle('playing', isPlaying);

      const num = row.querySelector('.song-num');
      if (isPlaying) {
        num.innerHTML = '<div class="eq-bars"><span></span><span></span><span></span></div>';
        row.querySelector('.song-name').style.color = 'var(--accent)';
      } else {
        num.textContent = i + 1;
        row.querySelector('.song-name').style.color = '';
      }
    });
  }

  // ─── Render Recommendations ────────────────────────────────────────────────
  function renderRecommendations() {
    const container = document.getElementById('recs-list');
    container.innerHTML = '';

    state.recommendations.forEach((song, i) => {
      const row = document.createElement('div');
      row.className = 'song-row';
      row.innerHTML = `
        <div class="song-num">${i + 1}</div>
        <div class="song-art">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/></svg>
        </div>
        <div class="song-info">
          <div class="song-name">${song.title}</div>
          <div class="song-detail">${song.artist} · ${song.album}</div>
        </div>
        <div class="song-duration">${formatTime(song.duration_secs)}</div>`;

      row.addEventListener('click', () => playSong(song));
      container.appendChild(row);
    });
  }

  // ─── Waveform ──────────────────────────────────────────────────────────────
  function generateWaveform(seed = 'default') {
    const container = document.getElementById('waveform');
    container.innerHTML = '';
    const numBars = 48;

    let hash = 0;
    for (const c of seed) hash = ((hash << 5) - hash + c.charCodeAt(0)) | 0;

    for (let i = 0; i < numBars; i++) {
      hash = (hash * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFF;
      const h = 6 + Math.abs(hash % 28);
      const bar = document.createElement('div');
      bar.className = 'wave-bar';
      bar.style.height = `${h}px`;
      bar.dataset.index = i;
      container.appendChild(bar);
    }
  }

  function updateWaveformProgress(pct) {
    const bars = document.querySelectorAll('.wave-bar');
    const total = bars.length;
    const activeIdx = Math.floor((pct / 100) * total);

    bars.forEach((bar, i) => {
      bar.classList.remove('played', 'current');
      if (i < activeIdx) bar.classList.add('played');
      else if (i === activeIdx) bar.classList.add('current');
    });
  }

  // ─── Utility ───────────────────────────────────────────────────────────────
  function formatTime(secs) {
    const s = Math.floor(secs);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, '0')}`;
  }

  // Expose for other modules
  return {
    init,
    state,
    playSong,
    togglePlay,
    updateProgress,
    navigateTo,
    formatTime,
  };
})();

// Boot
document.addEventListener('DOMContentLoaded', App.init);
