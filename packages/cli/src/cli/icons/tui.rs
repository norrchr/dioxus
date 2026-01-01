use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{prelude::*, widgets::{Block, BorderType, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState}};
use crate::Result;
use std::time::Duration;

pub(crate) struct App {
    pub(crate) search: Search,
    pub(crate) screen: Screen,
    pub(crate) focus: Focus,
    pub(crate) collection_state: State,
}

const COLLECTION_TILE_WIDTH: u16 = 30;
const COLLECTION_TILE_HEIGHT: u16 = 6;

const ICON_TILE_WIDTH: u16 = 8;
const ICON_TILE_HEIGHT: u16 = 4;

impl App {
    fn new() -> Self {
        Self {
            search: Search::default(),
            screen: Screen::default(),
            focus: Focus::default(),
            collection_state: State::default(),
        }
    }
}

/// Search state
#[derive(Default, PartialEq)]
pub(crate) struct Search {
    /// The area to display the search bar
    area: Rect,
    /// The current search query
    query: String,
}

#[derive(Default, PartialEq)]
pub(crate) struct State {
    selected_idx: Option<usize>,
    total_rows: usize,
    scroll_offset: usize,
    scroll: ScrollbarState,
}

/// The screen to display
#[derive(Default, PartialEq, Eq)]
pub(crate) enum Screen {
    #[default]
    /// Collection selection screen
    Collection,
    /// Icon browser screen
    IconBrowser,
}

/// Tab to focus area
#[derive(Default, PartialEq, Eq)]
pub(crate) enum Focus {
    Search,
    Sidebar,
    #[default]
    Content,
}

pub(crate) fn tui_main() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let run_result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    run_result
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> crate::Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;

        if event::poll(Duration::from_millis(50))? {
            let event = event::read()?;

            if event.is_mouse() {
                if let Event::Mouse(mouse) = event {
                    match mouse.kind {
                        MouseEventKind::Down(_) => {}, // TODO: handle mouse click
                        MouseEventKind::ScrollDown => {
                            match app.focus {
                                Focus::Content | Focus::Sidebar => match app.screen {
                                    Screen::Collection => {
                                        app.collection_state.scroll_offset = app.collection_state.scroll_offset.saturating_add(1).min(app.collection_state.total_rows);
                                        app.collection_state.scroll.next();
                                    },
                                    _ => {},
                                },
                                _ => {},
                            }
                        },
                        MouseEventKind::ScrollUp => {
                            match app.focus {
                                Focus::Content | Focus::Sidebar => {
                                    match app.screen {
                                        Screen::Collection => {
                                            app.collection_state.scroll_offset = app.collection_state.scroll_offset.saturating_sub(1);
                                            app.collection_state.scroll.prev();
                                        },
                                        _ => {},
                                    }
                                }
                                _ => {},
                            }
                        },
                        _ => {},
                    }
                }
            } else if event.is_resize() {
                // TODO: handle resize event(s)
                // recalculate the number of columns and rows to fit the terminal
            } else if let Some(event) = event.as_key_event() {

                // TODO: implement debounce for key events (either globally or per-event type)
           
                match app.focus {
                    Focus::Search => {
                        match event.code {
                            // `/` toggles search focus, but we are already focused -- ignore
                            KeyCode::Char('/') => {},
                            KeyCode::Esc => {
                                if app.search.query.is_empty() {
                                    app.focus = Focus::Content;
                                } else {
                                    app.search.query.clear();
                                }
                            },
                            KeyCode::Tab => {
                                match app.screen {
                                    Screen::Collection => app.focus = Focus::Content,
                                    Screen::IconBrowser => app.focus = Focus::Sidebar, 
                                }
                            },
                            KeyCode::Backspace => {
                                app.search.query.pop();
                            }
                            KeyCode::Enter => {
                                // update the filtered cells
                                app.focus = Focus::Content;
                            },
                            KeyCode::Char(c) => {
                                // sanitize the input 
                                app.search.query.push(c);
                            },
                            
                            _ => {},
                        }
                    },
                    Focus::Content | Focus::Sidebar => {
                        match event.code {
                            KeyCode::Char('q') => { break }, // exit 
                            KeyCode::Char('/') => app.focus = Focus::Search, // set focus to search
                            KeyCode::Tab => {
                                // cycle focus depending on the current screen and focus
                                match app.screen {
                                    // collection has no sidebar to focus
                                    Screen::Collection => {
                                        match app.focus {
                                            Focus::Search => app.focus = Focus::Content,
                                            Focus::Content => app.focus = Focus::Search,
                                            _ => {},
                                        }
                                    },
                                    Screen::IconBrowser => {
                                        match app.focus {
                                            Focus::Search => app.focus = Focus::Sidebar,
                                            Focus::Sidebar => app.focus = Focus::Content,
                                            Focus::Content => app.focus = Focus::Search,
                                        }
                                    }
                                }
                            },

                            // TODO: implement cell selection logic using arrow keys and kjhl
                            KeyCode::Up | KeyCode::Char('k') => {},
                            KeyCode::Down | KeyCode::Char('j') => {},
                            KeyCode::Left | KeyCode::Char('h') => {},
                            KeyCode::Right | KeyCode::Char('l') => {},
                            KeyCode::PageUp => {},
                            KeyCode::PageDown => {},
                            KeyCode::Enter => {}, // Behaviour dependant on screen and focus
                            KeyCode::Esc | KeyCode::Backspace => {}, // Behaviour dependant on screen and focus

                            // For testing/debug purposes
                            KeyCode::Char('`') => {
                                if event.is_repeat() { continue; };
                                match app.screen {
                                    Screen::Collection => app.screen = Screen::IconBrowser,
                                    Screen::IconBrowser => app.screen = Screen::Collection,
                                }
                            },
                            
                            _ => {},
                        }
                    },
                }
                
            }
        }
    }

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let area = f.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Search
            Constraint::Min(0),      // Content
            Constraint::Length(3),   // Help
        ])
        .split(area);

    let search_area = chunks[0];
    let content_area = chunks[1];
    let help_area = chunks[2];

    render_search(f, search_area, app);

    match app.screen {
        Screen::Collection => render_collection_screen(f, content_area, app),
        Screen::IconBrowser => render_icon_screen(f, content_area, app),
    }

    render_controls(f, help_area, app);
}

fn render_search(f: &mut Frame, area: Rect, app: &mut App) {
    app.search.area = area;

    let style = if app.focus == Focus::Search {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Search Icons (/) ")
        .style(style);
    
    let inner = block.inner(area);
    f.render_widget(block, area);


    
    let input = if app.search.query.is_empty() && app.focus != Focus::Search {
        "Type to search..."
    } else {
        &app.search.query
    };
    
    f.render_widget(Paragraph::new(input), inner);
}

fn render_collection_screen(f: &mut Frame, area: Rect, app: &mut App) {
    /*let style = if app.focus == Focus::Content {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };*/

    let border_type = if app.focus == Focus::Content {
        BorderType::Double
    } else {
        BorderType::default()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(border_type)
        .title(" Select Icon Collection ")
        .style(Style::default().fg(Color::DarkGray));

    f.render_widget(block, area);

    let grid_area = area.inner(Margin { horizontal: 1, vertical: 1 });

    // for testing/debugging
    let mut collections: Vec<String> = Vec::new();
    for i in 0..20 {
        collections.push(format!("Collection {}", i+1));
    }

    // number of columns to display per row, taking into account the width of the viewport
    let cols_per_row = (grid_area.width / COLLECTION_TILE_WIDTH).max(1) as usize;
    // number of rows to display, taking into account the height of the viewport
    let rows_in_viewport = (grid_area.height / COLLECTION_TILE_HEIGHT) as usize;

    //let start_row = app.collection_state.scroll_offset / cols_per_row;
    let start_row = app.collection_state.scroll_offset;
    let total_rows = collections.len().div_ceil(cols_per_row);
    let end_row = (start_row + rows_in_viewport + 1).min(total_rows);

    let mut row_constraints = Vec::with_capacity(rows_in_viewport);
    for _ in 0..rows_in_viewport {
        row_constraints.push(Constraint::Length(COLLECTION_TILE_HEIGHT));
    }

    let rows = Layout::default()
        .direction(Direction::Vertical)
        .constraints(row_constraints)
        .split(grid_area);

    let selected_idx = app.collection_state.selected_idx;

    for (row_idx, row_rect) in rows.into_iter().enumerate() {
        let row = start_row + row_idx;

        let mut col_constraints = Vec::with_capacity(cols_per_row);
        for _ in 0..cols_per_row {
            col_constraints.push(Constraint::Length(COLLECTION_TILE_WIDTH));
        }

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(col_constraints)
            .split(*row_rect);

        for (col_idx, col_rect) in cols.into_iter().enumerate() {
            let index = row * cols_per_row + col_idx;

            if index >= collections.len() {
                break;
            }

            let label = &collections[index];

            let (border_type, border_style) = if (selected_idx.is_none() && index == 0) || selected_idx == Some(index) {
                (BorderType::Double, Style::default().fg(Color::Yellow))
            } else {
                (BorderType::default(), Style::default().fg(Color::Gray))
            };

            f.render_widget(
                Block::default()
                    .title(label.clone())
                    .title_style(Style::default().red())
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(border_type),
                *col_rect,
            );
        }
    }

    app.collection_state.total_rows = total_rows;
    app.collection_state.scroll = app.collection_state.scroll.content_length(total_rows).viewport_content_length(rows_in_viewport).position(start_row);

    //f.render_widget(block, area);

    f.render_stateful_widget(
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓")),
            area.inner(Margin {
                vertical: 1,
                horizontal: 0,
            }
        ),
        &mut app.collection_state.scroll,
    );
}

fn render_icon_screen(f: &mut Frame, area: Rect, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(20), // Sidebar
            Constraint::Min(0),     // Grid
        ])
        .split(area);

    let sidebar_area = chunks[0];
    let grid_area = chunks[1];

    render_sidebar(f, sidebar_area, app);

    let style = if app.focus == Focus::Content {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Icon Browser ")
        .style(style);

    f.render_widget(block, grid_area);
}

fn render_sidebar(f: &mut Frame, area: Rect, app: &mut App) {
    let title = if !app.search.query.is_empty() {
        " Collections "
    } else {
        " Categories "
    };

    let style = if app.focus == Focus::Sidebar {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .style(style);

    f.render_widget(block, area);
}

fn render_controls(f: &mut Frame, area: Rect, app: &App) {   
    let help = Paragraph::new(Line::from(vec![
        Span::styled("Arrows/hjkl", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
        Span::raw(" - Navigate  |  "),
        Span::styled("/", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw(" - Search  |  "),
        Span::styled("Esc/q", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
        Span::raw(" - Back  |  "),
    ]))
    .block(Block::default().borders(Borders::ALL).title(" Controls "));
    
    f.render_widget(help, area);
}