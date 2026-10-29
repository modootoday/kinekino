(() => {
  const hosts = ['catalog.example.test', 'frame.example.test', 'media.example.test'];
  if (!hosts.includes(location.hostname)) return;
  let lastKey=0;
  addEventListener('keydown', () => {
    const now=performance.now();
    if(now-lastKey>200) { lastKey=now; window.ipc?.postMessage('{"input_activity":true}'); }
  });
  addEventListener('message', event => {
    const data = event.data;
    if (!['kinekino-player:pause', 'kinekino-player:control'].includes(data?.type)) return;
    try { if (!hosts.includes(new URL(event.origin).hostname)) return; } catch { return; }
    document.querySelectorAll('video,audio').forEach(media => {
      if (data.type.endsWith(':pause')) { media.pause(); return; }
      const value = Number(data.value);
      if (!Number.isFinite(value)) return;
      switch(data.action) {
        case 'pause': media.pause(); break;
        case 'play': media.play().catch(() => {}); break;
        case 'toggle': if (media.paused) media.play().catch(() => {}); else media.pause(); break;
        case 'seek': if (Number.isFinite(media.duration)) media.currentTime = Math.max(0, Math.min(value, media.duration)); break;
        case 'volume': media.volume = Math.max(0, Math.min(value / 100, 1)); break;
      }
    });
    for (let index = 0; index < frames.length; index++) frames[index].postMessage(data, '*');
  });
})();
