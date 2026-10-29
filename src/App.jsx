import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import StudioPlayer from "./components/StudioPlayer";
import {
  MODELS,
  bytes,
  fileName,
  modelName,
  movePending,
  restoreQueue,
  time,
} from "./lib/studio";
import "./studio.css";

const LABELS = {
  pending: "Na fila",
  running: "Processando",
  done: "Concluída",
  error: "Falhou",
  cancelled: "Cancelada",
};
const STAGES = {
  preparing: "Preparação",
  installing: "Instalação",
  downloading: "Download",
  converting: "Conversão",
  processing: "Separação",
  finalizing: "Finalização",
  cancelled: "Cancelamento",
};
const AUDIO = [
  {
    name: "Áudio",
    extensions: ["wav", "mp3", "flac", "ogg", "aiff", "aif", "m4a"],
  },
];
const uid = () => crypto.randomUUID();

export default function App() {
  const [ready, setReady] = useState(false);
  const [projects, setProjects] = useState([]);
  const [queue, setQueue] = useState([]);
  const [activeId, setActiveId] = useState(null);
  const [model, setModel] = useState("bs-roformer-cpp");
  const [backend, setBackend] = useState("auto");
  const [hardware, setHardware] = useState(null);
  const [models, setModels] = useState([]);
  const [tab, setTab] = useState("studio");
  const [auto, setAuto] = useState(false);
  const [busy, setBusy] = useState(null);
  const [progress, setProgress] = useState({
    stage: "preparing",
    percent: null,
    message: "",
  });
  const [elapsed, setElapsed] = useState(0);
  const [error, setError] = useState(null);
  const [notice, setNotice] = useState("");
  const [dragging, setDragging] = useState(false);
  const [search, setSearch] = useState("");
  const [source, setSource] = useState(null);
  const [compareModel, setCompareModel] = useState("demucs");
  const [clipStart, setClipStart] = useState(0);
  const [clipLength, setClipLength] = useState(15);
  const [compareGroup, setCompareGroup] = useState(null);
  const [resume, setResume] = useState(false);
  const [removing, setRemoving] = useState(null);
  const running = useRef(false);
  const stateRef = useRef({});
  const sourceRequest = useRef(0);
  const stageStarted = useRef(Date.now());
  const fail = useCallback((value) => setError(String(value)), []);
  const active = projects.find((p) => p.id === activeId);
  const pending = queue.filter((q) => q.status === "pending").length;
  stateRef.current = { queue, model, backend, activeId, compareGroup };

  const refreshModels = useCallback(
    () => invoke("model_status").then(setModels).catch(fail),
    [fail],
  );
  useEffect(() => {
    let disposed = false;
    Promise.allSettled([
      invoke("list_projects"),
      invoke("read_workspace"),
      invoke("hardware_info"),
      invoke("model_status"),
    ]).then((results) => {
      if (disposed) return;
      const [list, workspace, hw, status] = results;
      if (list.status === "fulfilled") setProjects(list.value);
      else fail(list.reason);
      if (workspace.status === "fulfilled") {
        const data = workspace.value;
        setQueue(restoreQueue(data.queue));
        if (MODELS.some((m) => m.id === data.model)) setModel(data.model);
        if (["auto", "cpu", "gpu"].includes(data.backend))
          setBackend(data.backend);
        setActiveId(data.activeId || null);
        setCompareGroup(data.compareGroup || null);
      } else fail(workspace.reason);
      if (hw.status === "fulfilled") setHardware(hw.value);
      else fail(hw.reason);
      if (status.status === "fulfilled") setModels(status.value);
      else fail(status.reason);
      setReady(true);
    });
    return () => {
      disposed = true;
    };
  }, [fail]);

  useEffect(() => {
    if (!ready) return;
    const timer = setTimeout(
      () => invoke("write_workspace", { value: stateRef.current }).catch(fail),
      250,
    );
    return () => clearTimeout(timer);
  }, [ready, queue, model, backend, activeId, compareGroup, fail]);

  useEffect(() => {
    let disposed = false,
      stop;
    listen("job_progress", ({ payload }) => {
      if (!disposed) setProgress(payload);
    })
      .then((fn) => {
        if (disposed) fn();
        else stop = fn;
      })
      .catch(fail);
    return () => {
      disposed = true;
      stop?.();
    };
  }, [fail]);

  useEffect(() => {
    if (!busy) {
      setElapsed(0);
      return;
    }
    const started = Date.now();
    const timer = setInterval(
      () => setElapsed((Date.now() - started) / 1000),
      500,
    );
    return () => clearInterval(timer);
  }, [busy]);

  useEffect(() => {
    stageStarted.current = Date.now();
  }, [progress.stage]);

  const inspectSource = useCallback(
    async (path) => {
      const request = ++sourceRequest.current;
      try {
        const info = await invoke("probe_file", { path });
        if (request === sourceRequest.current) {
          setSource({ path, ...info });
          setClipStart(0);
          setClipLength(Math.min(15, info.duration_secs));
        }
      } catch (e) {
        fail(`Não foi possível ler ${fileName(path)}: ${e}`);
      }
    },
    [fail],
  );

  const addFiles = useCallback(
    (paths) => {
      const accepted = paths.filter((path) =>
        /\.(wav|mp3|flac|ogg|aiff|aif|m4a)$/i.test(path),
      );
      if (!accepted.length) {
        fail("Selecione arquivos de áudio WAV, MP3, FLAC, OGG, AIFF ou M4A.");
        return;
      }
      setQueue((previous) => [
        ...previous,
        ...accepted.map((path) => ({
          id: uid(),
          path,
          model,
          backend,
          status: "pending",
        })),
      ]);
      inspectSource(accepted[0]);
      setNotice(`${accepted.length} arquivo(s) adicionado(s) à fila.`);
    },
    [model, backend, inspectSource, fail],
  );

  const pick = useCallback(async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: AUDIO,
        title: "Adicionar músicas à fila",
      });
      if (selected) addFiles(Array.isArray(selected) ? selected : [selected]);
    } catch (e) {
      fail(e);
    }
  }, [addFiles, fail]);

  useEffect(() => {
    let disposed = false,
      stop;
    getCurrentWebview()
      .onDragDropEvent(({ payload }) => {
        if (disposed) return;
        setDragging(payload.type === "over" || payload.type === "enter");
        if (payload.type === "drop" && payload.paths?.length)
          addFiles(payload.paths);
      })
      .then((fn) => {
        if (disposed) fn();
        else stop = fn;
      })
      .catch(fail);
    return () => {
      disposed = true;
      stop?.();
    };
  }, [addFiles, fail]);

  useEffect(() => {
    const handler = (event) => {
      if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === "o") {
        event.preventDefault();
        pick();
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [pick]);

  useEffect(() => {
    if (!ready || !auto || running.current || busy) return;
    const item = queue.find((q) => q.status === "pending");
    if (!item) {
      setAuto(false);
      return;
    }
    running.current = true;
    setBusy({ kind: "separation", id: item.id, name: fileName(item.path) });
    setProgress({
      stage: "preparing",
      percent: null,
      message: "Preparando a separação…",
    });
    setQueue((previous) =>
      previous.map((q) =>
        q.id === item.id ? { ...q, status: "running", error: null } : q,
      ),
    );
    invoke("process_file", {
      path: item.path,
      model: item.model,
      backendChoice: item.backend,
      clip: item.clip || null,
    })
      .then((result) => {
        const project = { id: result.project_id, result, state: {} };
        setProjects((previous) => [
          project,
          ...previous.filter((p) => p.id !== project.id),
        ]);
        setQueue((previous) =>
          previous.map((q) =>
            q.id === item.id
              ? { ...q, status: "done", projectId: project.id }
              : q,
          ),
        );
        setActiveId(project.id);
        setResume(false);
        setTab("studio");
        setNotice(`Separação concluída: ${result.name}`);
      })
      .catch((e) => {
        const message = String(e);
        const cancelled = /cancelad/i.test(message);
        setQueue((previous) =>
          previous.map((q) =>
            q.id === item.id
              ? {
                  ...q,
                  status: cancelled ? "cancelled" : "error",
                  error: message,
                }
              : q,
          ),
        );
        if (!cancelled) fail(`${fileName(item.path)}: ${message}`);
      })
      .finally(() => {
        running.current = false;
        setBusy(null);
        refreshModels();
      });
  }, [ready, auto, queue, busy, fail, refreshModels]);

  const cancel = async () => {
    setAuto(false);
    try {
      await invoke("cancel_operation");
    } catch (e) {
      fail(e);
    }
  };

  const prepare = async (id, reinstall = false) => {
    if (busy || running.current) return;
    running.current = true;
    setAuto(false);
    setBusy({ kind: "model", name: modelName(id) });
    setProgress({
      stage: "preparing",
      percent: null,
      message: "Preparando modelo…",
    });
    try {
      if (reinstall) await invoke("remove_model", { model: id });
      await invoke("prepare_model", { model: id, backend });
      setNotice(`${modelName(id)} preparado para uso offline.`);
    } catch (e) {
      fail(e);
    } finally {
      running.current = false;
      setBusy(null);
      refreshModels();
    }
  };

  const remove = async (id) => {
    try {
      await invoke("remove_model", { model: id });
      setRemoving(null);
      setNotice("Pesos removidos. Seus projetos foram preservados.");
      await refreshModels();
    } catch (e) {
      fail(e);
    }
  };

  const comparison = useMemo(
    () =>
      queue
        .filter((q) => q.group === compareGroup && q.projectId)
        .map((q) => projects.find((p) => p.id === q.projectId))
        .filter(Boolean),
    [queue, compareGroup, projects],
  );
  const startComparison = () => {
    if (!source || compareModel === model) return;
    const end = Math.min(source.duration_secs, clipStart + clipLength);
    if (clipStart < 0 || end <= clipStart || end - clipStart > 30) {
      fail("Escolha um trecho válido de até 30 segundos.");
      return;
    }
    const group = uid();
    setCompareGroup(group);
    setQueue((previous) => [
      ...previous,
      ...[model, compareModel].map((m) => ({
        id: uid(),
        path: source.path,
        model: m,
        backend,
        status: "pending",
        clip: [clipStart, end],
        group,
      })),
    ]);
    setAuto(true);
    setNotice(
      "Os dois modelos serão processados em sequência. Depois, alterne A/B no player.",
    );
  };

  const switchComparison = async (project) => {
    if (project.id === activeId) return;
    try {
      let snapshot = { position: 0, playing: false };
      if (active) snapshot = await invoke("audio_transport");
      const speed = active?.state?.study?.speed || 1;
      const state = {
        ...project.state,
        position: Math.min(
          snapshot.position * speed,
          project.result.duration_secs - 0.05,
        ),
      };
      // A/B uses the same listening mix and study settings for a fair comparison.
      if (active) {
        state.mix = active.state?.mix;
        state.study = active.state?.study;
        state.region = active.state?.region;
      }
      setProjects((previous) =>
        previous.map((p) => (p.id === project.id ? { ...p, state } : p)),
      );
      setResume(snapshot.playing);
      setActiveId(project.id);
      setTab("studio");
    } catch (e) {
      fail(e);
    }
  };

  const saveState = useCallback(
    (id, state) =>
      setProjects((previous) =>
        previous.map((p) => (p.id === id ? { ...p, state } : p)),
      ),
    [],
  );
  const measurements = useMemo(
    () =>
      MODELS.map((m) => {
        const targetBackend =
          m.id === "bs-roformer-cpp"
            ? backend === "cpu"
              ? "CPU"
              : hardware?.backend
            : "CPU";
        const samples = projects.filter(
          (p) =>
            p.result.model === m.id &&
            p.result.backend === targetBackend &&
            p.result.elapsed_secs > 0 &&
            p.result.duration_secs > 0,
        );
        const ratio = samples.length
          ? samples.reduce(
              (sum, p) => sum + p.result.elapsed_secs / p.result.duration_secs,
              0,
            ) / samples.length
          : null;
        return { ...m, ratio, count: samples.length };
      }),
    [projects, backend, hardware],
  );
  const measured = measurements
    .filter((m) => m.ratio !== null)
    .sort((a, b) => a.ratio - b.ratio)[0];
  const visibleProjects = projects.filter((p) =>
    p.result.name.toLowerCase().includes(search.toLowerCase()),
  );
  const estimate =
    progress.stage === "processing" && progress.percent > 10
      ? Math.max(
          0,
          ((Date.now() - stageStarted.current) / 1000) *
            (100 / progress.percent - 1),
        )
      : null;

  return (
    <div className={`studio-shell ${dragging ? "drag-active" : ""}`}>
      <header className="app-header">
        <a
          className="brand"
          href="#"
          onClick={(e) => {
            e.preventDefault();
            setTab("studio");
          }}
          aria-label="Stemway, estúdio"
        >
          <svg
            viewBox="0 0 24 24"
            aria-hidden="true"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
          >
            <path d="M4 8v8M8 4v16M12 7v10M16 3v18M20 9v6" />
          </svg>
          <strong>Stemway</strong>
        </a>
        <span className="local-label">Áudio local. Seu estúdio.</span>
        <nav aria-label="Área de trabalho">
          <button
            className={tab === "studio" ? "active" : ""}
            onClick={() => setTab("studio")}
            aria-pressed={tab === "studio"}
          >
            Estúdio
          </button>
          <button
            className={tab === "models" ? "active" : ""}
            onClick={() => setTab("models")}
            aria-pressed={tab === "models"}
          >
            Modelos e hardware
          </button>
        </nav>
      </header>
      <div className="workspace">
        <aside className="library">
          <div className="library-title">
            <h2>Projetos</h2>
            <span>{projects.length}</span>
          </div>
          <label className="sr-only" htmlFor="project-search">
            Buscar projetos
          </label>
          <input
            id="project-search"
            type="search"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Buscar na biblioteca"
          />
          {!ready ? (
            <p className="subtle">Abrindo biblioteca…</p>
          ) : visibleProjects.length ? (
            <ul className="project-list">
              {visibleProjects.map((p) => (
                <li key={p.id}>
                  <button
                    className={activeId === p.id ? "selected" : ""}
                    onClick={() => {
                      setActiveId(p.id);
                      setResume(false);
                      setTab("studio");
                    }}
                    aria-current={activeId === p.id ? "true" : undefined}
                  >
                    <strong>{p.result.name}</strong>
                    <span>{modelName(p.result.model)}</span>
                    <span>
                      {time(p.result.duration_secs)} ·{" "}
                      {p.result.clip_start > 0 ? "Trecho · " : ""}
                      {new Date(Number(p.id) / 1e6).toLocaleDateString("pt-BR")}
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          ) : (
            <p className="empty-library">
              {search
                ? "Nenhum projeto encontrado."
                : "Suas separações aparecem aqui e ficam salvas para você voltar depois."}
            </p>
          )}
          <p className="library-footnote">
            Projetos e mixagens são salvos neste computador.
          </p>
        </aside>
        <main>
          {tab === "studio" ? (
            <>
              <section
                className="import-section"
                aria-labelledby="import-title"
              >
                <div className="section-heading">
                  <div>
                    <h1 id="import-title">Separe. Escute. Toque junto.</h1>
                    <p>
                      Adicione músicas e escolha como separar os seis
                      instrumentos.
                    </p>
                  </div>
                  <button className="primary" onClick={pick} disabled={!ready}>
                    Adicionar músicas
                  </button>
                </div>
                <div className="import-options toolbar">
                  <label>
                    Modelo{" "}
                    <select
                      value={model}
                      onChange={(e) => setModel(e.target.value)}
                    >
                      {MODELS.map((m) => (
                        <option key={m.id} value={m.id}>
                          {m.name}
                        </option>
                      ))}
                    </select>
                  </label>
                  <label>
                    Processamento{" "}
                    <select
                      value={backend}
                      disabled={model !== "bs-roformer-cpp"}
                      onChange={(e) => setBackend(e.target.value)}
                    >
                      <option value="auto">Automático</option>
                      <option value="cpu">CPU</option>
                      <option value="gpu">GPU</option>
                    </select>
                  </label>
                  <span className="subtle">
                    {model === "bs-roformer-cpp"
                      ? hardware?.backend || "Verificando hardware…"
                      : "Este modelo usa CPU"}
                  </span>
                </div>
                <button className="drop-area" onClick={pick} disabled={!ready}>
                  Arraste arquivos para a janela ou escolha músicas{" "}
                  <span>WAV, MP3, FLAC, OGG, AIFF, M4A · Ctrl+O</span>
                </button>
              </section>

              {(queue.length > 0 || busy) && (
                <section
                  className="queue-section"
                  aria-labelledby="queue-title"
                >
                  <div className="section-heading">
                    <h2 id="queue-title">
                      Fila de processamento{" "}
                      <span className="subtle">{pending} pendente(s)</span>
                    </h2>
                    <div className="toolbar">
                      <button
                        onClick={() =>
                          setQueue((previous) =>
                            previous.filter((q) =>
                              ["pending", "running"].includes(q.status),
                            ),
                          )
                        }
                        disabled={
                          !queue.some(
                            (q) => !["pending", "running"].includes(q.status),
                          )
                        }
                      >
                        Limpar concluídas
                      </button>
                      {auto ? (
                        <button onClick={() => setAuto(false)}>
                          Pausar após atual
                        </button>
                      ) : (
                        <button
                          className="primary"
                          onClick={() => setAuto(true)}
                          disabled={!pending || !!busy}
                        >
                          Processar fila
                        </button>
                      )}
                    </div>
                  </div>
                  <ol className="queue-list">
                    {queue.map((q, index) => (
                      <li key={q.id}>
                        <div className="queue-file">
                          <button
                            title={q.path}
                            onClick={() => inspectSource(q.path)}
                          >
                            {fileName(q.path)}
                          </button>
                          <span>
                            {modelName(q.model)} ·{" "}
                            {q.clip
                              ? `Trecho ${time(q.clip[0])}–${time(q.clip[1])}`
                              : q.backend === "auto"
                                ? "Automático"
                                : q.backend.toUpperCase()}
                          </span>
                          {q.error && (
                            <span className="error-text">{q.error}</span>
                          )}
                        </div>
                        <span className={`job-state ${q.status}`}>
                          {LABELS[q.status]}
                        </span>
                        <div className="toolbar queue-actions">
                          {q.status === "pending" && (
                            <>
                              <button
                                aria-label={`Mover ${fileName(q.path)} para cima`}
                                disabled={
                                  index === 0 ||
                                  queue[index - 1]?.status !== "pending"
                                }
                                onClick={() =>
                                  setQueue((v) => movePending(v, q.id, -1))
                                }
                              >
                                ↑
                              </button>
                              <button
                                aria-label={`Mover ${fileName(q.path)} para baixo`}
                                disabled={
                                  queue[index + 1]?.status !== "pending"
                                }
                                onClick={() =>
                                  setQueue((v) => movePending(v, q.id, 1))
                                }
                              >
                                ↓
                              </button>
                              <button
                                aria-label={`Remover ${fileName(q.path)} da fila`}
                                onClick={() =>
                                  setQueue((v) =>
                                    v.filter((x) => x.id !== q.id),
                                  )
                                }
                              >
                                Remover
                              </button>
                            </>
                          )}
                          {["error", "cancelled"].includes(q.status) && (
                            <button
                              onClick={() =>
                                setQueue((v) =>
                                  v.map((x) =>
                                    x.id === q.id
                                      ? { ...x, status: "pending", error: null }
                                      : x,
                                  ),
                                )
                              }
                            >
                              Tentar novamente
                            </button>
                          )}
                          {q.projectId && (
                            <button
                              onClick={() => {
                                setActiveId(q.projectId);
                                setResume(false);
                              }}
                            >
                              Abrir
                            </button>
                          )}
                        </div>
                      </li>
                    ))}
                  </ol>
                </section>
              )}

              {source && (
                <details className="comparison-setup tool-section">
                  <summary>Comparar modelos em um trecho</summary>
                  <p className="subtle">
                    Fonte: {fileName(source.path)} ·{" "}
                    {time(source.duration_secs)}. Clique em uma música da fila
                    para mudar a fonte.
                  </p>
                  <div className="toolbar">
                    <label>
                      Modelo A <span>{modelName(model)}</span>
                    </label>
                    <label>
                      Modelo B{" "}
                      <select
                        value={compareModel}
                        onChange={(e) => setCompareModel(e.target.value)}
                      >
                        {MODELS.map((m) => (
                          <option key={m.id} value={m.id}>
                            {m.name}
                          </option>
                        ))}
                      </select>
                    </label>
                    <label>
                      Início (s){" "}
                      <input
                        type="number"
                        min="0"
                        max={source.duration_secs}
                        step="0.1"
                        value={clipStart}
                        onChange={(e) => setClipStart(Number(e.target.value))}
                      />
                    </label>
                    <label>
                      Duração (s){" "}
                      <input
                        type="number"
                        min="1"
                        max="30"
                        value={clipLength}
                        onChange={(e) => setClipLength(Number(e.target.value))}
                      />
                    </label>
                    <button
                      disabled={!!busy || model === compareModel}
                      onClick={startComparison}
                    >
                      Processar A e B
                    </button>
                  </div>
                </details>
              )}
              {comparison.length > 0 && (
                <div className="comparison-switch toolbar">
                  <strong>Comparação A/B</strong>
                  {comparison.map((p, i) => (
                    <button
                      key={p.id}
                      className={activeId === p.id ? "active" : ""}
                      aria-pressed={activeId === p.id}
                      onClick={() => switchComparison(p)}
                    >
                      {i === 0 ? "A" : "B"} · {modelName(p.result.model)}
                    </button>
                  ))}
                  <span className="subtle">Troca no mesmo ponto da música</span>
                </div>
              )}

              {active ? (
                <section
                  className="active-project"
                  aria-labelledby="project-title"
                >
                  <div className="section-heading">
                    <div>
                      <h2 id="project-title">{active.result.name}</h2>
                      <p>
                        {modelName(active.result.model)} ·{" "}
                        {active.result.backend} · {active.result.sample_rate} Hz
                        · {active.result.paths.length} stems
                      </p>
                    </div>
                    <button
                      onClick={() =>
                        invoke("open_output_folder", {
                          path: active.result.paths[0],
                        }).catch(fail)
                      }
                    >
                      Abrir pasta do projeto
                    </button>
                  </div>
                  <StudioPlayer
                    key={active.id}
                    project={active}
                    resume={resume}
                    onChange={(state) => saveState(active.id, state)}
                    onError={fail}
                  />
                </section>
              ) : (
                <div className="studio-empty">
                  <h2>Uma música, seis novas possibilidades.</h2>
                  <p>
                    Processe a fila para ouvir voz, bateria, baixo, guitarra,
                    piano e outros instrumentos. Depois, crie acompanhamentos e
                    pratique seus trechos favoritos.
                  </p>
                </div>
              )}
            </>
          ) : (
            <section className="models-section">
              <h1>Modelos e hardware</h1>
              <p>
                Prepare os modelos uma vez para separar músicas mesmo sem
                conexão.
              </p>
              <div className="hardware-panel">
                <h2>Seu computador</h2>
                <p>{hardware?.gpu || "GPU não identificada"}</p>
                <p className="subtle">
                  Backend automático: {hardware?.backend || "Verificando…"}
                </p>
                <p>
                  Recomendação pelo hardware:{" "}
                  <strong>
                    {modelName(hardware?.recommended || "demucs")}
                  </strong>
                </p>
                {measured && (
                  <p className="subtle">
                    Mais rápido nas suas medições: {measured.name}. As medições
                    variam com a música e o hardware.
                  </p>
                )}
                <label>
                  Preferência para o motor nativo{" "}
                  <select
                    value={backend}
                    onChange={(e) => setBackend(e.target.value)}
                  >
                    <option value="auto">Automático</option>
                    <option value="cpu">CPU</option>
                    <option value="gpu">GPU</option>
                  </select>
                </label>
              </div>
              <ul className="model-list">
                {MODELS.map((m) => {
                  const status = models.find((s) => s.id === m.id);
                  const measurement = measurements.find((s) => s.id === m.id);
                  return (
                    <li key={m.id}>
                      <div>
                        <h2>{m.name}</h2>
                        <p>{m.detail}</p>
                        <p className="subtle">
                          {status?.ready
                            ? "Pesos baixados"
                            : "Download necessário"}{" "}
                          · {bytes(status?.bytes || 0)} ocupados
                        </p>
                        {measurement.ratio !== null && (
                          <p className="subtle">
                            Medição local: {measurement.ratio.toFixed(2)} s de
                            processamento / s de áudio · {measurement.count}{" "}
                            amostra(s)
                          </p>
                        )}
                      </div>
                      <div className="toolbar">
                        <button
                          className="primary"
                          disabled={!!busy || running.current}
                          onClick={() => prepare(m.id)}
                        >
                          Preparar offline
                        </button>
                        <button
                          disabled={!!busy || !status?.ready}
                          onClick={() => prepare(m.id, true)}
                        >
                          Reinstalar
                        </button>
                        <button
                          disabled={!!busy || !status?.bytes}
                          onClick={() => setRemoving(m.id)}
                        >
                          Remover pesos
                        </button>
                        {removing === m.id && (
                          <div className="remove-confirm">
                            <span>
                              Remover somente os pesos e o motor? Os projetos
                              ficam salvos.
                            </span>
                            <button onClick={() => remove(m.id)}>
                              Confirmar remoção
                            </button>
                            <button onClick={() => setRemoving(null)}>
                              Manter
                            </button>
                          </div>
                        )}
                      </div>
                    </li>
                  );
                })}
              </ul>
            </section>
          )}

          {busy && (
            <section
              className="job-progress"
              aria-label="Operação em andamento"
            >
              <div className="section-heading">
                <div>
                  <strong>
                    {STAGES[progress.stage] || "Processamento"} · {busy.name}
                  </strong>
                  <p role="status">{progress.message || "Aguarde…"}</p>
                </div>
                <button onClick={cancel}>Cancelar operação</button>
              </div>
              {typeof progress.percent === "number" ? (
                <progress
                  max="100"
                  value={progress.percent}
                  aria-label="Progresso da etapa"
                />
              ) : (
                <progress aria-label="Etapa em andamento" />
              )}
              <div className="toolbar subtle">
                <span>{time(elapsed)} decorrido</span>
                {typeof progress.percent === "number" && (
                  <span>{Math.round(progress.percent)}% desta etapa</span>
                )}
                {estimate !== null && (
                  <span>Cerca de {time(estimate)} restantes · estimativa</span>
                )}
              </div>
            </section>
          )}
          {notice && (
            <div className="notice" role="status">
              <span>{notice}</span>
              <button
                aria-label="Dispensar aviso"
                onClick={() => setNotice("")}
              >
                Fechar
              </button>
            </div>
          )}
        </main>
      </div>
      <footer className="app-footer">
        Processamento local · Nenhum áudio enviado à nuvem
        <span>Stemway · Código aberto</span>
      </footer>
      {dragging && (
        <div className="drop-overlay">
          Solte as músicas para adicionar à fila
        </div>
      )}
      {error && (
        <div className="error-banner" role="alert">
          <span>{error}</span>
          <button onClick={() => setError(null)}>Fechar</button>
        </div>
      )}
    </div>
  );
}
