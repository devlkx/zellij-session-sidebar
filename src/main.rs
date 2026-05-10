use std::collections::BTreeMap;

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};
use zellij_tile::prelude::*;

register_plugin!(SessionSidebar);

#[derive(Default)]
struct SessionSidebar {
    sessions: Vec<SessionRow>,
    selected: usize,
    visible_start: usize,
    visible_end: usize,
    permission_status: PermissionState,
}

#[derive(Default)]
enum PermissionState {
    #[default]
    Requested,
    Granted,
    Denied,
}

#[derive(Clone, Debug)]
struct SessionRow {
    name: String,
    is_current: bool,
}

impl ZellijPlugin for SessionSidebar {
    fn load(&mut self, _configuration: BTreeMap<String, String>) {
        // Do not keep Zellij alive when all regular terminal panes have exited.
        set_selectable(false);

        subscribe(&[EventType::Mouse, EventType::PermissionRequestResult]);
        self.permission_status = PermissionState::Requested;
        request_permission(&[
            PermissionType::ReadApplicationState,
            PermissionType::ChangeApplicationState,
        ]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::SessionUpdate(session_infos, _resurrectable_sessions) => {
                if session_infos.is_empty() && !self.sessions.is_empty() {
                    return false;
                }

                self.update_sessions(session_infos);
                true
            }
            Event::Mouse(mouse) => self.handle_mouse(mouse),
            Event::PermissionRequestResult(status) => {
                if status == PermissionStatus::Granted {
                    self.permission_status = PermissionState::Granted;
                    subscribe(&[EventType::SessionUpdate]);
                } else {
                    self.permission_status = PermissionState::Denied;
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, rows: usize, cols: usize) {
        println!("Sessions");
        println!();

        if self.sessions.is_empty() {
            match self.permission_status {
                PermissionState::Requested => println!("loading sessions..."),
                PermissionState::Granted => println!("waiting for sessions..."),
                PermissionState::Denied => println!("permissions denied"),
            }
            return;
        }

        let list_top = 2;
        let max_visible = rows.saturating_sub(list_top);
        let (start, end) = visible_bounds(self.sessions.len(), max_visible, self.selected);
        self.visible_start = start;
        self.visible_end = end;

        for index in start..end {
            let row = &self.sessions[index];
            let y = list_top + index.saturating_sub(start);
            let line = self.format_row(row, cols);
            let mut text = if index == self.selected {
                Text::new(line).selected()
            } else {
                Text::new(line)
            };

            if row.is_current {
                text = text.color_range(2, 0..1);
            }

            print_text_with_coordinates(text, 0, y, Some(cols), None);
        }
    }
}

impl SessionSidebar {
    fn update_sessions(&mut self, session_infos: Vec<SessionInfo>) {
        let previously_selected = self.sessions.get(self.selected).map(|s| s.name.clone());

        let mut current = Vec::new();
        let mut others = Vec::new();

        for session in session_infos {
            let row = SessionRow {
                name: session.name,
                is_current: session.is_current_session,
            };

            if row.is_current {
                current.push(row);
            } else {
                others.push(row);
            }
        }

        // Keep the current session pinned at the top, and leave Zellij's order
        // for the rest. This avoids maintaining our own state/history file.
        current.append(&mut others);
        self.sessions = current;

        if self.sessions.is_empty() {
            self.selected = 0;
            return;
        }

        if let Some(previously_selected) = previously_selected {
            if let Some(index) = self
                .sessions
                .iter()
                .position(|session| session.name == previously_selected)
            {
                self.selected = index;
                return;
            }
        }

        self.selected = self.selected.min(self.sessions.len().saturating_sub(1));
    }

    fn handle_mouse(&mut self, mouse: Mouse) -> bool {
        if self.sessions.is_empty() {
            return false;
        }

        match mouse {
            Mouse::ScrollDown(lines) => {
                for _ in 0..lines.max(1) {
                    self.select_next();
                }
                true
            }
            Mouse::ScrollUp(lines) => {
                for _ in 0..lines.max(1) {
                    self.select_previous();
                }
                true
            }
            Mouse::LeftClick(line, _) => {
                if let Some(index) = self.session_index_at_line(line) {
                    self.selected = index;
                    self.switch_to_selected();
                    return true;
                }
                false
            }
            Mouse::Hover(line, _) => {
                if let Some(index) = self.session_index_at_line(line) {
                    if self.selected != index {
                        self.selected = index;
                        return true;
                    }
                }
                false
            }
            _ => false,
        }
    }

    fn select_next(&mut self) {
        self.selected = (self.selected + 1) % self.sessions.len();
    }

    fn select_previous(&mut self) {
        if self.selected == 0 {
            self.selected = self.sessions.len().saturating_sub(1);
        } else {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    fn switch_to_selected(&self) {
        if let Some(row) = self.sessions.get(self.selected) {
            if !row.is_current {
                switch_session(Some(&row.name));
            }
        }
    }

    fn session_index_at_line(&self, line: isize) -> Option<usize> {
        if line < 2 {
            return None;
        }

        let visible_line = (line as usize).saturating_sub(2);
        let index = self.visible_start.saturating_add(visible_line);
        if index < self.visible_end && index < self.sessions.len() {
            Some(index)
        } else {
            None
        }
    }

    fn format_row(&self, row: &SessionRow, cols: usize) -> String {
        let marker = if row.is_current { "•" } else { " " };
        let available_for_name = cols.saturating_sub(marker.width()).saturating_sub(1);
        let name = truncate_to_width(&row.name, available_for_name);
        let line = format!("{} {}", marker, name);
        pad_to_width(line, cols)
    }
}

fn visible_bounds(total: usize, max_visible: usize, selected: usize) -> (usize, usize) {
    if total == 0 || max_visible == 0 {
        return (0, 0);
    }

    if total <= max_visible {
        return (0, total);
    }

    let half = max_visible / 2;
    let mut start = selected.saturating_sub(half);
    if start + max_visible > total {
        start = total.saturating_sub(max_visible);
    }
    (start, start + max_visible)
}

fn truncate_to_width(input: &str, max_width: usize) -> String {
    if input.width() <= max_width {
        return input.to_owned();
    }

    if max_width == 0 {
        return String::new();
    }

    let ellipsis = "…";
    if max_width == 1 {
        return ellipsis.to_owned();
    }

    let mut output = String::new();
    let target = max_width.saturating_sub(ellipsis.width());

    for character in input.chars() {
        if output.width() + character.width().unwrap_or(0) > target {
            break;
        }
        output.push(character);
    }

    output.push_str(ellipsis);
    output
}

fn pad_to_width(mut input: String, width: usize) -> String {
    let current_width = input.width();
    if current_width < width {
        input.push_str(&" ".repeat(width - current_width));
    }
    input
}
