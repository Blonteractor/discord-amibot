use base64::{
    engine::general_purpose::{STANDARD, URL_SAFE},
    DecodeError, Engine as _,
};
use mongodb::bson::{doc, Bson};
use once_cell::sync::Lazy;
use serde::ser::{Serialize, SerializeStruct};
use tonic::metadata::AsciiMetadataValue;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};

use rand::RngCore;

pub type UserMetaData = AsciiMetadataValue;

static PRIVATE_KEY: Lazy<Vec<u8>> = Lazy::new(|| {
    STANDARD
        .decode(std::fs::read_to_string("./PRIVATE_KEY").expect("Failed to read encrytion key"))
        .expect("Bad encryption key")
    // STANDARD
    //     .decode(std::env::var("PRIVATE_ENCRYPTION_KEY").unwrap())
    //     .unwrap()
});

///The recommendation to use a 12-byte nonce for AES-GCM encryption comes from
/// the National Institute of Standards and Technology (NIST)
/// which specifies the algorithm in Special Publication 800-38D:
/// "Recommendation for Block Cipher Modes of Operation: Galois/Counter Mode (GCM) and GMAC"
/// (source: https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38d.pdf).
const NONCE_SIZE: usize = 12;

/// The nonce length when it is encoded to base 64
const NONCE_LENGTH: usize = ((NONCE_SIZE + 2) / 3) * 4;

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
        state.serialize_field("metadata", &self.get_metadata())?;

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

    pub fn get_auth(&self) -> AsciiMetadataValue {
        format!(
            "Basic {}",
            URL_SAFE.encode(format!("{}:{}", self.username, self.password))
        )
        .parse()
        .unwrap()
    }

    /// Metadata format (hopefully)
    /// 0-15: Nonce, Base64 encoded
    /// rest: encrypted creds, Base64 decode before decrypting
    pub fn get_metadata(&self) -> String {
        let cipher = Aes256Gcm::new(PRIVATE_KEY.as_slice().into());
        let (nonce, metadata) =
            Self::encrypt(&cipher, format!("{}:{}", self.username(), self.password()));

        let s = format!("{}{}", STANDARD.encode(nonce), STANDARD.encode(metadata));
        s
    }

    pub fn from_metadata(metadata: String) -> Result<Self, DecodeError> {
        let cipher = Aes256Gcm::new(PRIVATE_KEY.as_slice().into());

        // 12 byte nonce corresponds to 16 length string
        // First 16 bytes encoded into base64 is the nonce
        let nonce = STANDARD.decode(&metadata[0..NONCE_LENGTH]).unwrap();
        let secret = STANDARD.decode(&metadata[NONCE_LENGTH..]).unwrap();
        let secret = Self::decrypt(&cipher, nonce, secret);

        // 58 is the ASCII code for ':'
        // metadata is in the form username:password
        let mut split_at_colon = secret.split(|ascii| ascii == &58u8);

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

    // Thanks to @ditsuke for the crypto stuff
    /// Returns (nonce, encrypted data)
    fn encrypt<S: AsRef<str>>(cipher: &Aes256Gcm, secret: S) -> (Vec<u8>, Vec<u8>) {
        let mut rng = OsRng;

        // 12 byte nonce
        let mut nonce = [0u8; NONCE_SIZE];
        rng.fill_bytes(&mut nonce);

        let encrypted_credentials = cipher
            .encrypt(Nonce::from_slice(&nonce), secret.as_ref().as_ref())
            .unwrap();

        (nonce.into(), encrypted_credentials)
    }

    fn decrypt<S: AsRef<[u8]>>(cipher: &Aes256Gcm, nonce: S, encrypted: S) -> Vec<u8> {
        cipher
            .decrypt(
                Nonce::from_slice(nonce.as_ref()),
                encrypted.as_ref().as_ref(),
            )
            .unwrap()
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
