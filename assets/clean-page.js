(() => {
  if (!['catalog.example.test', 'frame.example.test', 'media.example.test'].includes(location.hostname)) return;
  const selectors = [
    'ins.adsbygoogle', '[id^="google_ads_"]', '[id^="aswift_"]',
    '.google-anno-sa', 'iframe[title="Advertisement"]',
    'iframe[src*="doubleclick.net"]', 'iframe[src*="googlesyndication.com"]',
    'a:has(img[alt="banner"])'
  ];
  const style = document.createElement('style');
  style.textContent = selectors.join(',') + '{display:none!important;height:0!important;min-height:0!important;margin:0!important}';
  const install = () => {
    if (!style.isConnected) (document.head || document.documentElement)?.append(style);
  };
  install();
  let scheduled = false;
  new MutationObserver(() => {
    if (scheduled) return;
    scheduled = true;
    requestAnimationFrame(() => { scheduled = false; install(); });
  }).observe(document, {childList:true, subtree:true});
})();
