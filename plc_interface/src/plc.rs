pub mod interface {
    use std::option::Option;
    use replace_stream::replace_mod::{self, ReplaceStream, replacment_builder};
    use tokio::io::{self, WriteHalf, ReadHalf, AsyncWriteExt, AsyncReadExt};
    use tokio::net::{TcpStream, ToSocketAddrs};
    use tokio::runtime::Handle;
    use tokio::task::block_in_place;
    use tokio_native_tls::{TlsConnector, TlsStream};
    use tokio_stream::StreamExt;
    use tokio_util::io::ReaderStream;
    use colored::Colorize;




    pub struct PlcInterface {
        pub read_stream: Option<ReadHalf<TlsStream<TcpStream>>>,
        pub write_stream: Option<WriteHalf<TlsStream<TcpStream>>>,
    }

    pub async fn new<A : ToSocketAddrs>(address : A) -> PlcInterface {

        let connector: TlsConnector = TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap(),
        );
    
        let stream_out = TcpStream::connect(address).await.unwrap();
            
            
        let tls_stream_server = connector
            .connect("example.com", stream_out).await.unwrap();

        let (server_read_tls, mut server_write_tls) = io::split(tls_stream_server);
        
        PlcInterface {
            read_stream: Some(server_read_tls),
            write_stream: Some(server_write_tls),
        }
    }

    impl PlcInterface {

        pub fn blocking_write<'a>(&mut self, src: &'a [u8]) {
            let f = self.write_stream.as_mut().unwrap().write_all(src);
        }

        pub async fn start_sec_sevices(write_stream: &mut tokio::io::WriteHalf<TlsStream<tokio::net::TcpStream>>, username: &str, password: &str) -> io::Result<()> {

            //if let Some(write_stream) = &mut self.write_stream {
                println!("Writing");
                //write_stream.write_all(&hex::decode("0300000000002000000000000000dcff52656d6f74696e6756657273696f6e446574656374696f6e5365727669636500").unwrap());
                
                println!("{:?}", 
                write_stream.write_all(&hex::decode("04c0070000000000000000000000f43f").unwrap()).await);
                std::thread::sleep(core::time::Duration::from_millis(100));
                write_stream.write_all(&hex::decode("04400d0000000400000000000000eabf417270004172702e506c632e446f6d61696e2e53657276696365732e49506c634d616e61676572536572766963653200").unwrap()).await?;
                std::thread::sleep(core::time::Duration::from_millis(100));
                write_stream.write_all(&hex::decode("04400b0000002e00000000000000c2bf02004172702e506c632e446f6d61696e2e53657276696365732e49506c634d616e61676572536572766963653200").unwrap()).await?;

                //write_stream.write_all(&hex::decode("0440020000000000010005000200f1bfff").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400d0000000900000000000000e5bf536563757269747900").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003e00000000000000b2bf0d004172702e53797374656d2e53656375726974792e53657276696365732e4950617373776f726441757468656e7469636174696f6e5365727669636500").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003c00000000000000b4bf0d004172702e53797374656d2e53656375726974792e53657276696365732e49536563757269747953657373696f6e496e666f536572766963653300").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003c00000000000000b4bf0d004172702e53797374656d2e53656375726974792e53657276696365732e49536563757269747953657373696f6e496e666f536572766963653200").unwrap()).await?;
                //write_stream.write_all(&hex::decode("0440020000000000030003000d00e6bfff").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400d0000000500000000000000e9bf45636c7200").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003c00000000000000b4bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4950726f436f6e4f53436f6e74726f6c536572766963653200").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003d00000000000000b3bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4953696d706c6546696c65416363657373536572766963653300").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003700000000000000b9bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e49446576696365496e666f536572766963653200").unwrap()).await?;
                //write_stream.write_all(&hex::decode("0440020000000000060003000c00e4bf1406010000000240ff").unwrap()).await?;
                //write_stream.write_all(&hex::decode("0440020000000000060003000c00e4bf1406010000001640ff").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003700000000000000b9bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4944617461416363657373536572766963653300").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003600000000000000babf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e49427265616b706f696e745365727669636500").unwrap()).await?;
                
                //write_stream.write_all(&hex::decode("04400b0000003e00000000000000b2bf02004172702e53797374656d2e53656375726974792e53657276696365").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003300000000000000bdbf02004172702e4465766963652e496e746572666163652e53657276696365732e49446576696365496e666f5365727669636500").unwrap()).await?;
                //write_stream.write_all(&hex::decode("04400b0000003800000000000000b8bf02004172702e53797374656d2e53656375726974792e53657276696365732e49536563757265446576696365496e666f5365727669636500").unwrap()).await?;
                //write_stream.write_all(&hex::decode(format!("0440020000000000020001000d00e9bf130600{}00240900{}00ff", hex::encode(username), hex::encode(password))).unwrap()).await?; //I have a feeling length of stuff is encoded in this, to test
                

            //}

            Ok(())
        }

        //This function takes our reader
        pub async fn enable_print_mode(&mut self) {
            let mut reader = std::mem::replace(&mut self.read_stream, None).unwrap();

            tokio::spawn(async move {
                while let Ok(read) = reader.read_u8().await {
                    print!("{}", format!(" {:02x?}", read).color("red"));
                }
            });
        }


        async fn set_extern_bool(&mut self, var: &str, state: bool) {
            
        }
    }




}