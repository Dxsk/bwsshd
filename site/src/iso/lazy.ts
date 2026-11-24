// Entry point kept free of three.js: the scene code (and three itself) is only
// fetched once a scene gets close to the viewport.
export function mountScenesLazily(): void {
  const cards = document.querySelectorAll<HTMLElement>("[data-scene-card]");
  if (!cards.length) return;
  const io = new IntersectionObserver(
    (entries) => {
      for (const e of entries) {
        if (!e.isIntersecting) continue;
        io.unobserve(e.target);
        import("./mount").then((m) => m.mountScene(e.target as HTMLElement));
      }
    },
    { rootMargin: "300px" },
  );
  cards.forEach((c) => io.observe(c));
}
