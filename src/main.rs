include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
use amibot::api;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let user = api::User::new(121212, "", "");
    let connection = api::new_connection("https://0.0.0.0:8081").await?;

    let mut client = user.get_client(connection)?;

    let sems = client
        .get_class_schedule(api::types::Date {
            year: 2023,
            day: 19,
            month: 4,
        })
        .await
        .unwrap();

    println!("{:#?}", sems);

    Ok(())
}
