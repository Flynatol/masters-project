use tokio::{net::UdpSocket, sync::mpsc, net::TcpStream, io::{AsyncWriteExt, AsyncReadExt, split}};
use std::{io, net::SocketAddr, sync::Arc};
use tokio_native_tls::TlsConnector;

#[tokio::main]
async fn main() -> io::Result<()> {
    let sock = UdpSocket::bind("192.168.121.144:41198".parse::<SocketAddr>().unwrap()).await?;
    let r = Arc::new(sock);
    let s = r.clone();
    let (tx, mut rx) = mpsc::channel::<SocketAddr>(1_000);
    
    let connector: TlsConnector = TlsConnector::from(
        native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true)
            .danger_accept_invalid_hostnames(true)
            .build()
            .unwrap(),
    );

    let stream_out = TcpStream::connect("192.168.121.98:41100")
        .await
        .unwrap();
        
    let tls_stream_server = connector
        .connect("googlasde.com", stream_out)
        .await
        .unwrap();


    let (mut read_tls, mut write_tls) = split(tls_stream_server);

    
    tokio::spawn(async move {
        let addr = rx.recv().await.expect("Missing return address");
        rx.close();
        loop {
            let mut buf = [0; 4096];

            if let Ok(len) = read_tls.read(&mut buf).await {
                println!("{:?} bytes received from plc: {:02x?}", len, &buf[..len]);
                let sent = s.send_to(&buf[..len], &addr).await.unwrap();
                println!("{:?} bytes sent", sent);
            } else {
		break;
	    };
        }
    });
     

    let mut buf = [0; 4096];
    loop {
        let (len, addr) = r.recv_from(&mut buf).await?;
        if !tx.is_closed() {
            tx.send(addr).await.unwrap();
        }
        println!("{:?} bytes received from {:?}: {:02x?}", len, addr, &buf[..len]);
        println!("{:?}", write_tls.write(&buf[..len]).await);
    }
}
