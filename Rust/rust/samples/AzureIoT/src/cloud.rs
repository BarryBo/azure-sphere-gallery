use crate::azureiot::{AzureIoT, FailureReason, IoTResult};
use azs::applibs::eventloop::{IoCallback, IoEvents};
use azure_sphere as azs;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::time::SystemTime;

const MODEL_ID: &str = "dtmi:com:example:azuresphere:thermometer;1";

#[derive(Debug)]
pub struct Telemetry {
    pub temperature: f32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CloudResult {
    NoNetwork,
    OtherFailure,
}

pub trait Callbacks {
    fn telemetry_upload_enabled_changed(&mut self, status: bool, from_cloud: bool) {
        drop(status);
        drop(from_cloud);
    }
    fn display_alert(&mut self, alert_message: &str) {
        drop(alert_message);
    }
    fn connection_changed(&mut self, connected: bool) {
        drop(connected);
    }
}

struct CloudData<FC> {
    last_acked_version: u32,
    date_time_buffer: String,
    fc: FC,
}

pub struct Cloud<FC> {
    azureiot: AzureIoT<CloudData<FC>>,
}

impl<FC> IoCallback for Cloud<FC> {
    fn event(&mut self, events: IoEvents) {
        self.azureiot.event(events)
    }

    unsafe fn fd(&self) -> i32 {
        self.azureiot.fd()
    }
}

impl<FC: crate::azureiot::FailureCallback> crate::azureiot::FailureCallback for CloudData<FC> {
    fn failure_callback(&mut self, reason: FailureReason) {
        self.fc.failure_callback(reason)
    }
}

fn azureiot_to_cloud_result(azureiot_result: Result<(), IoTResult>) -> Result<(), CloudResult> {
    match azureiot_result {
        Ok(_) => Ok(()),
        Err(IoTResult::NoNetwork) => Err(CloudResult::NoNetwork),
        Err(IoTResult::OtherFailure) => Err(CloudResult::OtherFailure),
    }
}

impl<FC> Cloud<FC> {
    pub fn new(
        failure_callback: FC,
        callbacks: crate::azureiot::Callbacks<'static>,
    ) -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let inner = CloudData {
            last_acked_version,
            date_time_buffer,
            fc: failure_callback,
        };
        let azureiot = AzureIoT::new(String::from(MODEL_ID), inner, callbacks)?;

        Ok(Self { azureiot })
    }

    pub fn test(&mut self) {
        azs::debug!("Cloud::test()\n");

        self.azureiot.test();
        self.do_work();
    }

    pub fn build_utc_datetime(t: SystemTime) -> String {
        // Ideally, we'd use chrono here.  But it adds 60kb to the binary size.
        //let dt: DateTime<Utc> = t.clone().into();
        //// %+ is 	ISO 8601 / RFC 3339 date & time format.
        //// Such as "2001-07-08T00:34:60.026490+09:30"
        //format!("{}", dt.format("%+"))

        // This reduces chrono to only 18kb...
        let dt: DateTime<Utc> = t.clone().into();
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
            dt.year(),
            dt.month(),
            dt.day(),
            dt.hour(),
            dt.minute(),
            dt.second()
        )
    }

    pub fn send_telemetry(
        &mut self,
        telemetry: &Telemetry,
        timestamp: Option<SystemTime>,
    ) -> Result<(), CloudResult> {
        let utc_datetime = if let Some(t) = timestamp {
            Some(Self::build_utc_datetime(t))
        } else {
            None
        };
        // Ideally, we'd use serde_json here.  But it adds 43kb to the binary size.
        //  {"temperature":28.3}
        let serialized_telemetry = format!("{{\"temperature\"={}}}", telemetry.temperature);
        azs::debug!(
            "Serialized telemetry = {} at {:?}\n",
            serialized_telemetry,
            utc_datetime
        );
        let result = self
            .azureiot
            .send_telemetry(serialized_telemetry, utc_datetime);
        azureiot_to_cloud_result(result)
    }

    pub fn do_work(&self) {
        self.azureiot.do_work();
    }
}
