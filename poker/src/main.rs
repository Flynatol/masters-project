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
    pin::Pin,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Corner, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use futures::{self, TryFutureExt, FutureExt};
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use tokio_stream::StreamExt;
use native_tls::Identity;


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

    fn push_select(&mut self, val: T) {
        self.items.push(val);
        self.state.select(Some(self.items.len()))
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
    message_list: StatefulList<(Vec<u8>, u8)>,
}

impl App {
    fn new(i: Vec<(Vec<u8>, u8)>) -> App {
        App {
            items: StatefulList::with_items(i),
            message_list: StatefulList::with_items(vec![]),
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    fn on_tick(&mut self) {

    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();

    let f = File::open(&args[1])?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let t = buffer.into_iter().tuples::<(u8, u8)>().fold(
        (Vec::<(Vec<u8>, u8)>::new()),
        |mut out, (tag, byte)| match out.last().map(|f| f.1) {
            Some(lr) if lr == tag => {
                out.last_mut().unwrap().0.push(byte);
                return out;
            }
            _ => {
                out.push((vec![byte], tag));
                return out;
            }
        },
    );

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(50);
    let app = App::new(t);
    run_app(&mut terminal, app, tick_rate).await;

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    

    Ok(())
}

static mut CACHE: format_cache = format_cache {
    width: None,
    utf8: vec![],
    hex: vec![],
    messages: vec![],
};


async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut supress = false;

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect("192.168.121.98:41100")
        .await
        .unwrap();
    
    let mut tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();

    //let tls_poll = futures::poll!(Box::pin(tls_stream_server.read_u8()));

    loop {
        let mut response = vec![];
        
        while let std::task::Poll::Ready(res) = futures::poll!(Box::pin(tls_stream_server.read_u8())) {
            match res {
                Ok(byte) => response.push(byte),
                Err(e) => return Err(e),
            }
        }

        //Write response into Log
        if !response.is_empty() {
            app.message_list.push_select((response, 1));
        }


        if crossterm::event::poll(Duration::from_millis(0))? {
            if let Event::Key(key) = event::read()? {
                let items = &mut app.items;
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Left => items.unselect(),
                    KeyCode::Down => items.next(),
                    KeyCode::Up => items.previous(),
                    KeyCode::Enter => {
                        if let Some(index) = items.state.selected() {
                            let selected = items.items.get(index).unwrap().clone();

                            //Write currently selected Hex into the output stream
                            for byte in &selected.0 {
                                tls_stream_server.write_u8(*byte).await;
                            }

                            //Additionally append it to the message board
                            app.message_list.push_select(selected);
                            //let item = it
                        }
                    }
                    _ => {}
                }
                terminal.draw(|f| {ui(f, &mut app);})?;
                supress = true;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
            if !supress {terminal.draw(|f| {ui(f, &mut app);})?;}
            else {supress = false;}
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
    messages: Vec<ListItem<'a>>,
}

impl <'a>format_cache<'a> {
    fn update(&'a mut self, width: Option<u16>, utf8: Vec<ListItem<'a>>, hex: Vec<ListItem<'a>>) {
        self.width = width;
        self.utf8 = utf8;
        self.hex = hex;
    }
}

// Create a List from all list items
fn itemize<'a>(items: Vec<ListItem<'a>>, title: &'a str) -> List<'a> {
    List::new(items)
        .block(Block::default().borders(Borders::ALL).title(title))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
}

fn ui<'a, B: Backend>(f: &mut Frame<B>, app: & mut App) {
    let mut chunks = vec![];

    // Create two chunks with equal horizontal screen space
    let vsplit = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    chunks.push(vsplit[0]);

    let hsplit = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(vsplit[1]);
        
    chunks.push(hsplit[0]);
    chunks.push(hsplit[1]);
    

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
                ListItem::new(lines.clone()).style(Style::default().fg(colour))
                //t
            })
            .collect()
    }

    


    unsafe {
        if CACHE.width == Some(width) {
            f.render_stateful_widget(itemize(CACHE.utf8.clone(), "UTF-8"), chunks[0], &mut app.items.state);
            f.render_stateful_widget(itemize(CACHE.hex.clone(), "Hex"), chunks[1], &mut app.items.state);
        } else {
            let hex_form: Vec<ListItem> = format_items(app.items.items.clone(), width, hex_formatter);
            let utf8_form: Vec<ListItem> = format_items(app.items.items.clone(), width, utf8_formatter);

            let utf8_items = itemize(utf8_form.clone(), "UTF-8");
            let hex_items = itemize(hex_form.clone(), "Hex");

            CACHE.width = Some(width);
            CACHE.hex = hex_form;
            CACHE.utf8 = utf8_form;

            f.render_stateful_widget(hex_items, chunks[0], &mut app.items.state);
            f.render_stateful_widget(utf8_items, chunks[1], &mut app.items.state);
        }
        f.render_stateful_widget(itemize(format_items(app.message_list.items.clone(), width, hex_formatter), "Messages"), chunks[2], &mut app.message_list.state);

    }
}
