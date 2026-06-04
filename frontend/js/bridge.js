/* ══════════════════════════════════════════════════════════════════════════════
   MUSICO — Tauri IPC Bridge
   Wraps Tauri invoke() calls with mock fallbacks for standalone preview.
   ══════════════════════════════════════════════════════════════════════════════ */

const Bridge = (() => {
  const isTauri = typeof window.__TAURI__ !== 'undefined';

  async function invoke(cmd, args = {}) {
    if (isTauri) {
      return window.__TAURI__.invoke(cmd, args);
    }
    // Mock fallbacks for standalone preview
    return mockCommand(cmd, args);
  }

  // Mock data for standalone HTML preview
  const DEMO_SONGS = [
    { id: '1', title: 'Midnight City', artist: 'M83', album: 'Hurry Up, We\'re Dreaming', duration_secs: 244 },
    { id: '2', title: 'Teardrop', artist: 'Massive Attack', album: 'Mezzanine', duration_secs: 329 },
    { id: '3', title: 'Breathe (In the Air)', artist: 'Pink Floyd', album: 'Dark Side of the Moon', duration_secs: 165 },
    { id: '4', title: 'Intro', artist: 'The xx', album: 'xx', duration_secs: 127 },
    { id: '5', title: 'Motion Picture Soundtrack', artist: 'Radiohead', album: 'Kid A', duration_secs: 421 },
    { id: '6', title: 'Blue (Da Ba Dee)', artist: 'Eiffel 65', album: 'Europop', duration_secs: 220 },
    { id: '7', title: 'Crystalised', artist: 'The xx', album: 'xx', duration_secs: 227 },
    { id: '8', title: 'Holocene', artist: 'Bon Iver', album: 'Bon Iver', duration_secs: 337 },
    { id: '9', title: 'Svefn-g-englar', artist: 'Sigur Rós', album: 'Ágætis byrjun', duration_secs: 610 },
    { id: '10', title: 'Porcelain', artist: 'Moby', album: 'Play', duration_secs: 238 },
    { id: '11', title: 'Teardrop', artist: 'José González', album: 'Veneer', duration_secs: 191 },
    { id: '12', title: 'Everything In Its Right Place', artist: 'Radiohead', album: 'Kid A', duration_secs: 252 },
  ];

  function mockCommand(cmd, args) {
    switch (cmd) {
      case 'get_library':
        return Promise.resolve(DEMO_SONGS);
      case 'get_queue':
        return Promise.resolve([]);
      case 'get_recommendations':
        return Promise.resolve(DEMO_SONGS.slice(3, 7));
      case 'get_playback_state':
        return Promise.resolve({
          current_song: null,
          status: 'Stopped',
          position_secs: 0,
          duration_secs: 0,
          volume: 0.68,
        });
      case 'search_library':
        const q = (args.query || '').toLowerCase();
        return Promise.resolve(
          DEMO_SONGS.filter(s =>
            s.title.toLowerCase().includes(q) ||
            s.artist.toLowerCase().includes(q) ||
            s.album.toLowerCase().includes(q)
          )
        );
      default:
        return Promise.resolve(null);
    }
  }

  // Event listener for Tauri events
  function listen(event, callback) {
    if (isTauri && window.__TAURI__.event) {
      return window.__TAURI__.event.listen(event, callback);
    }
    // No-op for standalone
    return () => {};
  }

  return { invoke, listen, isTauri, DEMO_SONGS };
})();
