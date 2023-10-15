use peroxide::fuga::*;
use reqwest::{
    self,
    header::{CONTENT_TYPE, HeaderMap, HeaderValue},
};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

fn main() {
    let df = DataFrame::read_parquet("./api.parquet").unwrap();
    let api_key: String = df["api_key"].at_raw(0);
    let api_secret: String = df["api_secret"].at_raw(0);

    let token = oauth2_token(&api_key, &api_secret);
    println!("{}", token);
}

pub fn oauth2_token(api_key: &str, api_secret: &str) -> String {
    let url_base = "https://openapivts.koreainvestment.com:29443";
    let token_url = format!("{}/oauth2/tokenP", url_base);

    // content-type : application/json
    let mut headers = HeaderMap::new();
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    let mut body = HashMap::new();
    body.insert("grant_type", "client_credentials");
    body.insert("appkey", api_key);
    body.insert("appsecret", api_secret);

    let client = reqwest::blocking::Client::new();
    let res = client
        .post(&token_url)
        .headers(headers)
        .json(&body)
        .send()
        .unwrap();

    let token: OAuth2 = res.json().unwrap();
    token.get_access_token().to_string()
}

#[derive(Debug, Serialize, Deserialize)]
struct OAuth2 {
    access_token: String,
    token_type: String,
    expires_in: i32,
}

impl OAuth2 {
    pub fn get_access_token(&self) -> &str {
        &self.access_token
    }
}
