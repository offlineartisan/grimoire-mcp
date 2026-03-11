use std::path::Path;

use crossterm::event::KeyCode;
use grimoire_mcp::{Pattern, load_all_patterns};
use ratatui::Frame;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, List, ListItem, ListState, Paragraph};
use tui_big_text::{BigText, PixelSize};

enum InputMode {
    Normal,
    Searching,
}

struct App {
    patterns: Vec<Pattern>,
    filtered_indices: Vec<usize>,
    list_state: ListState,
    mode: InputMode,
    search_query: String,
    detail_scroll: u16,
    running: bool,
    tick: u64,
}

impl App {
    fn new(patterns: Vec<Pattern>) -> Self {
        let filtered_indices: Vec<usize> = (0..patterns.len()).collect();
        Self {
            patterns,
            filtered_indices,
            list_state: ListState::default().with_selected(Some(0)),
            mode: InputMode::Normal,
            search_query: String::new(),
            detail_scroll: 0,
            running: true,
            tick: 0,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match self.mode {
            InputMode::Normal => match code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('j') | KeyCode::Down => {
                    self.detail_scroll = 0;
                    self.list_state.select_next();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.detail_scroll = 0;
                    self.list_state.select_previous();
                }
                KeyCode::Char('/') => self.mode = InputMode::Searching,
                KeyCode::PageDown => {
                    self.detail_scroll = self.detail_scroll.saturating_add(1);
                }
                KeyCode::PageUp => {
                    self.detail_scroll = self.detail_scroll.saturating_sub(1);
                }
                _ => {}
            },
            InputMode::Searching => match code {
                KeyCode::Esc => {
                    self.search_query.clear();
                    self.recompute_filter();
                    self.mode = InputMode::Normal;
                }
                KeyCode::Enter => {
                    self.mode = InputMode::Normal;
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                    self.recompute_filter();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                    self.recompute_filter();
                }
                _ => {}
            },
        }
    }

    fn recompute_filter(&mut self) {
        if self.search_query.is_empty() {
            self.filtered_indices = (0..self.patterns.len()).collect();
        } else {
            let query = self.search_query.to_lowercase();
            self.filtered_indices = self
                .patterns
                .iter()
                .enumerate()
                .filter(|(_, p)| {
                    p.metadata.pattern.to_lowercase().contains(&query)
                        || p.metadata.category.to_lowercase().contains(&query)
                        || p.metadata
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&query))
                        || p.content.to_lowercase().contains(&query)
                })
                .map(|(i, _)| i)
                .collect();
        }

        // Reset selection after filter change
        if self.filtered_indices.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
        self.detail_scroll = 0;
    }

    fn run(&mut self, terminal: &mut ratatui::DefaultTerminal) {
        let (tx, rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            loop {
                if crossterm::event::poll(std::time::Duration::from_millis(100)).unwrap_or(false) {
                    if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                        if tx.send(key_event).is_err() {
                            break; // receiver dropped, exit thread
                        }
                    }
                }
            }
        });

        while self.running {
            terminal.draw(|frame| draw(frame, self)).expect("failed to draw");

            if let Ok(key_event) = rx.recv_timeout(std::time::Duration::from_millis(50)) {
                if key_event.kind == crossterm::event::KeyEventKind::Press {
                    self.handle_key(key_event.code);
                }
            }

            self.tick += 1;
        }
    }
}

fn draw(frame: &mut Frame, app: &mut App) {
    let core_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(2)])
        .split(frame.area());

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(0)])
        .split(core_area[0]);

    let main_segment = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(0)])
        .split(chunks[1]);

    draw_title(frame, chunks[0]);
    draw_nav(frame, main_segment[0], app);
    draw_content(frame, main_segment[1], app);
    draw_status(frame, core_area[1], app);
}

fn draw_title(frame: &mut Frame, area: Rect) {
    let big_title = BigText::builder()
        .pixel_size(PixelSize::HalfHeight)
        .style(Style::new().blue())
        .lines(vec!["Grimoire".blue().into()])
        .alignment(Alignment::Center)
        .build();
    frame.render_widget(big_title, area);
}

fn draw_nav(frame: &mut Frame, area: Rect, app: &mut App) {
    let list_items: Vec<ListItem> = app
        .filtered_indices
        .iter()
        .map(|&i| ListItem::new(app.patterns[i].metadata.pattern.as_str()))
        .collect();

    let nav = List::new(list_items)
        .block(
            Block::bordered()
                .title("Patterns")
                .title_style(Color::Yellow),
        )
        .highlight_symbol(">>")
        .highlight_style(Style::new().bg(Color::DarkGray).bold());

    frame.render_stateful_widget(nav, area, &mut app.list_state);
}

fn draw_content(frame: &mut Frame, area: Rect, app: &App) {
    if app.filtered_indices.is_empty() {
        let empty = Paragraph::new("No patterns found. Add .md files with YAML frontmatter.")
            .block(Block::bordered().title("Content").title_style(Color::Cyan));
        frame.render_widget(empty, area);
        return;
    }

    let selected = app.list_state.selected().unwrap_or(0);
    let pattern_idx = app.filtered_indices[selected];
    let pattern = &app.patterns[pattern_idx];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Pattern: ", Style::new().yellow()),
        Span::styled(&pattern.metadata.pattern, Style::new().cyan().bold()),
    ]));
    lines.push(Line::from(vec![
        Span::styled("Category: ", Style::new().yellow()),
        Span::raw(&pattern.metadata.category),
    ]));
    if let Some(ref fw) = pattern.metadata.framework {
        lines.push(Line::from(vec![
            Span::styled("Framework: ", Style::new().yellow()),
            Span::raw(fw),
        ]));
    }
    if !pattern.metadata.projects.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Projects: ", Style::new().yellow()),
            Span::raw(pattern.metadata.projects.join(", ")),
        ]));
    }
    if !pattern.metadata.tags.is_empty() {
        lines.push(Line::from(vec![
            Span::styled("Tags: ", Style::new().yellow()),
            Span::styled(pattern.metadata.tags.join(", "), Style::new().light_green()),
        ]));
    }
    lines.push(Line::from(""));
    for line in pattern.content.lines() {
        lines.push(Line::from(line.to_string()));
    }

    let content = Paragraph::new(lines)
        .block(
            Block::bordered()
                .title(pattern.metadata.pattern.as_str())
                .title_style(Color::Cyan),
        )
        .scroll((app.detail_scroll, 0));

    frame.render_widget(content, area);
}

fn draw_status(frame: &mut Frame, area: Rect, app: &App) {
    let status = match app.mode {
        InputMode::Normal => {
            Paragraph::new("q: quit | j/k: navigate | /: search | PgUp/PgDn: scroll")
                .alignment(Alignment::Center)
        }
        InputMode::Searching => {
            Paragraph::new(Line::from(vec![
                Span::styled("Search: ", Style::new().yellow()),
                Span::raw(&app.search_query),
                Span::raw("▌"),
            ]))
        }
    };
    frame.render_widget(status, area);
}

fn main() -> Result<(), anyhow::Error> {
    // Validate PATTERNS_DIR before initializing the terminal
    let patterns_dir = match std::env::var("PATTERNS_DIR") {
        Ok(dir) => dir,
        Err(_) => {
            eprintln!("Error: PATTERNS_DIR environment variable is not set.");
            eprintln!("Set it to the directory containing your pattern .md files.");
            std::process::exit(1);
        }
    };

    if !Path::new(&patterns_dir).is_dir() {
        eprintln!(
            "Error: PATTERNS_DIR '{}' does not exist or is not a directory.",
            patterns_dir
        );
        std::process::exit(1);
    }

    let patterns = load_all_patterns();

    let mut terminal = ratatui::init();
    let mut app = App::new(patterns);
    app.run(&mut terminal);
    ratatui::restore();

    Ok(())
}
