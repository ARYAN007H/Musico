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
    recommendations: [],    sortField: 'title',
    sortAscending: true,
    viewMode: 'list',
    searchQuery: '',
    palette: 'nebula',
    settings: {},
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
    setupSettingsPersistence();
    setupEq();
    setupScanButton();

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

    // Live engine events (position, song changes, queue updates)
    initPlaybackEvents();

    // Restore engine truth on startup (volume, mid-play state)
    const ps = await Bridge.invoke('get_playback_state');
    if (ps) {
      Knob.setVolume(ps.volume ?? 0.68, false);
      if (ps.current_song) {
        state.currentSong = ps.current_song;
        state.duration = ps.duration_secs;
        state.position = ps.position_secs;
        setNowPlayingUi(ps.current_song);
        updateProgress();
      }
      state.isPlaying = ps.status === 'Playing';
      updatePlayState();
    }

    renderQueue(await Bridge.invoke('get_queue'));

    // Persisted settings: apply palette, toggles, folder, view mode
    state.settings = (await Bridge.invoke('get_settings')) || {};
    applySettings(state.settings);
  }

  // ─── Settings: apply + persist ────────────────────────────────────────────
  const TOGGLE_KEYS = {
    'toggle-eq': 'eq_enabled',
    'toggle-crossfade': 'crossfade_enabled',
    'toggle-art-tint': 'art_tint',
    'toggle-smart-rec': 'smart_recs',
  };

  function applySettings(s) {
    if (!s) return;
    document.documentElement.setAttribute('data-theme', s.theme === 'dark' ? 'dark' : 'light');
    document.querySelectorAll('[data-theme-pill]').forEach((p) =>
      p.classList.toggle('active', p.dataset.themePill === (s.theme === 'dark' ? 'dark' : 'light')));
    if (s.palette) {
      document.documentElement.setAttribute('data-palette', s.palette);
      document.querySelectorAll('.swatch').forEach((sw) =>
        sw.classList.toggle('selected', sw.dataset.palette === s.palette));
    }
    Object.entries(TOGGLE_KEYS).forEach(([id, key]) => {
      const el = document.getElementById(id);
      if (el) el.classList.toggle('on', !!s[key]);
    });
    if (s.music_folder) {
      const tag = document.getElementById('folder-tag');
      if (tag) {
        const short = s.music_folder.replace(/^\/home\/[^/]+/, '~');
        tag.lastChild.textContent = ` ${short}`;
      }
    }
    if (s.library_view_mode === 'grid') {
      document.getElementById('toggle-grid')?.click();
    }
    if (typeof s.shuffle_mode === 'number') {
      state.shuffleMode = s.shuffle_mode;
      document.getElementById('btn-shuffle')?.classList.toggle('active', s.shuffle_mode > 0);
    }
    if (typeof s.repeat_mode === 'number') {
      state.repeatMode = s.repeat_mode;
      document.getElementById('btn-repeat')?.classList.toggle('active', s.repeat_mode > 0);
    }
    updateModeIndicator();
    // Normalization pills
    document.querySelectorAll('#panel-audio .s-card:nth-of-type(2) .sort-pill').forEach((pill) => {
      pill.classList.toggle('active', pill.textContent.trim() === s.normalization);
    });
    // EQ: restore preset curve
    const presetName = s.eq_preset && EQ_PRESETS[s.eq_preset] ? s.eq_preset : 'Flat';
    document.querySelectorAll('#eq-presets .sort-pill').forEach((p) => {
      p.classList.toggle('active', p.dataset.eqPreset === presetName);
    });
    const gains = EQ_PRESETS[presetName];
    document.querySelectorAll('.eq-slider').forEach((sl, i) => {
      if (gains && gains[i] !== undefined) {
        sl.value = gains[i];
        sl.parentElement.querySelector('.eq-band-value').textContent = gains[i];
      }
    });
    if (s.eq_enabled && typeof Bridge !== 'undefined') applyEq(gains ?? []);
    // Crossfade slider
    const slider = document.querySelector('.s-slider');
    if (slider && s.crossfade_duration) {
      slider.value = s.crossfade_duration;
      const val = document.querySelector('.s-value');
      if (val) val.textContent = `${Number(s.crossfade_duration).toFixed(1)}s`;
    }
  }

  function persistSettings(patch) {
    Object.assign(state.settings, patch);
    Bridge.invoke('save_settings', { settings: state.settings });
  }

  function setupSettingsPersistence() {
    // Toggles
    Object.entries(TOGGLE_KEYS).forEach(([id, key]) => {
      const el = document.getElementById(id);
      if (!el) return;
      el.addEventListener('click', () => persistSettings({ [key]: el.classList.contains('on') }));
    });

    // Theme pills
    document.querySelectorAll('[data-theme-pill]').forEach((pill) => {
      pill.addEventListener('click', () => {
        document.querySelectorAll('[data-theme-pill]').forEach((p) => p.classList.remove('active'));
        pill.classList.add('active');
        const theme = pill.dataset.themePill;
        document.documentElement.setAttribute('data-theme', theme);
        persistSettings({ theme });
      });
    });

    // Music folder row → native picker
    document.getElementById('row-music-folder')?.addEventListener('click', async () => {
      const tag = document.getElementById('folder-tag');
      const chosen = await Bridge.invoke('pick_folder');
      if (chosen && tag) {
        const short = chosen.replace(/^\/home\/[^/]+/, '~');
        tag.lastChild.textContent = ` ${short}`;
      }
    });

    // Normalization pills (2nd s-card in audio panel)
    document.querySelectorAll('#panel-audio .s-card:nth-of-type(2) .sort-pill').forEach((pill) => {
      pill.addEventListener('click', () => {
        pill.parentElement.querySelectorAll('.sort-pill').forEach((p) => p.classList.remove('active'));
        pill.classList.add('active');
        persistSettings({ normalization: pill.textContent.trim() });
        Bridge.invoke('set_normalization', { mode: pill.textContent.trim() });
      });
    });

    // Crossfade duration slider
    const slider = document.querySelector('.s-slider');
    slider?.addEventListener('input', () => {
      const val = document.querySelector('.s-value');
      if (val) val.textContent = `${Number(slider.value).toFixed(1)}s`;
      persistSettings({ crossfade_enabled: true, crossfade_duration: Number(slider.value) });
      document.getElementById('toggle-crossfade')?.classList.add('on');
    });
  }

  // ─── Playback Event Handling (replaces fake progress simulation) ──────────
  function initPlaybackEvents() {
    Bridge.listen('playback-event', (e) => handlePlaybackEvent(e.payload ?? e));
    Bridge.listen('queue-changed', (e) => renderQueue(e.payload ?? e));
  }

  function handlePlaybackEvent(ev) {
    if (!ev || !ev.type) return;
    switch (ev.type) {
      case 'Playing': {
        const song = ev.song;
        const isNew = !state.currentSong || state.currentSong.id !== song.id;
        state.currentSong = song;
        state.isPlaying = true;
        if (isNew) {
          state.duration = song.duration_secs;
          state.position = 0;
          state.isLiked = false;
          setNowPlayingUi(song);
          loadCover(song);
          loadLyrics(song);
          generateWaveform(song.id);
          updateSongListPlaying();
        }
        updatePlayState();
        break;
      }
      case 'PositionUpdate':
      case 'Seeked':
        state.position = ev.position_secs;
        updateProgress();
        updateLyricsHighlight(ev.position_secs);
        break;
      case 'Paused':
      case 'Stopped':
      case 'SongEnded':
        state.isPlaying = false;
        if (ev.type !== 'Paused') {
          state.position = 0;
          updateProgress();
        }
        updatePlayState();
        break;
      case 'Resumed':
        state.isPlaying = true;
        updatePlayState();
        break;
      case 'VolumeChanged':
        Knob.setVolume(ev.volume, false);
        break;
      case 'Error':
        showError(ev.message || 'Playback error');
        break;
    }
  }

  // ─── Cover Art (disk-cached thumbnails, fetched per song) ─────────────────
  const coverCache = new Map();

  function loadCover(song) {
    if (coverCache.has(song.id)) {
      Vinyl.setAlbumArt(coverCache.get(song.id));
      return;
    }
    Bridge.invoke('get_cover', { songId: song.id }).then((url) => {
      coverCache.set(song.id, url ?? null);
      if (state.currentSong && state.currentSong.id === song.id) {
        Vinyl.setAlbumArt(url ?? null);
      }
    });
  }

  // ─── Lyrics (sidecar .lrc, karaoke highlight) ─────────────────────────────
  let lyricLines = []; // { time_secs, el }

  function loadLyrics(song) {
    lyricLines = [];
    const section = document.getElementById('lyrics-section');
    const list = document.getElementById('lyrics-list');
    section.style.display = 'none';
    Bridge.invoke('get_lyrics', { songId: song.id }).then((dto) => {
      if (!dto || !dto.lines || !dto.lines.length) return;
      if (state.currentSong && state.currentSong.id !== song.id) return;
      list.innerHTML = '';
      lyricLines = dto.lines.map((line) => {
        const el = document.createElement('div');
        el.className = 'lyric-line';
        el.textContent = line.text;
        list.appendChild(el);
        return { time_secs: line.time_secs, el };
      });
      section.style.display = '';
      updateLyricsHighlight(state.position);
    });
  }

  function updateLyricsHighlight(position) {
    if (!lyricLines.length) return;
    let active = -1;
    for (let i = 0; i < lyricLines.length; i++) {
      if (lyricLines[i].time_secs <= position) active = i;
      else break;
    }
    lyricLines.forEach((l, i) => {
      l.el.classList.toggle('active', i === active);
      l.el.classList.toggle('past', active !== -1 && i < active);
    });
    if (active >= 0) {
      lyricLines[active].el.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    }
  }

  // Push the song's identity into all the Now Playing + mini player surfaces
  function setNowPlayingUi(song) {
    document.getElementById('np-title').textContent = song.title;
    document.getElementById('np-artist').textContent = `${song.artist} · ${song.album}`;
    document.getElementById('time-total').textContent = formatTime(song.duration_secs);
    document.getElementById('time-current').textContent = formatTime(state.position);

    document.getElementById('mini-title').textContent = song.title;
    document.getElementById('mini-artist').textContent = song.artist;

    const likeBtn = document.getElementById('btn-like');
    likeBtn.classList.remove('liked');
    likeBtn.querySelector('svg').setAttribute('fill', 'none');
  }

  // ─── Queue Rendering ──────────────────────────────────────────────────────
  function renderQueue(songs) {
    state.queue = songs || [];
    const container = document.getElementById('queue-list');
    if (!container) return;
    document.getElementById('queue-count').textContent = `${state.queue.length} songs`;

    if (state.queue.length === 0) {
      container.innerHTML = `
        <div class="queue-empty" id="queue-empty">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
          <div style="text-align:center;">
            <div style="font-weight:var(--weight-semi); color:var(--text-primary); margin-bottom:4px;">Queue is empty</div>
            <div style="font-size:var(--text-caption);">Add songs from Library or let Smart Radio fill it</div>
          </div>
        </div>`;
      return;
    }

    container.innerHTML = '';
    state.queue.forEach((song, i) => {
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
        <button class="song-queue-btn" title="Remove from queue">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12"/></svg>
        </button>`;

      row.addEventListener('click', (e) => {
        if (e.target.closest('.song-queue-btn')) {
          Bridge.invoke('remove_from_queue', { songId: song.id });
          return;
        }
        playSong(song);
      });
      container.appendChild(row);
    });
  }

  // ─── Scan Button (async, progress via engine events) ──────────────────────
  function setupScanButton() {
    const btn = document.getElementById('btn-scan');
    if (!btn) return;

    Bridge.listen('scan-progress', (e) => {
      const p = e.payload ?? e;
      document.getElementById('lib-count').textContent = `Scanning ${p.done}/${p.total}…`;
    });

    Bridge.listen('library-updated', (e) => {
      state.library = e.payload ?? e;
      renderSongList(state.library);
      renderAlbumGrid(state.library);
      document.getElementById('lib-count').textContent = `${state.library.length} songs`;
      btn.disabled = false;
      // Refresh recs now that the library exists
      Bridge.invoke('get_recommendations').then((recs) => {
        state.recommendations = recs;
        renderRecommendations();
      });
      const hint = document.getElementById('scan-hint');
      if (hint) hint.textContent = `${state.library.length} songs · Scanned just now`;
    });

    Bridge.listen('index-progress', (e) => {
      const p = e.payload ?? e;
      const hint = document.getElementById('scan-hint');
      if (hint) hint.textContent = `Learning your library… ${p.done}/${p.total}`;
    });

    Bridge.listen('index-complete', () => {
      const hint = document.getElementById('scan-hint');
      if (hint && state.library.length) hint.textContent = `${state.library.length} songs · Ready`;
      Bridge.invoke('get_recommendations').then((recs) => {
        state.recommendations = recs;
        renderRecommendations();
      });
    });

    btn.addEventListener('click', () => {
      btn.disabled = true;
      document.getElementById('lib-count').textContent = 'Scanning…';
      Bridge.invoke('start_scan');
    });
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

    if (viewName === 'playlists') renderStats();
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
      persistSettings({ library_view_mode: 'list' });
    });

    document.getElementById('toggle-grid').addEventListener('click', () => {
      state.viewMode = 'grid';
      document.getElementById('toggle-grid').classList.add('active');
      document.getElementById('toggle-list').classList.remove('active');
      document.getElementById('song-list-view').style.display = 'none';
      document.getElementById('album-grid-view').style.display = '';
      persistSettings({ library_view_mode: 'grid' });
    });
  }

  // ─── Transport Controls ────────────────────────────────────────────────────
  function setupTransportControls() {
    document.getElementById('btn-play').addEventListener('click', togglePlay);
    document.getElementById('mini-play').addEventListener('click', togglePlay);
    document.getElementById('btn-prev').addEventListener('click', () => Bridge.invoke('previous'));
    document.getElementById('btn-next').addEventListener('click', () => Bridge.invoke('next'));
    document.getElementById('mini-next').addEventListener('click', () => Bridge.invoke('next'));

    document.getElementById('btn-shuffle').addEventListener('click', cycleShuffle);
    document.getElementById('btn-repeat').addEventListener('click', cycleRepeat);

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

    // Clicking the mini player (not its buttons) returns to Now Playing
    document.getElementById('mini-player')?.addEventListener('click', (e) => {
      if (e.target.closest('button')) return;
      navigateTo('now-playing');
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
      if (e.target.tagName === 'INPUT') return;
      switch (e.key) {
        case ' ': e.preventDefault(); togglePlay(); break;
        case 'n': case 'N': Bridge.invoke('next'); break;
        case 'p': case 'P': Bridge.invoke('previous'); break;
        case 's': case 'S': cycleShuffle(); break;
        case 'r': case 'R': cycleRepeat(); break;
        case 'ArrowUp': e.preventDefault(); Knob.setVolume(Math.min(1, state.volume + 0.05)); break;
        case 'ArrowDown': e.preventDefault(); Knob.setVolume(Math.max(0, state.volume - 0.05)); break;
      }
    });
  }

  function togglePlay() {
    if (!state.currentSong) {
      if (state.library.length > 0) playSong(state.library[0]);
      return;
    }
    // Optimistic flip; Paused/Resumed engine events confirm it
    state.isPlaying = !state.isPlaying;
    updatePlayState();
    Bridge.invoke(state.isPlaying ? 'resume' : 'pause');
  }

  function playSong(song) {
    state.currentSong = song;
    state.isPlaying = true;
    state.position = 0;
    state.duration = song.duration_secs;
    state.isLiked = false;

    setNowPlayingUi(song);
    generateWaveform(song.id);
    updatePlayState();
    updateSongListPlaying();
    updateProgress();

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

  function cycleShuffle() {
    state.shuffleMode = (state.shuffleMode + 1) % 3;
    document.getElementById('btn-shuffle').classList.toggle('active', state.shuffleMode > 0);
    updateModeIndicator();
    Bridge.invoke('set_playback_modes', { shuffle: state.shuffleMode, repeat: state.repeatMode });
  }

  function cycleRepeat() {
    state.repeatMode = (state.repeatMode + 1) % 3;
    document.getElementById('btn-repeat').classList.toggle('active', state.repeatMode > 0);
    updateModeIndicator();
    Bridge.invoke('set_playback_modes', { shuffle: state.shuffleMode, repeat: state.repeatMode });
  }

  function updateModeIndicator() {
    const modes = [];
    if (state.shuffleMode === 1) modes.push('<span class="mode-active">Shuffle</span>');
    if (state.shuffleMode === 2) modes.push('<span class="mode-active">Smart Radio ✦</span>');
    if (state.repeatMode === 1) modes.push('<span class="mode-active">Repeat One</span>');
    if (state.repeatMode === 2) modes.push('<span class="mode-active">Repeat All</span>');
    document.getElementById('mode-indicator').innerHTML = modes.join(' · ');
  }

  // ─── Progress (driven entirely by engine PositionUpdate events) ───────────
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
        persistSettings({ palette });
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
      row.dataset.id = song.id;
      row.dataset.index = i + 1;
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
        if (e.target.closest('.song-queue-btn')) {
          Bridge.invoke('add_to_queue', { song });
          return;
        }
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
    document.querySelectorAll('#song-list-view .song-row').forEach((row) => {
      const isPlaying = row.dataset.id === state.currentSong?.id;
      row.classList.toggle('playing', isPlaying);

      const num = row.querySelector('.song-num');
      if (isPlaying) {
        num.innerHTML = '<div class="eq-bars"><span></span><span></span><span></span></div>';
        row.querySelector('.song-name').style.color = 'var(--accent)';
      } else {
        num.textContent = row.dataset.index;
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

  // ─── Stats & Playlists View ───────────────────────────────────────────────
  function renderStats() {
    Bridge.invoke('get_stats').then((s) => {
      const grid = document.getElementById('stats-grid');
      const empty = document.getElementById('stats-empty');
      const streak = document.getElementById('stats-streak');
      if (!grid) return;
      if (!s || s.total_plays === 0) {
        grid.style.display = 'none';
        empty.style.display = '';
        streak.textContent = '';
        document.getElementById('top-songs-list').innerHTML = '';
        return;
      }
      grid.style.display = '';
      empty.style.display = 'none';
      const hours = (s.total_listened_secs / 3600).toFixed(1);
      const cells = [
        ['Plays', s.total_plays],
        ['Listened', `${hours} h`],
        ['Skips', s.total_skips],
        ['Completion', `${Math.round(s.avg_listen_ratio * 100)}%`],
        ['Streak', `${s.streak_days} d`],
        ['Top artist', s.top_artists[0] ? s.top_artists[0][0] : '—'],
      ];
      grid.innerHTML = cells.map(([k, v]) => `
        <div style="text-align:center;">
          <div style="font-size:var(--text-title); font-weight:var(--weight-bold); color:var(--accent);">${v}</div>
          <div style="font-size:var(--text-caption); color:var(--text-secondary);">${k}</div>
        </div>`).join('');
      streak.textContent = s.streak_days > 0 ? `🔥 ${s.streak_days}-day streak` : '';

      const list = document.getElementById('top-songs-list');
      list.innerHTML = '';
      (s.top_songs || []).slice(0, 5).forEach((song, i) => {
        const row = document.createElement('div');
        row.className = 'song-row';
        row.innerHTML = `
          <div class="song-num">${i + 1}</div>
          <div class="song-art">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><circle cx="12" cy="12" r="3"/></svg>
          </div>
          <div class="song-info">
            <div class="song-name">${song.title}</div>
            <div class="song-detail">${song.artist} · ${song.plays} plays</div>
          </div>`;
        const libSong = state.library.find((l) => l.id === song.id);
        if (libSong) row.addEventListener('click', () => playSong(libSong));
        list.appendChild(row);
      });
    });

    Bridge.invoke('get_playlists').then((playlists) => {
      const container = document.getElementById('playlists-list');
      if (!container) return;
      if (playlists.length === 0) return; // keep the empty state
      document.getElementById('playlists-empty').style.display = 'none';
      container.innerHTML = '';
      playlists.forEach((pl) => {
        const row = document.createElement('div');
        row.className = 'song-row';
        row.innerHTML = `
          <div class="song-num">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" style="width:16px;height:16px;"><path d="M8 6h13M8 12h13M8 18h13"/></svg>
          </div>
          <div class="song-info">
            <div class="song-name">${pl.name}</div>
            <div class="song-detail">${pl.rules.length} rules${pl.max_songs ? ` · max ${pl.max_songs}` : ''}</div>
          </div>`;
        container.appendChild(row);
      });
    });
  }

  // ─── EQ (10-band, presets, vertical sliders) ──────────────────────────────
  const EQ_BANDS = ['32', '64', '125', '250', '500', '1K', '2K', '4K', '8K', '16K'];
  const EQ_PRESETS = {
    'Flat': [0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    'Bass Boost': [6, 5, 4, 2, 0, 0, 0, 0, 0, 0],
    'Vocal': [-2, -1, 0, 2, 4, 4, 3, 1, 0, -1],
    'Treble': [0, 0, 0, 0, 0, 1, 2, 4, 5, 6],
    'Rock': [5, 4, 2, 0, -1, -1, 0, 2, 4, 5],
    'Classical': [4, 3, 2, 0, 0, 0, -1, -1, 2, 3],
  };

  function currentEqGains() {
    return [...document.querySelectorAll('.eq-slider')].map((s) => Number(s.value));
  }

  function applyEq(gains) {
    const enabled = document.getElementById('toggle-eq')?.classList.contains('on') ?? false;
    Bridge.invoke('set_eq', { enabled, gains });
  }

  function setupEq() {
    const container = document.getElementById('eq-bands');
    if (!container) return;
    container.innerHTML = EQ_BANDS.map((label, i) => `
      <div class="eq-band">
        <input type="range" class="eq-slider" data-idx="${i}" min="-12" max="12" step="1" value="0"/>
        <span class="eq-band-value">0</span>
        <span class="eq-band-label">${label}</span>
      </div>`).join('');

    container.querySelectorAll('.eq-slider').forEach((slider) => {
      slider.addEventListener('input', () => {
        slider.parentElement.querySelector('.eq-band-value').textContent = slider.value;
        // Custom curve — clear preset highlight
        document.querySelectorAll('#eq-presets .sort-pill').forEach((p) => p.classList.remove('active'));
        applyEq(currentEqGains());
      });
    });

    document.querySelectorAll('#eq-presets .sort-pill').forEach((pill) => {
      pill.addEventListener('click', () => {
        document.querySelectorAll('#eq-presets .sort-pill').forEach((p) => p.classList.remove('active'));
        pill.classList.add('active');
        const gains = EQ_PRESETS[pill.dataset.eqPreset] ?? EQ_PRESETS['Flat'];
        container.querySelectorAll('.eq-slider').forEach((s, i) => {
          s.value = gains[i];
          s.parentElement.querySelector('.eq-band-value').textContent = gains[i];
        });
        persistSettings({ eq_preset: pill.dataset.eqPreset });
        applyEq(gains);
      });
    });

    // EQ toggle also applies the current curve
    document.getElementById('toggle-eq')?.addEventListener('click', () => {
      applyEq(currentEqGains());
    });
  }

  // ─── Utility ───────────────────────────────────────────────────────────────
  function formatTime(secs) {
    const s = Math.floor(secs);
    const m = Math.floor(s / 60);
    const sec = s % 60;
    return `${m}:${sec.toString().padStart(2, '0')}`;
  }

  // Expose for other modules and browser-console testing
  return {
    init,
    state,
    playSong,
    togglePlay,
    updateProgress,
    navigateTo,
    formatTime,
    handlePlaybackEvent,
    renderQueue,
  };
})();

  // ─── Toasts (errors, engine warnings) ─────────────────────────────────────
  let toastTimer = null;

  function showError(message) {
    let toast = document.getElementById('musico-toast');
    if (!toast) {
      toast = document.createElement('div');
      toast.id = 'musico-toast';
      toast.style.cssText = [
        'position:fixed', 'bottom:86px', 'left:50%', 'transform:translateX(-50%)',
        'max-width:80%', 'padding:10px 18px', 'border-radius:12px',
        'background:var(--surface-raised, #222)', 'color:var(--text-primary, #fff)',
        'box-shadow:0 8px 32px rgba(0,0,0,0.25)', 'font-size:var(--text-caption, 12px)',
        'z-index:9999', 'opacity:0', 'transition:opacity 0.25s ease',
        'pointer-events:none', 'text-align:center',
      ].join(';');
      document.body.appendChild(toast);
    }
    toast.textContent = message;
    toast.style.opacity = '1';
    clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toast.style.opacity = '0'; }, 4000);
  }

  // Boot
  document.addEventListener('DOMContentLoaded', () => {
    Bridge.setErrorHandler(showError);
    App.init();
  });
