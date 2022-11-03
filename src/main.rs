use native_tls::{TlsConnector, TlsConnectorBuilder};
use std::io::{Read, Write};
use std::net::TcpStream;

fn main() {
    println!("Hello, world!");

    let connector = TlsConnector::builder()
        .danger_accept_invalid_certs(true)
        .danger_accept_invalid_hostnames(true)
        .build()
        .unwrap();

    let stream = TcpStream::connect("google.com:443").unwrap();
    let mut stream = connector.connect("google.com", stream).unwrap();

    stream.write_all(b"GET / HTTP/1.0\r\n\r\n").unwrap();
    let mut res = vec![];
    stream.read_to_end(&mut res).unwrap();
    println!("{}", String::from_utf8_lossy(&res));

}
