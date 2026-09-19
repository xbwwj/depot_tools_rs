use std::process::{Child, Command, Stdio};

pub struct CaffeinateGuard {
    proc: Option<Child>,
}

impl Drop for CaffeinateGuard {
    fn drop(&mut self) {
        if let Some(mut proc) = self.proc.take() {
            let _ = proc.kill();
            let _ = proc.wait();
        }
    }
}

/// Acts as a guard keeping a Mac awake, unless flagged off.
pub fn scope(actually_caffeinate: bool) -> CaffeinateGuard {
    if !cfg!(target_os = "macos") || !actually_caffeinate {
        return CaffeinateGuard { proc: None };
    }

    let pid = std::process::id();
    let proc = Command::new("caffeinate")
        .args(["-i", "-w", &pid.to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .ok();

    CaffeinateGuard { proc }
}
