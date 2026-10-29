import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { TailSpring } from '../src/physics/tail-spring.js';
import { LayerComposer } from '../src/runtime/composer.js';
import { UNIT } from '../src/geometry/svg-space.js';
// Viewer: clip/preset select, play/stop, spring-bone toggle, orbit camera.
// Screenshot mode: ?shot=1&clip=NAME&t=SEC&yaw=DEG&spring=0|1&whip=DEG (whip = sudden body roll at 0.2s for the physics test).
const q = new URLSearchParams(location.search);
const renderer = new THREE.WebGLRenderer({ antialias: true, preserveDrawingBuffer: true });
renderer.setSize(innerWidth, innerHeight); renderer.setPixelRatio(devicePixelRatio); document.body.append(renderer.domElement);
const scene = new THREE.Scene(); scene.background = new THREE.Color(0x0e0e10);
scene.add(new THREE.HemisphereLight(0xffffff, 0x2a1a14, 1.1));
const light = (i, x, y, z) => { const l = new THREE.DirectionalLight(0xffffff, i); l.position.set(x, y, z); scene.add(l); };
light(1.8, 3, 4, 6); light(0.6, -5, -1, 3); light(1.4, -2, 3, -5);
const camera = new THREE.PerspectiveCamera(30, innerWidth / innerHeight, 0.1, 100); camera.position.set(0, 0, 7.5);
const controls = new OrbitControls(camera, renderer.domElement);
const [gltf, physCfg, layersCfg] = await Promise.all([new GLTFLoader().loadAsync('../out/' + (q.get('glb') ?? 'kinekino-mascot.glb') + '?v=' + Date.now()), fetch('../src/physics/tail-physics.json?v=' + Date.now()).then((r) => r.json()), fetch('../src/presets/layers.json?v=' + Date.now()).then((r) => r.json())]);
// glTF drops clip.userData -> re-attach layer/loop/holdAt from parts.json (single source of truth).
const partsJson = await fetch('../src/presets/parts.json?v=' + Date.now()).then((r) => r.json());
const { resolveClip } = await import('../src/presets/build-part-clips.js');
for (const c of gltf.animations) if (partsJson.clips[c.name]) { const r = resolveClip(partsJson, c.name); c.userData = { layer: r.layer, loop: !!r.loop, holdAt: r.holdAt ?? null }; }
const pivot = new THREE.Group(); pivot.add(gltf.scene);
gltf.scene.updateMatrixWorld(true); const box = new THREE.Box3().setFromObject(gltf.scene); gltf.scene.position.sub(box.getCenter(new THREE.Vector3()));
pivot.rotation.y = THREE.MathUtils.degToRad(Number(q.get('yaw') ?? 0));
const tiltWrap = new THREE.Group(); scene.add(tiltWrap); tiltWrap.add(pivot); // ?tilt=DEG pitches the model (look up/down)
tiltWrap.rotation.x = THREE.MathUtils.degToRad(Number(q.get('tilt') ?? 0));
const comp = new LayerComposer(gltf.scene, gltf.animations, { ...layersCfg, unit: UNIT }); const mixer = comp.mixer; const clock = new THREE.Clock();
const spring = new TailSpring(gltf.scene, physCfg); spring.enabled = q.get('spring') !== '0';
const sel = document.getElementById('clip'); let action = null;
const presetNames = new Set(['idle_breath', 'alert', 'curious', 'tickle', 'content', 'startled', 'ear_perk', 'ear_flutter']);
const groups = { '기본 클립': document.createElement('optgroup'), '고양이 프리셋': document.createElement('optgroup'), '조합(파트 레이어)': document.createElement('optgroup'), '파트 클립': document.createElement('optgroup') };
for (const [label, g] of Object.entries(groups)) { g.label = label; sel.append(g); }
const isPart = (n) => n.includes('.');
for (const c of gltf.animations) groups[isPart(c.name) ? '파트 클립' : presetNames.has(c.name) ? '고양이 프리셋' : '기본 클립'].append(new Option(`${c.name} (${c.duration.toFixed(1)}s)`, c.name));
for (const n of Object.keys(layersCfg.combos)) groups['조합(파트 레이어)'].append(new Option(`조합: ${n}`, `combo:${n}`));
const springBox = document.getElementById('spring'); springBox.checked = spring.enabled;
springBox.onchange = () => { spring.enabled = springBox.checked; spring.reset(); };
const whip = THREE.MathUtils.degToRad(Number(q.get('whip') ?? 0));
const whipAngle = (t) => whip * THREE.MathUtils.smoothstep(t, 0.2, 0.28); // sudden roll for the physics test
const tailNodes = physCfg.bones.map((b) => gltf.scene.getObjectByName(b.name));
function frame(dt, t) {
  for (const n of tailNodes) { n.rotation.z = 0; n.scale.y = 1; } // un-animated tails must not accumulate the spring offset
  comp.update(dt); pivot.rotation.z = whipAngle(t);
  if (spring.enabled) spring.update(dt, pivot.rotation.z + gltf.scene.getObjectByName('root').rotation.z); else spring.apply();
}
function play(name) {
  mixer.stopAllAction(); comp.slots = {}; comp.queue = []; spring.reset();
  if (name.startsWith('combo:')) return comp.playCombo(name.slice(6));
  if (isPart(name)) { comp.play('reveal', 'reveal.show', { fade: 0 }); return comp.play(comp.layerOf(name), name, { fade: 0 }); }
  action = comp.play('baked', name, { fade: 0 });
}
// Cursor look-at: pointer -> normalised gaze relative to the canvas centre.
addEventListener('pointermove', (e) => comp.setLook([(e.clientX / innerWidth) * 2 - 1, 1 - (e.clientY / innerHeight) * 2]));
document.getElementById('play').onclick = () => play(sel.value);
document.getElementById('stop').onclick = () => { mixer.stopAllAction(); comp.slots = {}; comp.queue = []; action = null; spring.reset(); mixer.update(0); };
// Keep canvas/camera in sync with the real viewport (headless windows report their final size late -> off-centre frames).
function fit() { const w = document.documentElement.clientWidth, h = document.documentElement.clientHeight;
  renderer.setSize(w, h); camera.aspect = w / h; camera.updateProjectionMatrix(); }
addEventListener('resize', () => { fit(); if (q.get('shot')) renderer.render(scene, camera); }); fit();
if (q.get('sheet')) {
  // Keyframe sheet mode: ?sheet=part:NAME|combo:NAME|look:9&times=a,b,c (s). Fixed 1/120 s steps; each frame POSTed as PNG.
  document.getElementById('ui').classList.add('hide');
  renderer.setPixelRatio(1); renderer.setSize(360, 360); camera.aspect = 1; camera.updateProjectionMatrix();
  const spec = q.get('sheet'), [kind, arg] = spec.split(':'), h = 1 / 120; let t = 0, i = 0;
  const key = spec.replace(':', '_') + (q.get('tag') ? `_${q.get('tag')}` : ''); // ?tag=y15 -> extra-angle cut dir
  const post = async () => { renderer.render(scene, camera); const blob = await new Promise((r) => renderer.domElement.toBlob(r, 'image/png'));
    await fetch(`/frame?clip=${key}&i=${i++}`, { method: 'POST', body: blob }); };
  const run = (until) => { while (t + h <= until + 1e-9) { t += h; frame(h, t); } };
  if (kind === 'multi') {
    // v7 group sheets: ?sheet=multi:GROUP&clips=a,b,c&fr=.25,.5,.75 -> one Chrome renders many clips (frames in order).
    // Clips with holdAt also get a frame at holdAt (fr entry 'h').
    for (const name of q.get('clips').split(',')) {
      play(name); t = 0; const c = gltf.animations.find((x) => x.name === name), d = c.duration;
      for (const f of q.get('fr').split(',')) { run(f === 'h' ? (c.userData?.holdAt ?? d * 0.5) : Number(f) * d); await post(); }
    }
  } else if (kind === 'cam') {
    // v8: ?sheet=cam:cam.NAME&times=a,b,c -> runtime camera preset (src/runtime/camera.js) applied to the three camera; mascot at rest.
    const { resolveCamClip, cameraAt } = await import('../src/runtime/camera.js');
    const cfg = await fetch('../src/presets/camera.json?v=' + Date.now()).then((r) => r.json());
    const clip = resolveCamClip(cfg, arg); comp.play('reveal', 'reveal.show', { fade: 0 }); run(0.4);
    for (const T of q.get('times').split(',').map(Number)) {
      const v = cameraAt(clip, T), fx = (v.focus[0] - 60) / 40, fy = (60 - v.focus[1]) / 40, a = THREE.MathUtils.degToRad(v.yawDeg);
      camera.fov = v.fovDeg; camera.position.set(fx + Math.sin(a) * v.distance, fy, Math.cos(a) * v.distance); camera.lookAt(fx, fy, 0); camera.updateProjectionMatrix(); await post();
    }
  } else if (kind === 'look') {
    comp.play('reveal', 'reveal.show', { fade: 0 }); run(0.4);
    for (const gy of [1, 0, -1]) for (const gx of [-1, 0, 1]) { comp.setLook([gx, gy]); run(t + 0.8); await post(); }
  } else {
    play(kind === 'combo' ? `combo:${arg}` : arg);
    for (const T of q.get('times').split(',').map(Number)) { run(T); await post(); }
  }
  await fetch(`/frame?clip=${key}&done=1`, { method: 'POST', body: '' });
} else if (q.get('shot')) {
  // Deterministic: step mixer + physics at a fixed 120 Hz up to t, then render once.
  document.getElementById('ui').classList.add('hide'); play(q.get('clip') ?? 'idle');
  const T = Number(q.get('t') ?? 0), h = 1 / 120; let t = 0; mixer.update(0);
  while (t + h <= T + 1e-9) { t += h; frame(h, t); }
  fit(); renderer.render(scene, camera);
} else {
  sel.value = 'sequence'; play('sequence'); let t = 0;
  renderer.setAnimationLoop(() => { const dt = Math.min(clock.getDelta(), 0.05); t += dt; frame(dt, t); controls.update(); renderer.render(scene, camera); });
}
document.title = 'ready';
