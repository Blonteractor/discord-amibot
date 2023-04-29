use base64::{engine::general_purpose::URL_SAFE, DecodeError, Engine as _};
use mongodb::bson::{doc, Bson};
use serde::ser::{Serialize, SerializeStruct};
use tonic::metadata::AsciiMetadataValue;

pub type UserMetaData = AsciiMetadataValue;

#[derive(Clone)]
pub struct Credentials {
    username: String,
    password: String,
}

impl Serialize for Credentials {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Credentials", 1)?;
        state.serialize_field(
            "metadata",
            &self
                .get_metadata()
                .to_str()
                .unwrap_or_default()
                .split_ascii_whitespace()
                .nth(1)
                .unwrap(),
        )?;

        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for Credentials {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Credentials, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CredentialVisitor;

        impl<'de> serde::de::Visitor<'de> for CredentialVisitor {
            type Value = Credentials;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct Credentials")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Credentials, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let metadata = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

                if let Ok(creds) = Credentials::from_metadata(metadata) {
                    Ok(creds)
                } else {
                    Err(serde::de::Error::custom("Bad data received from database"))
                }
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<Credentials, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut metadata = None;
                while let Some(key) = map.next_key()? {
                    match key {
                        "metadata" => {
                            if metadata.is_some() {
                                return Err(serde::de::Error::duplicate_field("metadata"));
                            }
                            metadata = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<serde::de::IgnoredAny>()?;
                        }
                    }
                }

                let metadata =
                    metadata.ok_or_else(|| serde::de::Error::missing_field("metadata"))?;

                if let Ok(creds) = Credentials::from_metadata(metadata) {
                    Ok(creds)
                } else {
                    Err(serde::de::Error::custom("Bad data received from database"))
                }
            }
        }

        deserializer.deserialize_struct("Credentials", &["metadata"], CredentialVisitor)
    }
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
