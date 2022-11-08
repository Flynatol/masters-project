use native_tls::Identity;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};

const REPLACEMENTS: &'static [(&[u8], &[u8])] =
    &[("Host: 127.0.0.:1443".as_bytes(), "WOLOLOLO".as_bytes())];

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1443").await.unwrap();
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

    let stream_out = TcpStream::connect("reddit.com:443").await.unwrap();
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();

    let (mut client_read_tls, mut client_write_tls) = io::split(tls_stream_client);
    let (mut server_read_tls, mut server_write_tls) = io::split(tls_stream_server);

    tokio::spawn(async move {
        let mut outbuf: VecDeque<(u8, Vec<VecDeque<u8>>)> = VecDeque::new();

        loop {
            let mut testbuf = vec![0u8; 1];
            let read = match client_read_tls.read_u8().await {
                Ok(v) => v,
                Err(e) => {
                    println!("{}", e);
                    println!("Outgoing thread reading failed, terminating thread.");
                    return;
                }
            };

            print!("{}", read as char);

            //let running = outbuf.back().map(|(f, s)| s.clone()).unwrap_or_default();

            //Remove all elements that no longer match.
            outbuf = outbuf.into_iter().map(|(u, v)| (u, v.into_iter().filter(|f| f.front() == Some(&read)).collect())).collect::<VecDeque<_>>();



            //Add new element to the queue
            let base = REPLACEMENTS.iter().map(|(f, s)| *f).collect::<Vec<_>>();

            let newfilt = base
                .iter()
                .filter(|&&f| f.first() == Some(&read))
                .map(|f| VecDeque::from(f.to_vec()))
                .collect::<Vec<_>>();

            outbuf.push_back((read, newfilt));

            //Pop front off all remaining elements
            outbuf.iter_mut().for_each(|(_, v)| v.iter_mut().for_each(|f| {f.pop_front();}));

            //Print all front elements with no matches
            println!("{:?}", outbuf );

            server_write_tls.write_all(&testbuf).await;
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
            //print!("{}", testbuf[0] as char);
            client_write_tls.write_all(&testbuf).await;
        }
    });
}
