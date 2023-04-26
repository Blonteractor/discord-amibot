include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
use go_amizone::server::proto::v1::{
    self as goamizone, amizone_service_client::AmizoneServiceClient,
};

use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::channel::Channel;

pub type Date = google::r#type::Date;
pub type AttendanceRecord = goamizone::AttendanceRecord;
pub type ScheduledExam = goamizone::ScheduledExam;
pub type ScheduledClass = goamizone::ScheduledClass;
pub type AmizoneProfile = goamizone::Profile;
pub type Semester = goamizone::Semester;
pub type Course = goamizone::Course;
pub type WifiMacInfo = goamizone::WifiMacInfo;
pub type AmizoneConnection = Arc<Mutex<AmizoneServiceClient<Channel>>>;

pub type Result<T> = std::result::Result<T, tonic::Status>;
pub type DbOperationResult<T> = std::result::Result<T, mongodb::error::Error>;
