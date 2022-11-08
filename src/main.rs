use native_tls::{TlsConnector, TlsConnectorBuilder};
use std::io::{Read, Write, BufReader, BufRead};
use native_tls::{Identity, TlsAcceptor, TlsStream};
use std::fs::File;
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;

fn main() {
	/*
    println!("Hello, world!");

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream1 = TcpStream::connect("192.168.121.98:443").unwrap();
    let mut stream1 = connector.connect("godfgdfgogle.com", stream1).unwrap();

    stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    println!("{}", String::from_utf8_lossy(&res));
	
	*/
	
	let mut file = File::open("test.com.pfx").unwrap();
	let mut identity = vec![];
	file.read_to_end(&mut identity).unwrap();
	let identity = Identity::from_pkcs12(&identity, "password").unwrap();

	let listener = TcpListener::bind("0.0.0.0:8443").unwrap();
	let acceptor = TlsAcceptor::new(identity).unwrap();
	let acceptor = Arc::new(acceptor);

	fn handle_client(mut stream: TlsStream<TcpStream>) {
		//let mut client = vec![];
		//let mut client2 = Vec::with_capacity(32);
		//let mut server = vec![];
		
		let connector = TlsConnector::builder()
			.danger_accept_invalid_certs(true)
			.danger_accept_invalid_hostnames(true)
			.build()
			.unwrap();

		let stream1 = TcpStream::connect("google.com:443").unwrap();
		//let stream1 = TcpStream::connect("192.168.121.98:443").unwrap();
		let mut stream1 = connector.connect("godfgdfgogle.com", stream1).unwrap();

		//let mut breader = BufReader::new(stream.get_ref().try_clone().unwrap());


	    let mut line = String::default();



		//Handle out-going traffic forwarding in seperate thread
		thread::spawn(move || {
			let mut testbuf = vec![0u8; 1];
			let mut threadbuf = BufReader::new(stream);
			loop {
				if let Err(e) = threadbuf.read_exact(&mut testbuf) {
					println!("Outgoing thread reading failed, terminating thread.");
					return;
				}

				let byte = testbuf[0] as char;
				print!("{}", byte);
				stream1.write_all(&testbuf);
			}
		});







		/*
		breader.read_line(&mut line).unwrap();
		println!("{}", line);
		breader.read_line(&mut line).unwrap();
		println!("{}", line);
		breader.read_line(&mut line).unwrap();
		println!("{}", line);
		 */
		//while (num < 32) {
		//	num = stream.buffered_read_size().unwrap();
		//	println!("GOT: {}", num);
		//}
		//println!("Forwarding");

		//stream1.write_all(&client).unwrap();

		//stream1.read(&mut server).unwrap();

		//println!("Recived from google: {}", String::from_utf8_lossy(&server))

		//
		// stream1.read_to_end(buf: &mut Vec<u8>)

		/*
		while (true) {
			stream.read_to_end(&mut client).unwrap();
			println!("Client -> Server:\n {}", String::from_utf8_lossy(&client));
			stream1.write_all(&client).unwrap();
			
			stream1.read_to_end(&mut server).unwrap();
			println!("Server -> Client:\n {}", String::from_utf8_lossy(&server));
			stream.write_all(&server).unwrap();

		}
		*/
		
	}


	for stream in listener.incoming() {
		match stream {
			Ok(stream) => {
				let acceptor = acceptor.clone();
				thread::spawn(move || {
					let stream = acceptor.accept(stream).unwrap();
					handle_client(stream);
				});
			}
			Err(e) => { /* connection failed */ }
		}
	}

}
