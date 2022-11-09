use native_tls::Identity;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use colored::Colorize;

const DOMAIN : &str = "www.google.com";
const REPLACEMENTS: &'static [(&[u8], &[u8])] = &[("127.0.0.1:1444".as_bytes(), DOMAIN.as_bytes()),
                                                  ("fflkskkk".as_bytes(), "W2113dsf".as_bytes())];

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1444").await.unwrap();
    let mut file = File::open("test.com.pfx").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "password").unwrap();
    let acceptor = TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).expect("Failed to construct Identity"),
    );

    loop {
        let listener = listener.accept().await;

        match listener {
            Ok((stream, _)) => {
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    let stream = acceptor.accept(stream).await.unwrap();
                    handle_client(stream).await;
                });
            }
            Err(_) => {
                println!("TcpStream terminated")
            }
        }
    }
    Ok(())
}

async fn handle_client(tls_stream_client: TlsStream<TcpStream>) {
    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect("www.google.com:443").await.unwrap();
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();

    let (mut client_read_tls, mut client_write_tls) = io::split(tls_stream_client);
    let (mut server_read_tls, mut server_write_tls) = io::split(tls_stream_server);

    tokio::spawn(async move {
        let mut outbuf: VecDeque<(u8, Vec<VecDeque<u8>>)> = VecDeque::new();

        loop {
            //let mut testbuf = vec![0u8; 1];
            let read = match client_read_tls.read_u8().await {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", e);
                    println!("Outgoing thread reading failed, terminating thread.");
                    return;
                }
            };

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
                .any(|(a, b)| b.iter().any(|l| l.len() == 1 && l.contains(&read)));

            //Add new element to the queue
            let base = REPLACEMENTS.iter().map(|(f, s)| *f).collect::<Vec<_>>();

            let newfilt = base
                .iter()
                .filter(|&&f| f.first() == Some(&read))
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

                for (c, _) in &outbuf {
                    //println!("{:?}", candidates);
                    candidates = candidates
                        .into_iter()
                        .filter(|(a, _)| a.first() == Some(c))
                        .collect::<Vec<_>>();
                    if candidates.len() == 1 {
                        break;
                    }
                }

                let candidate = candidates.first().unwrap();
                //println!("DETECTED: {:?}", candidates.first().unwrap().1.iter().map(|&i| i as char).collect::<Vec<_>>());

                //Replace byte stream
                outbuf.drain(0..candidate.0.len().clone());

                let mut oldoutbuf = outbuf;

                outbuf = candidate.to_owned().1.into_iter().map(|f| (f, Vec::new())).collect::<VecDeque<_>>();
                outbuf.append(&mut oldoutbuf);
            }

            //write unmatched bytes to output stream
            let out = outbuf
                .iter()
                .take_while(|(_, s)| s.is_empty())
                .map(|(a, _)| a.clone())
                .collect::<Vec<_>>();

            outbuf.drain(0..out.len());


            out.iter().for_each(|&f| print!("{}", (f as char).to_string().green()));
            server_write_tls.write_all(&out).await;
        }
    });

    tokio::spawn(async move {
        loop {
            let mut testbuf = vec![0u8; 1];
            if let Err(e) = server_read_tls.read_exact(&mut testbuf).await {
                println!("{}", e);
                println!("Outgoing thread reading failed, terminating thread.");
                return;
            }
            print!("{}", (testbuf[0] as char).to_string().cyan());
            client_write_tls.write_all(&testbuf).await;
        }
    });
}
