use std::error::Error;

fn main() -> Result<(), Box<dyn Error>>{
    println!("Hello, world!");
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let t = client.get("https://192.168.144.98/_pxc_api/v1.2/auth/auth-token")
        .send()
        .unwrap();

    Ok(())
}


struct AuthTokenResponse {
    code: String,
    expires_in: usize,
}

struct UserAuthResponse {
    token_type: String,
    access_token: String,
    roles: Vec<String>,
}
