// The four scenes: the hero, and the three "how it works" demos. Each one
// builds its blocks on a Stage and returns an update(t) driven by a Player.
import { Block, PAL, type P3, type Stage, kf, lerp, lerpP3, setLeds, serverRack } from "./kit";

export interface Built {
  duration: number;
  update: (t: number, dt: number) => void;
  /** Frame to show when motion is reduced. */
  still: number;
  /** Optional scene-specific control (scene 3's vault toggle). */
  toggle?: () => void;
}

export interface SceneDef {
  bounds: readonly [P3, P3];
  background: number;
  pixel: number;
  build: (stage: Stage, root: HTMLElement) => Built;
}

const tile = (stage: Stage, tone = PAL.key, tip?: string) => stage.box(0, 0, 0, 0.45, 0.45, 0.12, tone, tip);

function moveTo(b: Block, p: P3): void {
  b.at(p[0], p[1], p[2]);
}

/** Position along a straight hop with a little arc, 0..1. */
function hop(a: P3, b: P3, u: number, lift = 0.6): P3 {
  const p = lerpP3(a, b, u);
  return [p[0], p[1], p[2] + Math.sin(Math.PI * u) * lift];
}

const without: SceneDef = {
  bounds: [
    [0, 0, 0],
    [9, 5, 3.6],
  ],
  background: PAL.panel,
  pixel: 1.5,
  build(stage) {
    stage.box(0, 0, 0, 9, 5, 0.3, PAL.plat);
    stage.path(
      [
        [1.7, 2.3, 0.31],
        [6.6, 2.3, 0.31],
      ],
      0x3a4260,
    );
    stage.box(0.8, 1.6, 0.3, 1.8, 1.4, 0.5, PAL.client, "Your machine: ssh pve1.example.com");
    stage.box(0.95, 2.95, 0.8, 1.5, 0.08, 0.9, PAL.dim);
    const queue = ["github", "backup [nobwsshd]", "pve1.example.com"];
    queue.forEach((name, i) => stage.box(2.0, 1.75, 0.8 + i * 0.15, 0.45, 0.45, 0.12, PAL.dim, `Still waiting: ${name}`));
    const leds = serverRack(stage, 6.6, 1.7, "pve1.example.com, MaxAuthTries 3");
    const names = ["deploy", "ansible", "gitlab runner"];
    const tiles = names.map((n) => tile(stage, PAL.bad, `Offered: ${n}`));
    const fail = stage.label("Too many authentication failures", [7.2, 2.3, 4.1], { className: "reject" });

    const start: P3 = [1.1, 1.8, 0.8];
    const hit: P3 = [5.8, 2.4, 1.6];
    const back: P3 = [5.55, 2.4, 1.5];
    const floor: P3 = [5.55, 2.4, 0.3];

    return {
      duration: 6,
      still: 3.9,
      update(t) {
        tiles.forEach((b, i) => {
          const u = t - i * 1.2;
          if (u < 0 || u > 1.6) {
            b.opacity = 0;
            return;
          }
          if (u < 0.9) moveTo(b, hop(start, hit, u / 0.9, 0.9));
          else if (u < 1.05) moveTo(b, lerpP3(hit, back, (u - 0.9) / 0.15));
          else moveTo(b, lerpP3(back, floor, Math.min(1, (u - 1.05) / 0.45) ** 2));
          b.opacity = kf(u, [
            [0, 0],
            [0.12, 1],
            [1.2, 1],
            [1.6, 0],
          ]);
        });
        const failing = t > 3.6 && t < 5.6;
        const blink = failing && !(t > 3.95 && t < 4.1);
        setLeds(leds, blink ? PAL.reject : PAL.idle);
        fail.style.opacity = String(kf(t, [
          [3.6, 0],
          [3.8, 1],
          [5.4, 1],
          [5.7, 0],
        ]));
      },
    };
  },
};

const withBwsshd: SceneDef = {
  bounds: [
    [0, 0, 0],
    [9, 5, 3.6],
  ],
  background: PAL.panel,
  pixel: 1.5,
  build(stage) {
    stage.box(0, 0, 0, 9, 5, 0.3, PAL.plat);
    stage.box(0.6, 1.4, 0.3, 1.8, 1.8, 1.6, PAL.vault, "Bitwarden vault: private keys never leave it");
    const door = stage.box(2.4, 1.9, 0.6, 0.03, 1.0, 0.9, [PAL.signal, PAL.signal, PAL.signal]);
    stage.path(
      [
        [2.4, 2.3, 0.31],
        [3.7, 2.3, 0.31],
      ],
      PAL.signal,
    );
    stage.box(3.7, 1.8, 0.3, 1.1, 1.1, 0.8, PAL.daemon, "bwsshd: one Host block per key, IdentitiesOnly yes");
    const pulse = stage.box(3.7, 1.8, 1.1, 1.1, 1.1, 0.02, [0xffffff, 0xffffff, 0xffffff]);
    stage.path(
      [
        [4.8, 2.3, 0.31],
        [6.6, 2.3, 0.31],
      ],
      PAL.signal,
    );
    const sheet = stage.box(3.9, 1.3, 1.25, 0.04, 0.9, 1.1, PAL.paper, "config.automatic.bw: Host pve1.example.com");
    const leds = serverRack(stage, 6.6, 1.7, "pve1.example.com: accepts the first key offered");
    const key = tile(stage, PAL.key, "pve1.example.com.pub, the only key offered");
    const ok = stage.label("1 key, 1 try", [7.2, 2.3, 4.1], { className: "signal" });
    stage.label("vault", [1.5, 2.3, 2.4]);

    const a: P3 = [2.45, 2.1, 1.0];
    const b: P3 = [3.95, 2.1, 1.12];
    const c: P3 = [6.2, 2.1, 1.6];
    const sheetBase = sheet.group.scale;

    return {
      duration: 6,
      still: 4.6,
      update(t) {
        door.opacity = kf(t, [
          [0, 0.15],
          [0.5, 0.9],
          [1.0, 0.15],
          [3.8, 0.15],
          [4.1, 0.95],
          [4.6, 0.15],
        ]);
        key.opacity = kf(t, [
          [0.4, 0],
          [0.6, 1],
          [4.2, 1],
          [4.45, 0],
        ]);
        if (t < 0.9) moveTo(key, a);
        else if (t < 2.1) moveTo(key, hop(a, b, (t - 0.9) / 1.2, 0.4));
        else if (t < 2.7) moveTo(key, b);
        else moveTo(key, hop(b, c, Math.min(1, (t - 2.7) / 1.5), 0.5));
        pulse.opacity = kf(t, [
          [2.0, 0],
          [2.2, 0.55],
          [2.8, 0],
        ]);
        const unfold = kf(t, [
          [2.2, 0.02],
          [2.8, 1],
          [5.6, 1],
          [5.95, 0.02],
        ]);
        sheetBase.set(1, unfold, 1);
        sheet.group.position.y = 1.25 + (1.1 * unfold) / 2;
        const lit = t > 4.3 && t < 5.7;
        setLeds(leds, lit ? PAL.signal : PAL.idle);
        ok.style.opacity = String(kf(t, [
          [4.3, 0],
          [4.5, 1],
          [5.5, 1],
          [5.8, 0],
        ]));
      },
    };
  },
};

const PRESENT = `Host pve1.example.com
    HostName pve1.example.com
    User debian
    IdentityFile ~/.ssh/bwsshd-keys/pve1.example.com.pub
    IdentitiesOnly yes`;
const ABSENT = "# bwsshd: Bitwarden SSH agent socket absent, no keys offered.";

const lifecycle: SceneDef = {
  bounds: [
    [0, 0, 0],
    [5, 4, 2.0],
  ],
  background: PAL.panel,
  pixel: 1.5,
  build(stage, root) {
    stage.box(0, 0, 0, 5, 4, 0.3, PAL.plat);
    const pads: P3[] = [];
    for (let i = 0; i < 3; i++)
      for (let j = 0; j < 2; j++) {
        const x = 2.8 + j * 0.8;
        const y = 0.8 + i * 0.9;
        stage.path(
          [
            [x, y, 0.31],
            [x + 0.6, y, 0.31],
            [x + 0.6, y + 0.6, 0.31],
            [x, y + 0.6, 0.31],
            [x, y, 0.31],
          ],
          0x3a4260,
        );
        pads.push([x, y, 0.3]);
      }
    const vault = stage.box(0.5, 0.6, 0.3, 1.6, 1.6, 1.4, PAL.vault, "Bitwarden desktop app");
    const door = stage.box(2.1, 1.1, 0.7, 0.03, 0.6, 0.6, [PAL.signal, PAL.signal, PAL.signal]);
    const hosts = ["pve1.example.com", "pve2.example.com", "db.example.com", "git.example.com", "10.0.0.5", "ansible"];
    const keys = pads.map((_, i) => stage.box(0, 0, 0, 0.6, 0.6, 0.12, PAL.key, `bwsshd-keys/${hosts[i]}.pub`));
    stage.label("bwsshd-keys/", [3.6, 2.2, 1.3]);

    const card = root.closest("[data-scene-card]");
    const readout = card?.querySelector<HTMLElement>("[data-readout]");
    const code = card?.querySelector<HTMLElement>("[data-config]");
    const button = card?.querySelector<HTMLElement>("[data-toggle]");

    // v eases toward the target: 1 = Bitwarden open, 0 = closed. Until the
    // visitor presses the button, the scene toggles by itself every 4 s.
    let v = 1;
    let manual = false;
    let target = 1;
    let shown = -1;

    const show = (open: boolean) => {
      if (shown === Number(open)) return;
      shown = Number(open);
      if (readout) {
        readout.textContent = open ? "state: present" : "state: absent";
        readout.dataset.state = open ? "present" : "absent";
      }
      if (code) code.textContent = open ? PRESENT : ABSENT;
      if (button) button.textContent = open ? "Close vault" : "Open vault";
    };

    return {
      duration: 8,
      still: 0,
      update(t, dt) {
        if (!manual) target = t < 4 ? 1 : 0;
        v += (target - v) * Math.min(1, dt * 3.2);
        if (dt === 0) v = target;
        vault.tone(PAL.dim, PAL.vault, v);
        door.opacity = lerp(0.06, 1, v);
        keys.forEach((k, i) => {
          const u = Math.min(1, Math.max(0, v * 1.5 - i * 0.08));
          const [x, y, z] = pads[i];
          k.at(x, y, z - (1 - u) * 0.35);
          k.opacity = u;
        });
        show(target === 1);
      },
      toggle() {
        manual = true;
        target = target === 1 ? 0 : 1;
        show(target === 1);
      },
    };
  },
};

const hero: SceneDef = {
  bounds: [
    [0, 0, 0],
    [12, 8, 3.6],
  ],
  background: PAL.ground,
  pixel: 2,
  build(stage) {
    stage.box(0, 0, 0, 12, 8, 0.3, PAL.plat);
    stage.box(1, 1.5, 0.3, 3, 3, 2.4, PAL.vault, "Bitwarden vault: keys stay here, signing happens in the app");
    const door = stage.box(4, 2.1, 0.7, 0.03, 1.8, 1.6, [0x161a38, 0x161a38, 0x161a38]);
    const lock = stage.box(4.02, 2.85, 1.35, 0.03, 0.3, 0.3, [PAL.signal, PAL.signal, PAL.signal]);
    stage.path(
      [
        [4, 3, 0.31],
        [5.6, 3, 0.31],
        [5.6, 3.6, 0.31],
      ],
      PAL.signal,
    );
    stage.box(5.6, 3, 0.3, 1.5, 1.5, 1.0, PAL.daemon, "bwsshd: maps each key name to a Host block");
    const servers = [
      { y: 0.6, name: "git.example.com" },
      { y: 3.1, name: "pve1.example.com" },
      { y: 5.6, name: "db.example.com" },
    ];
    const racks = servers.map((s) => serverRack(stage, 9.2, s.y, `${s.name}: one pinned key`));
    for (const s of servers)
      stage.path(
        [
          [7.1, 3.75, 0.31],
          [8.2, 3.75, 0.31],
          [8.2, s.y + 0.6, 0.31],
          [9.2, s.y + 0.6, 0.31],
        ],
        0x2a3e5a,
      );
    const key = tile(stage);
    stage.label("bitwarden vault", [2.5, 3, 3.6]);
    stage.label("bwsshd", [6.35, 3.75, 2.0], { className: "signal" });
    const names = servers.map((s) => stage.label(s.name, [9.8, s.y + 0.6, 3.75], { className: "small" }));

    const a: P3 = [4.05, 2.85, 1.2];
    const b: P3 = [6.15, 3.5, 1.32];

    return {
      duration: 9,
      still: 2.2,
      update(t) {
        const k = Math.floor(t / 3);
        const u = t - k * 3;
        const s = servers[k];
        const c: P3 = [8.6, s.y + 0.4, 1.8];
        lock.opacity = kf(u, [
          [0, 0.5],
          [0.2, 1],
          [0.6, 0.5],
          [2.0, 0.5],
          [2.15, 1],
          [2.5, 0.5],
        ]);
        door.opacity = 1;
        if (u < 1.0) moveTo(key, hop(a, b, u, 0.5));
        else if (u < 1.3) moveTo(key, b);
        else moveTo(key, hop(b, c, Math.min(1, (u - 1.3) / 0.8), 0.6));
        key.opacity = kf(u, [
          [0, 0],
          [0.15, 1],
          [2.0, 1],
          [2.2, 0],
        ]);
        racks.forEach((leds, i) => setLeds(leds, i === k && u > 2.1 ? PAL.signal : PAL.idle));
        names.forEach((el, i) => el.classList.toggle("signal", i === k));
      },
    };
  },
};

export const SCENES: Record<string, SceneDef> = { hero, without, with: withBwsshd, lifecycle };
