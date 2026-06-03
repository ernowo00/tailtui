use std::io;
use std::time::Duration;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::execute;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::widgets::ListState;

use crate::domain::error::AppError;
use crate::domain::model::{AppState, FocusSection, Machine};
use crate::domain::ports::{StatusReader, TailscaleController};
use crate::infra::clipboard::copy_to_clipboard;
use crate::infra::cli::command_runner::TailscaleCliService;
use crate::infra::cli::parser::first_ipv4_from_addrs;
use crate::ui::tui::event::{UiEvent, poll_ui_event, selection_to_event};
use crate::ui::tui::render::draw;

const MACHINE_PAGE_STEP: usize = 8;

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let _guard = TerminalGuard;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    let service = TailscaleCliService;
    let mut state = AppState::default();
    refresh_status(&service, &mut state);

    let mut should_quit = false;
    let mut machine_list_state = ListState::default();
    while !should_quit {
        terminal.draw(|frame| draw(frame, &state, &mut machine_list_state))?;
        let event = poll_ui_event(Duration::from_millis(250))?;
        if state.show_help && !matches!(event, UiEvent::None) {
            state.show_help = false;
            continue;
        }
        match event {
            UiEvent::Quit => should_quit = true,
            UiEvent::Refresh => refresh_status(&service, &mut state),
            UiEvent::Up => run_action(&service, &mut state, UiEvent::Up),
            UiEvent::Down => run_action(&service, &mut state, UiEvent::Down),
            UiEvent::MoveLeft => {
                state.focus = match state.focus {
                    FocusSection::Machines => FocusSection::Actions,
                    FocusSection::Actions => FocusSection::Machines,
                }
            }
            UiEvent::MoveRight => {
                state.focus = match state.focus {
                    FocusSection::Machines => FocusSection::Actions,
                    FocusSection::Actions => FocusSection::Machines,
                }
            }
            UiEvent::MoveUp => {
                match state.focus {
                    FocusSection::Actions => {
                        state.selected_action = state.selected_action.prev();
                    }
                    FocusSection::Machines => {
                        if !state.status.machines.is_empty() {
                            if state.selected_machine == 0 {
                                state.selected_machine = state.status.machines.len() - 1;
                            } else {
                                state.selected_machine -= 1;
                            }
                        }
                    }
                }
            }
            UiEvent::MoveDown => {
                match state.focus {
                    FocusSection::Actions => {
                        state.selected_action = state.selected_action.next();
                    }
                    FocusSection::Machines => {
                        if !state.status.machines.is_empty() {
                            state.selected_machine =
                                (state.selected_machine + 1) % state.status.machines.len();
                        }
                    }
                }
            }
            UiEvent::ActivateSelection => {
                let action = selection_to_event(state.selected_action);
                run_action(&service, &mut state, action);
            }
            UiEvent::CopyMachineIpv4 => copy_selected_machine_ip(&mut state),
            UiEvent::CopySelfIpv4 => copy_self_ipv4(&mut state),
            UiEvent::PageUp => {
                if state.focus == FocusSection::Machines && !state.status.machines.is_empty() {
                    state.selected_machine = state
                        .selected_machine
                        .saturating_sub(MACHINE_PAGE_STEP);
                }
            }
            UiEvent::PageDown => {
                if state.focus == FocusSection::Machines && !state.status.machines.is_empty() {
                    let last = state.status.machines.len().saturating_sub(1);
                    state.selected_machine =
                        (state.selected_machine + MACHINE_PAGE_STEP).min(last);
                }
            }
            UiEvent::ToggleHelp => state.show_help = !state.show_help,
            UiEvent::None => {}
        }
    }

    terminal.show_cursor()?;
    Ok(())
}

fn run_action(service: &TailscaleCliService, state: &mut AppState, action: UiEvent) {
    state.busy = true;
    state.feedback = "Running command...".to_string();

    let action_result = match action {
        UiEvent::Up => service.up(),
        UiEvent::Down => service.down(),
        UiEvent::Refresh => {
            refresh_status(service, state);
            state.busy = false;
            return;
        }
        _ => return,
    };

    match action_result {
        Ok(result) => {
            let prefix = if result.ok { "OK" } else { "ERR" };
            state.feedback = format!("{prefix}: {}", result.message);
            refresh_status(service, state);
        }
        Err(e) => {
            state.feedback = format_error(e);
        }
    }
    state.busy = false;
}

fn refresh_status(service: &TailscaleCliService, state: &mut AppState) {
    match service.read_status() {
        Ok(status) => {
            state.status = status;
            if state.selected_machine >= state.status.machines.len() {
                state.selected_machine = 0;
            }
            if !state.busy {
                state.feedback = "Status refreshed".to_string();
            }
        }
        Err(e) => {
            state.feedback = format_error(e);
        }
    }
}

fn format_error(err: AppError) -> String {
    format!("Error: {err}")
}

fn copy_self_ipv4(state: &mut AppState) {
    if let Some(ipv4) = first_ipv4_from_addrs(&state.status.ips) {
        match copy_to_clipboard(&ipv4) {
            Ok(()) => {
                state.feedback = format!("Copied this device's IPv4: {ipv4}");
            }
            Err(err) => {
                state.feedback = format!("Failed to copy IPv4: {err}");
            }
        }
        return;
    }
    if state.status.backend_state.eq_ignore_ascii_case("stopped") {
        state.feedback = "Tailscale is stopped; no local IPv4 to copy.".to_string();
    } else {
        state.feedback = "No Tailscale IPv4 for this device. Press 'r' to refresh.".to_string();
    }
}

fn copy_selected_machine_ip(state: &mut AppState) {
    if state.status.backend_state.eq_ignore_ascii_case("stopped") {
        state.feedback = "Tailscale is stopped. Press 'u' first.".to_string();
        return;
    }
    let (label, ipv4) = match selected_machine_ipv4(&state.status.machines, state.selected_machine) {
        Ok(v) => v,
        Err(msg) => {
            state.feedback = msg;
            return;
        }
    };
    match copy_to_clipboard(ipv4) {
        Ok(()) => {
            state.feedback = format!("Copied IPv4 for '{label}': {ipv4}");
        }
        Err(err) => {
            state.feedback = format!("Failed to copy IPv4: {err}");
        }
    }
}

fn selected_machine_ipv4(
    machines: &[Machine],
    selected_idx: usize,
) -> Result<(String, &str), String> {
    let machine = machines
        .get(selected_idx)
        .ok_or_else(|| "No machine selected".to_string())?;
    let label = machine.format_label();
    let ipv4 = machine
        .ipv4
        .as_deref()
        .ok_or_else(|| format!("Machine '{}' has no IPv4 address", label))?;
    Ok((label, ipv4))
}

struct TerminalGuard;

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}

#[cfg(test)]
mod tests {
    use super::selected_machine_ipv4;
    use crate::domain::model::Machine;

    #[test]
    fn selected_machine_ipv4_returns_name_and_ip() {
        let machines = vec![Machine {
            nickname: Some("web-1".to_string()),
            hostname: "web-1".to_string(),
            ipv4: Some("100.64.0.9".to_string()),
            online: true,
        }];
        let result = selected_machine_ipv4(&machines, 0).expect("machine should have ipv4");
        assert_eq!(result.0, "web-1");
        assert_eq!(result.1, "100.64.0.9");
    }

    #[test]
    fn selected_machine_ipv4_fails_for_missing_ip() {
        let machines = vec![Machine {
            nickname: None,
            hostname: "ipv6-only".to_string(),
            ipv4: None,
            online: true,
        }];
        let err = selected_machine_ipv4(&machines, 0).expect_err("should fail without ipv4");
        assert!(err.contains("no IPv4"));
    }
}
