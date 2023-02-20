use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use std::{
    error::Error,
    fs::File,
    io,
    io::{BufReader, Read},
    time::{Duration, Instant},
    env,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

impl<T> StatefulList<T> {
    fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn unselect(&mut self) {
        self.state.select(None);
    }
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
struct App {
    items: StatefulList<(Vec<u8>, u8)>,
    //events: Vec<(&'a str, &'a str)>,
}

impl App {
    fn new(i: Vec<(Vec<u8>, u8)>) -> App {
        App {
            items: StatefulList::with_items(i),
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {
        //let event = self.events.remove(0);
        //self.events.push(event);
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let f = File::open(&args[1])?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let formatted_data = buffer.into_iter().tuples::<(u8, u8)>().fold(
        (Vec::<(Vec<u8>, u8)>::new()),
        |mut out, (tag, byte)| match out.last().map(|f| f.1) {
            Some(last_read_tag) if last_read_tag == (tag % 2) as u8 => {
                out.last_mut().unwrap().0.push(byte);
                return out;
            }
            _ => {
                out.push((vec![byte], (tag % 2) as u8));
                return out;
            }
        },
    );

    let formatted_data = formatted_data.into_iter().map(|(a, b)| {
        let s = (a.chunks(1024).into_iter().map(|s| s.to_vec()).collect::<Vec<_>>(), b);
        s.0.into_iter().map(|v| (v, s.1)).collect_vec()
    }).collect::<Vec<Vec<(Vec<u8>, u8)>>>().into_iter().flatten().collect::<Vec<_>>();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(50);
    let app = App::new(formatted_data);
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

static mut CACHE: format_cache = format_cache {
    width: None,
    utf8: vec![],
    hex: vec![],
};


fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    //let mut cache = format_cache { width: None, utf8: vec![], hex: vec![] };
    //terminal.draw(|f|{ui(f, &mut app, &mut cache);})?;

    loop {
        //terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                let items = &mut app.items;
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => items.unselect(),
                    KeyCode::Down => items.next(),
                    KeyCode::Up => items.previous(),
                    _ => {}
                }
                terminal.draw(|f| {ui(f, &mut app);})?;

                    //terminal.draw(|f| ui(f, &mut app))?;
                //update = true;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
            //if update {
            //    terminal.draw(|f| ui(f, &mut app))?;
            //    update = false;
            //}
        }
    }
}

fn utf8_formatter(bytes: &Vec<u8>, width: u16) -> Vec<String> {
    let bytes2 = bytes.split(|&i| i == 10).map(|f| f.to_vec()).collect::<Vec<_>>();

    bytes2.into_iter().map(|f|
    f
        .chunks(width.into())
        .map(|f| String::from_utf8_lossy(f).to_string())
        .collect::<Vec<_>>()
    ).flatten().collect::<Vec<_>>()
}

fn hex_formatter(bytes: &Vec<u8>, width: u16) -> Vec<String> {
    bytes
        .chunks((width / 3 - 1).into())
        .map(|f| format!("{:02x?}", f).replace(&[',', '[', ']'][..], ""))
        .collect::<Vec<_>>()
}

struct format_cache<'a> {
    width: Option<u16>,
    utf8: Vec<ListItem<'a>>,
    hex: Vec<ListItem<'a>>,
}

impl <'a>format_cache<'a> {
    fn update(&'a mut self, width: Option<u16>, utf8: Vec<ListItem<'a>>, hex: Vec<ListItem<'a>>) {
        self.width = width;
        self.utf8 = utf8;
        self.hex = hex;
    }
}


fn ui<'a, B: Backend>(f: &mut Frame<B>, app: & mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    let width = chunks[0].width;

    // Iterate through all elements in the `items` app and append some debug text to it.
    fn format_items(
        app: Vec<(Vec<u8>, u8)>,
        width: u16,
        formatter: fn(&Vec<u8>, u16) -> Vec<String>,
    ) -> Vec<ListItem<'static>> {
        let t = app.iter().map(|(a, b)| {
            let s = (a.chunks(1024).into_iter().map(|s| s.to_vec()).collect::<Vec<_>>(), *b);
            s.0.into_iter().map(|v| (v, s.1)).collect_vec()
        })
        .collect::<Vec<Vec<(Vec<u8>, u8)>>>();

        let s = t.iter().flatten().collect::<Vec<_>>();


        s.iter()
            .map(|i| {
                //let split = i.0
                let lines = formatter(&i.0, width)
                    .into_iter()
                    .map(|f| Spans::from(f))
                    .collect::<Vec<_>>();
                let colour = if i.1 % 2 == 0 {
                    Color::Red
                } else {
                    Color::Green
                };
                //let t = lines.chunks(10).into_iter().map(|f| ListItem::new(f.to_vec()).style(Style::default().fg(colour)) ).collect::<Vec<_>>();
                ListItem::new(lines.clone()).style(Style::default().fg(colour))
                //t
            })
//            .flatten()
            .collect()
    }

    // Create a List from all list items and highlight the currently selected one
    fn itemize<'a>(items: Vec<ListItem<'a>>, title: &'a str) -> List<'a> {
        List::new(items)
            .block(Block::default().borders(Borders::ALL).title(title))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )
        //.highlight_symbol(">")
    }


    unsafe {
    if CACHE.width == Some(width) {
        f.render_stateful_widget(itemize(CACHE.utf8.clone(), "UTF-8"), chunks[0], &mut app.items.state);
        f.render_stateful_widget(itemize(CACHE.hex.clone(), "Hex"), chunks[1], &mut app.items.state);
    } else {
        //let test = app.items.items.clone();
        let hex_form: Vec<ListItem> = format_items(app.items.items.clone(), width, hex_formatter);
        let utf8_form: Vec<ListItem> = format_items(app.items.items.clone(), width, utf8_formatter);



        let utf8_items = itemize(utf8_form.clone(), "UTF-8");
        let hex_items = itemize(hex_form.clone(), "Hex");

        CACHE.width = Some(width);
        CACHE.hex = hex_form;
        CACHE.utf8 = utf8_form;

        //let t = hex_form.into_iter().map(|f| f.clone()).collect_vec();

        //cache.update(Some(width), utf8_form, hex_form);
        //cache.hex = hex_form;
        //f.render_stateful_widget(hex_items, chunks[0], &mut app.items.state);
        //f.render_stateful_widget(utf8_items, chunks[1], &mut app.items.state);
    }
    }
    //return format_cache { width: None, utf8: vec![], hex: vec![] };
}
