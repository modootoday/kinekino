(() => {
  const hosts = ['catalog.example.test', 'frame.example.test', 'media.example.test'];
  if (!hosts.includes(location.hostname)) return;
  const adCounts = () => {
    const ads = [...document.querySelectorAll('ins.adsbygoogle,[id^="aswift_"],.google-anno-sa,iframe[title="Advertisement"],a:has(img[alt="banner"])')];
    const visible = ads.filter(e => e.getClientRects().length && getComputedStyle(e).visibility !== 'hidden').length;
    return {hidden:ads.length-visible, visible};
  };
  const deliver = payload => {
    if (window === window.top) {
      const count = adCounts();
      window.ipc.postMessage(JSON.stringify({...payload, hidden_ads:payload.hidden_ads+count.hidden, visible_ads:payload.visible_ads+count.visible}));
    }
    else parent.postMessage({type:'kinekino-player:observation', payload}, '*');
  };
  addEventListener('message', event => {
    if (event.data?.type !== 'kinekino-player:observation') return;
    try { if (!hosts.includes(new URL(event.origin).hostname)) return; } catch { return; }
    deliver(event.data.payload);
  });
  setInterval(() => {
    const video = document.querySelector('video');
    if (!video || !Number.isFinite(video.duration)) return;
    const ads = [...document.querySelectorAll('ins.adsbygoogle,[id^="aswift_"],.google-anno-sa,iframe[title="Advertisement"],a:has(img[alt="banner"])')];
    const visible = ads.filter(e => e.getClientRects().length && getComputedStyle(e).visibility !== 'hidden').length;
    deliver({host:location.hostname, time:video.currentTime, duration:video.duration,
      paused:video.paused, muted:video.muted, volume:video.volume, ready:video.readyState,
      width:video.videoWidth, height:video.videoHeight, error:video.error?.code || 0,
      hidden_ads:ads.length-visible, visible_ads:visible,
      audio_bytes:video.webkitAudioDecodedByteCount || 0,
      decoded_frames:video.getVideoPlaybackQuality?.().totalVideoFrames || 0});
  }, 1000);
})();
