use crossterm::{
    event::{self, DisableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use itertools::Itertools;
use std::{
    error::Error,
    fs::File,
    io::{self, ErrorKind},
    io::{BufReader, Read},
    time::{Duration, Instant}, path::PathBuf, net::SocketAddr,
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};

use clap::Parser;

use futures::{self};
use tokio::{io::{AsyncReadExt, AsyncWriteExt}, time::timeout};
use tokio::net::TcpStream;
use tokio_native_tls::TlsConnector;
use thiserror::Error;

struct StatefulList<T> {
    state: ListState,
    items: Vec<T>,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip:port to connect to
    #[arg(short)]
    target_address: SocketAddr,

    /// Log file to open
    #[arg(short, value_name = "FILE")]
    file: PathBuf,
}

#[derive(Error, Debug)]
pub enum CustError {
    #[error("data store disconnected")]
    Disconnect(#[from] io::Error),
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
    fn on_tick(&mut self) {}
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let f = File::open(args.file)?;
    let mut reader = BufReader::new(f);
    let mut buffer = Vec::new();

    // Read file into vector.
    reader.read_to_end(&mut buffer)?;

    let formatted_data = buffer.into_iter().tuples::<(u8, u8)>().fold(
        Vec::<(Vec<u8>, u8)>::new(),
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

    let formatted_data = formatted_data
        .into_iter()
        .map(|(a, b)| {
            let s = (
                a.chunks(1024)
                    .into_iter()
                    .map(|s| s.to_vec())
                    .collect::<Vec<_>>(),
                b,
            );
            s.0.into_iter().map(|v| (v, s.1)).collect_vec()
        })
        .collect::<Vec<Vec<(Vec<u8>, u8)>>>()
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    
    // create app and run it
    let tick_rate = Duration::from_millis(50);
    let mut app = App::new(formatted_data);
    let mut err_message = None;

    while let Err(f) = run_app(&mut terminal, &mut app, tick_rate).await {
        if f.kind() == ErrorKind::Other {
            err_message = Some(f.to_string());
            break;
        };
        app.message_list.push_select((
            "PLC TERMINATED COMMUNICATION RESTABLISHING CONNECTION"
                .as_bytes()
                .to_vec(),
            4,
        ));
    };

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Some(message) = err_message {
        println!("{}", message);
    }

     Ok(())
}

static mut CACHE: FormatCache = FormatCache {
    width: None,
    utf8: vec![],
    hex: vec![],
    //messages: vec![],
    redraw: true,
    hex_mode: false,
};

async fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: &mut App,
    tick_rate: Duration,
) -> io::Result<()> {
    let args = Args::parse();
    let mut last_tick = Instant::now();
    let mut supress_redraw = false;

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let Ok(Ok(stream_out)) = timeout(Duration::from_millis(500), TcpStream::connect(args.target_address)).await
    else {
        return Err(std::io::Error::new(ErrorKind::Other, "Failed to connect to PLC"));
    };

    let Ok(Ok(mut tls_stream_server)) = timeout(Duration::from_millis(500), connector.connect("googlasde.com", stream_out)).await else {
        return Err(std::io::Error::new(ErrorKind::Other, "Failed to establish TLS connection"));
    };

    loop {
        let mut response = vec![];

        while let std::task::Poll::Ready(res) =
            futures::poll!(Box::pin(tls_stream_server.read_u8()))
        {
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
                    KeyCode::Char('h') => unsafe {
                        CACHE.hex_mode = !CACHE.hex_mode;
                    },
                    KeyCode::Left => items.unselect(),
                    KeyCode::Down => items.next(),
                    KeyCode::Up => items.previous(),
                    KeyCode::Char('i') => app.message_list.previous(),
                    KeyCode::Char('k') => app.message_list.next(),
                    KeyCode::Char('l') => {
                        if let Some(index) = app.message_list.state.selected() {
                            let mut selected =
                                (app.message_list.items.get(index).unwrap().clone().0, 0);
                            //let mut t = selected.0;
                            selected.0[2] = 2;
                            selected.0[10] = 3;
                            selected.0[14] = 227;
                            //Write currently selected Hex into the output stream
                            for byte in &selected.0 {
                                match tls_stream_server.write_u8(*byte).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }

                            //Additionally append it to the message board
                            app.message_list.push_select(selected);

                            items.next();
                            items.next();
                        }
                    }
                    KeyCode::Enter => {
                        if let Some(index) = items.state.selected() {
                            let selected = items.items.get(index).unwrap().clone();

                            //Write currently selected Hex into the output stream
                            for byte in &selected.0 {
                                match tls_stream_server.write_u8(*byte).await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }
                            }

                            //Additionally append it to the message board
                            app.message_list.push_select(selected);

                            items.next();
                            items.next();
                        }
                    }
                    KeyCode::Char('o') => {
                        if let Some(item) = items
                            .state
                            .selected()
                            .map(|f| items.items.get_mut(f).unwrap())
                        {
                            if item.1 > 1 {
                                item.1 -= 2;
                            } else {
                                item.1 += 2;
                            }

                            items.next();
                            items.next();

                            //Invalidate cache to force redraw
                            unsafe {
                                CACHE.redraw = true;
                            }
                        }
                    }
                    KeyCode::Char('u') => {
                        for item in &mut items.items {
                            if item.1 == 0 {
                                item.1 = 2;
                            }
                        }
                        unsafe {
                            CACHE.redraw = true;
                        }
                    }
                    KeyCode::Char('r') => {
                        //Start replay excluding deselected messages:
                        for item in items.items.clone().iter().filter(|(_a, b)| *b == 0) {
                            tls_stream_server.write_all(&item.0[..]).await.unwrap();
                            app.message_list.push_select(item.clone());

                            let mut response = vec![];

                            response.push(tls_stream_server.read_u8().await.unwrap());

                            terminal.draw(|f| {
                                ui(f, &mut app);
                            })?;

                            while let std::task::Poll::Ready(res) =
                                futures::poll!(Box::pin(tls_stream_server.read_u8()))
                            {
                                match res {
                                    Ok(byte) => response.push(byte),
                                    Err(e) => return Err(e),
                                }
                            }

                            //Write response into Log
                            if !response.is_empty() {
                                app.message_list.push_select((response, 1));
                            }
                        }
                        unsafe {
                            CACHE.redraw = true;
                        }
                    }
                    _ => {}
                }
                terminal.draw(|f| {
                    ui(f, &mut app);
                })?;
                supress_redraw = true;
            }
        }

        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
            if !supress_redraw {
                terminal.draw(|f| {
                    ui(f, &mut app);
                })?;
            } else {
                supress_redraw = false;
            }
        }
    }
}

fn utf8_formatter(bytes: &Vec<u8>, width: u16) -> Vec<String> {
    let bytes2 = bytes
        .split(|&i| i == 10)
        .map(|f| f.to_vec())
        .collect::<Vec<_>>();

    bytes2
        .into_iter()
        .map(|f| {
            f.chunks(width.into())
                .map(|f| String::from_utf8_lossy(f).to_string())
                .collect::<Vec<_>>()
        })
        .flatten()
        .collect::<Vec<_>>()
}

fn hex_formatter(bytes: &Vec<u8>, width: u16) -> Vec<String> {
    bytes
        .chunks((width / 3 - 1).into())
        .map(|f| format!("{:02x?}", f).replace(&[',', '[', ']'][..], ""))
        .collect::<Vec<_>>()
}

struct FormatCache<'a> {
    width: Option<u16>,
    utf8: Vec<ListItem<'a>>,
    hex: Vec<ListItem<'a>>,
    //messages: Vec<ListItem<'a>>,
    redraw: bool,
    hex_mode: bool,
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

fn ui<'a, B: Backend>(f: &mut Frame<B>, app: &mut App) {
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
        let t = app
            .iter()
            .map(|(a, b)| {
                let s = (
                    a.chunks(1024)
                        .into_iter()
                        .map(|s| s.to_vec())
                        .collect::<Vec<_>>(),
                    *b,
                );
                s.0.into_iter().map(|v| (v, s.1)).collect_vec()
            })
            .collect::<Vec<Vec<(Vec<u8>, u8)>>>();

        let _s = t.iter().flatten().collect::<Vec<_>>();

        app.iter()
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
                let bg_colour = if i.1 > 1 { Color::Black } else { Color::Reset };
                ListItem::new(lines.clone()).style(Style::default().fg(colour).bg(bg_colour))
            })
            .collect()
    }

    //TODO make safe by passing mutable struct ref
    unsafe {
        if CACHE.width == Some(width) && !CACHE.redraw {
            f.render_stateful_widget(
                itemize(CACHE.utf8.clone(), "UTF-8"),
                chunks[0],
                &mut app.items.state,
            );
            f.render_stateful_widget(
                itemize(CACHE.hex.clone(), "Hex"),
                chunks[1],
                &mut app.items.state,
            );
        } else {
            let hex_form: Vec<ListItem> =
                format_items(app.items.items.clone(), width, hex_formatter);
            let utf8_form: Vec<ListItem> =
                format_items(app.items.items.clone(), width, utf8_formatter);

            let utf8_items = itemize(utf8_form.clone(), "UTF-8");
            let hex_items = itemize(hex_form.clone(), "Hex");

            CACHE.width = Some(width);
            CACHE.hex = hex_form;
            CACHE.utf8 = utf8_form;

            f.render_stateful_widget(hex_items, chunks[1], &mut app.items.state);
            f.render_stateful_widget(utf8_items, chunks[0], &mut app.items.state);

            CACHE.redraw = false;
        }
        f.render_stateful_widget(
            itemize(
                format_items(
                    app.message_list.items.clone(),
                    width,
                    if CACHE.hex_mode {
                        hex_formatter
                    } else {
                        utf8_formatter
                    },
                ),
                "Messages",
            ),
            chunks[2],
            &mut app.message_list.state,
        );
    }
}
