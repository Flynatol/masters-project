pub mod interface {
    use std::{option::Option, net::{TcpStream, ToSocketAddrs}, io::{Write, Error, Read}};
    use colored::Colorize;
    use hex::decode;
    use native_tls::{TlsStream, TlsConnector};




    pub struct PlcInterface {
        pub stream: Option<TlsStream<TcpStream>>,
        key: Option<[u8; 4]>
    }

    pub fn new<A : ToSocketAddrs>(address : A) -> PlcInterface {

        let connector: TlsConnector = TlsConnector::from(
            native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true)
                .danger_accept_invalid_hostnames(true)
                .build()
                .unwrap(),
        );
    
        let stream_out = TcpStream::connect(address).unwrap();
            
            
        let tls_stream_server = connector
            .connect("example.com", stream_out).unwrap();
        
        PlcInterface {
            stream: Some(tls_stream_server),
            key: None,
        }
    }

    impl PlcInterface {

        pub fn blocking_write<'a>(&mut self, src: &'a [u8]) -> Result<(), Error> {
            return self.stream.as_mut().unwrap().write_all(src);
        }

        pub fn blocking_read<'a>(&mut self) -> Result<Vec<u8>, Error> {
            let mut buffer: [u8; 2048] = [0; 2048];

            self.stream.as_mut().unwrap()
            .read(&mut buffer)
            .map(|num_bytes| buffer[..num_bytes].to_vec())
        }

        pub fn write_read<'a>(&mut self, src: &'a [u8]) -> Result<Vec<u8>, Error> {
            match self.blocking_write(src) {
                Ok(_) => self.blocking_read(),
                Err(e) => Err(e),
            }
        }

        pub fn set_exten_bool<'a>(&mut self, var: &'a [u8], state: bool) {
            println!("{:02x?}", self.write_read(&decode(
                format!("0448020004000000070005000c00ddb7{}08010000001413050000000e0050726f436f6e4f535f65434c520002007b000e00526573476c6f62616c566172730002007b00{:02x?}00{}00141c0100000002{:02x?}ff", 
                hex::encode(self.key.unwrap()), var.len()+1, hex::encode(var), if state {1} else {0})).unwrap()));
        }

        pub fn login<'a>(&mut self, username: &'a [u8], password: &'a [u8]) -> Result<&[u8; 4], Error> {
            println!("{:02x?}", self.write_read(&decode("04c0070000000000000000000000f43f").unwrap())); //?
            println!("{:02x?}", self.write_read(&decode("04400d0000000400000000000000eabf41727000").unwrap()));//Arp
            println!("{:02x?}", self.write_read(&decode("04400b0000002e00000000000000c2bf02004172702e506c632e446f6d61696e2e53657276696365732e49506c634d616e61676572536572766963653200").unwrap()));//IPlcManSer2
            
            println!("{:02x?}", self.write_read(&decode("0440020000000000010005000200f1bfff").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400d0000000900000000000000e5bf536563757269747900").unwrap())); //Sec
            println!("{:02x?}", self.write_read(&decode("04400b0000003e00000000000000b2bf0d004172702e53797374656d2e53656375726974792e53657276696365732e4950617373776f726441757468656e7469636174696f6e5365727669636500").unwrap()));//Ipasswordauth
            println!("{:02x?}", self.write_read(&decode("04400b0000003c00000000000000b4bf0d004172702e53797374656d2e53656375726974792e53657276696365732e49536563757269747953657373696f6e496e666f536572766963653300").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400b0000003c00000000000000b4bf0d004172702e53797374656d2e53656375726974792e53657276696365732e49536563757269747953657373696f6e496e666f536572766963653200").unwrap()));
            println!("{:02x?}", self.write_read(&decode("0440020000000000030003000d00e6bfff").unwrap()));

            println!("{:02x?}", self.write_read(&decode("04400d0000000500000000000000e9bf45636c7200").unwrap())); //Eclr
            println!("{:02x?}", self.write_read(&decode("04400b0000003c00000000000000b4bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4950726f436f6e4f53436f6e74726f6c536572766963653200").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400b0000003d00000000000000b3bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4953696d706c6546696c65416363657373536572766963653300").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400b0000003700000000000000b9bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e49446576696365496e666f536572766963653200").unwrap()));
            println!("{:02x?}", self.write_read(&decode("0440020000000000060003000c00e4bf1406010000000240ff").unwrap()));
            println!("{:02x?}", self.write_read(&decode("0440020000000000060003000c00e4bf1406010000001640ff").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400b0000003700000000000000b9bf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e4944617461416363657373536572766963653300").unwrap()));
            println!("{:02x?}", self.write_read(&decode("04400b0000003600000000000000babf0c004164652e436f6d6d6f6e52656d6f74696e672e45636c722e53657276696365732e49427265616b706f696e745365727669636500").unwrap())); //Breakpoint service
            println!("{:02x?}", self.write_read(&decode("04400b0000003e00000000000000b2bf02004172702e53797374656d2e53656375726974792e53657276696365732e4950617373776f7264436f6e66696775726174696f6e536572766963653200").unwrap()));//Ipasswordconf2
            println!("{:02x?}", self.write_read(&decode("04400b0000003300000000000000bdbf02004172702e4465766963652e496e746572666163652e53657276696365732e49446576696365496e666f5365727669636500").unwrap()));//IdeviceInfoservice
            println!("{:02x?}", self.write_read(&decode("04400b0000003800000000000000b8bf02004172702e53797374656d2e53656375726974792e53657276696365732e49536563757265446576696365496e666f5365727669636500").unwrap())); //secDeviceInfoService

            println!("{:02x?}", self.write_read(&decode("04400200000000000b0001000200ebbfff").unwrap())); 
            println!("{:02x?}", self.blocking_read());
            println!("{:02x?}", self.blocking_read());
            let key_message = self.write_read(&hex::decode(format!("0440020000000000020001000d00e9bf13{:02x?}00{}0024{:02x?}00{}00ff", username.len() + 1, hex::encode(username), password.len() + 1, hex::encode(password))).unwrap());
            println!("{:02x?}", key_message);
            
            //Perform some verification to ensure we've got a good key
            return match key_message {
                Ok(vec) => {
                    let res: Result<&[u8; 4], Error> = if &vec[15..18] == [191, 39, 09] {
                        self.key = Some(vec[18..22].to_owned().try_into().unwrap());
                        Ok(&self.key.as_ref().unwrap())
                    } else {
                        //println!("{:02x?}", &vec[15..17]);
                        Err(Error::new(std::io::ErrorKind::Other, "Authentication failed"))
                    };
                    res
                },
                Err(e) => Err(e),
            }

        }

    }




}