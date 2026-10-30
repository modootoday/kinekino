// Progressive enhancement only: the site works without JS. No auto language redirect.
(function () {
  var d = document.documentElement; d.classList.add('js');
  var still = matchMedia('(prefers-reduced-motion: reduce)').matches;
  // Language menu and theme toggle come from jekyll-theme-modootoday.
  // Scroll reveal.
  var els = document.querySelectorAll('.reveal');
  if (still || !('IntersectionObserver' in window)) { els.forEach(function (el) { el.classList.add('in'); }); }
  else { var io = new IntersectionObserver(function (es) { es.forEach(function (x) { if (x.isIntersecting) { x.target.classList.add('in'); io.unobserve(x.target); } }); }, { threshold: 0, rootMargin: '0px 0px 120px 0px' });
    els.forEach(function (el) { io.observe(el); }); }
  if (still) return;
  // Eyes follow the pointer; the stage tilts slightly (parallax).
  var eyes = document.querySelectorAll('.cat .eyes'), stage = document.querySelector('[data-parallax]');
  addEventListener('pointermove', function (e) {
    eyes.forEach(function (g) {
      var r = g.ownerSVGElement.getBoundingClientRect();
      var dx = e.clientX - (r.left + r.width / 2), dy = e.clientY - (r.top + r.height * .58);
      var m = Math.hypot(dx, dy) || 1, k = Math.min(1, m / 300) * 4;
      g.style.transform = 'translate(' + (dx / m * k) + 'px,' + (dy / m * k) + 'px)';
    });
    if (stage) stage.style.transform = 'translate(' + ((e.clientX / innerWidth - .5) * 12) + 'px,' + ((e.clientY / innerHeight - .5) * 8) + 'px)';
  }, { passive: true });
})();
