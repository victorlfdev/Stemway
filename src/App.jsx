import { useState, useEffect, useRef, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import Dropzone from "./components/Dropzone";
import ModelSelector from "./components/ModelSelector";
import FileInfo from "./components/FileInfo";
import LoadingScreen from "./components/LoadingScreen";
import StemResults from "./components/StemResults";
import ErrorBanner from "./components/ErrorBanner";
import BlurText from "./components/ui/blur-text";
import GlowButton from "./components/ui/glow-button";
import LineWaves from "./components/ui/LineWaves";

const EVENT_MAP = {
  demucs: "demucs_progress",
  "bs-roformer": "bs_roformer_progress",
  "bs-roformer-cpp": "bs_roformer_cpp_progress",
};

const MODELS = [
  {
    value: "bs-roformer-cpp",
    label: "6-Track Premium",
    description:
      "Best quality (BS-RoFormer.cpp). 6 instruments. GPU accelerated (~30sec) or CPU (~25min)",
    recommended: true,
    meta: "Recommended for best results",
  },
  {
    value: "demucs",
    label: "6-Track Standard",
    description: "Good quality (HTDemucs). 6 instruments. Fast CPU processing",
    recommended: false,
    meta: "Fast CPU processing",
  },
  {
    value: "bs-roformer",
    label: "6-Track Essential",
    description: "Alternative (BS-RoFormer). 6 instruments. Slower, CPU only",
    recommended: false,
    meta: "CPU only · Slower",
  },
];

function App() {
  const [selectedModel, setSelectedModel] = useState("bs-roformer-cpp");
  const [selectedFile, setSelectedFile] = useState(null);
  const [fileInfo, setFileInfo] = useState(null);
  const [metadata, setMetadata] = useState(null);
  const [stemPaths, setStemPaths] = useState([]);
  const [processing, setProcessing] = useState(false);
  const [progress, setProgress] = useState({
    stage: "",
    percent: 0,
    message: "",
  });
  const [error, setError] = useState(null);
  const [downloading, setDownloading] = useState(false);
  const unlistenRef = useRef(null);

  const clearFile = useCallback(() => {
    setSelectedFile(null);
    setFileInfo(null);
    setMetadata(null);
    setStemPaths([]);
    setProcessing(false);
    setDownloading(false);
    setProgress({ stage: "", percent: 0, message: "" });
    setError(null);
    if (unlistenRef.current) {
      unlistenRef.current();
      unlistenRef.current = null;
    }
  }, []);

  const probeAndSelectFile = useCallback(
    async (path) => {
      clearFile();
      setSelectedFile(path);
      try {
        const info = await invoke("probe_file", { path });
        setFileInfo(info);
      } catch (err) {
        console.error("Failed to probe file:", err);
      }
    },
    [clearFile],
  );

  const dismissError = useCallback(() => {
    setError(null);
  }, []);

  const handleFileSelect = useCallback(
    (path) => {
      probeAndSelectFile(path);
    },
    [probeAndSelectFile],
  );

  const setupProgressListener = useCallback(async () => {
    if (unlistenRef.current) {
      unlistenRef.current();
    }

    const eventName = EVENT_MAP[selectedModel] || "demucs_progress";

    try {
      unlistenRef.current = await listen(eventName, (event) => {
        const data = event.payload;
        if (data && typeof data.percent === "number") {
          setProgress({
            stage: data.stage || data.message || "Processing",
            percent: data.percent,
            message: data.message || "",
          });
        } else if (data && data.stage === "downloading") {
          setDownloading(true);
          setProgress({
            stage: "Downloading model",
            percent: data.percent || 0,
            message: data.message || "This may take a moment...",
          });
        } else if (data && data.message && !data.percent) {
          setProgress({
            stage: data.message,
            percent: progress.percent,
            message: "",
          });
        }
      });
    } catch (err) {
      console.error("Failed to listen for progress:", err);
    }
  }, [selectedModel, progress.percent]);

  const handleSeparate = useCallback(async () => {
    if (!selectedFile) return;

    setProcessing(true);
    setDownloading(false);
    setProgress({
      stage: "Processing",
      percent: 0,
      message: "Analyzing file...",
    });

    await setupProgressListener();

    try {
      const result = await invoke("process_file", {
        path: selectedFile,
        model: selectedModel,
      });

      setMetadata(result);
      setStemPaths(result.paths);
      setProgress({ stage: "Complete", percent: 100, message: "Stems ready" });
      setProcessing(false);
      setDownloading(false);
    } catch (err) {
      setError(String(err));
      setProcessing(false);
      setDownloading(false);
    }
  }, [selectedFile, selectedModel, setupProgressListener]);

  const handleOpenOutputFolder = useCallback(async (path) => {
    try {
      await invoke("open_output_folder", { path });
    } catch (err) {
      setError(`Failed to open folder: ${err}`);
    }
  }, []);

  const fileName = selectedFile ? selectedFile.split("/").pop() : null;

  const selectedModelInfo = MODELS.find((m) => m.value === selectedModel);

  const isProcessing = processing || downloading;
  const showResults =
    selectedFile &&
    !processing &&
    !downloading &&
    metadata &&
    stemPaths.length > 0;

  return (
    <div className="min-h-screen bg-[#0a0a0a] text-white flex flex-col antialiased">
      <div
        className={`absolute inset-0 pointer-events-none transition-opacity duration-500 ${isProcessing ? "opacity-0" : "opacity-100"}`}
      >
        <LineWaves
          speed={0.3}
          innerLineCount={32}
          outerLineCount={36}
          warpIntensity={0.5}
          rotation={-45}
          edgeFadeWidth={0}
          colorCycleSpeed={0.5}
          brightness={0.25}
          color1="#16a34a"
          color2="#22c55e"
          color3="#15803d"
          enableMouseInteraction={false}
        />
      </div>

      <header className="px-6 py-3 border-b border-[#1a1a1a] flex items-center justify-between relative z-10">
        <div className="flex items-center gap-3">
          <div className="w-8 h-8 rounded-lg bg-green-600/20 flex items-center justify-center">
            <svg
              className="w-4 h-4 text-green-500"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2}
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                d="M9 19V6l12-3v13M9 19c0 1.126-0.895 2-2 2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2zm12-13c0 1.126-0.895 2-2 2s-2-0.874-2-2 0.895-2 2-2 2 0.874 2 2z"
              />
            </svg>
          </div>
          <div>
            <h1 className="text-sm font-bold tracking-tight">Stemway</h1>
            <span className="text-[11px] text-[#888]">v0.3.0</span>
          </div>
        </div>
        {metadata && (
          <button
            onClick={clearFile}
            className="text-xs text-[#666] hover:text-white transition-colors px-3 py-1.5 rounded-lg hover:bg-[#1a1a1a]"
          >
            New file
          </button>
        )}
      </header>

      <main className="flex-1 flex flex-col items-center px-6 py-8 overflow-y-auto relative z-10">
        <div className="w-full max-w-2xl sm:max-w-4xl">
          {!selectedFile && !processing && !downloading && (
            <div className="space-y-8">
              <div className="text-center mb-4">
                <h2 className="text-xl font-bold tracking-tight mb-2">
                  <BlurText text="Separate any track into stems" />
                </h2>
                <p className="text-sm text-[#888]">
                  Choose a model, drop your file, and get isolated drums, bass,
                  vocals, and more.
                </p>
              </div>

              <ModelSelector
                selectedModel={selectedModel}
                onSelect={setSelectedModel}
                models={MODELS}
              />

              <Dropzone onFileSelect={handleFileSelect} />
            </div>
          )}

          {downloading && selectedFile && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={fileInfo?.duration_secs || 0}
                sampleRate={fileInfo?.sample_rate || 0}
                channels={fileInfo?.channels || 0}
                model={selectedModel}
                backend={null}
                onNewFile={clearFile}
              />
              <LoadingScreen
                stage={progress.stage}
                message={progress.message || "Downloading model..."}
                percent={progress.percent}
              />
            </div>
          )}

          {processing && selectedFile && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={
                  fileInfo?.duration_secs || metadata?.duration_secs || 0
                }
                sampleRate={fileInfo?.sample_rate || metadata?.sample_rate || 0}
                channels={fileInfo?.channels || metadata?.channels || 0}
                model={selectedModel}
                backend={metadata?.backend || null}
                onNewFile={clearFile}
              />
              <LoadingScreen
                stage={progress.stage}
                message={progress.message || "Processing audio..."}
                percent={progress.percent}
              />
            </div>
          )}

          {showResults && (
            <div className="space-y-6">
              <FileInfo
                fileName={fileName}
                duration={metadata.duration_secs}
                sampleRate={metadata.sample_rate}
                channels={metadata.channels}
                model={metadata.model}
                backend={metadata.backend || null}
                onNewFile={clearFile}
              />
              <StemResults
                stemPaths={stemPaths}
                onOpenFolder={handleOpenOutputFolder}
                onNewFile={clearFile}
              />
            </div>
          )}

          {selectedFile && !processing && !downloading && !metadata && (
            <div className="space-y-4">
              <FileInfo
                fileName={fileName}
                duration={fileInfo?.duration_secs || 0}
                sampleRate={fileInfo?.sample_rate || 0}
                channels={fileInfo?.channels || 0}
                model={selectedModel}
                backend={null}
                onNewFile={clearFile}
              />
              <GlowButton
                onClick={handleSeparate}
                glowColor="#16a34a"
                className="w-full text-sm"
              >
                Separate Stems
              </GlowButton>
            </div>
          )}
        </div>
      </main>

      {error && <ErrorBanner message={error} onDismiss={dismissError} />}

      <footer className="px-6 py-3 border-t border-[#1a1a1a] flex items-center justify-between relative z-10">
        <span className="text-[10px] text-[#aaa] tracking-[0.08em]">
          Stemway — Local AI audio separation
        </span>
        <span className="text-[10px] text-[#aaa] tracking-[0.08em]">
          {selectedModelInfo?.label || MODELS[0].label}
        </span>
      </footer>
    </div>
  );
}

export default App;
