include!(concat!(env!("OUT_DIR"), "/_includes.rs"));
use go_amizone::server::proto::v1::{
    self as goamizone, amizone_service_client::AmizoneServiceClient,
};

pub type Date = google::r#type::Date;
pub type AttendanceRecord = goamizone::AttendanceRecord;
pub type ScheduledExam = goamizone::ScheduledExam;
pub type ScheduledClass = goamizone::ScheduledClass;
pub type AmizoneProfile = goamizone::Profile;
pub type Semester = goamizone::Semester;
pub type Course = goamizone::Course;
pub type AttendanceState = goamizone::AttendanceState;
pub type WifiMacInfo = goamizone::WifiMacInfo;
pub type AmizoneConnection =
    std::sync::Arc<tokio::sync::Mutex<AmizoneServiceClient<tonic::transport::channel::Channel>>>;
pub type DatabaseConnection = mongodb::Client;
pub type AmizoneApiError = tonic::Status;
pub type Result<T> = std::result::Result<T, AmizoneApiError>;
pub type StatusCode = tonic::Code;
pub type DbError = mongodb::error::Error;
pub type DbOperationResult<T> = std::result::Result<T, DbError>;

impl From<i32> for AttendanceState {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Pending,
            1 => Self::Present,
            2 => Self::Absent,
            3 => Self::Na,
            _ => Self::Invalid,
        }
    }
}
