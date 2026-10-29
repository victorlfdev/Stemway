use std::collections::HashSet;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex, OnceLock,
};
use tauri::Emitter;

pub static OPERATION: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
static CANCELLED: AtomicBool = AtomicBool::new(false);
static PIDS: Mutex<Option<HashSet<u32>>> = Mutex::new(None);
static APP: OnceLock<tauri::AppHandle> = OnceLock::new();
static SIGNAL: OnceLock<tokio::sync::watch::Sender<bool>> = OnceLock::new();

fn signal() -> &'static tokio::sync::watch::Sender<bool> {
    SIGNAL.get_or_init(|| tokio::sync::watch::channel(false).0)
}
pub async fn cancelled() {
    let mut rx = signal().subscribe();
    if *rx.borrow() {
        return;
    }
    let _ = rx.changed().await;
}

pub fn init(app: tauri::AppHandle) {
    let _ = APP.set(app);
}
pub fn begin() {
    CANCELLED.store(false, Ordering::SeqCst);
    signal().send_replace(false);
}
pub fn check() -> Result<(), String> {
    if CANCELLED.load(Ordering::SeqCst) {
        Err("Operação cancelada".into())
    } else {
        Ok(())
    }
}

pub fn progress(stage: &str, percent: Option<f64>, message: &str) {
    if let Some(app) = APP.get() {
        let _ = app.emit(
            "job_progress",
            serde_json::json!({"stage":stage,"percent":percent,"message":message}),
        );
    }
}

fn kill_tree(pid: u32) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let _ = std::process::Command::new("taskkill.exe")
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .creation_flags(0x08000000)
            .output();
    }
    #[cfg(unix)]
    {
        // Only children registered by this app have their own process group.
        let _ = std::process::Command::new("kill")
            .args(["-KILL", "--", &format!("-{pid}")])
            .output();
    }
}

pub struct ProcessGuard(u32);
impl Drop for ProcessGuard {
    fn drop(&mut self) {
        if let Ok(mut pids) = PIDS.lock() {
            if let Some(pids) = pids.as_mut() {
                pids.remove(&self.0);
            }
        }
    }
}
pub fn register(pid: u32) -> Result<ProcessGuard, String> {
    let mut pids = PIDS.lock().map_err(|e| e.to_string())?;
    if check().is_err() {
        kill_tree(pid);
        return Err("Operação cancelada".into());
    }
    pids.get_or_insert_with(HashSet::new).insert(pid);
    Ok(ProcessGuard(pid))
}

pub fn sync_output(cmd: &mut std::process::Command) -> Result<std::process::Output, String> {
    check()?;
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        cmd.process_group(0);
    }
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    let child = cmd.spawn().map_err(|e| e.to_string())?;
    let _guard = register(child.id())?;
    let output = child.wait_with_output().map_err(|e| e.to_string())?;
    check()?;
    Ok(output)
}

#[tauri::command]
pub async fn cancel_operation() -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(cancel)
        .await
        .map_err(|e| e.to_string())?
}

pub fn cancel() -> Result<(), String> {
    CANCELLED.store(true, Ordering::SeqCst);
    signal().send_replace(true);
    // Keep the registry locked until termination to prevent a PID-reuse race.
    let pids = PIDS.lock().map_err(|e| e.to_string())?;
    if let Some(pids) = pids.as_ref() {
        for pid in pids {
            kill_tree(*pid);
        }
    }
    progress("cancelled", None, "Cancelando a operação…");
    Ok(())
}
