pub mod plc;

use std::{convert::Infallible, net::TcpStream};

use plc::interface;
use tokio::{io::AsyncReadExt, net::tcp::ReadHalf};
use tokio_native_tls::TlsStream;

use crate::plc::interface::PlcInterface;
//use tokio_native_tls::native_tls::TlsStream;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    

    let mut interface = interface::new("192.168.121.98:41100").await;

    tokio::spawn(async move {
        print_stream(&mut interface.read_stream.unwrap()).await;
    });


    //interface.enable_print_mode().await;

    //println!("{:?}", PlcInterface::start_sec_sevices(&mut interface.write_stream.unwrap(), "admin", "2f0dc70d").await);

    

    

    let () = std::future::pending().await;
    //print_stream(&mut interface.read_stream.unwrap()).await;
    
}


async fn print_stream(reader: &mut tokio::io::ReadHalf<TlsStream<tokio::net::TcpStream>>) {
    println!("Starting printer");
    loop {
        let read= match reader.read_u8().await {
            Ok(num) => num,
            Err(e) => {
                println!("Thread timed out.");
                return;
            },
        };

        print!("{:02x?}", read); 
        
    }
}