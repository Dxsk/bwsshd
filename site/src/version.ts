// release-please bumps Cargo.toml, so the crate version is the single source of
// truth; reading it here keeps the site from drifting.
import cargo from "../../Cargo.toml?raw";

export const VERSION = /^version\s*=\s*"([^"]+)"/m.exec(cargo)?.[1] ?? "dev";
