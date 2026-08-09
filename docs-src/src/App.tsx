import { useEffect, useMemo, useRef, useState } from "react";
import { load as loadAquesTalk, type AquesTalk, type Voice } from "aquestalk.js";
import { load as loadKanji2Koe, type Kanji2Koe } from "kanji2koe-openjtalk";
import {
  AlertCircle,
  Download,
  Loader2,
  Mic2,
  Play,
  RefreshCcw,
  Sparkles,
  Volume2,
} from "lucide-react";
import "./App.css";

const VOICES: { id: Voice; label: string; tone: string }[] = [
  { id: "f1", label: "女声 f1", tone: "#d946ef" },
  { id: "f2", label: "女声 f2", tone: "#ec4899" },
  { id: "m1", label: "男声 m1", tone: "#2563eb" },
  { id: "m2", label: "男声 m2", tone: "#0891b2" },
  { id: "imd1", label: "中性 imd1", tone: "#16a34a" },
  { id: "jgr", label: "機械 jgr", tone: "#ca8a04" },
  { id: "dvd", label: "機械 dvd", tone: "#ea580c" },
  { id: "r1", label: "ロボ r1", tone: "#64748b" },
];

const PRESETS = [
  "今日は良い天気ですね。AquesTalk.jsで読み上げます。",
  "漢字かな交じり文を、音声記号列に変換してから再生します。",
  "ゆっくりしていってね。",
];

type Status = "idle" | "loading" | "speaking" | "error";

function App() {
  const [text, setText] = useState(PRESETS[0]);
  const [koe, setKoe] = useState("");
  const [voice, setVoice] = useState<(typeof VOICES)[number]>(VOICES[0]);
  const [speed, setSpeed] = useState(100);
  const [status, setStatus] = useState<Status>("loading");
  const [message, setMessage] = useState("初期化中");
  const [sampleIndex, setSampleIndex] = useState(0);
  const kanjiRef = useRef<Kanji2Koe | null>(null);
  const talkRef = useRef<AquesTalk | null>(null);
  const lastWavUrlRef = useRef<string | null>(null);

  const isBusy = status === "loading" || status === "speaking";
  const canGenerateAudio = !isBusy && Boolean(text.trim()) && talkRef.current !== null;
  const bars = useMemo(() => makeBars(koe || text), [koe, text]);

  useEffect(() => {
    let disposed = false;

    async function boot() {
      setStatus("loading");
      setMessage("変換器を読み込み中");
      try {
        kanjiRef.current = await loadKanji2Koe();
        if (disposed) return;
        await loadVoice(voice.id);
        if (disposed) return;
        const converted = kanjiRef.current.convert(text);
        setKoe(converted);
        setStatus("idle");
        setMessage("準備完了");
      } catch (error) {
        if (!disposed) {
          setStatus("error");
          setMessage(errorMessage(error));
        }
      }
    }

    boot();

    return () => {
      disposed = true;
      revokeLastWavUrl();
      void talkRef.current?.destroy();
      talkRef.current = null;
    };
  }, []);

  async function loadVoice(nextVoice: Voice) {
    await talkRef.current?.destroy();
    talkRef.current = await loadAquesTalk(nextVoice, {
      memorySize: 1024 * 1024 * 1024,
    });
  }

  async function handleVoiceChange(nextVoiceId: Voice) {
    const nextVoice = VOICES.find((item) => item.id === nextVoiceId);
    if (!nextVoice) return;

    setVoice(nextVoice);
    setStatus("loading");
    setMessage("音声を切り替え中");
    try {
      await loadVoice(nextVoice.id);
      setStatus("idle");
      setMessage("準備完了");
    } catch (error) {
      setStatus("error");
      setMessage(errorMessage(error));
    }
  }

  function convertCurrentText(): string {
    if (!kanjiRef.current) {
      throw new Error("kanji2koe-openjtalk is not loaded");
    }
    const converted = kanjiRef.current.convert(text);
    setKoe(converted);
    return converted;
  }

  async function speak() {
    if (!talkRef.current) return;

    setStatus("speaking");
    setMessage("音声を生成中");
    try {
      const currentKoe = convertCurrentText();
      const wav = talkRef.current.run(currentKoe, speed);
      const url = setLastWav(wav);
      await playWav(url);
      setStatus("idle");
      setMessage("再生完了");
    } catch (error) {
      setStatus("error");
      setMessage(errorMessage(error));
    }
  }

  function downloadWav() {
    if (!talkRef.current || !text.trim()) return;

    setStatus("speaking");
    setMessage("WAVを生成中");
    try {
      const currentKoe = convertCurrentText();
      const wav = talkRef.current.run(currentKoe, speed);
      const url = setLastWav(wav);
      const link = document.createElement("a");
      link.href = url;
      link.download = `kanji2koe-${voice.id}-${speed}.wav`;
      document.body.append(link);
      link.click();
      link.remove();
      setStatus("idle");
      setMessage("WAVを保存しました");
    } catch (error) {
      setStatus("error");
      setMessage(errorMessage(error));
    }
  }

  function setLastWav(wav: Uint8Array) {
    revokeLastWavUrl();
    const blob = wavToBlob(wav);
    const url = URL.createObjectURL(blob);
    lastWavUrlRef.current = url;
    return url;
  }

  function revokeLastWavUrl() {
    if (lastWavUrlRef.current) {
      URL.revokeObjectURL(lastWavUrlRef.current);
      lastWavUrlRef.current = null;
    }
  }

  function applyPreset() {
    const nextIndex = (sampleIndex + 1) % PRESETS.length;
    setSampleIndex(nextIndex);
    const nextText = PRESETS[nextIndex];
    setText(nextText);
    if (kanjiRef.current) {
      setKoe(kanjiRef.current.convert(nextText));
    }
  }

  return (
    <main className="app">
      <section className="workbench">
        <header className="topbar">
          <div className="brand">
            <span className="brand-mark">
              <Mic2 size={22} aria-hidden="true" />
            </span>
            <div>
              <h1>kanji2koe-openjtalk</h1>
              <p>OpenJTalk to AquesTalk.js</p>
            </div>
          </div>
          <div className={`status status-${status}`}>
            {status === "loading" && <Loader2 size={16} className="spin" aria-hidden="true" />}
            {status === "speaking" && <Volume2 size={16} aria-hidden="true" />}
            {status === "error" && <AlertCircle size={16} aria-hidden="true" />}
            {status === "idle" && <Sparkles size={16} aria-hidden="true" />}
            <span>{message}</span>
          </div>
        </header>

        <div className="visual" aria-hidden="true">
          {bars.map((height, index) => (
            <span key={index} style={{ height: `${height}%` }} />
          ))}
        </div>

        <div className="editor-grid">
          <section className="panel input-panel">
            <div className="panel-title">
              <span>Text</span>
              <button className="icon-button" onClick={applyPreset} disabled={isBusy} title="サンプルを切り替え">
                <RefreshCcw size={18} aria-hidden="true" />
              </button>
            </div>
            <textarea
              value={text}
              onChange={(event) => {
                setText(event.target.value);
                if (kanjiRef.current) {
                  setKoe(kanjiRef.current.convert(event.target.value));
                }
              }}
              disabled={isBusy}
              spellCheck={false}
            />
          </section>

          <section className="panel output-panel">
            <div className="panel-title">
              <span>Koe</span>
              <span className="format-label">AquesTalk notation</span>
            </div>
            <textarea value={koe} readOnly spellCheck={false} />
          </section>
        </div>

        <footer className="controls">
          <label className="field">
            <span>Voice</span>
            <select
              value={voice.id}
              onChange={(event) => void handleVoiceChange(event.target.value as Voice)}
              disabled={isBusy}
              style={{ borderColor: voice.tone }}
            >
              {VOICES.map((item) => (
                <option key={item.id} value={item.id}>
                  {item.label}
                </option>
              ))}
            </select>
          </label>

          <label className="field speed-field">
            <span>Speed {speed}</span>
            <input
              type="range"
              min="50"
              max="300"
              value={speed}
              onChange={(event) => setSpeed(Number(event.target.value))}
              disabled={isBusy}
            />
          </label>

          <div className="action-buttons">
            <button className="download-button" onClick={downloadWav} disabled={!canGenerateAudio} title="WAVをダウンロード">
              <Download size={20} aria-hidden="true" />
              <span>WAV</span>
            </button>
            <button className="play-button" onClick={() => void speak()} disabled={!canGenerateAudio}>
            {status === "speaking" ? (
              <Loader2 size={20} className="spin" aria-hidden="true" />
            ) : (
              <Play size={20} aria-hidden="true" />
            )}
            <span>Play</span>
            </button>
          </div>
        </footer>
      </section>
    </main>
  );
}

function wavToBlob(wav: Uint8Array) {
  const bytes = new ArrayBuffer(wav.byteLength);
  new Uint8Array(bytes).set(wav);
  return new Blob([bytes], { type: "audio/wav" });
}

async function playWav(url: string) {
  const audio = new Audio(url);
  await audio.play();
}

function makeBars(seed: string) {
  const source = seed || "kanji2koe";
  return Array.from({ length: 56 }, (_, index) => {
    const code = source.charCodeAt(index % source.length) || 64;
    return 18 + ((code * (index + 7)) % 72);
  });
}

function errorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error);
}

export default App;
