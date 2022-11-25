use native_tls::Identity;
use std::collections::VecDeque;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::fs::OpenOptions;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_native_tls::{TlsAcceptor, TlsConnector, TlsStream};
use colored::{Colorize, Color};

const TARGET : &str = "192.168.121.98";
const PORT : &str = ":41100";
const MY_PORT: &str = ":41100";
const MY_IP : &str = "192.168.121.144"; //TODO grab this automatically
const REPLACEMENTS: &'static [(&[u8], &[u8])] = &[(MY_IP.as_bytes(), TARGET.as_bytes()),
												  ("192.168.zz".as_bytes(), MY_IP.as_bytes()),
                                                  (TARGET.as_bytes(), MY_IP.as_bytes()),
												  ("www.wikipedia.org".as_bytes(), MY_IP.as_bytes()),
                                                  ("fflkskkk".as_bytes(), "W2113dsf".as_bytes())];

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind(format!("{}{}", MY_IP, MY_PORT)).await.unwrap();
    let mut file = File::open("test.com.pfx").unwrap();
    let mut identity = vec![];
    file.read_to_end(&mut identity).unwrap();
    let identity = Identity::from_pkcs12(&identity, "password").unwrap();
    let acceptor = TlsAcceptor::from(
        native_tls::TlsAcceptor::new(identity).expect("Failed to construct Identity"),
    );

    let mut stream_num = 0;

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

async fn handle_client(tls_stream_client: TlsStream<TcpStream>, num : usize) {

    println!("{} {}", "Stream created".red(), num);

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect(format!("{}{}", TARGET, PORT)).await.unwrap();
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();

    let (client_read_tls, client_write_tls) = io::split(tls_stream_client);
    let (server_read_tls, server_write_tls) = io::split(tls_stream_server);


    tokio::spawn(async move {
        replace_bridge(client_read_tls, server_write_tls, num).await;
    });

    tokio::spawn(async move {
        replace_bridge(server_read_tls, client_write_tls, num+1).await;
    });
}

async fn replace_bridge(mut read_tls : tokio::io::ReadHalf<TlsStream<TcpStream>>, mut write_tls : tokio::io::WriteHalf<TlsStream<TcpStream>>, threadnum : usize) {
        let mut outbuf: VecDeque<(u8, Vec<VecDeque<u8>>)> = VecDeque::new();
		
		let mut log = create_log(threadnum).unwrap();
		
        let colours = vec![
            Color::TrueColor {r : 255,  g : 179, b : 0},
            Color::TrueColor {r : 128,  g : 62,  b : 117},
            Color::TrueColor {r : 255,  g : 104, b : 0},
            Color::TrueColor {r : 166,  g : 189, b : 215},
            Color::TrueColor {r : 193,  g : 0,   b : 32},
            Color::TrueColor {r : 206,  g : 162, b : 98},
            Color::TrueColor {r : 129,  g : 112, b : 102},
            Color::TrueColor {r : 0,    g : 125, b : 52},
            Color::TrueColor {r : 246,  g : 118, b : 142},
            Color::TrueColor {r : 0,    g : 83,  b : 138},
            Color::TrueColor {r : 255,  g : 122, b : 92},
            Color::TrueColor {r : 83,   g : 55,  b : 122},
            Color::TrueColor {r : 255,  g : 142, b : 0},
            Color::TrueColor {r : 179,  g : 40,  b : 81},
            Color::TrueColor {r : 244,  g : 200, b : 0},
            Color::TrueColor {r : 127,  g : 24,  b : 13},
            Color::TrueColor {r : 147,  g : 170, b : 0},
            Color::TrueColor {r : 89,   g : 51,  b : 21},
            Color::TrueColor {r : 241,  g : 58,  b : 19},
            Color::TrueColor {r : 35,   g : 44,  b : 22},
        ];

        let col = colours.get(threadnum % colours.len()).unwrap();


        loop {
            let read = match read_tls.read_u8().await {
                Ok(v) => v,
                Err(_) => {
                    println!("Outgoing thread died, terminating thread.");
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
                        //.map(|(a, b)| (Vec::from(&a[1..]), b))
                        .collect::<Vec<_>>();

					//println!("filter pass {} {}", c, i);
					//println!("candidates: {:?}", candidates);

                    if candidates.len() == 1 {
                        println!("Found");
                        break;
                    }
                }

                let candidate = match candidates.first() {
                    Some(x) => x,
                    None => {println!("Caught failed match on {:?}", outbuf);
                             break},
                };
				println!("replacing with {:?}", candidate);
                //Replace byte stream
                outbuf.drain(0..candidate.0.len().clone());

                let mut oldoutbuf = outbuf;

                outbuf = candidate.to_owned().1.into_iter().map(|f| (f, Vec::new())).collect::<VecDeque<_>>();
                outbuf.append(&mut oldoutbuf);
            }

            //write unmatched bytes to output stream
            //Somehow this version is faster?
            let out = outbuf
                .iter()
                .take_while(|(_, s)| s.is_empty())
                .map(|(a, _)| a.clone())
                .collect::<Vec<_>>();

            outbuf.drain(0..out.len());

            out.iter().for_each(|&f| print!("{}", (f as char).to_string().color(*col)));
            log.write_all(&out);
			write_tls.write_all(&out).await;


            /*
            while !outbuf.is_empty() {
                if outbuf.front().unwrap().1.is_empty() {
                    let v = outbuf.pop_front().unwrap();
                    print!("{}", (v.0 as char).to_string().color(*col));
                    write_tls.write_u8(v.0).await.unwrap();
                }else {
                    break;
                }
            }
            */
        }
}

fn create_log(lognum: usize) -> io::Result<File> {
        OpenOptions::new().write(true).create(true).truncate(false).open(format!("./logs/thread_{}.log", lognum))
}
