(() => {
 if (window !== top || location.hostname !== 'catalog.example.test') return;
 let previous='';
 const observe=()=>{
  if (!/^\/(c|e)\//.test(location.pathname)) return;
  const series = location.pathname.startsWith('/c/') ? location.href : document.querySelector('a[href^="/c/"]')?.href;
  if (!series) return;
  const entries=Array.from(document.querySelectorAll('.wr-subject a[href^="/e/"]')).map(a=>({title:a.textContent.trim(),url:a.href})).reverse();
  if (!entries.length || entries.length>2000) return;
  const payload=JSON.stringify({page:location.href,series,title:document.querySelector('h1')?.textContent.trim() || document.title,entries});
  if (payload===previous || payload.length>500000) return;
  previous=payload; window.ipc.postMessage(JSON.stringify({episodes:JSON.parse(payload)}));
 };
 setInterval(observe,500);
})();
