use colored::Colorize;
use colours::COLOURS;
use itertools::Itertools;
use native_tls::Identity;

use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use tokio_stream::StreamExt;

use clap::Parser;

use replace_stream::replace_mod::replacment_builder;
mod colours;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip:port to connect to
    #[arg(short, long)]
    target_address: std::net::SocketAddr,

    /// ip:port to bind to
    #[arg(short, long)]
    bind_address: std::net::SocketAddr,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let listener = TcpListener::bind(args.bind_address).await?;

    let mut public = File::open("./fullchain.pem").unwrap();
    let mut pub_buf = vec![];
    public.read_to_end(&mut pub_buf).unwrap();

    let mut private = File::open("./privkey.pem").unwrap();
    let mut priv_buf = vec![];
    private.read_to_end(&mut priv_buf).unwrap();

    let identity = Identity::from_pkcs8(&pub_buf, &priv_buf).unwrap();

    let acceptor = TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).expect("Failed to construct Identity"),
    );

    let mut stream_num = 0;

    //clean up logs
    fs::read_dir("./logs")
        .unwrap()
        .filter(|f| f.as_ref().unwrap().path().extension() == Some(OsStr::new("log")))
        .for_each(|f| {
            fs::rename(
                f.as_ref().unwrap().path().to_str().unwrap(),
                format!("{}.old", f.unwrap().path().to_str().unwrap()),
            )
            .expect("Failed to rename");
        });

    loop {
        let listener = listener.accept().await;

        match listener {
            Ok((stream, _)) => {
                stream_num += 2;
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    if let Ok(stream) = acceptor.accept(stream).await {
                        handle_client(stream, stream_num).await;
                    }
                });
            }
            Err(_) => {
                println!("TcpStream terminated")
            }
        }
    }
}

async fn handle_client(tls_stream_client: TlsStream<TcpStream>, num: usize) {
    let args = Args::parse();
    println!("{} {}", "Stream created".red(), num);

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .use_sni(false)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect(args.target_address).await.unwrap();

    let tls_stream_server = connector.connect("AXC F 2152", stream_out).await.unwrap();

    let (client_read_tls, client_write_tls) = io::split(tls_stream_client);
    let (server_read_tls, server_write_tls) = io::split(tls_stream_server);

    let merged_log = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open("./logs/combined.log")
        .unwrap();

    let merged_log = Arc::new(Mutex::new(merged_log));

    let mutex_1 = Arc::clone(&merged_log);
    let mutex_2 = Arc::clone(&merged_log);

    tokio::spawn(async move {
        replace_bridge(client_read_tls, server_write_tls, num, mutex_1).await;
    });

    tokio::spawn(async move {
        replace_bridge(server_read_tls, client_write_tls, num + 1, mutex_2).await;
    });
}

async fn replace_bridge(
    read_tls: tokio::io::ReadHalf<TlsStream<TcpStream>>,
    mut write_tls: tokio::io::WriteHalf<TlsStream<TcpStream>>,
    threadnum: usize,
    merged_log: std::sync::Arc<std::sync::Mutex<std::fs::File>>,
) {
    let mut read_tls = replacment_builder(read_tls, vec![]);

    read_tls.add_repl(b"13625".to_vec(), b"99999".to_vec());

    let mut log = create_log(threadnum).unwrap();

    let col = COLOURS.get(threadnum % COLOURS.len()).unwrap();

    loop {
        let read = match read_tls.next().await {
            Some(num) => num,
            None => {
                println!("Thread {threadnum} timed out.");
                return;
            }
        };

        print!("{}", format!(" {:02x?}", read).color(*col));
        log.write_all(&[read]).expect("Failed to write to log");
        write_tls.write_all(&[read]).await.expect("Failed to write to TLS stream");
        synced_write(&merged_log, threadnum as u8, &vec![read]);
    }
}

fn create_log(lognum: usize) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(format!("./logs/thread_{}.log", lognum))
}

fn synced_write(
    merged_log: &std::sync::Arc<std::sync::Mutex<std::fs::File>>,
    prefix: u8,
    data: &Vec<u8>,
) {
    let mut file = merged_log.lock().unwrap();
    let d2 = vec![prefix; data.len()]
        .into_iter()
        .interleave(data.clone().into_iter())
        .collect::<Vec<_>>();
    file.write_all(&d2).expect("Could not write to file");
}
