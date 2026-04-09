use ratatui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, List, ListItem},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, enable_raw_mode, disable_raw_mode},
};
use std::io;

const TOOLS: [&str; 4] = ["cursor", "claude-code", "cline", "goose"];

pub struct ToolPicker {
    selected: Vec<bool>,
}

impl ToolPicker {
    pub fn new() -> Self {
        Self {
            selected: vec![false, false, false, false],
        }
    }
    
    pub fn run() -> io::Result<Vec<String>> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        let mut picker = Self::new();
        
        loop {
            terminal.draw(|f| picker.render(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('1') => picker.selected[0] = !picker.selected[0],
                    KeyCode::Char('2') => picker.selected[1] = !picker.selected[1],
                    KeyCode::Char('3') => picker.selected[2] = !picker.selected[2],
                    KeyCode::Char('4') => picker.selected[3] = !picker.selected[3],
                    KeyCode::Enter => break,
                    KeyCode::Esc => {
                        disable_raw_mode()?;
                        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                        return Ok(vec![]);
                    },
                    _ => {}
                }
            }
        }
        
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        
        Ok(TOOLS.iter()
            .zip(picker.selected.iter())
            .filter_map(|(t, s)| if *s { Some(t.to_string()) } else { None })
            .collect())
    }
    
    fn render(&self, f: &mut ratatui::Frame) {
        let items: Vec<ListItem> = TOOLS.iter()
            .enumerate()
            .zip(self.selected.iter())
            .map(|((i, t), s)| {
                let symbol = if *s { "[x]" } else { "[ ]" };
                ListItem::new(format!("{}. {} {}", i + 1, symbol, t))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title("Select AI Tools (1-4 to toggle, Enter to confirm)")
                .borders(Borders::ALL));
        
        f.render_widget(list, f.size());
    }
}