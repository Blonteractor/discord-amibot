pub mod client;
pub mod types;
pub mod user;
use mongodb::{
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::*;

use go_amizone::server::proto::v1::amizone_service_client::AmizoneServiceClient;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Status;

pub async fn new_amizone_connection(addr: impl ToString) -> Result<AmizoneConnection> {
    let pem = std::fs::read_to_string("./tls/lets-encrypt.pem")
        .map_err(|_| Status::internal("Error reading TLS cert"))?;

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(Certificate::from_pem(pem))
        .domain_name("amizone.fly.dev");

    if let Ok(ch) = Channel::from_shared(addr.to_string())
        .map_err(|_| Status::internal("Invalid URL for amizone backend"))?
        .tls_config(tls_config)
        .map_err(|_| Status::internal("Invlaid TLS config"))?
        .connect()
        .await
    {
        Ok(Arc::new(Mutex::new(AmizoneServiceClient::new(ch))))
    } else {
        Err(Status::internal(
            "Couldn't establish connection to amizone API",
        ))
    }
}

pub async fn new_db_connection(addr: impl ToString) -> DbOperationResult<DatabaseConnection> {
    let mut client_options = ClientOptions::parse(addr.to_string()).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    Ok(client)
}
