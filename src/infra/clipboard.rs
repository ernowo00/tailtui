use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

const CLIPBOARD_WAIT_TIMEOUT: Duration = Duration::from_millis(200);

pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    let candidates: [(&str, &[&str]); 3] = [
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];

    let mut errors = Vec::new();
    for (bin, args) in candidates {
        match run_copy_command(bin, args, text) {
            Ok(()) => return Ok(()),
            Err(err) => errors.push(format!("{bin}: {err}")),
        }
    }

    Err(format!(
        "no clipboard command available (tried wl-copy, xclip, xsel): {}",
        errors.join("; ")
    ))
}

fn run_copy_command(bin: &str, args: &[&str], text: &str) -> Result<(), String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| e.to_string())?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(text.as_bytes())
            .map_err(|e| format!("failed to write stdin: {e}"))?;
    }
    drop(child.stdin.take());

    // Some clipboard tools (notably xclip/xsel) may stay alive to own clipboard selection.
    // Treat that as success after a short wait to avoid freezing the TUI.
    match child
        .wait_timeout(CLIPBOARD_WAIT_TIMEOUT)
        .map_err(|e| format!("failed waiting command: {e}"))?
    {
        Some(status) => {
            if status.success() {
                Ok(())
            } else {
                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("failed reading stderr: {e}"))?;
                let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                if err.is_empty() {
                    Err(format!("clipboard command exited with status: {status}"))
                } else {
                    Err(err)
                }
            }
        }
        None => Ok(()),
    }
}
