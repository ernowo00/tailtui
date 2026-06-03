use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use std::time::Duration;

use crate::domain::model::ActionSelection;

#[derive(Debug, Clone, Copy)]
pub enum UiEvent {
    Quit,
    Refresh,
    Up,
    Down,
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    ActivateSelection,
    CopyMachineIpv4,
    CopySelfIpv4,
    PageUp,
    PageDown,
    ToggleHelp,
    None,
}

pub fn poll_ui_event(timeout: Duration) -> std::io::Result<UiEvent> {
    if !event::poll(timeout)? {
        return Ok(UiEvent::None);
    }

    match event::read()? {
        Event::Key(key) if key.kind == KeyEventKind::Press => Ok(map_key(key.code)),
        _ => Ok(UiEvent::None),
    }
}

pub fn selection_to_event(selection: ActionSelection) -> UiEvent {
    match selection {
        ActionSelection::Up => UiEvent::Up,
        ActionSelection::Down => UiEvent::Down,
        ActionSelection::Refresh => UiEvent::Refresh,
    }
}

fn map_key(code: KeyCode) -> UiEvent {
    match code {
        KeyCode::Char('q') => UiEvent::Quit,
        KeyCode::Char('r') => UiEvent::Refresh,
        KeyCode::Char('u') => UiEvent::Up,
        KeyCode::Char('d') => UiEvent::Down,
        KeyCode::Char('h') => UiEvent::MoveLeft,
        KeyCode::Char('l') => UiEvent::MoveRight,
        KeyCode::Char('k') => UiEvent::MoveUp,
        KeyCode::Char('j') => UiEvent::MoveDown,
        KeyCode::Char('?') => UiEvent::ToggleHelp,
        KeyCode::Char('y') => UiEvent::CopyMachineIpv4,
        KeyCode::Char('Y') => UiEvent::CopySelfIpv4,
        KeyCode::Enter => UiEvent::ActivateSelection,
        KeyCode::PageUp => UiEvent::PageUp,
        KeyCode::PageDown => UiEvent::PageDown,
        _ => UiEvent::None,
    }
}
