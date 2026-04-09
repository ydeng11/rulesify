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
use crate::models::Skill;
use std::io;

pub struct SkillSelector {
    skills: Vec<(String, Skill)>,
    selected_indices: Vec<usize>,
    current_index: usize,
}

impl SkillSelector {
    pub fn new(skills: Vec<(String, Skill)>) -> Self {
        Self {
            skills,
            selected_indices: vec![],
            current_index: 0,
        }
    }
    
    pub fn run(mut self) -> io::Result<Vec<(String, Skill)>> {
        if self.skills.is_empty() {
            return Ok(vec![]);
        }
        
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        
        loop {
            terminal.draw(|f| self.render(f))?;
            
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Up => {
                        if self.current_index > 0 {
                            self.current_index -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if self.current_index < self.skills.len() - 1 {
                            self.current_index += 1;
                        }
                    }
                    KeyCode::Char(' ') => {
                        if self.selected_indices.contains(&self.current_index) {
                            self.selected_indices.retain(|&i| i != self.current_index);
                        } else {
                            self.selected_indices.push(self.current_index);
                        }
                    }
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
        
        Ok(self.selected_indices
            .into_iter()
            .map(|i| self.skills[i].clone())
            .collect())
    }
    
    fn render(&self, f: &mut ratatui::Frame) {
        let items: Vec<ListItem> = self.skills.iter()
            .enumerate()
            .map(|(i, (_id, skill))| {
                let marker = if self.selected_indices.contains(&i) { "[x]" } else { "[ ]" };
                let cursor = if i == self.current_index { ">" } else { " " };
                ListItem::new(format!(
                    "{}{} {} - {} (★{})",
                    cursor, marker, skill.name, skill.description, skill.popularity
                ))
            })
            .collect();
        
        let list = List::new(items)
            .block(Block::default()
                .title("Select Skills (↑↓ navigate, Space select, Enter confirm)")
                .borders(Borders::ALL));
        
        f.render_widget(list, f.size());
    }
}