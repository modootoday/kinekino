import * as THREE from 'three';
// Spring-bone layer for the tail. Runs after the AnimationMixer each frame:
// each bone gets a damped angular offset driven by the parent's angular acceleration (inertia),
// clamped to a per-joint limit, and a thickness scale that compensates linear-blend pinching (volume preservation).
export class TailSpring {
  constructor(root, cfg) {
    this.cfg = cfg; this.enabled = true; this.acc = 0;
    this.bones = cfg.bones.map((b) => ({ ...b, node: root.getObjectByName(b.name), phi: 0, vel: 0 }));
    this.driver = root; this.prevAngle = null; this.prevVel = 0;
  }
  reset() { for (const b of this.bones) { b.phi = 0; b.vel = 0; } this.prevAngle = null; this.prevVel = 0; }
  // driverAngle: world Z rotation of the body (radians); called once per rendered frame after mixer.update.
  update(dt, driverAngle) {
    const h = this.cfg.fixedStep; this.acc += dt;
    while (this.acc >= h) { this.step(h, driverAngle); this.acc -= h; }
    this.apply();
  }
  step(h, angle) {
    const vel = this.prevAngle === null ? 0 : (angle - this.prevAngle) / h;
    const accel = (vel - this.prevVel) / h; this.prevAngle = angle; this.prevVel = vel;
    let parentAccel = accel * this.cfg.inertiaGain;
    for (const b of this.bones) {
      const a = (-b.stiffness * b.phi - b.damping * b.vel) / b.mass - parentAccel;
      b.vel += a * h; b.phi += b.vel * h;
      const lim = THREE.MathUtils.degToRad(b.limitDeg);
      if (Math.abs(b.phi) > lim) { b.phi = Math.sign(b.phi) * lim; b.vel = 0; }
      parentAccel = a * this.cfg.chainLag; // child feels the parent's swing, attenuated
    }
  }
  apply() {
    for (const b of this.bones) {
      if (!b.node) continue;
      if (!this.enabled) { b.node.scale.y = 1; continue; }
      b.node.rotation.z += b.phi;
      const lim = THREE.MathUtils.degToRad(b.limitDeg + 6), z = b.node.rotation.z;
      if (Math.abs(z) > lim) b.node.rotation.z = Math.sign(z) * lim; // hard joint limit on the combined pose
      b.node.scale.y = 1 + this.cfg.volumePreserve * Math.abs(b.node.rotation.z);
    }
  }
}
