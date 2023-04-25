include!(concat!(env!("OUT_DIR"), "/_includes.rs"));

use go_amizone::server::proto::v1::amizone_service_client::AmizoneServiceClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut amizone = AmizoneServiceClient::connect("http://0.0.0.0:8081").await?;

    Ok(())
}
