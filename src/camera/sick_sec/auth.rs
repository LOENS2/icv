use crate::data::config::SickSecConfig;
use crate::data::sec_api::{
    SecChallengeRequest, SecChallengeRequestData, SecChallengeResponse, SecChallengeResponseData,
    SecPostRequest, SecPostRequestHeader, SecPostResponse, SecPostResponseHeader,
};
use bytes::Bytes;
use reqwest::{Client, StatusCode};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::error::Error;
use std::fmt::{Display, Formatter};

pub struct SecAuth {
    config: SickSecConfig,
    client: Client,
    challenge_response_data: SecChallengeResponseData,
    ha1: String,
}

enum HttpMethod {
    GET,
    POST,
}

pub enum ResponseType {
    Json { data: Option<Value> },
    Jpeg { data: Bytes },
    Mp4 { data: Bytes },
    PlainText { data: String },
    OctetStream { data: Bytes },
}

impl Display for HttpMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            HttpMethod::GET => write!(f, "GET"),
            HttpMethod::POST => write!(f, "POST"),
        }
    }
}

impl SecAuth {
    const CHALLENGE_PATH: &'static str = "/getChallenge";

    pub async fn new(config: SickSecConfig) -> Result<Self, Box<dyn Error>> {
        let client = Client::new();

        let challenge_response =
            Self::get_challenge(&config.host(), &client, config.username()).await?;

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
        let uri = format!("http://{host}/api{path}");

        let res = self.client.get(uri).send().await?.json::<Value>().await?;

        Ok(res)
    }

    pub async fn post(
        &self,
        path: &str,
        data: Option<Value>,
    ) -> Result<ResponseType, Box<dyn Error>> {
        let host = self.config.host();
        let uri = format!("http://{host}/api{path}");
        let ha1 = &self.ha1.to_string();
        let ha2 = Self::calc_ha2(HttpMethod::POST, path).await?;
        let nonce = &self.challenge_response_data.nonce;

        let response_digest = Sha256::digest(format!("{ha1}:{nonce}:{ha2}"));

        let post_request_data = SecPostRequest {
            header: SecPostRequestHeader {
                nonce: self.challenge_response_data.nonce.to_owned(),
                opaque: self.challenge_response_data.opaque.to_owned(),
                realm: self.challenge_response_data.realm.to_owned(),
                response: hex::encode(response_digest),
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

        let content_type = match res.headers().get("Content-Type") {
            Some(content_type) => content_type,
            None => Err("Incorrect or unsupported content type received")?,
        };

        match content_type.to_str()? {
            "application/json" => {
                let status = res.status();
                let res_data = res.json::<SecPostResponse>().await?;
                Self::check_status(&status, &res_data.header)?;

                Ok(ResponseType::Json {
                    data: res_data.data,
                })
            }
            "image/jpeg" => {
                let bytes = res.bytes().await?;
                Self::validate_jpeg_bytes(&bytes)?;

                Ok(ResponseType::Jpeg { data: bytes })
            }
            "video/mp4" => {
                let bytes = res.bytes().await?;
                Self::validate_mp4_bytes(&bytes)?;

                Ok(ResponseType::Mp4 { data: bytes })
            }
            "text/plain" => Ok(ResponseType::PlainText {
                data: res.text().await?,
            }),
            "application/octet-stream" => Ok(ResponseType::OctetStream {
                data: res.bytes().await?,
            }),
            unsupported => {
                Err(format!("Incorrect or unsupported content type received: {unsupported}").into())
            }
        }
    }

    async fn calc_ha2(method: HttpMethod, path: &str) -> Result<String, Box<dyn Error>> {
        let ha2 = Sha256::digest(format!("{method}:{path}"));

        Ok(hex::encode(ha2))
    }
    async fn update_ha1(&mut self) -> Result<(), Box<dyn Error>> {
        let challenge_response =
            Self::get_challenge(&self.config.host(), &self.client, self.config.username()).await?;
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

        Ok(hex::encode(ha1))
    }

    async fn get_challenge(
        host: &String,
        client: &Client,
        username: &str,
    ) -> Result<SecChallengeResponse, Box<dyn Error>> {
        let request_data = SecChallengeRequest {
            data: SecChallengeRequestData {
                user: username.to_owned(),
            },
        };

        let path = Self::CHALLENGE_PATH;
        let uri = format!("http://{host}/api{path}");

        let response: SecChallengeResponse = client
            .post(uri)
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

    fn validate_jpeg_bytes(bytes: &Bytes) -> Result<(), Box<dyn Error>> {
        if bytes.len() < 3 || &bytes[0..3] != &[0xFF, 0xD8, 0xFF] {
            return Err("Invalid JPEG signature".into());
        }
        Ok(())
    }

    fn validate_mp4_bytes(bytes: &Bytes) -> Result<(), Box<dyn Error>> {
        if bytes.len() < 8 || &bytes[4..8] != b"ftyp" {
            return Err("Invalid MP4 signature".into());
        }
        Ok(())
    }
}
