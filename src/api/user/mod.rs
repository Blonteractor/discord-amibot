pub mod credentials;

use super::client::UserClient;
use super::types::*;
use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use credentials::Credentials;
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, Client};
use serde::{Deserialize, Serialize};
use tonic::{
    metadata::{AsciiMetadataKey, AsciiMetadataValue},
    Status,
};

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    id: u32,
    pub credentials: Credentials,
}

impl User {
    pub async fn new<S: ToString>(
        id: u32,
        username: S,
        password: S,
        mongo_client: &Client,
    ) -> DbOperationResult<Self> {
        if let Some(user) = Self::from_id(id, mongo_client).await? {
            Ok(user)
        } else {
            let db = mongo_client.database("users");
            let creds = db.collection::<User>("credentials");
            let object = Self {
                id,
                credentials: Credentials::new(username, password),
            };
            creds.insert_one(object.clone(), None).await?;
            Ok(object)
        }
    }

    pub async fn forget(id: u32, mongo_client: &Client) -> DbOperationResult<Option<User>> {
        let db = mongo_client.database("users");
        let creds = db.collection::<User>("credentials");

        creds.find_one_and_delete(doc! { "id": id }, None).await
    }

    pub async fn update<S: ToString>(
        id: u32,
        username: S,
        password: S,
        mongo_client: &Client,
    ) -> DbOperationResult<Option<User>> {
        let db = mongo_client.database("users");
        let creds = db.collection::<User>("credentials");

        creds
            .find_one_and_update(
                doc! { "id": id },
                doc! { "$set": { "credentials": Credentials::new(username, password) } },
                None,
            )
            .await
    }

    pub async fn from_id(id: u32, mongo_client: &Client) -> DbOperationResult<Option<Self>> {
        let db = mongo_client.database("users");
        let creds = db.collection::<User>("credentials");

        let mut cursor = creds.find(doc! { "id": id }, None).await?;

        if let Some(user) = cursor.try_next().await? {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub fn get_client(&self, connection: AmizoneConnection) -> Result<UserClient> {
        let key = AsciiMetadataKey::from_static("authorization");

        let value: AsciiMetadataValue = if let Ok(v) = format!(
            "Basic {}",
            URL_SAFE.encode(format!(
                "{}:{}",
                self.credentials.username(),
                self.credentials.password()
            ))
        )
        .parse()
        {
            v
        } else {
            return Err(Status::unauthenticated("Badly formatted credentials"));
        };

        Ok(UserClient::new(key, value, connection))
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}
