use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap};
use ratatui::{Frame, prelude::Alignment};

use crate::domain::model::{ActionSelection, AppState, FocusSection};

pub fn draw(frame: &mut Frame, state: &AppState, machine_list_state: &mut ListState) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(8),
            Constraint::Min(6),
            Constraint::Length(3),
        ])
        .split(frame.area());

    let header = Paragraph::new("tailtui  |  h/l  j/k  PgUp/Dn  u/d/r  y peer  Y self-ip  q  ?:help")
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled("Header", Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .alignment(Alignment::Left);
    frame.render_widget(header, chunks[0]);

    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(chunks[1]);

    let mut status_lines = vec![
        Line::from(vec![
            Span::raw("  "),
            Span::styled("Status", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from(""),
    ];
    status_lines.extend([
        Line::from(format!("  Backend: {}", state.status.backend_state)),
        Line::from(format!(
            "  Tailnet: {}",
            state.status.tailnet_name.as_deref().unwrap_or("-")
        )),
        Line::from(format!(
            "  Device: {}",
            state.status.self_name.as_deref().unwrap_or("-")
        )),
        Line::from(format!(
            "  IPs: {}",
            if state.status.ips.is_empty() {
                "-".to_string()
            } else {
                state.status.ips.join(", ")
            }
        )),
        Line::from(format!(
            "  Exit node: {}",
            state.status.exit_node.as_deref().unwrap_or("-")
        )),
    ]);

    let status_panel = Paragraph::new(status_lines)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    frame.render_widget(status_panel, top[0]);

    let actions_focused = state.focus == FocusSection::Actions;
    let actions = [ActionSelection::Up, ActionSelection::Down, ActionSelection::Refresh];
    let action_items: Vec<ListItem> = actions
        .iter()
        .map(|action| {
            let (label, key) = match action {
                ActionSelection::Up => ("Bring network up", "u"),
                ActionSelection::Down => ("Bring network down", "d"),
                ActionSelection::Refresh => ("Refresh status", "r"),
            };

            let selected = *action == state.selected_action;
            let show_selected = actions_focused && selected;
            let prefix = if show_selected { ">" } else { " " };
            let style = if show_selected {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!("{prefix} {label}"), style),
                Span::raw(format!(" ({key})")),
            ]))
        })
        .collect();

    let action_list = List::new(action_items).block(
        Block::default()
            .title("Actions")
            .borders(Borders::ALL)
            .border_style(focus_style(state.focus == FocusSection::Actions)),
    );
    frame.render_widget(action_list, top[1]);

    let total_machines = state.status.machines.len();
    let online_machines = state
        .status
        .machines
        .iter()
        .filter(|m| m.online)
        .count();
    let machines_title = format!("Machines ({online_machines}/{total_machines})");

    let machines_focused = state.focus == FocusSection::Machines;
    let backend_stopped = state.status.backend_state.eq_ignore_ascii_case("stopped");
    let machine_items: Vec<ListItem> = if backend_stopped {
        machine_list_state.select(None);
        vec![
            ListItem::new("  tailscale is stopped"),
            ListItem::new("  press 'u' to bring network up"),
            ListItem::new("  then press 'r' to refresh machines"),
        ]
    } else if state.status.machines.is_empty() {
        machine_list_state.select(None);
        vec![ListItem::new("  no machines found")]
    } else {
        let len = state.status.machines.len();
        let idx = state.selected_machine.min(len.saturating_sub(1));
        if machines_focused {
            machine_list_state.select(Some(idx));
        } else {
            machine_list_state.select(None);
        }
        state
            .status
            .machines
            .iter()
            .map(|machine| {
                let online = if machine.online { "online" } else { "offline" };
                let ip = machine.ipv4.as_deref().unwrap_or("-");
                let line_style = if machine.online {
                    Style::default()
                } else {
                    Style::default().fg(Color::DarkGray)
                };
                ListItem::new(Line::from(Span::styled(
                    format!("{} [{}] {ip}", machine.format_label(), online),
                    line_style,
                )))
            })
            .collect()
    };
    let machines_list = List::new(machine_items)
        .block(
            Block::default()
                .title(machines_title.as_str())
                .borders(Borders::ALL)
                .border_style(focus_style(state.focus == FocusSection::Machines)),
        )
        .highlight_style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .highlight_symbol("> ")
        .scroll_padding(1);
    frame.render_stateful_widget(machines_list, chunks[2], machine_list_state);

    let feedback_title = if state.busy { "Feedback (busy)" } else { "Feedback" };
    let feedback = Paragraph::new(state.feedback.clone())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled(feedback_title, Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });
    frame.render_widget(feedback, chunks[3]);

    if state.show_help {
        let area = centered_rect(frame.area(), 70, 40);
        frame.render_widget(Clear, area);
        let help = Paragraph::new(vec![
            Line::from("Vim navigation: h/l = Machines ↔ Actions, j/k = move within panel"),
            Line::from("Machines list: PgUp / PgDn jump by page when Machines focused"),
            Line::from("Actions: u (up), d (down), r (refresh), Enter (selected action)"),
            Line::from("Machines: y copy selected peer IPv4, Y / Shift+Y copy this device's IPv4"),
            Line::from("General: press any key to close help"),
        ])
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(Span::styled("Help", Style::default().fg(Color::Yellow)))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
        frame.render_widget(help, area);
    }
}

fn focus_style(is_focused: bool) -> Style {
    if is_focused {
        Style::default().fg(Color::Green)
    } else {
        Style::default()
    }
}

fn centered_rect(area: ratatui::layout::Rect, percent_x: u16, percent_y: u16) -> ratatui::layout::Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}
