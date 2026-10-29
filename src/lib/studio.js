export const MODELS = [
  {
    id: "bs-roformer-cpp",
    name: "BS-RoFormer nativo",
    detail: "Seis instrumentos · GPU ou CPU",
  },
  { id: "demucs", name: "HTDemucs", detail: "Seis instrumentos · CPU" },
  {
    id: "bs-roformer",
    name: "BS-RoFormer Python",
    detail: "Seis instrumentos · CPU",
  },
];
export const STEMS = {
  vocals: { label: "Voz", color: "#f18a91" },
  drums: { label: "Bateria", color: "#e8b66c" },
  bass: { label: "Baixo", color: "#87b0ef" },
  guitar: { label: "Guitarra", color: "#74caa2" },
  piano: { label: "Piano", color: "#8dd3da" },
  other: { label: "Outros", color: "#b5a0e8" },
};
export const fileName = (path) => path?.split(/[\\/]/).pop() || "";
export const stemId = (path) =>
  fileName(path)
    .replace(/\.wav$/i, "")
    .toLowerCase();
export const modelName = (id) => MODELS.find((m) => m.id === id)?.name || id;
export const time = (seconds) => {
  const value = Math.max(0, Math.floor(Number(seconds) || 0));
  return `${Math.floor(value / 60)}:${String(value % 60).padStart(2, "0")}`;
};
export const bytes = (n) =>
  n > 1e9 ? `${(n / 1e9).toFixed(2)} GB` : `${(n / 1e6).toFixed(1)} MB`;
export function gains(mix, ids) {
  const anySolo = ids.some((id) => mix[id]?.soloed);
  return ids.map((id) => {
    const s = mix[id] || { volume: 1 };
    return s.muted || (anySolo && !s.soloed) ? 0 : (s.volume ?? 1);
  });
}
export function presetMix(preset, ids) {
  return Object.fromEntries(
    ids.map((id) => [
      id,
      {
        volume: preset === "bass" && id !== "bass" ? 0.25 : 1,
        muted:
          (preset === "karaoke" && id === "vocals") ||
          (preset === "no-drums" && id === "drums"),
        soloed: preset === "vocals" && id === "vocals",
      },
    ]),
  );
}
export function restoreQueue(queue = []) {
  return queue
    .filter(
      (q) =>
        q && typeof q.path === "string" && MODELS.some((m) => m.id === q.model),
    )
    .map((q) => ({
      ...q,
      status: q.status === "running" ? "pending" : q.status,
    }));
}
export function movePending(queue, id, direction) {
  const index = queue.findIndex((q) => q.id === id);
  const other = index + direction;
  if (
    index < 0 ||
    other < 0 ||
    other >= queue.length ||
    queue[index].status !== "pending" ||
    queue[other].status !== "pending"
  )
    return queue;
  const next = [...queue];
  [next[index], next[other]] = [next[other], next[index]];
  return next;
}
