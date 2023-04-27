use super::types::*;
use go_amizone::server::proto::v1::{
    ClassScheduleRequest, DeregisterWifiMacRequest, EmptyMessage, FillFacultyFeedbackRequest,
    RegisterWifiMacRequest, SemesterRef,
};
use tonic::{
    metadata::{AsciiMetadataKey, AsciiMetadataValue},
    Request,
};

pub struct UserClient {
    key: AsciiMetadataKey,
    value: AsciiMetadataValue,
    connection: AmizoneConnection,
}

impl UserClient {
    pub fn new(
        key: AsciiMetadataKey,
        value: AsciiMetadataValue,
        connection: AmizoneConnection,
    ) -> Self {
        Self {
            key,
            value,
            connection,
        }
    }

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
