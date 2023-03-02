use std::{process::{Command, Stdio}, io::{Read, Write, self}};

#[tokio::main]
async fn main() -> io::Result<()> {
    println!("Hello, world!");

    let mut child = Command::new("openssl")
        .arg("s_server")
        .arg("-cert /media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/certificate.pem")
        .arg("-accept 41199")
        .arg("-keyform engine")
        .arg("-engine tpm")
        .arg("-key /media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/tpmkey.pem")
        .arg("-cert_chain dev_crt_chain_modified.pem")
        .arg("-quiet")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to launch openSSL server");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = child.stdout.take().unwrap();

    
    
    let mut buf = String::new();
    stdout.read_to_string(&mut buf)?;

    println!("{}", buf);

    //Do password stuff here
    stdin.write_all(b"notasecret")?;
    
    //Single threaded listener that spawns one async and a non async handler so we only have one client at a time
    //Wait for byte stream to begin from stdout

    

    


    Ok(())

}


//openssl s_server -cert /media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/certificate.pem -accept 41199 -keyform engine -engine tpm -key /media/rfs/ro/etc/plcnext/Security/IdentityStores/IDevID/tpmkey.pem -cert_chain dev_crt_chain_modified.pem -quiet