// A tiny isometric scene kit on top of three.js, shared by every demo.
//
// World coordinates follow the drawings: x and y lie on the ground, z points up.
// three.js is y-up, so iso(x, y, z) maps to Vector3(x, z, y). Blocks are flat
// shaded with three fixed tones (top, left, right) instead of lights: it keeps
// the illustrated look and there is nothing to tune.
import {
  BoxGeometry,
  BufferGeometry,
  Color,
  EdgesGeometry,
  Group,
  Line,
  LineBasicMaterial,
  LineDashedMaterial,
  LineSegments,
  Mesh,
  MeshBasicMaterial,
  Object3D,
  OrthographicCamera,
  Raycaster,
  Scene,
  Vector2,
  Vector3,
  WebGLRenderer,
} from "three";
import { EffectComposer } from "three/addons/postprocessing/EffectComposer.js";
import { RenderPass } from "three/addons/postprocessing/RenderPass.js";
import { ShaderPass } from "three/addons/postprocessing/ShaderPass.js";

export type Tone = readonly [top: number, left: number, right: number];
export type P3 = readonly [number, number, number];

export const PAL = {
  ground: 0x0a0d14,
  panel: 0x0e1220,
  edge: 0x0a0d14,
  plat: [0x121828, 0x0c1020, 0x0f1424],
  vault: [0x2e3460, 0x1c2044, 0x252a55],
  daemon: [0x4fe3c8, 0x258c7b, 0x33ae99],
  server: [0x2a3250, 0x171d33, 0x1f2640],
  key: [0x4fe3c8, 0x258c7b, 0x33ae99],
  bad: [0xffb547, 0xa8701f, 0xd18f30],
  dim: [0x3a4260, 0x232942, 0x2c3350],
  client: [0x3a4268, 0x20263f, 0x2a3152],
  paper: [0xe8ebf4, 0xb8c0d4, 0xd2d8e6],
  slot: [0x0a0d14, 0x0a0d14, 0x0a0d14],
  signal: 0x4fe3c8,
  reject: 0xffb547,
  idle: 0x3a4260,
} satisfies Record<string, Tone | number>;

export const iso = (x: number, y: number, z: number) => new Vector3(x, z, y);

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));
export const smooth = (x: number) => x * x * (3 - 2 * x);
export const lerp = (a: number, b: number, t: number) => a + (b - a) * t;

/** Keyframe interpolation: keys are [time, value] pairs sorted by time. */
export function kf(t: number, keys: readonly (readonly [number, number])[], ease = smooth): number {
  if (t <= keys[0][0]) return keys[0][1];
  for (let i = 1; i < keys.length; i++) {
    const [t1, v1] = keys[i];
    if (t <= t1) {
      const [t0, v0] = keys[i - 1];
      return lerp(v0, v1, ease(clamp01((t - t0) / (t1 - t0 || 1))));
    }
  }
  return keys[keys.length - 1][1];
}

export function lerpP3(a: P3, b: P3, t: number): P3 {
  return [lerp(a[0], b[0], t), lerp(a[1], b[1], t), lerp(a[2], b[2], t)];
}

/** A box with its three tones and an outline, positioned by its min corner. */
export class Block {
  readonly group = new Group();
  private readonly faces: MeshBasicMaterial[];
  private readonly edges: LineBasicMaterial;
  private readonly size: P3;

  constructor(w: number, d: number, h: number, tone: Tone) {
    this.size = [w, d, h];
    const geo = new BoxGeometry(w, h, d);
    const [top, left, right] = tone.map((c) => new MeshBasicMaterial({ color: c, transparent: true }));
    // BoxGeometry face order: +x, -x, +y, -y, +z, -z. From the iso camera the
    // visible sides are +x (right), +y (top) and +z (left); the hidden ones
    // reuse the same tones so an orbit never reveals an odd color.
    this.faces = [right, top, left];
    const mesh = new Mesh(geo, [right, right, top, top, left, left]);
    this.edges = new LineBasicMaterial({ color: PAL.edge, transparent: true });
    const lines = new LineSegments(new EdgesGeometry(geo), this.edges);
    this.group.add(mesh, lines);
  }

  at(x: number, y: number, z: number): this {
    const [w, d, h] = this.size;
    this.group.position.copy(iso(x + w / 2, y + d / 2, z + h / 2));
    return this;
  }

  set opacity(o: number) {
    for (const m of this.faces) m.opacity = o;
    this.edges.opacity = o;
    this.group.visible = o > 0.01;
  }

  tone(a: Tone, b: Tone = a, t = 0): void {
    const [right, top, left] = this.faces;
    top.color.setHex(a[0]).lerp(new Color(b[0]), t);
    left.color.setHex(a[1]).lerp(new Color(b[1]), t);
    right.color.setHex(a[2]).lerp(new Color(b[2]), t);
  }

  get object(): Object3D {
    return this.group;
  }
}

// Ordered (Bayer 4x4) dither over a soft vertical shade: gives the flat scene
// the printed, pixel-art texture of the mockups for a few lines of GLSL.
const DitherShader = {
  uniforms: { tDiffuse: { value: null }, levels: { value: 28.0 } },
  vertexShader: /* glsl */ `
    varying vec2 vUv;
    void main() { vUv = uv; gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0); }`,
  fragmentShader: /* glsl */ `
    uniform sampler2D tDiffuse;
    uniform float levels;
    varying vec2 vUv;
    float bayer(vec2 p) {
      int x = int(mod(p.x, 4.0));
      int y = int(mod(p.y, 4.0));
      int i = x + y * 4;
      float m[16] = float[16](0., 8., 2., 10., 12., 4., 14., 6., 3., 11., 1., 9., 15., 7., 13., 5.);
      return m[i] / 16.0 - 0.5;
    }
    // The composer renders in linear space and this is the last pass, so it
    // owns the sRGB encode; dithering after it keeps the steps even.
    vec3 toSrgb(vec3 c) {
      return mix(c * 12.92, 1.055 * pow(c, vec3(1.0 / 2.4)) - 0.055, step(0.0031308, c));
    }
    void main() {
      vec3 c = toSrgb(texture2D(tDiffuse, vUv).rgb);
      float shade = mix(0.74, 1.04, smoothstep(0.0, 1.0, vUv.y));
      vec3 col = floor(c * shade * levels + bayer(gl_FragCoord.xy) + 0.5) / levels;
      gl_FragColor = vec4(col, 1.0);
    }`,
};

export interface StageOptions {
  /** Ground-space bounds used to frame the camera: [min, max]. */
  bounds: readonly [P3, P3];
  /** Scene background, matching the card behind the canvas. */
  background?: number;
  /** CSS pixels per rendered pixel; 2 gives the chunky pixel-art look. */
  pixel?: number;
  /** Extra framing margin, 1 = tight. */
  pad?: number;
  /** Let the visitor drag to orbit around the vertical axis. */
  orbit?: boolean;
}

interface Label {
  el: HTMLElement;
  at: Vector3;
  follow?: Object3D;
  offset: Vector3;
}

const BASE_AZIMUTH = Math.PI / 4;
const ELEVATION = Math.atan(1 / Math.SQRT2); // 35.264 deg: true isometric
const ORBIT_LIMIT = 0.7;

/** Owns the renderer, camera, labels, hover tooltips and the drag orbit. */
export class Stage {
  readonly scene = new Scene();
  readonly camera = new OrthographicCamera(-1, 1, 1, -1, 0.1, 200);
  readonly renderer: WebGLRenderer;
  private readonly composer: EffectComposer;
  private readonly overlay: HTMLDivElement;
  private readonly tip: HTMLDivElement;
  private readonly labels: Label[] = [];
  private readonly hoverables: Object3D[] = [];
  private readonly raycaster = new Raycaster();
  private readonly target: Vector3;
  private readonly corners: Vector3[];
  private readonly pixel: number;
  private readonly pad: number;
  private azimuth = BASE_AZIMUTH;
  private azimuthGoal = BASE_AZIMUTH;
  private dirty = true;
  private readonly ro: ResizeObserver;

  constructor(
    private readonly root: HTMLElement,
    opts: StageOptions,
  ) {
    this.pixel = opts.pixel ?? 1.5;
    this.pad = opts.pad ?? 1.08;
    this.scene.background = new Color(opts.background ?? PAL.panel);

    const [min, max] = opts.bounds;
    this.target = iso((min[0] + max[0]) / 2, (min[1] + max[1]) / 2, (min[2] + max[2]) / 2);
    this.corners = [];
    for (const x of [min[0], max[0]])
      for (const y of [min[1], max[1]]) for (const z of [min[2], max[2]]) this.corners.push(iso(x, y, z));

    this.renderer = new WebGLRenderer({ antialias: false, powerPreference: "low-power" });
    const canvas = this.renderer.domElement;
    canvas.className = "iso-canvas";
    canvas.setAttribute("aria-hidden", "true");
    root.append(canvas);

    this.composer = new EffectComposer(this.renderer);
    this.composer.addPass(new RenderPass(this.scene, this.camera));
    this.composer.addPass(new ShaderPass(DitherShader));

    this.overlay = document.createElement("div");
    this.overlay.className = "iso-labels";
    this.tip = document.createElement("div");
    this.tip.className = "iso-tip";
    this.tip.hidden = true;
    root.append(this.overlay, this.tip);

    this.placeCamera();
    this.ro = new ResizeObserver(() => this.resize());
    this.ro.observe(root);
    this.resize();

    canvas.addEventListener("pointermove", (e) => this.hover(e));
    canvas.addEventListener("pointerleave", () => (this.tip.hidden = true));
    if (opts.orbit !== false) this.enableOrbit(canvas);
  }

  add<T extends Object3D>(o: T): T {
    this.scene.add(o);
    this.dirty = true;
    return o;
  }

  box(x: number, y: number, z: number, w: number, d: number, h: number, tone: Tone, tip?: string): Block {
    const b = new Block(w, d, h, tone).at(x, y, z);
    this.add(b.group);
    if (tip) this.hoverable(b.group, tip);
    return b;
  }

  hoverable(o: Object3D, tip: string): void {
    o.userData.tip = tip;
    this.hoverables.push(o);
  }

  /** A ground-level polyline; dashed by default, like the drawings. */
  path(points: readonly P3[], color: number, dashed = true): Line {
    const geo = new BufferGeometry().setFromPoints(points.map((p) => iso(...p)));
    const mat = dashed
      ? new LineDashedMaterial({ color, dashSize: 0.18, gapSize: 0.18, transparent: true })
      : new LineBasicMaterial({ color, transparent: true });
    const line = new Line(geo, mat);
    if (dashed) line.computeLineDistances();
    return this.add(line);
  }

  /** An HTML label pinned to a world point, or following an object. */
  label(text: string, at: P3, opts: { className?: string; follow?: Object3D } = {}): HTMLElement {
    const el = document.createElement("span");
    el.className = `iso-label ${opts.className ?? ""}`;
    el.textContent = text;
    this.overlay.append(el);
    const p = iso(...at);
    this.labels.push({
      el,
      at: p,
      follow: opts.follow,
      offset: opts.follow ? p.clone().sub(opts.follow.position) : new Vector3(),
    });
    return el;
  }

  invalidate(): void {
    this.dirty = true;
  }

  /** Draws a frame if anything changed. Call once per animation frame. */
  render(): void {
    if (Math.abs(this.azimuthGoal - this.azimuth) > 1e-4) {
      this.azimuth = lerp(this.azimuth, this.azimuthGoal, 0.18);
      this.placeCamera();
      this.dirty = true;
    }
    if (!this.dirty) return;
    this.dirty = false;
    this.composer.render();
    this.placeLabels();
  }

  dispose(): void {
    this.ro.disconnect();
    this.renderer.dispose();
  }

  private placeCamera(): void {
    const r = 40;
    const dir = new Vector3(
      Math.cos(ELEVATION) * Math.cos(this.azimuth),
      Math.sin(ELEVATION),
      Math.cos(ELEVATION) * Math.sin(this.azimuth),
    );
    this.camera.position.copy(this.target).addScaledVector(dir, r);
    this.camera.lookAt(this.target);
    this.camera.updateMatrixWorld();
  }

  private resize(): void {
    const w = Math.max(1, this.root.clientWidth);
    const h = Math.max(1, this.root.clientHeight);
    const ratio = Math.min(window.devicePixelRatio || 1, 2) / this.pixel;
    this.renderer.setPixelRatio(ratio);
    this.renderer.setSize(w, h, false);
    this.composer.setPixelRatio(ratio);
    this.composer.setSize(w, h);

    // Frame the bounds at the base azimuth only, so orbiting never zooms.
    const saved = this.azimuth;
    this.azimuth = BASE_AZIMUTH;
    this.placeCamera();
    let hx = 0;
    let hy = 0;
    for (const c of this.corners) {
      const v = c.clone().applyMatrix4(this.camera.matrixWorldInverse);
      hx = Math.max(hx, Math.abs(v.x));
      hy = Math.max(hy, Math.abs(v.y));
    }
    this.azimuth = saved;
    this.placeCamera();

    const aspect = w / h;
    let halfW = hx * this.pad;
    let halfH = hy * this.pad;
    if (halfW / halfH > aspect) halfH = halfW / aspect;
    else halfW = halfH * aspect;
    Object.assign(this.camera, { left: -halfW, right: halfW, top: halfH, bottom: -halfH });
    this.camera.updateProjectionMatrix();
    this.dirty = true;
    this.render();
  }

  private placeLabels(): void {
    const w = this.root.clientWidth;
    const h = this.root.clientHeight;
    const v = new Vector3();
    for (const l of this.labels) {
      if (l.follow) {
        v.copy(l.follow.position).add(l.offset);
        l.el.style.opacity = l.follow.visible ? "" : "0";
      } else v.copy(l.at);
      v.project(this.camera);
      l.el.style.transform = `translate(${((v.x + 1) / 2) * w}px, ${((1 - v.y) / 2) * h}px) translate(-50%, -100%)`;
    }
  }

  private hover(e: PointerEvent): void {
    if (!this.hoverables.length) return;
    const rect = this.root.getBoundingClientRect();
    const ndc = new Vector2(((e.clientX - rect.left) / rect.width) * 2 - 1, -((e.clientY - rect.top) / rect.height) * 2 + 1);
    this.raycaster.setFromCamera(ndc, this.camera);
    const hit = this.raycaster.intersectObjects(this.hoverables, true).find((i) => this.tipOf(i.object));
    const tip = hit && this.tipOf(hit.object);
    if (!tip) {
      this.tip.hidden = true;
      return;
    }
    this.tip.textContent = tip;
    this.tip.hidden = false;
    this.tip.style.transform = `translate(${e.clientX - rect.left + 14}px, ${e.clientY - rect.top + 14}px)`;
  }

  private tipOf(o: Object3D | null): string | undefined {
    for (let n = o; n; n = n.parent) {
      if (!n.visible) return undefined;
      if (typeof n.userData.tip === "string") return n.userData.tip;
    }
    return undefined;
  }

  private enableOrbit(canvas: HTMLCanvasElement): void {
    let startX = 0;
    let startAz = BASE_AZIMUTH;
    let dragging = false;
    canvas.addEventListener("pointerdown", (e) => {
      dragging = true;
      startX = e.clientX;
      startAz = this.azimuthGoal;
      canvas.setPointerCapture(e.pointerId);
      canvas.classList.add("dragging");
    });
    canvas.addEventListener("pointermove", (e) => {
      if (!dragging) return;
      const dx = (e.clientX - startX) / Math.max(200, this.root.clientWidth);
      this.azimuthGoal = Math.min(BASE_AZIMUTH + ORBIT_LIMIT, Math.max(BASE_AZIMUTH - ORBIT_LIMIT, startAz - dx * 2.2));
    });
    const end = () => {
      dragging = false;
      canvas.classList.remove("dragging");
    };
    canvas.addEventListener("pointerup", end);
    canvas.addEventListener("pointercancel", end);
    canvas.addEventListener("dblclick", () => (this.azimuthGoal = BASE_AZIMUTH));
  }
}

/** Drives a scene: a looping timeline, play/pause, and pausing off screen. */
export class Player {
  t = 0;
  playing: boolean;
  private last = 0;
  private raf = 0;
  private visible = false;
  private readonly io: IntersectionObserver;

  constructor(
    private readonly stage: Stage,
    private readonly opts: {
      duration: number;
      update: (t: number, dt: number) => void;
      autoplay: boolean;
      onTick?: (t: number) => void;
      /** Frame shown when not playing (reduced motion). */
      still?: number;
    },
  ) {
    this.playing = opts.autoplay;
    if (!this.playing && opts.still !== undefined) this.t = opts.still;
    this.opts.update(this.t, 0);
    this.stage.invalidate();
    this.stage.render();
    this.io = new IntersectionObserver(([e]) => {
      this.visible = e.isIntersecting;
      if (this.visible) this.kick();
    });
    this.io.observe(stage.renderer.domElement);
  }

  toggle(): boolean {
    this.playing = !this.playing;
    if (this.playing && this.t >= this.opts.duration) this.t = 0;
    this.kick();
    return this.playing;
  }

  restart(): void {
    this.t = 0;
    this.playing = true;
    this.kick();
  }

  private kick(): void {
    if (this.raf) return;
    this.last = performance.now();
    this.raf = requestAnimationFrame((n) => this.frame(n));
  }

  private frame(now: number): void {
    this.raf = 0;
    // A rAF timestamp can predate the performance.now() taken in kick(), so
    // clamp at 0: a negative step would run the timeline backwards.
    const dt = Math.min(0.1, Math.max(0, (now - this.last) / 1000));
    this.last = now;
    if (this.playing) {
      this.t = (this.t + dt) % this.opts.duration;
      this.opts.update(this.t, dt);
      this.opts.onTick?.(this.t);
      this.stage.invalidate();
    }
    this.stage.render();
    // Keep ticking while on screen even when paused: orbit easing and hover
    // still need frames, and render() is a no-op when nothing changed.
    if (this.visible) this.raf = requestAnimationFrame((n) => this.frame(n));
  }
}

export const prefersReducedMotion = () => window.matchMedia("(prefers-reduced-motion: reduce)").matches;

export function webglAvailable(): boolean {
  try {
    const c = document.createElement("canvas");
    return !!c.getContext("webgl2");
  } catch {
    return false;
  }
}

/** Three LED slots on a server's right face; returns the LED blocks. */
export function serverRack(stage: Stage, x: number, y: number, tip: string): Block[] {
  stage.box(x, y, 0.3, 1.2, 1.2, 3, PAL.server, tip);
  const leds: Block[] = [];
  for (let i = 0; i < 3; i++) {
    const z = 0.8 + i * 0.85;
    stage.box(x + 1.2, y + 0.2, z, 0.03, 0.8, 0.12, PAL.slot);
    const led = stage.box(x + 1.215, y + 0.25, z + 0.02, 0.03, 0.2, 0.08, [PAL.idle, PAL.idle, PAL.idle]);
    leds.push(led);
  }
  return leds;
}

export function setLeds(leds: Block[], color: number): void {
  for (const l of leds) l.tone([color, color, color]);
}
