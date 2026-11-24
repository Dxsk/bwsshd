import { Player, Stage, prefersReducedMotion, webglAvailable } from "./kit";
import { SCENES } from "./scenes";

/**
 * Turns a [data-scene-card] element into a live scene. Without WebGL the
 * static SVG still inside the stage simply stays in place.
 */
export function mountScene(card: HTMLElement): void {
  const host = card.querySelector<HTMLElement>("[data-stage]");
  const def = host && SCENES[host.dataset.scene ?? ""];
  if (!host || !def || !webglAvailable()) return;

  let stage: Stage;
  try {
    stage = new Stage(host, { bounds: def.bounds, background: def.background, pixel: def.pixel });
  } catch {
    return;
  }
  const built = def.build(stage, host);
  const progress = card.querySelector<HTMLElement>("[data-progress]");
  const player = new Player(stage, {
    duration: built.duration,
    update: built.update,
    still: built.still,
    autoplay: !prefersReducedMotion(),
    onTick: (t) => progress?.style.setProperty("--p", String(t / built.duration)),
  });
  host.classList.add("live");

  const play = card.querySelector<HTMLButtonElement>("[data-play]");
  const syncPlay = () => {
    if (!play) return;
    play.dataset.playing = String(player.playing);
    play.setAttribute("aria-label", `${player.playing ? "Pause" : "Play"} ${play.dataset.name ?? "scene"}`);
  };
  play?.addEventListener("click", () => {
    player.toggle();
    syncPlay();
  });
  syncPlay();

  const toggle = card.querySelector<HTMLButtonElement>("[data-toggle]");
  if (toggle && built.toggle) {
    const fn = built.toggle;
    toggle.addEventListener("click", () => {
      fn();
      // Paused (reduced motion): jump straight to the new state.
      if (!player.playing) {
        built.update(player.t, 0);
        stage.invalidate();
        stage.render();
      }
    });
  }
}

