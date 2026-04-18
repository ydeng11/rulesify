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
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Tabs, Wrap},
    Terminal,
};
use std::collections::{HashMap, HashSet};
use std::io;

struct SkillSelectorState {
    all_skills: Vec<(String, Skill)>,
    filtered_skills: Vec<(String, Skill)>,
    domains: Vec<String>,
    domain_index: usize,
    all_tags: Vec<(String, usize)>,
    selected_tags: HashSet<String>,
    sort_mode: SortMode,
    current_skill_index: usize,
    selected_skill_indices: Vec<usize>,
    show_tag_popup: bool,
    tag_search_query: String,
    tag_scroll_offset: usize,
    tag_popup_index: usize,
    tag_popup_selected: HashSet<usize>,
}

impl SkillSelectorState {
    fn new(skills: Vec<(String, Skill)>) -> Self {
        let domains = Self::extract_domains(&skills);
        let all_tags = Self::extract_tags_with_counts(&skills);
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
            tag_search_query: String::new(),
            tag_scroll_offset: 0,
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

    fn extract_tags_with_counts(skills: &[(String, Skill)]) -> Vec<(String, usize)> {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();
        for (_, skill) in skills {
            for tag in &skill.tags {
                *tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        let mut tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        tags.sort_by(|a, b| a.0.cmp(&b.0));
        tags
    }

    fn get_filtered_tags(&self) -> Vec<(String, usize)> {
        if self.tag_search_query.is_empty() {
            self.all_tags.clone()
        } else {
            self.all_tags
                .iter()
                .filter(|(tag, _)| {
                    tag.to_lowercase()
                        .contains(&self.tag_search_query.to_lowercase())
                })
                .cloned()
                .collect()
        }
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
                let style = if i == self.current_skill_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Line::from(vec![
                    Span::styled(format!("{}{} ", cursor, marker), style),
                    Span::styled(skill.name.clone(), style),
                    Span::styled(" ", Style::default()),
                    Span::styled(
                        format!("★{}", skill.stars),
                        Style::default().fg(Color::Gray),
                    ),
                    Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                    Span::styled(score_text, Style::default().fg(Color::Cyan)),
                ]))
            })
            .collect();

        let title = format!("Skills ({})", self.filtered_skills.len());
        let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

        f.render_widget(list, area);
    }

    fn render_skill_details(&self, f: &mut ratatui::Frame, area: Rect) {
        if self.filtered_skills.is_empty() {
            let empty = Paragraph::new("No skills selected")
                .block(Block::default().title("Details").borders(Borders::ALL));
            f.render_widget(empty, area);
            return;
        }

        let (_, skill) = &self.filtered_skills[self.current_skill_index];

        let mut lines: Vec<Line> = vec![
            Line::from(vec![
                Span::styled("Name: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    skill.name.clone(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(vec![
                Span::styled("Score: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    skill
                        .score
                        .map(|s| format!("{:.0}", s))
                        .unwrap_or_else(|| "-".to_string()),
                    Style::default().fg(Color::Yellow),
                ),
                Span::styled("  Stars: ", Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!("★{}", skill.stars),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::styled("Domain: ", Style::default().fg(Color::Cyan)),
                Span::styled(skill.domain.clone(), Style::default().fg(Color::White)),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Description:",
                Style::default().fg(Color::Cyan),
            )),
        ];

        let desc_lines = textwrap::wrap(&skill.description, area.width.saturating_sub(4) as usize);
        for line in desc_lines {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(Color::White),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Tags:",
            Style::default().fg(Color::Cyan),
        )));

        if skill.tags.is_empty() {
            lines.push(Line::from(Span::styled(
                "  (none)",
                Style::default().fg(Color::DarkGray),
            )));
        } else {
            let tags_str = skill
                .tags
                .iter()
                .map(|t| format!("  • {}", t))
                .collect::<Vec<_>>();
            for tag in tags_str {
                lines.push(Line::from(Span::styled(
                    tag,
                    Style::default().fg(Color::White),
                )));
            }
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "Source:",
            Style::default().fg(Color::Cyan),
        )));
        lines.push(Line::from(Span::styled(
            skill.source_url.clone(),
            Style::default().fg(Color::DarkGray),
        )));

        let details = Paragraph::new(Text::from(lines))
            .block(Block::default().title("Details").borders(Borders::ALL))
            .wrap(Wrap { trim: false });

        f.render_widget(details, area);
    }

    fn render_tag_popup(&self, f: &mut ratatui::Frame, area: Rect) {
        let popup_width = area.width.saturating_sub(4);
        let popup_height = area.height.saturating_sub(4);
        let popup_area = Rect {
            x: area.x + 2,
            y: area.y + 2,
            width: popup_width,
            height: popup_height,
        };

        f.render_widget(Clear, popup_area);

        let filtered_tags = self.get_filtered_tags();

        if filtered_tags.is_empty() {
            let empty_msg = Paragraph::new("No matching tags").block(
                Block::default()
                    .title(format!("Search: {}", self.tag_search_query))
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black)),
            );
            f.render_widget(empty_msg, popup_area);
            return;
        }

        let inner_width = popup_width.saturating_sub(2);
        let search_bar_height = 2;
        let help_bar_height = 1;
        let available_height = popup_height.saturating_sub(search_bar_height + help_bar_height + 2);

        let max_tag_len = filtered_tags
            .iter()
            .map(|(tag, _)| tag.len())
            .max()
            .unwrap_or(0);
        let max_count_digits = filtered_tags
            .iter()
            .map(|(_, count)| count.to_string().len())
            .max()
            .unwrap_or(1);

        let item_width = 4 + max_tag_len + 2 + max_count_digits + 1;
        let cols = std::cmp::max(1, inner_width as usize / item_width);
        let rows_per_page = available_height as usize;

        let mut lines: Vec<Line> = Vec::new();

        let search_line = Line::from(vec![
            Span::styled("Search: ", Style::default().fg(Color::Cyan)),
            Span::styled(
                if self.tag_search_query.is_empty() {
                    "(type to filter)"
                } else {
                    &self.tag_search_query
                },
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);
        lines.push(search_line);
        lines.push(Line::from(""));

        let total_rows = (filtered_tags.len() + cols - 1) / cols;
        let start_row = self.tag_scroll_offset;
        let end_row = std::cmp::min(start_row + rows_per_page, total_rows);

        let mut row_items: Vec<Span> = Vec::new();

        for row_idx in start_row..end_row {
            row_items.clear();

            for col_idx in 0..cols {
                let flat_idx = row_idx * cols + col_idx;
                if flat_idx >= filtered_tags.len() {
                    break;
                }

                let (tag, count) = &filtered_tags[flat_idx];
                let original_idx = self
                    .all_tags
                    .iter()
                    .position(|(t, _)| t == tag)
                    .unwrap_or(flat_idx);

                let is_selected = self.tag_popup_selected.contains(&original_idx);
                let is_cursor = flat_idx == self.tag_popup_index;

                let marker = if is_selected { "[x]" } else { "[ ]" };
                let cursor = if is_cursor { ">" } else { " " };

                let style = if is_cursor {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else if is_selected {
                    Style::default().fg(Color::Green)
                } else {
                    Style::default().fg(Color::White)
                };

                let item = Span::styled(
                    format!(
                        "{}{} {:tag_len$}({:count_len$})",
                        cursor,
                        marker,
                        tag,
                        count,
                        tag_len = max_tag_len,
                        count_len = max_count_digits
                    ),
                    style,
                );

                row_items.push(item);
                if col_idx < cols - 1 {
                    row_items.push(Span::raw("  "));
                }
            }

            lines.push(Line::from(row_items.clone()));
        }

        if total_rows > rows_per_page {
            let scroll_info = format!(
                "  --- Row {}-{} of {} (scroll with ↑↓) ---",
                start_row + 1,
                end_row,
                total_rows
            );
            lines.push(Line::from(Span::styled(
                scroll_info,
                Style::default().fg(Color::DarkGray),
            )));
        }

        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled(
            "↑↓ Nav | Space Toggle | Backspace Clear | Enter Apply | Esc Cancel",
            Style::default().fg(Color::DarkGray),
        )));

        let title = format!(
            "Select Tags ({}/{})",
            filtered_tags.len(),
            self.all_tags.len()
        );
        let tags_text = Text::from(lines);
        let list = Paragraph::new(tags_text)
            .style(Style::default().bg(Color::Reset))
            .block(
                Block::default()
                    .title(title)
                    .borders(Borders::ALL)
                    .style(Style::default().bg(Color::Black)),
            );

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

        let panels = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(chunks[2]);

        self.render_skill_list(f, panels[0]);
        self.render_skill_details(f, panels[1]);
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
                self.tag_search_query.clear();
                self.tag_scroll_offset = 0;
                self.tag_popup_index = 0;
                self.tag_popup_selected = self
                    .selected_tags
                    .iter()
                    .filter_map(|t| self.all_tags.iter().position(|(tag, _)| tag == t))
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
        let filtered_tags = self.get_filtered_tags();
        let cols = self.calculate_cols();

        match key {
            KeyCode::Up => {
                if self.tag_popup_index >= cols {
                    self.tag_popup_index -= cols;
                } else if self.tag_popup_index > 0 {
                    self.tag_popup_index = 0;
                }
                self.update_scroll_offset(filtered_tags.len(), cols);
            }
            KeyCode::Down => {
                let max_idx = filtered_tags.len().saturating_sub(1);
                if self.tag_popup_index < max_idx {
                    self.tag_popup_index = std::cmp::min(self.tag_popup_index + cols, max_idx);
                    self.update_scroll_offset(filtered_tags.len(), cols);
                }
            }
            KeyCode::Left => {
                if self.tag_popup_index > 0 {
                    self.tag_popup_index -= 1;
                    self.update_scroll_offset(filtered_tags.len(), cols);
                }
            }
            KeyCode::Right => {
                if self.tag_popup_index < filtered_tags.len().saturating_sub(1) {
                    self.tag_popup_index += 1;
                    self.update_scroll_offset(filtered_tags.len(), cols);
                }
            }
            KeyCode::Char(c) => {
                if c == ' ' && !filtered_tags.is_empty() {
                    if self.tag_popup_index < filtered_tags.len() {
                        let (tag, _) = &filtered_tags[self.tag_popup_index];
                        let original_idx = self
                            .all_tags
                            .iter()
                            .position(|(t, _)| t == tag)
                            .unwrap_or(self.tag_popup_index);

                        if self.tag_popup_selected.contains(&original_idx) {
                            self.tag_popup_selected.remove(&original_idx);
                        } else {
                            self.tag_popup_selected.insert(original_idx);
                        }
                    }
                } else if c != ' ' {
                    self.tag_search_query.push(c);
                    self.tag_popup_index = 0;
                    self.tag_scroll_offset = 0;
                }
            }
            KeyCode::Backspace => {
                self.tag_search_query.pop();
                self.tag_popup_index = 0;
                self.tag_scroll_offset = 0;
            }
            KeyCode::Enter => {
                self.selected_tags = self
                    .tag_popup_selected
                    .iter()
                    .map(|&i| self.all_tags[i].0.clone())
                    .collect();
                self.show_tag_popup = false;
                self.tag_search_query.clear();
                self.tag_scroll_offset = 0;
                self.apply_filters();
            }
            KeyCode::Esc => {
                self.show_tag_popup = false;
                self.tag_search_query.clear();
                self.tag_scroll_offset = 0;
            }
            _ => {}
        }
        false
    }

    fn calculate_cols(&self) -> usize {
        let filtered_tags = self.get_filtered_tags();
        let max_tag_len = filtered_tags
            .iter()
            .map(|(tag, _)| tag.len())
            .max()
            .unwrap_or(0);
        let max_count_digits = filtered_tags
            .iter()
            .map(|(_, count)| count.to_string().len())
            .max()
            .unwrap_or(1);
        let item_width = 4 + max_tag_len + 2 + max_count_digits + 1;
        let estimated_width = 80;
        std::cmp::max(1, estimated_width / item_width)
    }

    fn update_scroll_offset(&mut self, filtered_len: usize, cols: usize) {
        if filtered_len == 0 || cols == 0 {
            return;
        }
        let popup_height_estimate: usize = 20;
        let search_bar_height: usize = 3;
        let help_bar_height: usize = 1;
        let rows_per_page =
            popup_height_estimate.saturating_sub(search_bar_height + help_bar_height + 2);

        let current_row = self.tag_popup_index / cols;
        if current_row < self.tag_scroll_offset {
            self.tag_scroll_offset = current_row;
        } else if current_row >= self.tag_scroll_offset + rows_per_page {
            self.tag_scroll_offset = current_row - rows_per_page + 1;
        }
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
