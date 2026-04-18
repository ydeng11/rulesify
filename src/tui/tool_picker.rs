use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use std::io;

const TOOLS: [&str; 5] = ["claude-code", "codex", "cursor", "opencode", "pi"];

pub struct ToolPicker {
    selected: Vec<bool>,
    cursor: usize,
}

impl ToolPicker {
    pub fn new() -> Self {
        Self {
            selected: vec![false; TOOLS.len()],
            cursor: 0,
        }
    }

    pub fn new_with_selected(selected_tools: Vec<String>) -> Self {
        let selected = TOOLS
            .iter()
            .map(|t| selected_tools.contains(&t.to_string()))
            .collect();
        Self {
            selected,
            cursor: 0,
        }
    }

    pub fn run() -> io::Result<Vec<String>> {
        Self::run_with_selected(vec![])
    }

    pub fn run_with_selected(selected_tools: Vec<String>) -> io::Result<Vec<String>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let mut picker = Self::new_with_selected(selected_tools);

        loop {
            terminal.draw(|f| picker.render(f))?;

            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if picker.cursor > 0 {
                            picker.cursor -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if picker.cursor < TOOLS.len() - 1 {
                            picker.cursor += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        picker.selected[picker.cursor] = !picker.selected[picker.cursor];
                    }
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        return Ok(vec![]);
                    }
                    _ => {}
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(TOOLS
            .iter()
            .zip(picker.selected.iter())
            .filter_map(|(t, s)| if *s { Some(t.to_string()) } else { None })
            .collect())
    }

    fn render(&self, f: &mut ratatui::Frame) {
        let items: Vec<ListItem> = TOOLS
            .iter()
            .enumerate()
            .zip(self.selected.iter())
            .map(|((i, t), s)| {
                let symbol = if *s { "[x]" } else { "[ ]" };
                let marker = if i == self.cursor { ">" } else { " " };
                ListItem::new(format!("{} {} {}", marker, symbol, t))
            })
            .collect();

        let list = List::new(items).block(
            Block::default()
                .title("Select AI Tools (↑↓ to move, Space to toggle, Enter to confirm)")
                .borders(Borders::ALL),
        );

        f.render_widget(list, f.size());
    }
}
