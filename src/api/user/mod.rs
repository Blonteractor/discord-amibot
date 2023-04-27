pub mod credentials;
use super::client::UserClient;
use super::types::*;
use credentials::Credentials;
use futures::stream::TryStreamExt;
use mongodb::{bson::doc, Client};
use serde::ser::{Serialize, SerializeStruct};

#[derive(Clone)]
pub struct User {
    id: u32,
    pub credentials: Credentials,
}

impl Serialize for User {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("User", 3)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field(
            "metadata",
            &self.credentials.get_metadata().to_str().unwrap_or_default(),
        )?;

        state.end()
    }
}

impl<'de> serde::Deserialize<'de> for User {
    fn deserialize<D>(deserializer: D) -> std::result::Result<User, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct UserVisitor;

        impl<'de> serde::de::Visitor<'de> for UserVisitor {
            type Value = User;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("struct User")
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<User, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let id = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;
                let metadata: String = seq
                    .next_element()?
                    .ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

                Ok(User {
                    id,
                    credentials: Credentials::decode_login_from_metadata(metadata).unwrap(),
                })
            }

            fn visit_map<A>(self, mut map: A) -> std::result::Result<User, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut id = None;
                let mut metadata = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
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

                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let metadata =
                    metadata.ok_or_else(|| serde::de::Error::missing_field("metadata"))?;

                let user = User {
                    id,
                    credentials: Credentials::decode_login_from_metadata(metadata).unwrap(),
                };

                Ok(user)
            }
        }

        deserializer.deserialize_struct("User", &["id", "credentials"], UserVisitor)
    }
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
        Ok(UserClient::new(self.credentials.get_metadata(), connection))
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize() {
        let example = r#"{
            "id": 1010,
            "username": "blonteractor",
            "metadata": "Basic YmxvbnRlcmFjdG9yOnZlcnluaWNlcGFzcw=="
          }"#;

        let desirialized = serde_json::from_str::<User>(example).unwrap();

        assert_eq!(desirialized.id, 1010);
        assert_eq!(desirialized.credentials.username(), "blonteractor");
        assert_eq!(desirialized.credentials.password(), "verynicepass");
    }

    #[test]
    fn serialize() {
        let example = User {
            id: 1010,
            credentials: Credentials::new("blonteractor", "verynicepass"),
        };

        println!("{}", serde_json::to_string_pretty(&example).unwrap());
    }
}
