import { memo, useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { STEMS, stemId, time, gains, presetMix } from "../lib/studio";

// Source changes and unloads are serialized so a late render cannot replace a newer project.
let audioChain = Promise.resolve();
const audioTask = (task) => {
  const next = audioChain.then(task);
  audioChain = next.catch(() => {});
  return next;
};
const EMPTY = [];
const Waveform = memo(function Waveform({ peaks, color }) {
  const path = useMemo(() => {
    const max = Math.max(0.01, ...peaks);
    return peaks
      .map((p, i) => {
        const h = Math.max(0.6, (p / max) * 22);
        return `M${i} ${25 - h}v${h * 2}`;
      })
      .join("");
  }, [peaks]);
  return (
    <svg
      viewBox={`0 0 ${peaks.length || 700} 50`}
      preserveAspectRatio="none"
      aria-hidden="true"
    >
      <path d={path} stroke={color} strokeWidth="1" />
    </svg>
  );
});

export default function StudioPlayer({
  project,
  onChange,
  onError,
  resume = false,
}) {
  const paths = project.result.paths;
  const ids = useMemo(() => paths.map(stemId), [paths]);
  const initial = project.state || {};
  const [mix, setMix] = useState(() => ({
    ...presetMix("reset", ids),
    ...initial.mix,
  }));
  const [markers, setMarkers] = useState(initial.markers || []);
  const [region, setRegion] = useState(
    initial.region || {
      a: 0,
      b: Math.min(15, project.result.duration_secs),
      enabled: false,
    },
  );
  const [study, setStudy] = useState(
    initial.study || { speed: 1, semitones: 0 },
  );
  const [draft, setDraft] = useState(
    initial.study || { speed: 1, semitones: 0 },
  );
  const [transport, setTransport] = useState({
    playing: false,
    position: (initial.position || 0) / (initial.study?.speed || 1),
  });
  const [peaks, setPeaks] = useState([]);
  const [loading, setLoading] = useState(true);
  const [rendering, setRendering] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [changing, setChanging] = useState(false);
  const [format, setFormat] = useState("wav24");
  const [exportMix, setExportMix] = useState(true);
  const [exportRegion, setExportRegion] = useState(false);
  const [exportStudy, setExportStudy] = useState(true);
  const [selected, setSelected] = useState(ids);
  const [destination, setDestination] = useState("");
  const [exported, setExported] = useState(null);
  const [markerName, setMarkerName] = useState("");
  const [saveStatus, setSaveStatus] = useState("");
  const latest = useRef({
    mix,
    region,
    study,
    markers,
    position: initial.position || 0,
  });
  const callback = useRef(onChange);
  callback.current = onChange;
  const mounted = useRef(true);
  const duration = project.result.duration_secs;
  const position = Math.min(duration, transport.position * study.speed);
  latest.current = { mix, region, study, markers, position };

  const fail = useCallback((error) => onError(String(error)), [onError]);
  const applyMix = useCallback(
    async (value) => {
      for (const id of ids) {
        await invoke("audio_set_volume", {
          id,
          volume: value[id]?.volume ?? 1,
        });
        await invoke("audio_set_mute", { id, muted: !!value[id]?.muted });
        await invoke("audio_set_solo", { id, solo: !!value[id]?.soloed });
      }
    },
    [ids],
  );

  useEffect(() => {
    mounted.current = true;
    let disposed = false;
    invoke("audio_waveforms", { paths })
      .then((value) => {
        if (!disposed) setPeaks(value);
      })
      .catch(fail);
    audioTask(async () => {
      if (disposed) return;
      const settings = initial.study || { speed: 1, semitones: 0 };
      const rendered =
        settings.speed !== 1 || settings.semitones !== 0
          ? await invoke("render_study", { paths, ...settings })
          : paths;
      if (disposed) return;
      await invoke("audio_load_stems", {
        stemPaths: ids.map((id, i) => [id, rendered[i]]),
      });
      await applyMix(latest.current.mix);
      const start =
        Math.min(initial.position || 0, Math.max(0, duration - 0.05)) /
        settings.speed;
      if (start > 0) await invoke("audio_seek", { seconds: start });
      const r = latest.current.region;
      await invoke("audio_set_loop", {
        region: r.enabled ? [r.a / settings.speed, r.b / settings.speed] : null,
      });
      if (resume) await invoke("audio_toggle_playback");
      if (!disposed) {
        setLoading(false);
        setTransport(await invoke("audio_transport"));
      }
    }).catch((error) => {
      if (!disposed) {
        setLoading(true);
        fail(error);
      }
    });
    return () => {
      disposed = true;
      mounted.current = false;
      audioTask(() => invoke("audio_load_stems", { stemPaths: [] })).catch(
        () => {},
      );
    };
    // A keyed player owns one immutable project source; edits use latest.current.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (loading || rendering) return;
    let disposed = false;
    let pending = false;
    const timer = setInterval(async () => {
      if (pending) return;
      pending = true;
      try {
        const value = await invoke("audio_transport");
        if (!disposed) setTransport(value);
      } catch (error) {
        if (!disposed) fail(error);
      } finally {
        pending = false;
      }
    }, 120);
    return () => {
      disposed = true;
      clearInterval(timer);
    };
  }, [loading, rendering, fail]);

  useEffect(() => {
    callback.current(latest.current);
  }, [mix, markers, region, study]);

  useEffect(() => {
    const timer = setTimeout(() => {
      const state = latest.current;
      callback.current(state);
      setSaveStatus("Salvando…");
      invoke("save_project", { id: project.id, state })
        .then(() => {
          if (mounted.current) setSaveStatus("Salvo neste computador");
        })
        .catch((error) => {
          if (mounted.current) {
            setSaveStatus("Falha ao salvar");
            fail(error);
          }
        });
    }, 450);
    return () => clearTimeout(timer);
  }, [mix, markers, region, study, transport.playing, project.id, fail]);

  useEffect(
    () => () => {
      callback.current(latest.current);
      invoke("save_project", { id: project.id, state: latest.current }).catch(
        fail,
      );
    },
    [project.id, fail],
  );

  const toggle = useCallback(async () => {
    if (loading || rendering) return;
    try {
      await invoke("audio_toggle_playback");
      setTransport(await invoke("audio_transport"));
    } catch (error) {
      fail(error);
    }
  }, [loading, rendering, fail]);

  const seek = useCallback(
    async (seconds) => {
      if (loading || rendering) return;
      try {
        const value = Math.min(duration, Math.max(0, seconds));
        await invoke("audio_seek", { seconds: value / study.speed });
        setTransport(await invoke("audio_transport"));
        const state = { ...latest.current, position: value };
        callback.current(state);
        await invoke("save_project", { id: project.id, state });
      } catch (error) {
        fail(error);
      }
    },
    [loading, rendering, duration, study.speed, project.id, fail],
  );

  useEffect(() => {
    const handler = (event) => {
      if (
        /INPUT|SELECT|TEXTAREA|BUTTON/.test(event.target.tagName) ||
        event.ctrlKey ||
        event.metaKey ||
        event.altKey
      )
        return;
      if (event.code === "Space") {
        event.preventDefault();
        toggle();
      }
      if (event.key === "ArrowLeft") {
        event.preventDefault();
        seek(latest.current.position - 5);
      }
      if (event.key === "ArrowRight") {
        event.preventDefault();
        seek(latest.current.position + 5);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggle, seek]);

  const changeMix = async (value) => {
    setChanging(true);
    try {
      await applyMix(value);
      setMix(value);
    } catch (error) {
      fail(error);
    } finally {
      setChanging(false);
    }
  };
  const changeStem = async (id, field, value) => {
    const cmd = {
      volume: "audio_set_volume",
      muted: "audio_set_mute",
      soloed: "audio_set_solo",
    }[field];
    try {
      await invoke(cmd, { id, [field === "soloed" ? "solo" : field]: value });
      setMix((previous) => ({
        ...previous,
        [id]: { ...previous[id], [field]: value },
      }));
    } catch (error) {
      fail(error);
    }
  };

  const updateRegion = async (next) => {
    if (
      next.enabled &&
      (!(next.b > next.a) || next.b > duration || next.a < 0)
    ) {
      fail("Defina A e B dentro da música, com B depois de A.");
      return;
    }
    try {
      await invoke("audio_set_loop", {
        region: next.enabled
          ? [next.a / study.speed, next.b / study.speed]
          : null,
      });
      setRegion(next);
    } catch (error) {
      fail(error);
    }
  };

  const applyStudy = async () => {
    setRendering(true);
    try {
      await audioTask(async () => {
        const snapshot = await invoke("audio_transport");
        if (snapshot.playing) await invoke("audio_toggle_playback");
        const outputs =
          draft.speed === 1 && draft.semitones === 0
            ? paths
            : await invoke("render_study", { paths, ...draft });
        if (!mounted.current) return;
        await invoke("audio_load_stems", {
          stemPaths: ids.map((id, i) => [id, outputs[i]]),
        });
        await applyMix(latest.current.mix);
        await invoke("audio_seek", {
          seconds: latest.current.position / draft.speed,
        });
        await invoke("audio_set_loop", {
          region: region.enabled
            ? [region.a / draft.speed, region.b / draft.speed]
            : null,
        });
        setStudy({ ...draft });
        if (snapshot.playing) await invoke("audio_toggle_playback");
        setTransport(await invoke("audio_transport"));
      });
    } catch (error) {
      fail(error);
    } finally {
      if (mounted.current) setRendering(false);
    }
  };

  const doExport = async () => {
    if (!selected.length) {
      fail("Selecione pelo menos uma faixa.");
      return;
    }
    setExporting(true);
    setExported(null);
    try {
      const directory =
        destination ||
        (await open({
          directory: true,
          title: "Escolher pasta de exportação",
        }));
      if (!directory) return;
      setDestination(directory);
      const levels = gains(mix, ids);
      if (
        exportMix &&
        !ids.some((id, i) => selected.includes(id) && levels[i] > 0)
      ) {
        fail(
          "A mixagem selecionada está silenciosa. Ajuste mute, solo ou volume.",
        );
        return;
      }
      const output = await invoke("export_audio", {
        request: {
          tracks: ids
            .map((id, i) => ({ id, path: paths[i], gain: levels[i] }))
            .filter((t) => selected.includes(t.id)),
          directory,
          name: project.result.name,
          format,
          mix: exportMix,
          speed: exportStudy ? study.speed : 1,
          semitones: exportStudy ? study.semitones : 0,
          region: exportRegion ? [region.a, region.b] : null,
        },
      });
      setExported(output);
    } catch (error) {
      fail(error);
    } finally {
      setExporting(false);
    }
  };

  const disabled = loading || rendering || changing;
  return (
    <div className="player">
      <div className="transport toolbar">
        <button className="primary" onClick={toggle} disabled={disabled}>
          {loading
            ? "Carregando áudio…"
            : transport.playing
              ? "Pausar"
              : "Reproduzir"}
        </button>
        <button onClick={() => seek(0)} disabled={disabled}>
          Início
        </button>
        <output className="timecode" aria-label="Posição de reprodução">
          {time(position)} / {time(duration)}
        </output>
        <span className="subtle shortcut">Espaço: play · ← →: 5 s</span>
      </div>
      <label className="timeline-label">
        Posição na música
        <input
          aria-label="Posição na música"
          type="range"
          min="0"
          max={duration || 1}
          step="0.1"
          value={position}
          disabled={disabled}
          onChange={(e) => seek(Number(e.target.value))}
        />
      </label>
      <div className="toolbar presets">
        <label>
          Preset{" "}
          <select
            aria-label="Preset da mixagem"
            defaultValue=""
            disabled={disabled}
            onChange={(e) => {
              changeMix(presetMix(e.target.value, ids));
              e.target.value = "";
            }}
          >
            <option value="" disabled>
              Escolher mixagem
            </option>
            <option value="reset">Restaurar volumes</option>
            <option value="karaoke">Karaokê</option>
            <option value="no-drums">Sem bateria</option>
            <option value="bass">Baixo em destaque</option>
            <option value="vocals">Somente voz</option>
          </select>
        </label>
        <span className="subtle">M: silenciar · S: ouvir isoladamente</span>
      </div>
      <div className="tracks">
        {ids.map((id, i) => {
          const config = STEMS[id] || { label: id, color: "#aaa" };
          const state = mix[id];
          return (
            <div className={`track ${state.muted ? "muted" : ""}`} key={id}>
              <span className="track-name" style={{ color: config.color }}>
                {config.label}
              </span>
              <div className="waveform">
                <Waveform peaks={peaks[i] || EMPTY} color={config.color} />
                {!peaks[i] && (
                  <span className="wave-loading">Analisando áudio…</span>
                )}
                <span
                  className="playhead"
                  style={{
                    left: `${duration ? (position / duration) * 100 : 0}%`,
                  }}
                />
                {region.enabled && (
                  <span
                    className="loop-overlay"
                    style={{
                      left: `${(region.a / duration) * 100}%`,
                      width: `${((region.b - region.a) / duration) * 100}%`,
                    }}
                  />
                )}
                <input
                  className="wave-seek"
                  aria-label={`Posição em ${config.label}`}
                  type="range"
                  min="0"
                  max={duration || 1}
                  step="0.1"
                  value={position}
                  disabled={disabled}
                  onChange={(e) => seek(Number(e.target.value))}
                />
              </div>
              <label className="volume">
                <span className="sr-only">Volume de {config.label}</span>
                <input
                  aria-label={`Volume de ${config.label}`}
                  type="range"
                  min="0"
                  max="1"
                  step="0.01"
                  value={state.volume}
                  disabled={disabled}
                  onChange={(e) =>
                    changeStem(id, "volume", Number(e.target.value))
                  }
                />
                <output>{Math.round(state.volume * 100)}%</output>
              </label>
              <button
                className={state.muted ? "active mute" : ""}
                aria-label={`Silenciar ${config.label}`}
                aria-pressed={state.muted}
                title={`Silenciar ${config.label}`}
                disabled={disabled}
                onClick={() => changeStem(id, "muted", !state.muted)}
              >
                M
              </button>
              <button
                className={state.soloed ? "active" : ""}
                aria-label={`Ouvir somente ${config.label}`}
                aria-pressed={state.soloed}
                title={`Ouvir somente ${config.label}`}
                disabled={disabled}
                onClick={() => changeStem(id, "soloed", !state.soloed)}
              >
                S
              </button>
            </div>
          );
        })}
      </div>
      <details className="tool-section" open>
        <summary>Estudo e trechos</summary>
        <div className="toolbar study-controls">
          <label>
            Velocidade{" "}
            <select
              value={draft.speed}
              disabled={disabled}
              onChange={(e) =>
                setDraft({ ...draft, speed: Number(e.target.value) })
              }
            >
              {[0.5, 0.65, 0.75, 0.85, 1, 1.15, 1.25, 1.5, 2].map((v) => (
                <option key={v} value={v}>
                  {v}×
                </option>
              ))}
            </select>
          </label>
          <label>
            Tom{" "}
            <select
              value={draft.semitones}
              disabled={disabled}
              onChange={(e) =>
                setDraft({ ...draft, semitones: Number(e.target.value) })
              }
            >
              {Array.from({ length: 25 }, (_, i) => i - 12).map((v) => (
                <option key={v} value={v}>
                  {v > 0 ? "+" : ""}
                  {v} semitons
                </option>
              ))}
            </select>
          </label>
          <button
            onClick={applyStudy}
            disabled={
              disabled ||
              (draft.speed === study.speed &&
                draft.semitones === study.semitones)
            }
          >
            {rendering ? "Preparando estudo…" : "Aplicar ao áudio"}
          </button>
        </div>
        <p className="subtle">
          Velocidade e tom são independentes. Aplicar prepara uma cópia local; o
          áudio original fica preservado.
        </p>
        <div className="toolbar loop-controls">
          <button
            disabled={disabled}
            onClick={() => updateRegion({ ...region, a: position })}
          >
            Marcar A: {time(region.a)}
          </button>
          <button
            disabled={disabled}
            onClick={() => updateRegion({ ...region, b: position })}
          >
            Marcar B: {time(region.b)}
          </button>
          <label>
            <input
              type="checkbox"
              checked={region.enabled}
              disabled={disabled}
              onChange={(e) =>
                updateRegion({ ...region, enabled: e.target.checked })
              }
            />{" "}
            Repetir A–B
          </label>
        </div>
        <form
          className="toolbar marker-form"
          onSubmit={(e) => {
            e.preventDefault();
            if (!markerName.trim()) return;
            setMarkers((v) =>
              [
                ...v,
                { id: crypto.randomUUID(), name: markerName.trim(), position },
              ].sort((a, b) => a.position - b.position),
            );
            setMarkerName("");
          }}
        >
          <input
            aria-label="Nome do marcador"
            placeholder="Nome do trecho, ex.: refrão"
            maxLength="60"
            value={markerName}
            onChange={(e) => setMarkerName(e.target.value)}
          />
          <button disabled={disabled || !markerName.trim()}>
            Adicionar marcador
          </button>
        </form>
        {markers.length > 0 && (
          <ul className="markers">
            {markers.map((m) => (
              <li key={m.id}>
                <button onClick={() => seek(m.position)} disabled={disabled}>
                  {time(m.position)} · {m.name}
                </button>
                <button
                  aria-label={`Excluir marcador ${m.name}`}
                  onClick={() =>
                    setMarkers((v) => v.filter((x) => x.id !== m.id))
                  }
                >
                  Excluir
                </button>
              </li>
            ))}
          </ul>
        )}
      </details>
      <details className="tool-section">
        <summary>Exportar áudio</summary>
        <div className="toolbar">
          <label>
            Conteúdo{" "}
            <select
              value={String(exportMix)}
              onChange={(e) => setExportMix(e.target.value === "true")}
              disabled={exporting}
            >
              <option value="true">Mixagem com volumes, mute e solo</option>
              <option value="false">
                Stems individuais sem ajustes de volume
              </option>
            </select>
          </label>
          <label>
            Formato{" "}
            <select
              value={format}
              onChange={(e) => setFormat(e.target.value)}
              disabled={exporting}
            >
              <option value="wav16">WAV 16-bit</option>
              <option value="wav24">WAV 24-bit</option>
              <option value="flac">FLAC</option>
              <option value="mp3">MP3 320 kbps</option>
            </select>
          </label>
        </div>
        <fieldset className="export-stems">
          <legend>Faixas incluídas</legend>
          {ids.map((id) => (
            <label key={id}>
              <input
                type="checkbox"
                checked={selected.includes(id)}
                disabled={exporting}
                onChange={(e) =>
                  setSelected((v) =>
                    e.target.checked ? [...v, id] : v.filter((x) => x !== id),
                  )
                }
              />
              {STEMS[id]?.label || id}
            </label>
          ))}
        </fieldset>
        <div className="toolbar">
          <label>
            <input
              type="checkbox"
              checked={exportRegion}
              disabled={exporting}
              onChange={(e) => setExportRegion(e.target.checked)}
            />{" "}
            Somente trecho A–B
          </label>
          <label>
            <input
              type="checkbox"
              checked={exportStudy}
              disabled={exporting}
              onChange={(e) => setExportStudy(e.target.checked)}
            />{" "}
            Aplicar velocidade e tom atuais
          </label>
        </div>
        <div className="toolbar">
          <button
            disabled={exporting}
            onClick={async () => {
              try {
                const value = await open({
                  directory: true,
                  title: "Pasta de exportação",
                });
                if (value) setDestination(value);
              } catch (e) {
                fail(e);
              }
            }}
          >
            Escolher pasta
          </button>
          <span className="path subtle" title={destination}>
            {destination || "Você escolherá a pasta ao exportar"}
          </span>
          <button
            className="primary"
            onClick={doExport}
            disabled={exporting || !selected.length}
          >
            {exporting ? "Exportando…" : "Exportar"}
          </button>
        </div>
        {exported && (
          <p role="status">
            {exported.length} arquivo(s) exportado(s).{" "}
            <button
              onClick={() =>
                invoke("open_output_folder", { path: exported[0] }).catch(fail)
              }
            >
              Abrir pasta
            </button>
          </p>
        )}
      </details>
      <p className="subtle save-status" role="status">
        {saveStatus}
      </p>
    </div>
  );
}
