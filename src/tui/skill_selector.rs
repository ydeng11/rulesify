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
    widgets::{Block, Borders, Clear, Paragraph, Tabs, Wrap},
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
    skill_search_query: String,
    skill_search_active: bool,
    skill_scroll_offset: usize,
    skill_list_height: usize,
    show_tag_popup: bool,
    tag_search_query: String,
    tag_scroll_offset: usize,
    tag_popup_index: usize,
    tag_popup_selected: HashSet<usize>,
    tag_popup_rows_per_page: usize,
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
            skill_search_query: String::new(),
            skill_search_active: false,
            skill_scroll_offset: 0,
            skill_list_height: 15,
            show_tag_popup: false,
            tag_search_query: String::new(),
            tag_scroll_offset: 0,
            tag_popup_index: 0,
            tag_popup_selected: HashSet::new(),
            tag_popup_rows_per_page: 20,
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
            .filter(|(_, skill)| {
                if self.skill_search_query.is_empty() {
                    true
                } else {
                    skill
                        .name
                        .to_lowercase()
                        .contains(&self.skill_search_query.to_lowercase())
                }
            })
            .cloned()
            .collect();

        self.filtered_skills = Self::apply_sort(filtered, self.sort_mode);
        self.current_skill_index = 0;
        self.skill_scroll_offset = 0;
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
        } else if self.skill_search_active {
            "Type to search | Backspace Delete | Enter Done | Esc Cancel Search"
        } else {
            "↑↓ Nav | Space Select | ←→ Domain | s Sort | t Tags | / Search | Enter Done | Esc Cancel"
        };

        let help = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));

        f.render_widget(help, area);
    }

    fn render_skill_list(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let mut lines: Vec<Line> = Vec::new();

        let search_indicator = if self.skill_search_active {
            format!("Search: {}", self.skill_search_query)
        } else {
            "Press / to search".to_string()
        };
        lines.push(Line::from(vec![Span::styled(
            search_indicator,
            Style::default().fg(Color::Cyan),
        )]));
        lines.push(Line::from(""));

        let list_height = area.height.saturating_sub(5) as usize;
        self.skill_list_height = list_height;
        let start_idx = self.skill_scroll_offset;
        let end_idx = std::cmp::min(start_idx + list_height, self.filtered_skills.len());

        for i in start_idx..end_idx {
            let (_, skill) = &self.filtered_skills[i];
            let is_selected = self.selected_skill_indices.contains(&i);
            let is_cursor = i == self.current_skill_index;

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

            let score_text = skill
                .score
                .map(|s| format!("{:.0}", s))
                .unwrap_or_else(|| "-".to_string());

            lines.push(Line::from(vec![
                Span::styled(format!("{}{} ", cursor, marker), style),
                Span::styled(skill.name.clone(), style),
                Span::raw(" "),
                Span::styled(
                    format!("★{}", skill.stars),
                    Style::default().fg(Color::Gray),
                ),
                Span::styled(" | ", Style::default().fg(Color::DarkGray)),
                Span::styled(score_text, Style::default().fg(Color::Cyan)),
            ]));
        }

        if self.filtered_skills.len() > list_height {
            lines.push(Line::from(""));
            lines.push(Line::from(Span::styled(
                format!(
                    "--- {}-{} of {} ---",
                    start_idx + 1,
                    end_idx,
                    self.filtered_skills.len()
                ),
                Style::default().fg(Color::DarkGray),
            )));
        }

        let title = format!("Skills ({})", self.filtered_skills.len());
        let text = Text::from(lines);
        let list = Paragraph::new(text).block(Block::default().title(title).borders(Borders::ALL));

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

    fn render_tag_popup(&mut self, f: &mut ratatui::Frame, area: Rect) {
        let popup_width = (area.width * 3 / 4).min(100);
        let popup_height = (area.height * 3 / 4).min(30);
        let popup_x = (area.width - popup_width) / 2;
        let popup_y = (area.height - popup_height) / 2;
        let popup_area = Rect {
            x: area.x + popup_x,
            y: area.y + popup_y,
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

        let cols: usize = 3;
        let rows_per_page = available_height as usize;
        self.tag_popup_rows_per_page = rows_per_page;

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

    fn render(&mut self, f: &mut ratatui::Frame) {
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

        if self.skill_search_active {
            match key {
                KeyCode::Char(c) => {
                    self.skill_search_query.push(c);
                    self.apply_filters();
                }
                KeyCode::Backspace => {
                    self.skill_search_query.pop();
                    self.apply_filters();
                }
                KeyCode::Esc => {
                    self.skill_search_active = false;
                    self.skill_search_query.clear();
                    self.apply_filters();
                }
                KeyCode::Enter => {
                    self.skill_search_active = false;
                }
                _ => {}
            }
            return false;
        }

        match key {
            KeyCode::Up => {
                if self.current_skill_index > 0 {
                    self.current_skill_index -= 1;
                    self.update_skill_scroll_offset();
                }
            }
            KeyCode::Down => {
                if self.current_skill_index < self.filtered_skills.len().saturating_sub(1) {
                    self.current_skill_index += 1;
                    self.update_skill_scroll_offset();
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
            KeyCode::Char('/') => {
                self.skill_search_active = true;
                self.skill_search_query.clear();
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
        3
    }

    fn update_scroll_offset(&mut self, filtered_len: usize, cols: usize) {
        if filtered_len == 0 || cols == 0 {
            return;
        }
        let rows_per_page = self.tag_popup_rows_per_page;

        let current_row = self.tag_popup_index / cols;
        if current_row < self.tag_scroll_offset {
            self.tag_scroll_offset = current_row;
        } else if current_row >= self.tag_scroll_offset + rows_per_page {
            self.tag_scroll_offset = current_row - rows_per_page + 1;
        }
    }

    fn update_skill_scroll_offset(&mut self) {
        if self.filtered_skills.is_empty() {
            return;
        }
        let list_height = self.skill_list_height;
        if self.current_skill_index < self.skill_scroll_offset {
            self.skill_scroll_offset = self.current_skill_index;
        } else if self.current_skill_index >= self.skill_scroll_offset + list_height {
            self.skill_scroll_offset = self.current_skill_index - list_height + 1;
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
