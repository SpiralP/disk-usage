import filesize from "filesize";

export function bytes(b: number): string {
  // const [n, s]: [number, string] = filesize(b, { output: "array" });
  // return `${n.toFixed(1)} ${s}`;
  return filesize(b, { standard: "iec" });
}

export function time<T>(name: string, cb: () => T): T {
  const before = new Date();
  const ret = cb();
  const after = new Date();

  const millis = after.getTime() - before.getTime();
  if (millis > 50) {
    console.warn(`${name} took ${millis} ms`);
  }

  return ret;
}
