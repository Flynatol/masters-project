
use replace_stream::replace_mod::ReplaceStream;
use tokio_stream::StreamExt;
use std::fs;
use tokio::io::{self, AsyncWriteExt, ReadHalf};
use tokio::net::{TcpStream};
use tokio_native_tls::{TlsConnector, TlsStream};
use colored::{Colorize, Color};
use std::io::{Write};
use std::ffi::OsStr;
use tokio_util::io::ReaderStream;
use std::sync::mpsc;
use std::time::{Instant};
use replace_stream::replace_mod::replacment_builder;


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
    let connector: TlsConnector = TlsConnector::from(
		native_tls::TlsConnector::builder()
			.danger_accept_invalid_certs(true)
			.danger_accept_invalid_hostnames(true)
			.build()
			.unwrap(),
    );

	//clean up logs
	//fs::read_dir("./logs").unwrap()
	//	.filter(|f| f.as_ref().unwrap().path().extension() == Some(OsStr::new("log")))
	//	.for_each(|f| { fs::rename(f.as_ref().unwrap().path().to_str().unwrap(), format!("{}.old", f.unwrap().path().to_str().unwrap())); });

	let test = tokio::fs::File::open("payload_program.log").await.expect("Could not read file");

	let (tx, rx) = mpsc::sync_channel(512);

	//let cls = |f : &mut ReplaceStream<ReaderStream<tokio::io::ReadHalf<_>>>| 
	//	f.message(tx, Signal::Key);
	//let cls = rpl_boxed(vec![], repl)

	//let test2 = ReplaceStream::<'_, ReaderStream<tokio::fs::File>>::message(tx, Signal::Key);

	let mut file_stream = replacment_builder(test, vec![
		//(&[0xe8, 0xbf, 0x27, 0x09], test2),
		//("Dieses".as_bytes(), (&|f| f.replace("Dieses".as_bytes(), "tested".as_bytes()))),
	]);

	

	

    let stream_out = TcpStream::connect(format!("{}{}", TARGET, PORT)).await.unwrap();
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();



	let (server_read_tls, mut server_write_tls) = io::split(tls_stream_server);

	let message_fn = ReplaceStream::<ReaderStream<ReadHalf<TlsStream<TcpStream>>>>::message(tx, Signal::Key);

	let mut server_read_tls = replacment_builder(server_read_tls, vec![
		([0xe8, 0xbf, 0x27, 0x09].to_vec(), message_fn),
			//("Dieses".as_bytes(), (&|f| f.replace("Dieses".as_bytes(), "tested".as_bytes()))),
	]);	
	
	while let Some(tag) = file_stream.next().await {
		if tag == 0xff {
			println!("read ff tag");
			break;
		};
		let data = file_stream.next().await.expect("File stream ended unexpectedly");
		if tag % 2 == 0  {
			server_write_tls.write_u8(data).await.unwrap();
			print!("{}", format!(" {:02x?}", data).color(Color::TrueColor {r : 193,  g : 0,   b : 32}));
		} else {
			let rec = server_read_tls.next().await.expect("No data recived");

			if let Ok(_x) = rx.try_recv() {
				println!("SIGNAL");
				//Do the buffer modifications here
				//Skip as many bytes in the file as we consume from stream to keep them in sync.
				//server_read_tls = server_read_tls.skip(4);
				
				let mut trg = vec![];

				for _ in 0..3 {
					//server_read_tls.next().await;
					println!("skp: {:02x?}", server_read_tls.next().await.unwrap())
				}

				for _ in 0..4 {
					let byte = server_read_tls.next().await.unwrap();
					println!("trg {:02x?}", byte);
					trg.push(0x04);
					trg.push(byte);
					
					//trg.push(byte);
				}
				
				let mut to_replace = vec![];

				for i in 0..7 {
					let v = file_stream.next().await.unwrap();
					println!("skp f: {:02x?}", v);
					if i % 2 == 1 {
						//to_replace.push(v)
					}
				}

				for i in 0..7 {
					let v = file_stream.next().await.unwrap();
					println!("skp f: {:02x?}", v);
					if i % 2 == 0 {
						to_replace.push(0x04);
						to_replace.push(v)
					}
				}

				println!("Pushing replacement of {:02x?} to {:02x?}", &to_replace, &trg);
				let lmb = ReplaceStream::rpl_boxed(to_replace.clone(), trg);
				file_stream.triggers.push((to_replace, lmb));

			}

			if rec == data {
				print!("{}", format!(" {:02x?}", data).color(Color::TrueColor {r : 206,  g : 162, b : 98}));
			} else {
				print!("[Rc : {:02x?}, Exp: {:02x?}]", rec, data);
			}
		}
	}
	let now = Instant::now();
	server_write_tls.write_u8(0xff).await;

	server_read_tls.next().await;
	let elapsed_time = now.elapsed();

	println!("Response took {}", elapsed_time.as_micros());


	while let Some(x) = server_read_tls.next().await {
		print!("{}", format!(" {:02x?}", x).color(Color::TrueColor {r : 206,  g : 162, b : 98}));
		std::io::stdout().flush().unwrap()
	}
	
	
}

#[derive(Debug, Copy, Clone)]
enum Signal{Key}