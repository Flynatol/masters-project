use colored::Colorize;
use colours::COLOURS;
use itertools::Itertools;
use native_tls::Identity;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Read;
use std::io::Write;
use std::sync::{Arc, Mutex};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use tokio_stream::StreamExt;
use tokio_util::io::ReaderStream;

mod colours;
mod replace_stream;

use crate::replace_stream::replace_mod::replacment_builder;

const TARGET: &str = "192.168.121.98";
const PORT: &str = ":41100";
const MY_PORT: &str = ":41100";
const MY_IP: &str = "192.168.121.144"; //TODO grab this automatically
const REPLACEMENTS: &'static [(&[u8], &[u8])] = &[
    (MY_IP.as_bytes(), TARGET.as_bytes()),
    (
        "A8:74:1D:04:9D:4A".as_bytes(),
        "08:00:27:A6:D5:86".as_bytes(),
    ),
    //("Dieses".as_bytes(), "WOLOLOLOLO".as_bytes()),
];

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(format!("{}{}", MY_IP, MY_PORT))
        .await
        .unwrap();
    let mut file = File::open("identity.pfx").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "test").unwrap();
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
                    let stream = acceptor.accept(stream).await.unwrap(); //an unknown error occured while proccessing the certificate
                    handle_client(stream, stream_num).await;
                });
            }
            Err(_) => {
                println!("TcpStream terminated")
            }
        }
    }
    //Ok(())
}

async fn handle_client(tls_stream_client: TlsStream<TcpStream>, num: usize) {
    println!("{} {}", "Stream created".red(), num);

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect(format!("{}{}", TARGET, PORT))
        .await
        .unwrap();
        
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();

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
    let mut outbuf: VecDeque<(u8, Vec<VecDeque<u8>>)> = VecDeque::new();

    let mut read_tls = replacment_builder(
        read_tls,
        vec![
            //("Dieses".as_bytes(), (|f| f.replace("Dieses".as_bytes(), "tested".as_bytes())))
        ],
    );

    let mut log = create_log(threadnum).unwrap();

    let col = COLOURS.get(threadnum % COLOURS.len()).unwrap();

    loop {
        let read = read_tls.next().await.unwrap();

        //Remove all elements that no longer match.
        outbuf = outbuf
            .into_iter()
            .map(|(u, v)| {
                (
                    u,
                    v.into_iter().filter(|f| f.front() == Some(&read)).collect(),
                )
            })
            .collect::<VecDeque<_>>();

        //Detect a completion
        let det = outbuf
            .iter()
            .any(|(_, b)| b.iter().any(|l| l.len() == 1 && l.contains(&read)));

        //Add new element to the queue
        let newfilt = REPLACEMENTS
            .into_iter()
            .map(|(f, _s)| *f)
            .filter(|&f| f.first() == Some(&read))
            .map(|f| VecDeque::from(f.to_vec()))
            .collect::<Vec<_>>();

        outbuf.push_back((read, newfilt));

        //Pop front off all remaining elements
        outbuf.iter_mut().for_each(|(_, v)| {
            v.iter_mut().for_each(|f| {
                f.pop_front();
            })
        });
        //Rebuild the completion and edit the buffer.
        if det {
            let mut candidates = REPLACEMENTS
                .to_vec()
                .iter()
                .map(|(a, b)| (a.to_vec(), b.to_vec()))
                .collect::<Vec<_>>();

            println!("Candidates: {:?}", candidates);
            for (i, (c, _)) in outbuf.iter().enumerate() {
                candidates = candidates
                    .into_iter()
                    .filter(|(a, _)| a.get(i) == Some(c))
                    .collect::<Vec<_>>();

                if candidates.len() == 1 {
                    println!("Found");
                    break;
                }
            }

            let candidate = match candidates.first() {
                Some(x) => x,
                None => {
                    println!("Caught failed match on {:?}", outbuf);
                    break;
                }
            };
            println!("replacing with {:?}", candidate);

            //Replace byte stream
            outbuf.drain(0..candidate.0.len().clone());
            let mut oldoutbuf = outbuf;

            outbuf = candidate
                .to_owned()
                .1
                .into_iter()
                .map(|f| (f, Vec::new()))
                .collect::<VecDeque<_>>();
            outbuf.append(&mut oldoutbuf);
        }

        //write unmatched bytes to output stream
        let out = outbuf
            .iter()
            .take_while(|(_, s)| s.is_empty())
            .map(|(a, _)| a.clone())
            .collect::<Vec<_>>();

        outbuf.drain(0..out.len());

        print!("{}", format!(" {:02x?}", read).color(*col));
        log.write_all(&[read]);
        write_tls.write_all(&[read]).await;
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
    file.write_all(&d2);
}
