//! # [Rustytop] A rust based top

use std::{any::type_name_of_val, env::consts, error::Error, io};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        event::{
            self, poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Constraint, Layout, Margin, Rect},
    style::{self, Color, Modifier, Style, Stylize},
    terminal::{Frame, Terminal},
    text::Line,
    widgets::{
        Block, BorderType, Cell, HighlightSpacing, Paragraph, Row, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Table, TableState,
    },
};
use style::palette::tailwind;
use users::{get_current_uid, get_user_by_uid};

const PALETTES: [tailwind::Palette; 4] = [
    tailwind::BLUE,
    tailwind::EMERALD,
    tailwind::INDIGO,
    tailwind::RED,
];
const INFO_TEXT: &str = "(Esc) quit | (↑) move up | (↓) move down";

const ITEM_HEIGHT: usize = 4;

struct TableColors {
    buffer_bg: Color,
    header_bg: Color,
    header_fg: Color,
    row_fg: Color,
    selected_style_fg: Color,
    normal_row_color: Color,
    alt_row_color: Color,
    footer_border_color: Color,
}

impl TableColors {
    const fn new(color: &tailwind::Palette) -> Self {
        Self {
            buffer_bg: tailwind::SLATE.c950,
            header_bg: color.c900,
            header_fg: tailwind::SLATE.c200,
            row_fg: tailwind::SLATE.c200,
            selected_style_fg: color.c400,
            normal_row_color: tailwind::SLATE.c950,
            alt_row_color: tailwind::SLATE.c900,
            footer_border_color: color.c400,
        }
    }
}

struct ProcessMap {
    pid: u32,
    name: String,
    path: String,
    user: String,
    // memory: u64,
}

struct App {
    state: TableState,
    items: [Vec<String>; 4],
    input: String,
    longest_item_lens: (u16, u16, u16, u16), // order is (pid, name, path,, user, memory)
    scroll_state: ScrollbarState,
    colors: TableColors,
    color_index: usize,
    input_mode: InputMode,
    message: Vec<String>,
    character_index: usize,
}

impl App {
    fn new() -> Self {
        use sysinfo::{Components, Disks, Networks, System};

        // "new_all" to ensure that all list;/mof components, network interfaces,
        // disks and users are already filled!
        let mut sys = System::new_all();

        let user = get_user_by_uid(get_current_uid()).unwrap();
        // println!("Hello, {}!", user.name().to_string_lossy());

        // Update all information of our `System` struct.
        sys.refresh_all();

        // let mut user_uid: String = String::new();

        let mut table_process_map = vec![];

        for (pid, process) in sys.processes() {
            // std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            let user_uid = get_user_by_uid(
                process
                    .user_id()
                    .unwrap()
                    .to_string()
                    .parse::<u32>()
                    .unwrap(),
            )
            .unwrap()
            .name()
            .to_os_string()
            .as_os_str()
            .to_str()
            .unwrap()
            .to_string();
            table_process_map.push(ProcessMap {
                pid: pid.as_u32(),
                name: String::from(process.name()),
                path: String::from(process.exe().unwrap().to_str().unwrap()),
                user: user_uid,
                // memory: process.memory(),
            });
        }

        table_process_map.sort_by_key(|element: &ProcessMap| element.pid.clone());

        let mut pids = vec![];
        let mut names = vec![];
        let mut path = vec![];
        let mut users = vec![];

        for element in table_process_map {
            pids.push(element.pid.to_string());
            names.push(element.name);
            path.push(element.path);
            users.push(element.user);
        }

        let data_vec = [pids, names, path, users];
        Self {
            state: TableState::default().with_selected(0),
            longest_item_lens: (100, 100, 100, 100),
            scroll_state: ScrollbarState::new((data_vec[0].len() - 1) * ITEM_HEIGHT),
            colors: TableColors::new(&PALETTES[0]),
            color_index: 0,
            items: data_vec,
            input_mode: InputMode::Normal,
            input: String::new(),
            message: Vec::new(),
            character_index: 0,
        }
    }
    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items[0].len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items[0].len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
        self.scroll_state = self.scroll_state.position(i * ITEM_HEIGHT);
    }

    pub fn set_colors(&mut self) {
        self.colors = TableColors::new(&PALETTES[self.color_index]);
    }

    pub fn submit_message(&mut self) {
        self.message.push(self.input.clone());
        self.input.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.character_index = self.character_index.saturating_add(1);
    }

    pub fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn clear(&mut self) {
        self.input = String::new();
        self.message = Vec::new();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let app = App::new();
    let res = run_app(&mut terminal, app);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => {
                    if key.kind == KeyEventKind::Press {
                        match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Char('j') | KeyCode::Down => app.next(),
                            KeyCode::Char('k') | KeyCode::Up => app.previous(),
                            KeyCode::Char('f') | KeyCode::Insert => {
                                app.input_mode = InputMode::Editing
                            }
                            KeyCode::Char('c') | KeyCode::Home => app.clear(),
                            _ => {}
                        }
                    }
                }

                InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                    KeyCode::Enter => app.submit_message(),
                    KeyCode::Char(to_insert) => app.enter_char(to_insert),
                    _ => println!("Error. Enter letter or press return"),
                },
                InputMode::Editing => {}
            }
        }
    }
}

enum InputMode {
    Normal,
    Editing,
}

fn ui(f: &mut Frame, app: &mut App) {
    let vertical = &Layout::vertical([Constraint::Min(5), Constraint::Length(4)]);
    let rects = vertical.split(f.size());

    app.set_colors();

    render_table(f, app, rects[0]);

    render_scrollbar(f, app, rects[0]);

    render_footer(f, app, rects[1]);
}

fn render_table(f: &mut Frame, app: &mut App, area: Rect) {
    let header_style = Style::default()
        .fg(app.colors.header_fg)
        .bg(app.colors.header_bg);
    let selected_style = Style::default()
        .add_modifier(Modifier::REVERSED)
        .fg(app.colors.selected_style_fg);

    let header = ["PID", "PATH", "File", "User"]
        .into_iter()
        .map(Cell::from)
        .collect::<Row>()
        .style(header_style)
        .height(1);

    // Note: TableState should be stored in your application state (not constructed in your render
    // method) so that the selected row is preserved across renders
    // TODO!!!! Filter items
    let len = app.items[0].len();
    let mut rows = vec![];

    //let filter = app.message[0].clone();
    // transform array of array
    if app.message.len() > 0 {
        for n in 0..len {
            if app.items[2][n].contains(&app.message[0]) {
                rows.push(Row::new(vec![
                    app.items[0][n].clone(),
                    app.items[1][n].clone(),
                    app.items[2][n].clone(),
                    app.items[3][n].clone(),
                ]));
            }
        }
    } else {
        for n in 0..len {
            rows.push(Row::new(vec![
                app.items[0][n].clone(),
                app.items[1][n].clone(),
                app.items[2][n].clone(),
                app.items[3][n].clone(),
            ]));
        }
    }

    let widths = [
        //  _ => app.colors.alt_row_color,
        Constraint::Length(10),
        Constraint::Length(50),
        Constraint::Length(100),
        Constraint::Length(30),
    ];

    let table = Table::new(rows, widths)
        .block(Block::new().title("Processes"))
        .column_spacing(1)
        .style(Style::new().blue())
        .header(header)
        .highlight_style(Style::new().reversed());

    f.render_stateful_widget(table, area, &mut app.state);
}

fn render_scrollbar(f: &mut Frame, app: &mut App, area: Rect) {
    f.render_stateful_widget(
        Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(None)
            .end_symbol(None),
        area.inner(Margin {
            vertical: 1,
            horizontal: 1,
        }),
        &mut app.scroll_state,
    );
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    //println!("filter is: {:?}", app.message);
    let info_footer = Paragraph::new(Line::from(INFO_TEXT))
        .style(Style::new().fg(app.colors.row_fg).bg(app.colors.buffer_bg))
        .centered()
        .block(
            Block::bordered()
                .border_type(BorderType::Double)
                .border_style(Style::new().fg(app.colors.footer_border_color)),
        );
    f.render_widget(info_footer, area);
}
