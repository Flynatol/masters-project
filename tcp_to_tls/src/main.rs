use std::{convert::Infallible, net::{TcpStream, TcpListener}, io::{Read, Write}};

use native_tls::TlsConnector;
use anyhow::Result;

fn main() -> Result<()> {
    let listener = TcpListener::bind("192.168.121.144:41198")?;

    for stream in listener.incoming() {
        handle_client(stream?)?;
    }

    Ok(())
}

fn handle_client(tcp_in_stream: TcpStream) -> Result<()> {
    let tcp_out_stream = TcpStream::connect("192.168.121.98:41100")?;

    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()?
    );
 
    let mut tls_stream = connector
        .connect("test.com", tcp_out_stream)?;


    for b in tcp_in_stream.bytes() {
        let res = b?;
        print!("{:02x?} ", res);
        tls_stream.write(&[res])?;
    }

    Ok(())
}