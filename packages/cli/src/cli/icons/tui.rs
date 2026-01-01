use crossterm::{event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind}, execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode}};
use ratatui::{prelude::*, widgets::{Block, Borders, Paragraph}};
use crate::Result;
use std::time::Duration;

pub(crate) struct App {
    pub(crate) search: Search,
    pub(crate) screen: Screen,
    pub(crate) focus: Focus,
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
                        MouseEventKind::ScrollUp => {}, // TODO: handle mouse scroll
                        MouseEventKind::ScrollDown => {}, // TODO: handle mouse scroll
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
    let style = if app.focus == Focus::Content {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Select Icon Collection ")
        .style(style);

    let inner = block.inner(area);

    // number of columns to display per row, taking into account the width of the viewport
    let num_cols = (inner.width / COLLECTION_TILE_WIDTH).max(1) as usize;
    // number of rows to display, taking into account the height of the viewport
    let num_rows = (inner.height / COLLECTION_TILE_HEIGHT) as usize;

    // TODO: 
    // implement scroll
    // calculate the start_row from scroll offset
    // calculate total number of required rows 
    // calculate the end_row

    // start_row..end_row are the visible rows to render to the viewport
    // calculate the starting index from current row and current column to render the visible tiles

    // if possible, use the built-in Layout engine to create the grid with constraints

    f.render_widget(block, area);
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