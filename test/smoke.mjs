import { load } from "../dist/index.js";

const converter = await load();

const kana = converter.convert("日本語のテキストです。");
const roman = converter.convertRoman("日本語のテキストです。");

if (typeof kana !== "string" || kana.length === 0) {
  throw new Error("convert() did not return a non-empty string");
}

if (typeof roman !== "string" || roman.length === 0) {
  throw new Error("convertRoman() did not return a non-empty string");
}

console.log({ kana, roman });
