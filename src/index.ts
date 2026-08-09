import init, { Kanji2Koe as WasmKanji2Koe } from "../pkg/aqkanji2koe_wasm.js";

export interface LoadOptions {
  wasmPath?: string | URL;
}

export class Kanji2Koe {
  readonly #inner: WasmKanji2Koe;

  constructor(inner: WasmKanji2Koe) {
    this.#inner = inner;
  }

  convert(text: string): string {
    return this.#inner.convert(text);
  }

  convertRoman(text: string): string {
    return this.#inner.convertRoman(text);
  }
}

const DEFAULT_WASM_URL = new URL("../pkg/aqkanji2koe_wasm_bg.wasm", import.meta.url);

let initPromise: Promise<void> | undefined;

export async function load(options: LoadOptions = {}): Promise<Kanji2Koe> {
  if (!initPromise) {
    initPromise = initWasm(options.wasmPath);
  }

  await initPromise;
  return new Kanji2Koe(new WasmKanji2Koe());
}

async function initWasm(wasmPath?: string | URL): Promise<void> {
  const source = wasmPath ?? DEFAULT_WASM_URL;

  if (isNodeLike()) {
    const bytes = await readNodeWasm(source);
    await init({ module_or_path: bytes });
    return;
  }

  await init({ module_or_path: source });
}

function isNodeLike(): boolean {
  return (
    typeof process !== "undefined" &&
    typeof process.versions === "object" &&
    typeof process.versions.node === "string"
  );
}

async function readNodeWasm(source: string | URL): Promise<Uint8Array> {
  const { readFile } = await import("node:fs/promises");
  const { fileURLToPath } = await import("node:url");

  const filePath =
    source instanceof URL
      ? fileURLToPath(source)
      : source.startsWith("file://")
        ? fileURLToPath(source)
        : source;

  return readFile(filePath);
}

export type { Kanji2Koe as WasmKanji2Koe };
