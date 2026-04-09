use std::{io, path::Path, time::Duration};

use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::{
    model::{ProjectSummary, SessionSummary},
    scanner::{self, SessionSource},
};

const BG: Color = Color::Rgb(16, 20, 24);
const PANEL: Color = Color::Rgb(23, 29, 35);
const PANEL_ALT: Color = Color::Rgb(27, 34, 41);
const INK: Color = Color::Rgb(231, 225, 214);
const MUTED: Color = Color::Rgb(143, 151, 156);
const BRASS: Color = Color::Rgb(214, 180, 98);
const OLIVE: Color = Color::Rgb(140, 170, 120);
const RUST: Color = Color::Rgb(188, 112, 91);
const SKY: Color = Color::Rgb(120, 168, 196);

pub fn run_dashboard(repo: Option<String>) -> Result<()> {
    let preferred_repo = preferred_repo(repo.as_deref());
    let (sessions, source) = scanner::load_sessions()?;
    let projects = scanner::summarize_projects(&sessions);
    let mut app = DashboardApp::new(projects, sessions, source, preferred_repo);
    let mut terminal = init_terminal()?;
    let result = run_app(&mut terminal, &mut app);
    restore_terminal(&mut terminal)?;
    result
}

fn preferred_repo(explicit_repo: Option<&str>) -> Option<std::path::PathBuf> {
    match explicit_repo {
        Some(path) => scanner::current_repo_root(Some(path)).ok(),
        None => scanner::current_repo_root(None).ok(),
    }
}

fn init_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend).context("failed to initialize terminal")
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode().context("failed to disable raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)
        .context("failed to leave alternate screen")?;
    terminal.show_cursor().context("failed to show cursor")
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut DashboardApp,
) -> Result<()> {
    loop {
        terminal.draw(|frame| app.render(frame))?;

        if !event::poll(Duration::from_millis(200))? {
            continue;
        }

        let Event::Key(key) = event::read()? else {
            continue;
        };

        if key.kind != KeyEventKind::Press {
            continue;
        }

        if app.handle_key(key)? {
            break;
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusPane {
    Projects,
    Sessions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputMode {
    Normal,
    Search,
}

struct DashboardApp {
    projects: Vec<ProjectSummary>,
    sessions: Vec<SessionSummary>,
    visible_sessions: Vec<SessionSummary>,
    project_state: ListState,
    session_state: ListState,
    focus: FocusPane,
    input_mode: InputMode,
    source: SessionSource,
    search_query: String,
    status: String,
}

impl DashboardApp {
    fn new(
        projects: Vec<ProjectSummary>,
        sessions: Vec<SessionSummary>,
        source: SessionSource,
        preferred_repo: Option<std::path::PathBuf>,
    ) -> Self {
        let preferred_key = preferred_repo.as_ref().map(|path| normalize_path(path));
        let mut project_state = ListState::default();
        let selected_project = preferred_key
            .as_deref()
            .and_then(|needle| {
                projects
                    .iter()
                    .position(|project| normalize_path(&project.repo_root) == needle)
            })
            .unwrap_or(0);
        if !projects.is_empty() {
            project_state.select(Some(selected_project));
        }

        let mut app = Self {
            projects,
            sessions,
            visible_sessions: Vec::new(),
            project_state,
            session_state: ListState::default(),
            focus: FocusPane::Projects,
            input_mode: InputMode::Normal,
            source,
            search_query: String::new(),
            status: "Continuity board ready. Use Tab to move, / to search, i to reindex."
                .to_owned(),
        };
        app.refresh_visible_sessions();
        app
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<bool> {
        match self.input_mode {
            InputMode::Search => self.handle_search_key(key),
            InputMode::Normal => self.handle_normal_key(key),
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Esc => {
                self.input_mode = InputMode::Normal;
                self.status = "Search cancelled.".to_owned();
            }
            KeyCode::Enter => {
                self.input_mode = InputMode::Normal;
                self.refresh_visible_sessions();
                if self.search_query.trim().is_empty() {
                    self.status = "Search cleared. Showing project sessions.".to_owned();
                } else {
                    self.status = format!(
                        "Search applied: {} result(s) for \"{}\".",
                        self.visible_sessions.len(),
                        self.search_query
                    );
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                self.refresh_visible_sessions();
            }
            KeyCode::Char(ch) => {
                self.search_query.push(ch);
                self.refresh_visible_sessions();
            }
            _ => {}
        }

        Ok(false)
    }

    fn handle_normal_key(&mut self, key: KeyEvent) -> Result<bool> {
        match key.code {
            KeyCode::Char('q') => return Ok(true),
            KeyCode::Tab => {
                self.focus = match self.focus {
                    FocusPane::Projects => FocusPane::Sessions,
                    FocusPane::Sessions => FocusPane::Projects,
                };
            }
            KeyCode::Char('g') => self.focus = FocusPane::Projects,
            KeyCode::Char('s') | KeyCode::Enter => self.focus = FocusPane::Sessions,
            KeyCode::Esc => {
                if !self.search_query.is_empty() {
                    self.search_query.clear();
                    self.refresh_visible_sessions();
                    self.status = "Search cleared. Showing project sessions.".to_owned();
                }
            }
            KeyCode::Char('/') => {
                self.input_mode = InputMode::Search;
                self.status = "Search mode. Type to filter current project sessions.".to_owned();
            }
            KeyCode::Char('i') => {
                self.status = "Reindexing archive...".to_owned();
                let selected_repo = self
                    .selected_project()
                    .map(|project| project.repo_root.clone());
                let sessions = scanner::rebuild_session_index()?;
                let projects = scanner::summarize_projects(&sessions);
                self.sessions = sessions;
                self.projects = projects;
                self.source = SessionSource::Cache;
                self.reselect_project(selected_repo.as_deref());
                self.refresh_visible_sessions();
                self.status = format!(
                    "Reindex complete. {} projects, {} sessions.",
                    self.projects.len(),
                    self.sessions.len()
                );
            }
            KeyCode::Up | KeyCode::Char('k') => self.move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => self.move_selection(1),
            _ => {}
        }

        Ok(false)
    }

    fn move_selection(&mut self, delta: isize) {
        match self.focus {
            FocusPane::Projects => {
                let Some(next) =
                    move_index(self.project_state.selected(), self.projects.len(), delta)
                else {
                    return;
                };
                self.project_state.select(Some(next));
                self.refresh_visible_sessions();
                if let Some(project) = self.selected_project() {
                    self.status = format!(
                        "Project selected: {} ({} session{}).",
                        project.repo_root.display(),
                        project.session_count,
                        if project.session_count == 1 { "" } else { "s" }
                    );
                }
            }
            FocusPane::Sessions => {
                let Some(next) = move_index(
                    self.session_state.selected(),
                    self.visible_sessions.len(),
                    delta,
                ) else {
                    return;
                };
                self.session_state.select(Some(next));
                if let Some(session) = self.selected_session() {
                    self.status = format!("Session selected: {}.", session.id);
                }
            }
        }
    }

    fn refresh_visible_sessions(&mut self) {
        let repo = self
            .selected_project()
            .map(|project| project.repo_root.clone());
        self.visible_sessions = match repo.as_ref() {
            Some(repo_root) if !self.search_query.trim().is_empty() => scanner::search_sessions(
                &self.sessions,
                &self.search_query,
                Some(repo_root.as_path()),
                24,
            )
            .into_iter()
            .map(|hit| hit.session)
            .collect(),
            Some(repo_root) => self
                .sessions
                .iter()
                .filter(|session| {
                    normalize_path(&session.attributed_repo_root) == normalize_path(repo_root)
                })
                .take(24)
                .cloned()
                .collect(),
            None => Vec::new(),
        };

        let next_session = if self.visible_sessions.is_empty() {
            None
        } else {
            Some(
                self.session_state
                    .selected()
                    .unwrap_or(0)
                    .min(self.visible_sessions.len().saturating_sub(1)),
            )
        };
        self.session_state.select(next_session);
    }

    fn reselect_project(&mut self, repo_root: Option<&Path>) {
        let selected = repo_root
            .map(normalize_path)
            .and_then(|needle| {
                self.projects
                    .iter()
                    .position(|project| normalize_path(&project.repo_root) == needle)
            })
            .unwrap_or(0);

        if self.projects.is_empty() {
            self.project_state.select(None);
        } else {
            self.project_state.select(Some(selected));
        }
    }

    fn selected_project(&self) -> Option<&ProjectSummary> {
        self.project_state
            .selected()
            .and_then(|index| self.projects.get(index))
    }

    fn selected_session(&self) -> Option<&SessionSummary> {
        self.session_state
            .selected()
            .and_then(|index| self.visible_sessions.get(index))
    }

    fn render(&mut self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        let background = Block::default().style(Style::default().bg(BG));
        frame.render_widget(background, area);

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(18),
                Constraint::Length(3),
            ])
            .split(area);

        self.render_header(frame, layout[0]);
        self.render_body(frame, layout[1]);
        self.render_footer(frame, layout[2]);

        if self.input_mode == InputMode::Search {
            self.render_search_overlay(frame, centered_rect(76, 7, area));
        }
    }

    fn render_header(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let header = Block::default()
            .title(Line::from(vec![
                Span::styled(
                    " CCX ",
                    Style::default()
                        .fg(BG)
                        .bg(BRASS)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " Continuity Board",
                    Style::default().fg(INK).add_modifier(Modifier::BOLD),
                ),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BRASS))
            .style(Style::default().bg(PANEL));
        frame.render_widget(header, area);

        let inner = inner(area, 1);
        let columns = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(46),
                Constraint::Percentage(27),
                Constraint::Percentage(27),
            ])
            .split(inner);

        let title = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Project-aware memory over your Codex archive",
                Style::default().fg(INK).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Navigate projects, inspect sessions, and recover next-step context without leaving the terminal.",
                Style::default().fg(MUTED),
            )),
        ]))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(PANEL));
        frame.render_widget(title, columns[0]);

        let source_text = match self.source {
            SessionSource::Cache => "Hot path: reading from cache",
            SessionSource::Scan => "Cold path: reading directly from archive",
        };
        let stats = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "PROJECTS ",
                    Style::default().fg(BRASS).add_modifier(Modifier::BOLD),
                ),
                Span::styled(self.projects.len().to_string(), Style::default().fg(INK)),
            ]),
            Line::from(vec![
                Span::styled(
                    "SESSIONS ",
                    Style::default().fg(OLIVE).add_modifier(Modifier::BOLD),
                ),
                Span::styled(self.sessions.len().to_string(), Style::default().fg(INK)),
            ]),
            Line::from(Span::styled(source_text, Style::default().fg(MUTED))),
        ]))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(PANEL));
        frame.render_widget(stats, columns[1]);

        let focus_label = match self.focus {
            FocusPane::Projects => "Projects",
            FocusPane::Sessions => "Sessions",
        };
        let mode_label = match self.input_mode {
            InputMode::Normal => "Normal",
            InputMode::Search => "Search",
        };
        let status = Paragraph::new(Text::from(vec![
            Line::from(vec![
                Span::styled(
                    "FOCUS ",
                    Style::default().fg(SKY).add_modifier(Modifier::BOLD),
                ),
                Span::styled(focus_label, Style::default().fg(INK)),
            ]),
            Line::from(vec![
                Span::styled(
                    "MODE  ",
                    Style::default().fg(RUST).add_modifier(Modifier::BOLD),
                ),
                Span::styled(mode_label, Style::default().fg(INK)),
            ]),
            Line::from(Span::styled(
                if self.search_query.is_empty() {
                    "No active filter".to_owned()
                } else {
                    format!("Filter: {}", self.search_query)
                },
                Style::default().fg(MUTED),
            )),
        ]))
        .wrap(Wrap { trim: true })
        .style(Style::default().bg(PANEL));
        frame.render_widget(status, columns[2]);
    }

    fn render_body(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let sections = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
            .split(area);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(44), Constraint::Percentage(56)])
            .split(sections[0]);

        self.render_projects(frame, left[0]);
        self.render_sessions(frame, left[1]);
        self.render_detail(frame, sections[1]);
    }

    fn render_projects(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let border = if self.focus == FocusPane::Projects {
            BRASS
        } else {
            Color::Rgb(83, 92, 99)
        };
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(
                    " Projects ",
                    Style::default()
                        .fg(BG)
                        .bg(BRASS)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(" repo clusters ", Style::default().fg(MUTED)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(PANEL_ALT));

        let items = self
            .projects
            .iter()
            .map(|project| {
                let repo_label = shorten_path(&project.repo_root, 36);
                let goal = project
                    .latest_goal
                    .as_deref()
                    .map(|text| scanner::limit_text(text, 60))
                    .unwrap_or_else(|| "no meaningful user goal extracted".to_owned());
                ListItem::new(vec![
                    Line::from(vec![
                        Span::styled(
                            repo_label,
                            Style::default().fg(INK).add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("  [{} sessions]", project.session_count),
                            Style::default().fg(OLIVE),
                        ),
                    ]),
                    Line::from(Span::styled(goal, Style::default().fg(MUTED))),
                ])
            })
            .collect::<Vec<_>>();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(44, 52, 58))
                    .fg(INK)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▶ ");
        frame.render_stateful_widget(list, area, &mut self.project_state);
    }

    fn render_sessions(&mut self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let border = if self.focus == FocusPane::Sessions {
            OLIVE
        } else {
            Color::Rgb(83, 92, 99)
        };
        let subtitle = if self.search_query.is_empty() {
            " recent sessions "
        } else {
            " filtered sessions "
        };
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(
                    " Sessions ",
                    Style::default()
                        .fg(BG)
                        .bg(OLIVE)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(subtitle, Style::default().fg(MUTED)),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border))
            .style(Style::default().bg(PANEL));

        let items = if self.visible_sessions.is_empty() {
            vec![ListItem::new(Line::from(Span::styled(
                "No sessions for this selection.",
                Style::default().fg(MUTED),
            )))]
        } else {
            self.visible_sessions
                .iter()
                .map(|session| {
                    let goal = session
                        .first_user_goal
                        .as_deref()
                        .map(|text| scanner::limit_text(text, 72))
                        .unwrap_or_else(|| "no meaningful user goal extracted".to_owned());
                    ListItem::new(vec![
                        Line::from(vec![
                            Span::styled(
                                &session.id,
                                Style::default().fg(INK).add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(
                                format!("  {}", scanner::limit_text(&session.started_at, 20)),
                                Style::default().fg(SKY),
                            ),
                        ]),
                        Line::from(Span::styled(goal, Style::default().fg(MUTED))),
                    ])
                })
                .collect()
        };

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::Rgb(42, 50, 56))
                    .fg(INK)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("▸ ");
        frame.render_stateful_widget(list, area, &mut self.session_state);
    }

    fn render_detail(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let block = Block::default()
            .title(Line::from(vec![
                Span::styled(
                    " Selected Context ",
                    Style::default().fg(BG).bg(SKY).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " what happened, where, and what next ",
                    Style::default().fg(MUTED),
                ),
            ]))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(SKY))
            .style(Style::default().bg(PANEL_ALT));
        frame.render_widget(block, area);

        let inner = inner(area, 1);
        if inner.height < 4 || inner.width < 20 {
            return;
        }

        let sections = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),
                Constraint::Length(6),
                Constraint::Min(7),
                Constraint::Length(6),
            ])
            .split(inner);

        frame.render_widget(
            detail_paragraph(self.detail_header_text(), "Snapshot"),
            sections[0],
        );
        frame.render_widget(
            detail_paragraph(self.detail_goal_text(), "Goal"),
            sections[1],
        );
        frame.render_widget(
            detail_paragraph(self.detail_outcome_text(), "Outcome + Files"),
            sections[2],
        );
        frame.render_widget(
            detail_paragraph(self.detail_next_text(), "What To Do Next"),
            sections[3],
        );
    }

    fn detail_header_text(&self) -> Text<'static> {
        let Some(project) = self.selected_project() else {
            return Text::from("No project selected.");
        };

        let selected_session = self.selected_session();
        let session_id = selected_session
            .map(|session| session.id.clone())
            .unwrap_or_else(|| "none".to_owned());
        let started_at = selected_session
            .map(|session| session.started_at.clone())
            .unwrap_or_else(|| "no session selected".to_owned());

        Text::from(vec![
            Line::from(vec![
                Span::styled(
                    shorten_path(&project.repo_root, 68),
                    Style::default().fg(INK).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("  [{} sessions]", project.session_count),
                    Style::default().fg(OLIVE),
                ),
            ]),
            Line::from(vec![
                Span::styled("Selected session: ", Style::default().fg(BRASS)),
                Span::styled(session_id, Style::default().fg(INK)),
            ]),
            Line::from(vec![
                Span::styled("Started at: ", Style::default().fg(BRASS)),
                Span::styled(started_at, Style::default().fg(MUTED)),
            ]),
        ])
    }

    fn detail_goal_text(&self) -> Text<'static> {
        let Some(session) = self.selected_session() else {
            return Text::from("Choose a session to inspect its goal.");
        };

        Text::from(vec![
            Line::from(Span::styled(
                excerpt_or_default(
                    session.first_user_goal.as_deref(),
                    260,
                    "No meaningful user goal extracted.",
                ),
                Style::default().fg(INK),
            )),
            Line::from(vec![
                Span::styled("Workspace: ", Style::default().fg(BRASS)),
                Span::styled(
                    shorten_path(&session.repo_root, 72),
                    Style::default().fg(MUTED),
                ),
            ]),
        ])
    }

    fn detail_outcome_text(&self) -> Text<'static> {
        let Some(session) = self.selected_session() else {
            return Text::from("Select a session to see the outcome and touched files.");
        };

        let mut lines = vec![
            Line::from(Span::styled(
                excerpt_or_default(
                    session.last_assistant_outcome.as_deref(),
                    320,
                    "No meaningful assistant outcome extracted.",
                ),
                Style::default().fg(INK),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Files that mattered",
                Style::default().fg(OLIVE).add_modifier(Modifier::BOLD),
            )),
        ];

        let files = prioritized_files_for_session(session);
        if files.is_empty() {
            lines.push(Line::from(Span::styled(
                "No repo-relevant files extracted.",
                Style::default().fg(MUTED),
            )));
        } else {
            for file in files.into_iter().take(6) {
                lines.push(Line::from(vec![
                    Span::styled("• ", Style::default().fg(BRASS)),
                    Span::styled(shorten_text(&file, 92), Style::default().fg(MUTED)),
                ]));
            }
        }

        Text::from(lines)
    }

    fn detail_next_text(&self) -> Text<'static> {
        let Some(project) = self.selected_project() else {
            return Text::from("Pick a project to see the next-step hint.");
        };

        let related = self
            .sessions
            .iter()
            .filter(|session| {
                normalize_path(&session.attributed_repo_root) == normalize_path(&project.repo_root)
            })
            .take(3)
            .collect::<Vec<_>>();

        let latest = related.first().copied();
        let anchor = related
            .iter()
            .max_by_key(|session| {
                session
                    .last_assistant_outcome
                    .as_deref()
                    .map(|text| text.len())
                    .unwrap_or(0)
                    + session.mentioned_files.len() * 4
            })
            .copied();

        let Some(latest) = latest else {
            return Text::from("No sessions available for this project.");
        };
        let anchor = anchor.unwrap_or(latest);

        Text::from(vec![
            Line::from(Span::styled(
                "This project already has enough continuity to resume immediately.",
                Style::default().fg(INK).add_modifier(Modifier::BOLD),
            )),
            Line::from(vec![
                Span::styled("Latest checkpoint: ", Style::default().fg(BRASS)),
                Span::styled(latest.id.clone(), Style::default().fg(INK)),
            ]),
            Line::from(vec![
                Span::styled("Richest anchor: ", Style::default().fg(BRASS)),
                Span::styled(anchor.id.clone(), Style::default().fg(INK)),
            ]),
            Line::from(vec![
                Span::styled("Suggested move: ", Style::default().fg(BRASS)),
                Span::styled(
                    format!(
                        "use `ccx pack --repo {}` before opening the next Codex chat.",
                        project.repo_root.display()
                    ),
                    Style::default().fg(MUTED),
                ),
            ]),
        ])
    }

    fn render_footer(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        let footer = Paragraph::new(Text::from(vec![
            Line::from(vec![
                footer_key("Tab", "switch pane"),
                footer_sep(),
                footer_key("/", "search"),
                footer_sep(),
                footer_key("Esc", "clear search"),
                footer_sep(),
                footer_key("i", "reindex"),
                footer_sep(),
                footer_key("q", "quit"),
            ]),
            Line::from(Span::styled(
                shorten_text(&self.status, area.width.saturating_sub(4) as usize),
                Style::default().fg(MUTED),
            )),
        ]))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(72, 81, 88)))
                .style(Style::default().bg(PANEL)),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(footer, area);
    }

    fn render_search_overlay(&self, frame: &mut ratatui::Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);
        let overlay = Paragraph::new(Text::from(vec![
            Line::from(Span::styled(
                "Filter sessions inside the selected project",
                Style::default().fg(INK).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    "Query: ",
                    Style::default().fg(BRASS).add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    if self.search_query.is_empty() {
                        "type here...".to_owned()
                    } else {
                        self.search_query.clone()
                    },
                    Style::default().fg(INK),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Enter applies the filter. Esc closes search mode.",
                Style::default().fg(MUTED),
            )),
        ]))
        .block(
            Block::default()
                .title(Line::from(vec![
                    Span::styled(
                        " Search ",
                        Style::default()
                            .fg(BG)
                            .bg(RUST)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(" live continuity filter ", Style::default().fg(MUTED)),
                ]))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(RUST))
                .style(Style::default().bg(PANEL_ALT)),
        )
        .wrap(Wrap { trim: true });
        frame.render_widget(overlay, area);
    }
}

fn move_index(current: Option<usize>, len: usize, delta: isize) -> Option<usize> {
    if len == 0 {
        return None;
    }

    let current = current.unwrap_or(0) as isize;
    let next = (current + delta).clamp(0, len as isize - 1) as usize;
    Some(next)
}

fn detail_paragraph(text: Text<'static>, title: &str) -> Paragraph<'static> {
    Paragraph::new(text)
        .block(
            Block::default()
                .title(Line::from(Span::styled(
                    format!(" {title} "),
                    Style::default().fg(BRASS).add_modifier(Modifier::BOLD),
                )))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(70, 79, 86)))
                .style(Style::default().bg(PANEL)),
        )
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(INK))
}

fn centered_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(area.height.saturating_sub(height) / 2),
            Constraint::Length(height),
            Constraint::Min(0),
        ])
        .split(area);
    let width = area.width.saturating_mul(percent_x).saturating_div(100);
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(area.width.saturating_sub(width) / 2),
            Constraint::Length(width.max(20)),
            Constraint::Min(0),
        ])
        .split(vertical[1]);
    horizontal[1]
}

fn footer_key(key: &str, label: &str) -> Span<'static> {
    Span::styled(
        format!("[{key}] {label}"),
        Style::default().fg(INK).add_modifier(Modifier::BOLD),
    )
}

fn footer_sep() -> Span<'static> {
    Span::styled("  •  ", Style::default().fg(MUTED))
}

fn inner(area: Rect, margin: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(margin),
        y: area.y.saturating_add(margin),
        width: area.width.saturating_sub(margin.saturating_mul(2)),
        height: area.height.saturating_sub(margin.saturating_mul(2)),
    }
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}

fn excerpt_or_default(text: Option<&str>, max_len: usize, fallback: &str) -> String {
    text.map(|value| scanner::limit_text(value, max_len))
        .unwrap_or_else(|| fallback.to_owned())
}

fn shorten_path(path: &Path, max_len: usize) -> String {
    shorten_text(&path.display().to_string(), max_len)
}

fn shorten_text(text: &str, max_len: usize) -> String {
    scanner::limit_text(text, max_len)
}

fn prioritized_files_for_session(session: &SessionSummary) -> Vec<String> {
    let attributed_root = normalize_path(&session.attributed_repo_root);
    let workspace_root = normalize_path(&session.repo_root);
    let mut values = session
        .mentioned_files
        .iter()
        .filter(|value| is_repo_file_candidate(value, &attributed_root, &workspace_root))
        .cloned()
        .collect::<Vec<_>>();

    values.sort_by_key(|value| file_priority(value));
    values.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    values
}

fn is_repo_file_candidate(value: &str, attributed_root: &str, workspace_root: &str) -> bool {
    let normalized = value.replace('\\', "/").to_lowercase();

    if normalized.contains("/.agents/skills/")
        || normalized.contains("/.codex/skills/")
        || normalized.contains("/.codex/memories/")
    {
        return false;
    }

    if normalized.contains(attributed_root) || normalized.contains(workspace_root) {
        return true;
    }

    normalized.starts_with("./")
        || normalized.starts_with("src/")
        || normalized.starts_with("docs/")
        || normalized.starts_with("backend/")
        || normalized.starts_with("frontend/")
        || normalized.starts_with("scripts/")
        || normalized.starts_with("app/")
        || matches!(
            normalized.as_str(),
            "readme.md" | "agents.md" | "cargo.toml" | "cargo.lock" | "continuity.md"
        )
}

fn file_priority(value: &str) -> (usize, String) {
    let lower = value.replace('\\', "/").to_lowercase();
    let bucket =
        if lower.contains("/backend/") || lower.contains("/frontend/") || lower.contains("/src/") {
            0
        } else if lower.ends_with("/state_of_play.md")
            || lower.ends_with("/prompt_profiles.md")
            || lower.ends_with("/agents.md")
            || lower.ends_with("/architecture.md")
            || lower.ends_with("/continuity.md")
        {
            1
        } else if lower.contains("/docs/") {
            2
        } else if lower.contains("/.agent/compare/") {
            3
        } else if lower.contains("/.agent/e2e/") {
            4
        } else if lower.contains("/.agent/history/") {
            6
        } else if lower.starts_with("./scripts/") {
            7
        } else {
            5
        };

    (bucket, lower)
}
