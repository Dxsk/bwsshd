// Values shared by pages, components and the install scripts.
export const AUTHOR = "https://github.com/Dxsk";
export const REPO = `${AUTHOR}/bwsshd`;

export const siteUrl = (path: string) => new URL(path, import.meta.env.SITE).href;
