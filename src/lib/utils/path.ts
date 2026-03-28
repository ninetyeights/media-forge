/** Detect path separator */
export function pathSep(p: string): string {
  return p.includes("\\") ? "\\" : "/";
}

/** Split a file path into directory, filename, and detected separator */
export function splitPath(p: string): { dir: string; name: string; sep: string } {
  const sep = pathSep(p);
  const lastSep = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
  if (lastSep < 0) return { dir: "", name: p, sep };
  return { dir: p.substring(0, lastSep), name: p.substring(lastSep + 1), sep };
}

/** Get just the filename from a path */
export function fileName(p: string): string {
  const lastSep = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
  return lastSep < 0 ? p : p.substring(lastSep + 1);
}

/** Get the parent directory from a path */
export function parentDir(p: string): string {
  const lastSep = Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
  return lastSep < 0 ? "" : p.substring(0, lastSep);
}

/** Get the last segment (folder name or file name) */
export function lastSegment(p: string): string {
  return fileName(p);
}

/** Join path segments with the given separator, filtering out empty parts */
export function joinPath(parts: string[], sep: string): string {
  return parts.filter(Boolean).join(sep);
}

/** Split path by both / and \\ */
export function splitSegments(p: string): string[] {
  return p.split(/[/\\]/).filter(Boolean);
}

/** Find last separator index in path */
export function lastSepIndex(p: string): number {
  return Math.max(p.lastIndexOf("/"), p.lastIndexOf("\\"));
}

/** Strip leading separator */
export function stripLeadingSep(p: string): string {
  return p.replace(/^[/\\]/, "");
}
