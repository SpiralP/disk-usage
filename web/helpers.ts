import filesize from "filesize";

export function bytes(b: number): string {
  // const [n, s]: [number, string] = filesize(b, { output: "array" });
  // return `${n.toFixed(1)} ${s}`;
  return filesize(b, { standard: "iec" });
}
