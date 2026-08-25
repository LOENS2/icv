use crate::data::config::SickSecConfig;
use crate::data::sec_api::{
    SecChallengeRequest, SecChallengeRequestData, SecChallengeResponse, SecChallengeResponseData,
    SecPostRequest, SecPostRequestHeader, SecPostResponse, SecPostResponseHeader,
};
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Display, Formatter};

struct SickAuth {
    config: SickSecConfig,
    client: Client,
    challenge_response_data: SecChallengeResponseData,
    ha1: String,
}

enum HttpMethod {
    GET,
    POST,
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
        }
    }
}

impl SickAuth {
    const CHALLENGE_PATH: &'static str = "/getChallenge";

    pub async fn new(config: SickSecConfig) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();

        let challenge_response = Self::get_challenge(&client, config.username()).await?;

        let ha1 = Self::calc_ha1(&config, &challenge_response.challenge).await?;

        Ok(Self {
            config,
            client,
            challenge_response_data: challenge_response.challenge,
            ha1,
        })
    }

    pub async fn get(&self, path: String) -> Result<Value, Box<dyn Error>> {
        let host = self.config.host();
        let uri = format!("http://{host}{path}");

        let res = self.client.get(uri).send().await?.json::<Value>().await?;

        Ok(res)
    }

    pub async fn post(
        &self,
        path: String,
        data: Option<Value>,
    ) -> Result<Option<Value>, Box<dyn Error>> {
        let host = self.config.host();
        let uri = format!("http://{host}{path}");
        let ha1 = &self.ha1.to_string();
        let ha2 = Self::calc_ha2(HttpMethod::POST, path).await?;
        let nonce = &self.challenge_response_data.nonce;

        let response_digest = Sha256::digest(format!("{ha1}:{nonce}:{ha2}"));

        let post_request_data = SecPostRequest {
            header: SecPostRequestHeader {
                nonce: self.challenge_response_data.nonce.to_owned(),
                opaque: self.challenge_response_data.opaque.to_owned(),
                realm: self.challenge_response_data.realm.to_owned(),
                response: String::from_utf8(response_digest.to_ascii_lowercase())?,
                user: self.config.username().to_owned(),
            },
            data,
        };

        let res = self
            .client
            .post(uri)
            .json(&post_request_data)
            .send()
            .await?;

        let status = res.status();

        let res_data = res.json::<SecPostResponse>().await?;

        Self::check_status(&status, &res_data.header)?;

        Ok(res_data.data)
    }

    async fn calc_ha2(method: HttpMethod, path: String) -> Result<String, Box<dyn Error>> {
        let ha2 = Sha256::digest(format!("{method}:{path}"));

        Ok(String::from_utf8(ha2.to_ascii_lowercase())?)
    }
    async fn update_ha1(&mut self) -> Result<(), Box<dyn Error>> {
        let challenge_response = Self::get_challenge(&self.client, self.config.username()).await?;
        let challenge_response_data = challenge_response.challenge;

        self.ha1 = Self::calc_ha1(&self.config, &challenge_response_data).await?;

        self.challenge_response_data = self.challenge_response_data.clone();
        Ok(())
    }

    async fn calc_ha1(
        config: &SickSecConfig,
        sec_challenge_response_data: &SecChallengeResponseData,
    ) -> Result<String, Box<dyn Error>> {
        let username = config.username();
        let realm = &sec_challenge_response_data.realm;
        let password = config.password();
        let salt = &sec_challenge_response_data.salt;
        let ha1_base = format!("{username}:{realm}:{password}");

        let ha1 = match salt.is_empty() {
            true => Sha256::digest(ha1_base),
            false => Sha256::digest([ha1_base.as_bytes(), salt.as_slice()].concat()),
        };

        Ok(String::from_utf8(ha1.to_ascii_lowercase())?)
    }

    async fn get_challenge(
        client: &Client,
        username: &String,
    ) -> Result<SecChallengeResponse, Box<dyn Error>> {
        let request_data = SecChallengeRequest {
            data: SecChallengeRequestData {
                user: username.clone(),
            },
        };

        let response: SecChallengeResponse = client
            .post(Self::CHALLENGE_PATH)
            .json(&request_data)
            .send()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    fn check_status(
        status_code: &StatusCode,
        header: &SecPostResponseHeader,
    ) -> Result<(), Box<dyn Error>> {
        if Self::is_access_denied(status_code, header) {
            return Err("Sick SEC: Post: Access denied".into());
        }

        if !Self::is_valid(status_code, header) {
            return Err("Sick SEC: Post: Data invalid".into());
        }

        Ok(())
    }

    fn is_valid(status_code: &StatusCode, header: &SecPostResponseHeader) -> bool {
        status_code.is_success() && header.status == 0
    }

    fn is_access_denied(status_code: &StatusCode, header: &SecPostResponseHeader) -> bool {
        status_code.is_success() && header.status == 4
    }
}
