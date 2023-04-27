use base64::{engine::general_purpose::URL_SAFE, DecodeError, Engine as _};
use mongodb::bson::{doc, Bson};
use serde::{Deserialize, Serialize};
use tonic::metadata::AsciiMetadataValue;

pub type UserMetaData = AsciiMetadataValue;

#[derive(Deserialize, Serialize, Clone)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Credentials {
    pub fn new<S: ToString>(username: S, password: S) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    pub fn get_metadata(&self) -> AsciiMetadataValue {
        format!(
            "Basic {}",
            URL_SAFE.encode(format!("{}:{}", self.username, self.password))
        )
        .parse()
        .unwrap()
    }

    pub fn from_metadata(metadata: String) -> Result<Self, DecodeError> {
        let decoded = URL_SAFE.decode(metadata)?;

        // 58 is the ASCII code for ':'
        // metadata is in the form username:password
        let mut split_at_colon = decoded.split(|ascii| ascii == &58u8);

        Ok(Self::new(
            String::from_utf8_lossy(split_at_colon.next().unwrap()),
            String::from_utf8_lossy(split_at_colon.next().unwrap()),
        ))
    }

    pub fn username(&self) -> &str {
        self.username.as_ref()
    }

    pub fn password(&self) -> &str {
        self.password.as_ref()
    }
}

impl From<Credentials> for Bson {
    fn from(credentials: Credentials) -> Self {
        let doc = doc! {
            "username": credentials.username,
            "password": credentials.password,
        };
        Bson::Document(doc)
    }
}
