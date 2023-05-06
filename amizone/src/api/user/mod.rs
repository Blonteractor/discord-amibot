use futures::stream::TryStreamExt;
pub mod credentials;
use super::client::UserClient;
use super::types::*;
use credentials::Credentials;
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};

static DATABSE_NAME: &str = "amibot_users";
static COLLECTION_NAME: &str = "login_credentials";

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    id: String,

    #[serde(flatten)]
    pub credentials: Credentials,
}

impl User {
    pub async fn new<S: ToString>(
        id: S,
        username: S,
        password: S,
        mongo_client: &DatabaseConnection,
    ) -> DbOperationResult<Self> {
        if let Some(user) = Self::from_id(id.to_string(), mongo_client).await? {
            Ok(user)
        } else {
            let db = mongo_client.database(DATABSE_NAME);
            let creds = db.collection::<User>(COLLECTION_NAME);
            let object = Self {
                id: id.to_string(),
                credentials: Credentials::new(username, password),
            };
            creds.insert_one(object.clone(), None).await?;
            Ok(object)
        }
    }

    pub async fn forget(
        id: impl ToString,
        mongo_client: &DatabaseConnection,
    ) -> DbOperationResult<Option<User>> {
        let db = mongo_client.database(DATABSE_NAME);
        let creds = db.collection::<User>(COLLECTION_NAME);

        let result = creds
            .find_one_and_delete(doc! { "id": id.to_string() }, None)
            .await;

        Self::sanitize_result(id, mongo_client, result).await
    }

    async fn sanitize_result(
        id: impl ToString,
        mongo_client: &DatabaseConnection,
        result: DbOperationResult<Option<User>>,
    ) -> DbOperationResult<Option<User>> {
        if let Err(err) = &result {
            match err.kind.as_ref() {
                mongodb::error::ErrorKind::BsonDeserialization(_) => {
                    // Attempt requery using from_id
                    User::from_id(id, mongo_client).await
                }
                _ => result,
            }
        } else {
            result
        }
    }

    pub async fn update<S: ToString>(
        id: S,
        username: S,
        password: S,
        mongo_client: &DatabaseConnection,
    ) -> DbOperationResult<Option<User>> {
        let db = mongo_client.database(DATABSE_NAME);
        let creds = db.collection::<User>(COLLECTION_NAME);

        let result = creds
            .find_one_and_update(
                doc! { "id": id.to_string() },
                doc! { "$set": { "metadata": Credentials::new(username, password).get_metadata() } },
                None,
            )
            .await;

        Self::sanitize_result(id, mongo_client, result).await
    }

    pub async fn from_id<S: ToString>(
        id: S,
        mongo_client: &DatabaseConnection,
    ) -> DbOperationResult<Option<Self>> {
        let db = mongo_client.database(DATABSE_NAME);
        let creds = db.collection::<User>(COLLECTION_NAME);

        let mut cursor = creds.find(doc! { "id": id.to_string() }, None).await?;

        if let Some(user) = cursor.try_next().await? {
            Ok(Some(user))
        } else {
            Ok(None)
        }
    }

    pub fn get_client(&self, connection: AmizoneConnection) -> Result<UserClient> {
        Ok(UserClient::new(self.credentials.get_auth(), connection))
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dotenv::dotenv;

    static PASS: &'static str = "$196*(^%@1DSjDSx@";
    static USERNAME: &'static str = "sampleuser";
    static ID: &'static str = "619800189372465153";

    #[test]
    fn deserialize() {
        dotenv().ok();
        let example = r#"{
            "id": "619800189372465153",
            "metadata": "Gc47KbQk36YLrkxmxAZhYbqWWOR4cnSX9xaIRo1nC9R5h7t8OHS6mZyzlPfMh0vHiNw1oM6QG20="
          }"#;

        let desirialized = serde_json::from_str::<User>(example).unwrap();

        assert_eq!(desirialized.id, ID);
        assert_eq!(desirialized.credentials.username(), USERNAME);
        assert_eq!(desirialized.credentials.password(), PASS);
    }

    #[test]
    fn serialize() {
        dotenv().ok();
        let example = User {
            id: ID.to_string(),
            credentials: Credentials::new(USERNAME, PASS),
        };

        println!("{}", serde_json::to_string_pretty(&example).unwrap());
    }
}
