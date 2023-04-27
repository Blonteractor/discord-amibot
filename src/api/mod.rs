pub mod types;
use futures::stream::TryStreamExt;
use mongodb::{
    bson::{doc, Bson},
    options::{ClientOptions, ServerApi, ServerApiVersion},
    Client,
};
use std::sync::Arc;
use tokio::sync::Mutex;
use types::*;

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use go_amizone::server::proto::v1::{
    amizone_service_client::AmizoneServiceClient, ClassScheduleRequest, DeregisterWifiMacRequest,
    EmptyMessage, FillFacultyFeedbackRequest, RegisterWifiMacRequest, SemesterRef,
};
use serde::{Deserialize, Serialize};
use tonic::{
    metadata::{AsciiMetadataKey, AsciiMetadataValue},
    Request, Status,
};

pub async fn new_amizone_connection(addr: impl ToString) -> Result<AmizoneConnection> {
    if let Ok(connection) = AmizoneServiceClient::connect(addr.to_string()).await {
        Ok(Arc::new(Mutex::new(connection)))
    } else {
        Err(Status::internal(
            "Couldn't establish connection to amizone API",
        ))
    }
}

pub async fn new_db_connection(addr: impl ToString) -> DbOperationResult<Client> {
    let mut client_options = ClientOptions::parse(addr.to_string()).await?;

    let server_api = ServerApi::builder().version(ServerApiVersion::V1).build();
    client_options.server_api = Some(server_api);

    let client = Client::with_options(client_options)?;

    Ok(client)
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Credentials {
    pub username: String,
    password: String,
}

impl Credentials {
    pub fn new<S: ToString>(username: S, password: S) -> Self {
        Self {
            username: username.to_string(),
            password: password.to_string(),
        }
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

#[derive(Deserialize, Serialize, Clone)]
pub struct User {
    pub id: u32,
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
                self.credentials.username, self.credentials.password
            ))
        )
        .parse()
        {
            v
        } else {
            return Err(Status::unauthenticated("Badly formatted credentials"));
        };

        Ok(UserClient {
            key,
            value,
            connection,
        })
    }
}

pub struct UserClient {
    key: AsciiMetadataKey,
    value: AsciiMetadataValue,
    connection: AmizoneConnection,
}

impl UserClient {
    pub async fn get_attendance(&mut self) -> Result<Vec<AttendanceRecord>> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_attendance(request).await?.into_inner();
        drop(amizone);

        Ok(response.records)
    }

    pub async fn get_exam_schedule(&mut self) -> Result<(String, Vec<ScheduledExam>)> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_exam_schedule(request).await?.into_inner();
        drop(amizone);

        Ok((response.title, response.exams))
    }

    pub async fn get_semesters(&mut self) -> Result<Vec<Semester>> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_semesters(request).await?.into_inner();
        drop(amizone);

        Ok(response.semesters)
    }

    pub async fn get_current_courses(&mut self) -> Result<Vec<Course>> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_current_courses(request).await?.into_inner();
        drop(amizone);

        Ok(response.courses)
    }

    pub async fn get_user_profile(&mut self) -> Result<AmizoneProfile> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_user_profile(request).await?.into_inner();
        drop(amizone);

        Ok(response)
    }

    pub async fn get_wifi_mac_info(&mut self) -> Result<WifiMacInfo> {
        let mut request = Request::new(EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_wifi_mac_info(request).await?.into_inner();
        drop(amizone);

        Ok(response)
    }

    pub async fn get_courses(&mut self, num: usize) -> Result<Vec<Course>> {
        let mut request = Request::new(SemesterRef {
            semester_ref: num.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_courses(request).await?.into_inner();
        drop(amizone);

        Ok(response.courses)
    }

    pub async fn register_wifi_mac(&mut self, addr: impl ToString) -> Result<()> {
        let mut request = Request::new(RegisterWifiMacRequest {
            address: addr.to_string(),
            override_limit: true,
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        amizone.register_wifi_mac(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn deregister_wifi_mac(&mut self, addr: impl ToString) -> Result<()> {
        let mut request = Request::new(DeregisterWifiMacRequest {
            address: addr.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        amizone.deregister_wifi_mac(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn fill_faculty_feedback(
        &mut self,
        rating: i32,
        query_rating: i32,
        comment: impl ToString,
    ) -> Result<()> {
        let mut request = Request::new(FillFacultyFeedbackRequest {
            rating,
            query_rating,
            comment: comment.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        amizone.fill_faculty_feedback(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn get_class_schedule(&mut self, date: Date) -> Result<Vec<ScheduledClass>> {
        let mut request = Request::new(ClassScheduleRequest { date: Some(date) });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock().await;
        let response = amizone.get_class_schedule(request).await?.into_inner();
        drop(amizone);
        Ok(response.classes)
    }
}
