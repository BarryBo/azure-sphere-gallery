use crate::azureiot::AzureIoT;
use crate::azureiot::{Callbacks, FailureCallback, IoTResult};
use azs::applibs::eventloop::{IoCallback, IoEvents};
use azure_sphere as azs;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::time::SystemTime;

const MODEL_ID: &str = "dtmi:com:example:azuresphere:thermometer;1";

pub trait CloudCallbacks {
    fn telemetry_upload_enabled_change(&mut self, status: bool, from_cloud: bool) {
        azs::debug!("WARNING: Cloud - no handler registered for TelemetryUploadEnabled - status {:?} from_cloud {:?}\n", status, from_cloud);
    }

    fn display_alert(&mut self, message: String) {
        azs::debug!(
            "WARNING: Cloud - no handler registered for DisplayAlert - message {:?}\n",
            message
            );        
    }

    fn connection_change(&mut self, connected: bool) {
        azs::debug!(
            "WARNING: Cloud - no handler registered for ConnectionChanged - connected {:?}\n",
            connected
        );
    }

}

#[derive(Debug)]
pub struct Telemetry {
    pub temperature: f32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CloudResult {
    NoNetwork,
    OtherFailure,
}

pub struct Cloud<'a> {
    last_acked_version: u32,
    date_time_buffer: String,
    cloud_callbacks: &'a mut dyn CloudCallbacks,
    azureiot: AzureIoT,
}

impl<'a> IoCallback for Cloud<'a> {
    fn event(&mut self, events: IoEvents) {
        self.azureiot.event(events)
    }

    unsafe fn fd(&self) -> i32 {
        self.azureiot.fd()
    }
}

fn azureiot_to_cloud_result(azureiot_result: Result<(), IoTResult>) -> Result<(), CloudResult> {
    match azureiot_result {
        Ok(_) => Ok(()),
        Err(IoTResult::NoNetwork) => Err(CloudResult::NoNetwork),
        Err(IoTResult::OtherFailure) => Err(CloudResult::OtherFailure),
    }
}

impl<'a> Cloud<'a> {
    pub fn initialize(
        failure_callback: FailureCallback,
        cloud_callbacks: &'a mut dyn CloudCallbacks
    ) -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let connection_changed_callback = Box::new(Self::default_connection_change_callback);

        let callbacks = Callbacks {
            connection_status: Some(connection_changed_callback),
            device_twin_received: None,
            device_twin_report_state_ack: None,
            send_telemetry: None,
            device_method: None,
            cloud_to_device: None,
        };

        let mut azureiot = AzureIoT::new(String::from(MODEL_ID), failure_callback, callbacks)?;
        azureiot.initialize()?;

        Ok(Self {
            last_acked_version,
            date_time_buffer,
            cloud_callbacks,
            azureiot,
        })
    }

    pub fn test(&mut self) {
        azs::debug!("Cloud::test()\n");
        self.cloud_callbacks.display_alert(String::from("Hello from Cloud::test()"));
        self.azureiot.test()
    }

    fn default_connection_change_callback(_connected: bool) {
        // bugbug:
        //self.cloud_callbacks.connection_change(connected);
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
}
