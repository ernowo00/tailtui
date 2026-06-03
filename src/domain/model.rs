#[derive(Debug, Clone, Default)]
pub struct TailscaleStatus {
    pub backend_state: String,
    pub tailnet_name: Option<String>,
    pub self_name: Option<String>,
    pub ips: Vec<String>,
    pub exit_node: Option<String>,
    pub machines: Vec<Machine>,
}

#[derive(Debug, Clone, Default)]
pub struct Machine {
    /// Tailnet / MagicDNS machine name (admin-editable), when derivable from `DNSName`.
    pub nickname: Option<String>,
    /// OS hostname from `HostName`.
    pub hostname: String,
    pub ipv4: Option<String>,
    pub online: bool,
}

impl Machine {
    /// Primary sort key: MagicDNS name if present, else hostname.
    pub fn sort_key(&self) -> &str {
        self.nickname.as_deref().unwrap_or(self.hostname.as_str())
    }

    /// Human-readable label for UI and messages.
    pub fn format_label(&self) -> String {
        match &self.nickname {
            Some(nick)
                if !nick.is_empty()
                    && nick != self.hostname.as_str()
                    && !self.hostname.is_empty()
                    && self.hostname != "unknown" =>
            {
                format!("{nick} ({})", self.hostname)
            }
            Some(nick) if !nick.is_empty() => nick.clone(),
            _ => self.hostname.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActionResult {
    pub ok: bool,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusSection {
    /// Peer list (j/k, y).
    Machines,
    /// Up/down/refresh list (j/k, Enter, global u/d/r).
    Actions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionSelection {
    Up,
    Down,
    Refresh,
}

impl ActionSelection {
    pub fn next(self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Down => Self::Refresh,
            Self::Refresh => Self::Up,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Up => Self::Refresh,
            Self::Down => Self::Up,
            Self::Refresh => Self::Down,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppState {
    pub status: TailscaleStatus,
    pub busy: bool,
    pub feedback: String,
    pub show_help: bool,
    pub focus: FocusSection,
    pub selected_action: ActionSelection,
    pub selected_machine: usize,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            status: TailscaleStatus::default(),
            busy: false,
            feedback: "y: copy peer IPv4 · Y: copy this device's IPv4 · r: refresh · q: quit.".to_string(),
            show_help: false,
            focus: FocusSection::Machines,
            selected_action: ActionSelection::Refresh,
            selected_machine: 0,
        }
    }
}
