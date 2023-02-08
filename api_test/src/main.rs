use std::error::Error;
use reqwest::blocking::Client;
use std::collections::HashMap;
use serde::Deserialize;

fn main() -> Result<(), Box<dyn Error>>{
    println!("Hello, world!");
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let mut auth_body = HashMap::new();
    auth_body.insert("scope", "variables");

    let t : AuthTokenResponse = client.clone().post("https://192.168.121.98/_pxc_api/v1.2/auth/auth-token")
        .json(&auth_body)
        .send()
        .unwrap()
        .json()?;

    println!("{:?}", t);

    let mut user_auth_body = HashMap::new();
    user_auth_body.insert("code", t.code.as_str());
    user_auth_body.insert("grant_type", "authorization_code");
    user_auth_body.insert("username", "tester");
    user_auth_body.insert("password", "tester");

    let t = client.post("https://192.168.121.98/_pxc_api/v1.2/auth/access-token")
        .json(&user_auth_body)
        .send()
        .unwrap();
    //.text()?; 

    println!("{:?}", t.headers());

    Ok(())
}

#[derive(Deserialize, Debug)]
struct AuthTokenResponse {
    code: String,
    expires_in: usize,
}

#[derive(Deserialize, Debug)]
struct UserAuthResponse {
    //token_type: String,
    access_token: String,
    roles: Vec<String>,
}
