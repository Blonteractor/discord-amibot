pub mod api;

#[cfg(test)]
mod test {
    use super::api::{new_db_connection, user::User};
    use dotenv::dotenv;

    #[tokio::test]
    async fn crud() {
        dotenv().ok();

        const ID: &'static str = "486590017150517248";
        let client = new_db_connection(std::env::var("DATABASE_URL").unwrap().to_string())
            .await
            .unwrap();

        User::new(ID, "samepluser", "samplepass", &client)
            .await
            .unwrap();

        let new_user = User::from_id(ID, &client).await.unwrap().unwrap();

        assert_eq!(new_user.id(), ID);

        let updated_user = User::update(ID, "samepluserupdated", "samplepassupdated", &client)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(updated_user.credentials.username(), "samepluserupdated");

        User::forget(ID, &client).await.unwrap();
    }
}
