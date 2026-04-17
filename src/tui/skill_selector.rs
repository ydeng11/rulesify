use crate::models::Skill;
use crate::tui::SortMode;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Terminal,
};
use std::collections::HashSet;
use std::io;

struct SkillSelectorState {
    all_skills: Vec<(String, Skill)>,
    filtered_skills: Vec<(String, Skill)>,
    domains: Vec<String>,
    domain_index: usize,
    all_tags: Vec<String>,
    selected_tags: HashSet<String>,
    sort_mode: SortMode,
    current_skill_index: usize,
    selected_skill_indices: Vec<usize>,
    show_tag_popup: bool,
    tag_popup_index: usize,
    tag_popup_selected: HashSet<usize>,
}

impl SkillSelectorState {
    fn new(skills: Vec<(String, Skill)>) -> Self {
        let domains = Self::extract_domains(&skills);
        let all_tags = Self::extract_tags(&skills);
        let filtered_skills = skills.clone();

        Self {
            all_skills: skills,
            filtered_skills,
            domains,
            domain_index: 0,
            all_tags,
            selected_tags: HashSet::new(),
            sort_mode: SortMode::default(),
            current_skill_index: 0,
            selected_skill_indices: vec![],
            show_tag_popup: false,
            tag_popup_index: 0,
            tag_popup_selected: HashSet::new(),
        }
    }

    fn extract_domains(skills: &[(String, Skill)]) -> Vec<String> {
        let mut domains: Vec<String> = skills
            .iter()
            .map(|(_, s)| s.domain.clone())
            .filter(|d| !d.is_empty())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        domains.sort();
        domains.insert(0, "All".to_string());
        domains
    }

    fn extract_tags(skills: &[(String, Skill)]) -> Vec<String> {
        let mut tags: Vec<String> = skills
            .iter()
            .flat_map(|(_, s)| s.tags.iter().cloned())
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        tags.sort();
        tags
    }

    fn apply_filters(&mut self) {
        let domain_filter = if self.domain_index == 0 {
            None
        } else {
            Some(self.domains[self.domain_index].clone())
        };

        let filtered: Vec<(String, Skill)> = self
            .all_skills
            .iter()
            .filter(|(_, skill)| {
                if let Some(domain) = &domain_filter {
                    skill.domain == *domain
                } else {
                    true
                }
            })
            .filter(|(_, skill)| {
                if self.selected_tags.is_empty() {
                    true
                } else {
                    self.selected_tags.iter().all(|t| skill.tags.contains(t))
                }
            })
            .cloned()
            .collect();

        self.filtered_skills = Self::apply_sort(filtered, self.sort_mode);
        self.current_skill_index = 0;
    }

    fn apply_sort(skills: Vec<(String, Skill)>, mode: SortMode) -> Vec<(String, Skill)> {
        let mut sorted = skills;
        match mode {
            SortMode::StarsDesc => sorted.sort_by(|a, b| b.1.stars.cmp(&a.1.stars)),
            SortMode::StarsAsc => sorted.sort_by(|a, b| a.1.stars.cmp(&b.1.stars)),
            SortMode::ScoreDesc => sorted.sort_by(|a, b| {
                b.1.score
                    .unwrap_or(0.0)
                    .partial_cmp(&a.1.score.unwrap_or(0.0))
                    .unwrap()
            }),
            SortMode::ScoreAsc => sorted.sort_by(|a, b| {
                a.1.score
                    .unwrap_or(0.0)
                    .partial_cmp(&b.1.score.unwrap_or(0.0))
                    .unwrap()
            }),
            SortMode::NameAsc => sorted.sort_by(|a, b| a.1.name.cmp(&b.1.name)),
            SortMode::NameDesc => sorted.sort_by(|a, b| b.1.name.cmp(&a.1.name)),
        }
        sorted
    }

    fn render_domain_tabs(&self, f: &mut ratatui::Frame, area: Rect) {
        let titles: Vec<Span> = self
            .domains
            .iter()
            .enumerate()
            .map(|(i, d)| {
                if i == self.domain_index {
                    Span::styled(
                        d.clone(),
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD),
                    )
                } else {
                    Span::styled(d.clone(), Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::BOTTOM))
            .divider(Span::raw(" | "))
            .style(Style::default().fg(Color::White));

        f.render_widget(tabs, area);
    }

    fn render_status_bar(&self, f: &mut ratatui::Frame, area: Rect) {
        let sort_text = format!("Sort: {}", self.sort_mode.label());
        let tags_text = if self.selected_tags.is_empty() {
            "Tags: none".to_string()
        } else {
            format!(
                "Tags: {}",
                self.selected_tags
                    .iter()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        let text = format!("{} | {}", sort_text, tags_text);

        let status = Paragraph::new(text).style(Style::default().fg(Color::Cyan));

        f.render_widget(status, area);
    }

    fn render_help_bar(&self, f: &mut ratatui::Frame, area: Rect) {
        let help_text = if self.show_tag_popup {
            "↑↓ Nav | Space Toggle | Enter Apply | Esc Cancel"
        } else {
            "↑↓ Nav | Space Select | ←→ Domain | s Sort | t Tags | Enter Done | Esc Cancel"
        };

        let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

        f.render_widget(help, area);
    }

    fn render_skill_list(&self, f: &mut ratatui::Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .filtered_skills
            .iter()
            .enumerate()
            .map(|(i, (_id, skill))| {
                let marker = if self.selected_skill_indices.contains(&i) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let cursor = if i == self.current_skill_index {
                    ">"
                } else {
                    " "
                };
                let score_text = skill
                    .score
                    .map(|s| format!("{:.0}", s))
                    .unwrap_or_else(|| "-".to_string());
                ListItem::new(format!(
                    "{}{} {} - {} (★{} | {})",
                    cursor, marker, skill.name, skill.description, skill.stars, score_text
                ))
            })
            .collect();

        let title = format!("Skills ({})", self.filtered_skills.len());
        let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(list, area);
    }

    fn render_tag_popup(&self, f: &mut ratatui::Frame, area: Rect) {
        let popup_area = Rect {
            x: area.x + area.width / 4,
            y: area.y + area.height / 4,
            width: area.width / 2,
            height: area.height / 2,
        };

        let items: Vec<ListItem> = self
            .all_tags
            .iter()
            .enumerate()
            .map(|(i, tag)| {
                let marker = if self.tag_popup_selected.contains(&i) {
                    "[x]"
                } else {
                    "[ ]"
                };
                let cursor = if i == self.tag_popup_index { ">" } else { " " };
                ListItem::new(format!("{}{} {}", cursor, marker, tag))
            })
            .collect();

        let list =
            List::new(items).block(Block::default().title("Select Tags").borders(Borders::ALL));

        f.render_widget(list, popup_area);
    }

    fn render(&self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Length(1),
                Constraint::Min(1),
                Constraint::Length(1),
            ])
            .split(f.size());

        self.render_domain_tabs(f, chunks[0]);
        self.render_status_bar(f, chunks[1]);
        self.render_skill_list(f, chunks[2]);
        self.render_help_bar(f, chunks[3]);

        if self.show_tag_popup {
            self.render_tag_popup(f, f.size());
        }
    }

    fn handle_key(&mut self, key: KeyCode) -> bool {
        if self.show_tag_popup {
            return self.handle_tag_popup_key(key);
        }

        match key {
            KeyCode::Up => {
                if self.current_skill_index > 0 {
                    self.current_skill_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.current_skill_index < self.filtered_skills.len().saturating_sub(1) {
                    self.current_skill_index += 1;
                }
            }
            KeyCode::Left => {
                if self.domain_index > 0 {
                    self.domain_index -= 1;
                    self.apply_filters();
                }
            }
            KeyCode::Right => {
                if self.domain_index < self.domains.len().saturating_sub(1) {
                    self.domain_index += 1;
                    self.apply_filters();
                }
            }
            KeyCode::Char(' ') => {
                if self.filtered_skills.is_empty() {
                    return false;
                }
                if self
                    .selected_skill_indices
                    .contains(&self.current_skill_index)
                {
                    self.selected_skill_indices
                        .retain(|&i| i != self.current_skill_index);
                } else {
                    self.selected_skill_indices.push(self.current_skill_index);
                }
            }
            KeyCode::Char('s') => {
                self.sort_mode = self.sort_mode.cycle();
                self.apply_filters();
            }
            KeyCode::Char('t') => {
                self.show_tag_popup = true;
                self.tag_popup_index = 0;
                self.tag_popup_selected = self
                    .selected_tags
                    .iter()
                    .filter_map(|t| self.all_tags.iter().position(|tag| tag == t))
                    .collect();
            }
            KeyCode::Enter => return true,
            KeyCode::Esc => {
                self.selected_skill_indices.clear();
                return true;
            }
            _ => {}
        }
        false
    }

    fn handle_tag_popup_key(&mut self, key: KeyCode) -> bool {
        match key {
            KeyCode::Up => {
                if self.tag_popup_index > 0 {
                    self.tag_popup_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.tag_popup_index < self.all_tags.len().saturating_sub(1) {
                    self.tag_popup_index += 1;
                }
            }
            KeyCode::Char(' ') => {
                if self.tag_popup_selected.contains(&self.tag_popup_index) {
                    self.tag_popup_selected.remove(&self.tag_popup_index);
                } else {
                    self.tag_popup_selected.insert(self.tag_popup_index);
                }
            }
            KeyCode::Enter => {
                self.selected_tags = self
                    .tag_popup_selected
                    .iter()
                    .map(|&i| self.all_tags[i].clone())
                    .collect();
                self.show_tag_popup = false;
                self.apply_filters();
            }
            KeyCode::Esc => {
                self.show_tag_popup = false;
            }
            _ => {}
        }
        false
    }
}

pub struct SkillSelector {
    skills: Vec<(String, Skill)>,
}

impl SkillSelector {
    pub fn new(skills: Vec<(String, Skill)>) -> Self {
        Self { skills }
    }

    pub fn run(self) -> io::Result<Vec<(String, Skill)>> {
        if self.skills.is_empty() {
            return Ok(vec![]);
        }

        let mut state = SkillSelectorState::new(self.skills);

        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        loop {
            terminal.draw(|f| state.render(f))?;

            if let Event::Key(key) = event::read()? {
                if state.handle_key(key.code) {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

        Ok(state
            .selected_skill_indices
            .into_iter()
            .map(|i| state.filtered_skills[i].clone())
            .collect())
    }
}
