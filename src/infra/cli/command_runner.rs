use std::process::{Command, Stdio};
use std::time::Duration;

use wait_timeout::ChildExt;

use crate::domain::error::AppError;
use crate::domain::model::{ActionResult, TailscaleStatus};
use crate::domain::ports::{StatusReader, TailscaleController};
use crate::infra::cli::parser::parse_status_json;

const TAILSCALE_BIN: &str = "tailscale";
const COMMAND_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Debug, Default)]
pub struct TailscaleCliService;

impl TailscaleCliService {
    fn execute(&self, args: &[&str]) -> Result<String, AppError> {
        let mut child = Command::new(TAILSCALE_BIN)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(map_spawn_error)?;

        match child.wait_timeout(COMMAND_TIMEOUT).map_err(|e| {
            AppError::CommandFailed(format!("failed to wait for command completion: {e}"))
        })? {
            Some(_) => {
                let output = child.wait_with_output().map_err(|e| {
                    AppError::CommandFailed(format!("failed to read command output: {e}"))
                })?;
                if output.status.success() {
                    String::from_utf8(output.stdout)
                        .map_err(|e| AppError::ParseFailed(format!("stdout is not utf-8: {e}")))
                } else {
                    let err = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if err.to_lowercase().contains("permission denied") {
                        return Err(AppError::PermissionDenied);
                    }
                    Err(AppError::CommandFailed(if err.is_empty() {
                        "tailscale command returned non-zero exit code".to_string()
                    } else {
                        err
                    }))
                }
            }
            None => {
                let _ = child.kill();
                let _ = child.wait();
                Err(AppError::Timeout)
            }
        }
    }
}

impl StatusReader for TailscaleCliService {
    fn read_status(&self) -> Result<TailscaleStatus, AppError> {
        let out = self.execute(&["status", "--json"])?;
        parse_status_json(&out)
    }
}

impl TailscaleController for TailscaleCliService {
    fn up(&self) -> Result<ActionResult, AppError> {
        let out = self.execute(&["up"])?;
        Ok(ActionResult {
            ok: true,
            message: normalize_output("tailscale up executed", &out),
        })
    }

    fn down(&self) -> Result<ActionResult, AppError> {
        let out = self.execute(&["down"])?;
        Ok(ActionResult {
            ok: true,
            message: normalize_output("tailscale down executed", &out),
        })
    }
}

fn normalize_output(default_msg: &str, out: &str) -> String {
    let trimmed = out.trim();
    if trimmed.is_empty() {
        default_msg.to_string()
    } else {
        trimmed.to_string()
    }
}

fn map_spawn_error(err: std::io::Error) -> AppError {
    use std::io::ErrorKind;
    match err.kind() {
        ErrorKind::NotFound => AppError::CliNotFound,
        ErrorKind::PermissionDenied => AppError::PermissionDenied,
        _ => AppError::CommandFailed(err.to_string()),
    }
}
