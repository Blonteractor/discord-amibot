use std::sync::{Arc, Mutex};

use base64::{engine::general_purpose::URL_SAFE, Engine as _};
use go_amizone::server::proto::v1::{
    self as goamizone, amizone_service_client::AmizoneServiceClient,
};
use tonic::{
    metadata::{errors::InvalidMetadataValue, AsciiMetadataKey, AsciiMetadataValue},
    transport::Channel,
    Request,
};

include!(concat!(env!("OUT_DIR"), "/_includes.rs"));

pub type Date = google::r#type::Date;
type AmizoneConnection = Arc<Mutex<AmizoneServiceClient<Channel>>>;

pub async fn new_connection(
    addr: impl ToString,
) -> Result<AmizoneConnection, tonic::transport::Error> {
    Ok(Arc::new(Mutex::new(
        AmizoneServiceClient::connect(addr.to_string()).await?,
    )))
}

pub struct Credentials {
    username: String,
    password: String,
}

pub struct User {
    pub id: usize,
    credentials: Credentials,
}

pub struct UserClient {
    key: AsciiMetadataKey,
    value: AsciiMetadataValue,
    connection: AmizoneConnection,
}

impl User {
    pub fn new<S: ToString>(id: usize, username: S, password: S) -> Self {
        Self {
            id,
            credentials: Credentials {
                username: username.to_string(),
                password: password.to_string(),
            },
        }
    }

    pub fn from_id(id: usize) -> Self {
        //TODO: Read creds from a db
        Self {
            id,
            credentials: Credentials {
                username: String::from(""),
                password: String::from(""),
            },
        }
    }
    pub fn get_client(
        &self,
        connection: AmizoneConnection,
    ) -> Result<UserClient, InvalidMetadataValue> {
        let key = AsciiMetadataKey::from_static("authorization");

        let value: AsciiMetadataValue = format!(
            "Basic {}",
            URL_SAFE.encode(format!(
                "{}:{}",
                self.credentials.username, self.credentials.password
            ))
        )
        .parse()?;

        Ok(UserClient {
            key,
            value,
            connection,
        })
    }
}

impl UserClient {
    pub async fn get_attendance(
        &mut self,
    ) -> Result<Vec<goamizone::AttendanceRecord>, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_attendance(request).await?.into_inner();
        drop(amizone);

        Ok(response.records)
    }

    pub async fn get_exam_schedule(
        &mut self,
    ) -> Result<(String, Vec<goamizone::ScheduledExam>), Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_exam_schedule(request).await?.into_inner();
        drop(amizone);

        Ok((response.title, response.exams))
    }

    pub async fn get_semesters(
        &mut self,
    ) -> Result<Vec<goamizone::Semester>, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_semesters(request).await?.into_inner();
        drop(amizone);

        Ok(response.semesters)
    }

    pub async fn get_current_courses(
        &mut self,
    ) -> Result<Vec<goamizone::Course>, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_current_courses(request).await?.into_inner();
        drop(amizone);

        Ok(response.courses)
    }

    pub async fn get_user_profile(
        &mut self,
    ) -> Result<goamizone::Profile, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_user_profile(request).await?.into_inner();
        drop(amizone);

        Ok(response)
    }

    pub async fn get_wifi_mac_info(
        &mut self,
    ) -> Result<goamizone::WifiMacInfo, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::EmptyMessage {});
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_wifi_mac_info(request).await?.into_inner();
        drop(amizone);

        Ok(response)
    }

    pub async fn get_courses(
        &mut self,
        num: usize,
    ) -> Result<Vec<goamizone::Course>, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::SemesterRef {
            semester_ref: num.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_courses(request).await?.into_inner();
        drop(amizone);

        Ok(response.courses)
    }

    pub async fn register_wifi_mac(
        &mut self,
        addr: impl ToString,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::RegisterWifiMacRequest {
            address: addr.to_string(),
            override_limit: true,
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        amizone.register_wifi_mac(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn deregister_wifi_mac(
        &mut self,
        addr: impl ToString,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::DeregisterWifiMacRequest {
            address: addr.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        amizone.deregister_wifi_mac(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn fill_faculty_feedback(
        &mut self,
        rating: i32,
        query_rating: i32,
        comment: impl ToString,
    ) -> Result<(), Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::FillFacultyFeedbackRequest {
            rating,
            query_rating,
            comment: comment.to_string(),
        });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        amizone.fill_faculty_feedback(request).await?;
        drop(amizone);

        Ok(())
    }

    pub async fn get_class_schedule(
        &mut self,
        date: Date,
    ) -> Result<Vec<goamizone::ScheduledClass>, Box<dyn std::error::Error + '_>> {
        let mut request = Request::new(goamizone::ClassScheduleRequest { date: Some(date) });
        request
            .metadata_mut()
            .insert(self.key.clone(), self.value.clone());

        let mut amizone = self.connection.lock()?;
        let response = amizone.get_class_schedule(request).await?.into_inner();
        drop(amizone);

        Ok(response.classes)
    }
}