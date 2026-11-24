// Port of src/target.rs, so the playground shows exactly what bwsshd writes.
// Keep the two in step when the naming rules change.

export interface Target {
  /** The Host pattern: what you type after `ssh`. */
  host: string;
  /** Where SSH connects; differs from `host` only with `(alias,address)`. */
  hostname: string;
  user?: string;
  port?: string;
}

export const skipKey = (name: string) => name.includes("[nobwsshd]");

const isDigits = (s: string) => /^[0-9]+$/.test(s);

function isIpv4(s: string): boolean {
  const parts = s.split(".");
  return parts.length === 4 && parts.every((p) => /^(0|[1-9][0-9]{0,2})$/.test(p) && Number(p) <= 255);
}

function isIpv6(s: string): boolean {
  if (!s.includes(":") || !/^[0-9A-Fa-f:.]+$/.test(s)) return false;
  try {
    new URL(`http://[${s}]/`);
    return true;
  } catch {
    return false;
  }
}

function splitHostPort(s: string): [string, string | undefined] {
  let host: string | undefined;
  let port: string | undefined;
  if (s.startsWith("[")) {
    const i = s.indexOf("]:");
    if (i >= 0) {
      host = s.slice(1, i);
      port = s.slice(i + 2);
      if (/[[\]]/.test(host)) host = undefined;
    }
  } else {
    const i = s.lastIndexOf(":");
    if (i >= 0) {
      host = s.slice(0, i);
      port = s.slice(i + 1);
      if (/[:[\]]/.test(host)) host = undefined;
    }
  }
  return host !== undefined && port !== undefined && isDigits(port) ? [host, port] : [s, undefined];
}

export function looksLikeHost(s: string): boolean {
  if (/[/\\ ]/.test(s)) return false;
  if (isIpv4(s) || isIpv6(s)) return true;
  const dot = s.lastIndexOf(".");
  if (dot < 0) return false;
  return /^[A-Za-z]+$/.test(s.slice(dot + 1));
}

const isAlias = (s: string) => /^[A-Za-z0-9_][A-Za-z0-9._-]*$/.test(s);

function isAddress(s: string): boolean {
  if (isIpv4(s) || isIpv6(s)) return true;
  return s !== "" && s.split(".").every((l) => /^[A-Za-z0-9]([A-Za-z0-9-]*[A-Za-z0-9])?$/.test(l));
}

/** `(alias,address)[:port]`, mirroring split_alias in target.rs. */
function splitAlias(s: string): [string, string, string | undefined] | undefined {
  const close = s.indexOf(")");
  if (!s.startsWith("(") || close < 0) return undefined;
  const inner = s.slice(1, close);
  const after = s.slice(close + 1);
  let port: string | undefined;
  if (after) {
    if (!after.startsWith(":") || !isDigits(after.slice(1))) return undefined;
    port = after.slice(1);
  }
  const comma = inner.indexOf(",");
  if (comma < 0) return undefined;
  const alias = inner.slice(0, comma);
  let address = inner.slice(comma + 1);
  if (address.startsWith("[") && address.endsWith("]")) address = address.slice(1, -1);
  return isAlias(alias) && isAddress(address) ? [alias, address, port] : undefined;
}

export function parseTarget(name: string): Target | undefined {
  for (const raw of name.split(/\s+/).filter(Boolean)) {
    const field = raw.replace(/^\.+|\.+$/g, "");
    const at = field.lastIndexOf("@");
    const user = at >= 0 ? field.slice(0, at) : undefined;
    const hostport = at >= 0 ? field.slice(at + 1) : field;
    if (hostport.startsWith("(")) {
      const alias = splitAlias(hostport);
      if (alias) return { host: alias[0], hostname: alias[1], user, port: alias[2] };
      continue;
    }
    const [host, port] = splitHostPort(hostport);
    if (looksLikeHost(host)) return { host, hostname: host, user, port };
  }
  return undefined;
}

export const sanitize = (s: string) => s.trim().replace(/[^A-Za-z0-9._-]/g, "_");
