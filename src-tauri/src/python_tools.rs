use std::path::{Path, PathBuf};

fn scripts_dir(venv: &Path) -> PathBuf {
    venv.join(if cfg!(windows) { "Scripts" } else { "bin" })
}

fn executable(name: &str) -> String {
    format!("{}{}", name, if cfg!(windows) { ".exe" } else { "" })
}

pub fn find_tool(name: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(venv) = std::env::var("VIRTUAL_ENV") {
        candidates.push(scripts_dir(Path::new(&venv)).join(executable(name)));
    }
    if cfg!(debug_assertions) {
        candidates.push(
            scripts_dir(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../.venv"))
                .join(executable(name)),
        );
    }
    if let Some(dirs) = directories::ProjectDirs::from("", "", "stemway") {
        candidates
            .push(scripts_dir(&dirs.data_local_dir().join("python-env")).join(executable(name)));
    }
    candidates.push(PathBuf::from(executable(name)));
    candidates.into_iter().find(|candidate| {
        let mut cmd = std::process::Command::new(candidate);
        cmd.arg("--help");
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        cmd.output().map(|o| o.status.success()).unwrap_or(false)
    })
}

pub fn ensure_tool(name: &str, package: &str) -> Result<PathBuf, String> {
    crate::jobs::check()?;
    if let Some(path) = find_tool(name) {
        return Ok(path);
    }
    let dirs = directories::ProjectDirs::from("", "", "stemway")
        .ok_or("Cannot find the local application directory")?;
    let venv = dirs.data_local_dir().join("python-env");
    let python = scripts_dir(&venv).join(executable("python"));
    if !python.exists() {
        let launcher = ["python", "python3", "py"]
            .into_iter()
            .find(|name| {
                std::process::Command::new(name)
                    .arg("--version")
                    .output()
                    .map(|o| o.status.success())
                    .unwrap_or(false)
            })
            .ok_or("Python 3.10–3.12 is required. Install Python, then restart Stemway.")?;
        crate::jobs::progress("installing", None, "Criando ambiente Python privado…");
        let output = crate::jobs::sync_output(
            std::process::Command::new(launcher)
                .args(["-m", "venv"])
                .arg(&venv),
        )?;
        if !output.status.success() {
            return Err(format!(
                "Failed to create Python environment: {}",
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    crate::jobs::progress(
        "installing",
        None,
        "Instalando dependências no ambiente privado…",
    );
    let output = crate::jobs::sync_output(std::process::Command::new(&python).args([
        "-m",
        "pip",
        "install",
        "torch==2.5.1",
        "torchaudio==2.5.1",
        "soundfile==0.14.0",
        package,
    ]))?;
    if !output.status.success() {
        return Err(format!(
            "Failed to install {package}: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    find_tool(name).ok_or_else(|| format!("{name} was installed but could not start"))
}
