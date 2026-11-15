// Values shared by pages, components and the install scripts.
export const REPO = "https://github.com/Dxsk/bwsshd";
export const FORGE = "https://git.infrasouveraine.fr/dxsk/bwsshd";

export const siteUrl = (path: string) => new URL(path, import.meta.env.SITE).href;
