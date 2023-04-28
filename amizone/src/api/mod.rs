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
use tonic::Status;

pub async fn new_amizone_connection(addr: impl ToString) -> Result<AmizoneConnection> {
    if let Ok(connection) = AmizoneServiceClient::connect(addr.to_string()).await {
        Ok(Arc::new(Mutex::new(connection)))
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
