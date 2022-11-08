use std::io::{Read, Write, BufReader, BufRead};
use native_tls::{Identity};
use tokio_native_tls::{TlsAcceptor, TlsStream, TlsConnector};
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::fs::File;

#[tokio::main]
async fn main() -> io::Result<()> {
	let listener = TcpListener::bind("127.0.0.1:1443").await?;


	let mut file = File::open("test.com.pfx").unwrap();
	let mut identity = vec![];
	file.read_to_end(&mut identity).unwrap();
	let identity = Identity::from_pkcs12(&identity, "password").unwrap();

	let acceptor = TlsAcceptor::from(native_tls::TlsAcceptor::new(identity).expect("Failed to construct Identity"));


	loop {

		let (mut socket, _) = listener.accept().await?;


		let tls_stream_client = acceptor.accept(socket).await.unwrap();



		let connector : TlsConnector = TlsConnector::from(
			native_tls::TlsConnector::builder()
				.danger_accept_invalid_certs(true)
				.danger_accept_invalid_hostnames(true)
				.build()
				.unwrap()
		);

		let stream_out = TcpStream::connect("youtube.com:443").await?;
		let tls_stream_server = connector.connect("googlasde.com", stream_out).await.unwrap();


		let (mut client_read_tls, mut client_write_tls) = io::split(tls_stream_client);
		let (mut server_read_tls, mut server_write_tls) = io::split(tls_stream_server);


		tokio::spawn(async move {
			loop {
				let mut testbuf = vec![0u8; 1];
				if let Err(e) = client_read_tls.read_exact(&mut testbuf).await {
					println!("{}", e);
					println!("Outgoing thread reading failed, terminating thread.");
					return;
				}
				print!("{}", testbuf[0] as char);
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
				print!("{}", testbuf[0] as char);
				client_write_tls.write_all(&testbuf).await;
			}
		});


		//server_write_tls.write_all(b"GET / HTTP/1.0\r\n\r\n").await;

		println!("test");
	}

}

//async fn handle_client() {
