mod tests;

use crate::error::Error;
use http::Response;
use hyper::{body::HttpBody as _, client::HttpConnector, Body, Client};
use hyper_tls::HttpsConnector;
use serde::Deserialize;

/// https://github.com/nextdotid/proof-server/blob/master/docs/api.apib
#[derive(Deserialize, Debug)]
pub struct ProofQueryResponse {
    pub ids: Vec<ProofPersona>,
}

#[derive(Deserialize, Debug)]
pub struct ProofPersona {
    pub persona: String,
    pub proofs: Vec<Proof>,
}

#[derive(Deserialize, Debug)]
pub struct Proof {
    pub platform: String,
    pub identity: String,
    pub created_at: String,
    pub last_checked_at: String,
    pub is_valid: bool,
    pub invalid_reason: String,
}

#[derive(Deserialize, Debug)]
pub struct ErrorResponse {
    pub message: String
}

pub fn make_client() -> Client<HttpsConnector<HttpConnector>> {
    let https = HttpsConnector::new();
    let client = Client::builder().build::<_, hyper::Body>(https);
    client
}

async fn parse_body<T>(resp: &mut Response<Body>) -> Result<T, Error>
where
    T: for<'de> Deserialize<'de>,
{
    let mut body_bytes: Vec<u8> = vec![];
    while let Some(chunk) = resp.body_mut().data().await {
        let mut chunk_bytes = chunk.unwrap().to_vec();
        body_bytes.append(&mut chunk_bytes);
    }
    let body = std::str::from_utf8(&body_bytes).unwrap();

    Ok(serde_json::from_str(&body)?)
}

pub async fn query(base: &str, persona: &str) -> Result<ProofQueryResponse, Error> {
    let client = make_client();
    let uri = format!("{}/v1/proof?platform=nextid&identity={}", base, persona)
        .parse()
        .unwrap();
    let mut resp = client.get(uri).await?;
    if !resp.status().is_success() {
        let body: ErrorResponse = parse_body(&mut resp).await?;
        return Err(Error::General(
            format!("ProofService error: {}", body.message),
            resp.status(),
        ));
    }
    let body: ProofQueryResponse = parse_body(&mut resp).await?;
    Ok(body)
}
