use crate::bs_roformer_cpp_cli::{self as cpp, BsRoformerBackend};
use std::path::{Path, PathBuf};

pub fn model_dir(model: &str) -> Result<PathBuf, String> {
    if !["demucs", "bs-roformer", "bs-roformer-cpp"].contains(&model) {
        return Err("Modelo inválido".into());
    }
    if model == "bs-roformer-cpp" {
        return Ok(cpp::get_cache_dirs()?.2);
    }
    let root = directories::ProjectDirs::from("", "", "stemway")
        .ok_or("Cache indisponível")?
        .cache_dir()
        .join("models")
        .join(if model == "bs-roformer" {
            "bs-roformer-python"
        } else {
            model
        });
    std::fs::create_dir_all(&root).map_err(|e| e.to_string())?;
    Ok(root)
}

pub fn selected_backend(choice: &str) -> Result<BsRoformerBackend, String> {
    match choice {
        "cpu" => Ok(BsRoformerBackend::Cpu),
        "auto" => Ok(cpp::detect_gpu_backend()),
        "gpu" => {
            let backend = cpp::detect_gpu_backend();
            if backend == BsRoformerBackend::Cpu {
                Err("Nenhuma GPU compatível detectada; selecione CPU ou automático".into())
            } else {
                Ok(backend)
            }
        }
        _ => Err("Backend inválido".into()),
    }
}

fn size(path: &Path) -> u64 {
    if path.is_symlink() {
        return 0;
    }
    if path.is_file() {
        return path.metadata().map(|m| m.len()).unwrap_or(0);
    }
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|e| size(&e.path()))
                .sum()
        })
        .unwrap_or(0)
}

fn has_weights(path: &Path) -> bool {
    if path.is_symlink() {
        return false;
    }
    if path.is_file() {
        return path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|e| ["gguf", "ckpt", "th"].contains(&e))
            && size(path) > 100_000;
    }
    std::fs::read_dir(path)
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .any(|e| has_weights(&e.path()))
        })
        .unwrap_or(false)
}

#[derive(serde::Serialize)]
pub struct ModelStatus {
    pub id: String,
    pub bytes: u64,
    pub ready: bool,
    pub path: String,
}

#[tauri::command]
pub async fn model_status() -> Result<Vec<ModelStatus>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        ["bs-roformer-cpp", "demucs", "bs-roformer"]
            .into_iter()
            .map(|id| {
                let path = model_dir(id)?;
                Ok(ModelStatus {
                    id: id.into(),
                    bytes: size(&path),
                    ready: has_weights(&path),
                    path: path.to_string_lossy().into_owned(),
                })
            })
            .collect()
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn hardware_info() -> Result<serde_json::Value, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let backend = cpp::detect_gpu_backend();
        let mut gpu = String::new();
        #[cfg(windows)] {
            use std::os::windows::process::CommandExt;
            if let Ok(out) = std::process::Command::new("powershell.exe").args(["-NoProfile","-NonInteractive","-Command","(Get-CimInstance Win32_VideoController).Name"]).creation_flags(0x08000000).output() { gpu = String::from_utf8_lossy(&out.stdout).trim().to_string(); }
        }
        #[cfg(target_os="linux")] {
            if let Ok(out) = std::process::Command::new("lspci").output() { gpu = String::from_utf8_lossy(&out.stdout).lines().filter(|s| s.contains("VGA") || s.contains("3D controller")).collect::<Vec<_>>().join(" · "); }
        }
        serde_json::json!({ "gpu": gpu, "backend": cpp::get_backend_name(&backend), "gpu_available": backend != BsRoformerBackend::Cpu, "recommended": if backend == BsRoformerBackend::Cpu { "demucs" } else { "bs-roformer-cpp" } })
    }).await.map_err(|e|e.to_string())
}

pub async fn prepare(model: &str, backend: &str) -> Result<(), String> {
    crate::jobs::check()?;
    crate::jobs::progress("preparing", None, "Preparando o modelo para uso offline…");
    if model == "bs-roformer-cpp" {
        cpp::get_binary_path_for(&selected_backend(backend)?).await?;
        cpp::get_model_path().await?;
        return Ok(());
    }
    let model = model.to_owned();
    tauri::async_runtime::spawn_blocking(move || {
        let (tool,package,code) = match model.as_str() {
            "demucs" => ("demucs","demucs==4.0.1","from demucs.pretrained import get_model; get_model('htdemucs_6s')"),
            "bs-roformer" => ("bs-roformer-infer","bs-roformer-infer==0.1.5","from bs_roformer import DEFAULT_MODEL, ensure_model_assets; ensure_model_assets(DEFAULT_MODEL)"),
            _ => return Err("Modelo inválido".into()),
        };
        let executable = crate::python_tools::ensure_tool(tool,package)?;
        let python = executable.parent().filter(|p|!p.as_os_str().is_empty()).map(|p|p.join(if cfg!(windows){"python.exe"}else{"python"})).unwrap_or_else(|| PathBuf::from("python"));
        let dir = model_dir(&model)?;
        crate::jobs::progress("downloading", None, "Baixando os pesos do modelo. O tempo depende da conexão…");
        let output = crate::jobs::sync_output(std::process::Command::new(python).args(["-c",code]).env("PYTHONUTF8","1").env("TORCH_HOME", &dir).env("BS_ROFORMER_MODELS_PATH", &dir))?;
        if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).to_string()); }
        Ok(())
    }).await.map_err(|e|e.to_string())?
}

#[tauri::command]
pub async fn prepare_model(model: String, backend: String) -> Result<(), String> {
    let _guard = crate::jobs::OPERATION
        .try_lock()
        .map_err(|_| "Outra operação está em andamento")?;
    crate::jobs::begin();
    prepare(&model, &backend).await
}

#[tauri::command]
pub async fn remove_model(model: String) -> Result<(), String> {
    let _guard = crate::jobs::OPERATION
        .try_lock()
        .map_err(|_| "Aguarde a operação atual terminar")?;
    tauri::async_runtime::spawn_blocking(move || {
        // Only known, application-owned directories may be removed. Reject links.
        let path = model_dir(&model)?;
        let root = directories::ProjectDirs::from("", "", "stemway")
            .ok_or("Cache indisponível")?
            .cache_dir()
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let canonical = path.canonicalize().map_err(|e| e.to_string())?;
        if path.is_symlink() || !canonical.starts_with(&root) || canonical == root {
            return Err("Pasta de modelo inválida".into());
        }
        std::fs::remove_dir_all(&canonical).map_err(|e| e.to_string())?;
        if model == "bs-roformer-cpp" {
            let binaries = cpp::get_cache_dirs()?.1;
            let canonical = binaries.canonicalize().map_err(|e| e.to_string())?;
            if binaries.is_symlink() || !canonical.starts_with(&root) || canonical == root {
                return Err("Pasta do motor inválida".into());
            }
            std::fs::remove_dir_all(canonical).map_err(|e| e.to_string())?;
        }
        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}
