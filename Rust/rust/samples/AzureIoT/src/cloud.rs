use crate::azureiot::{AzureIoT, AzureIoTEvent, FailureReason, IoTResult};
use azs::applibs::eventloop::{IoCallback, IoEvents};
use azure_sphere as azs;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::cell::RefCell;
use std::time::SystemTime;

const MODEL_ID: &str = "dtmi:com:example:azuresphere:thermometer;1";

#[derive(Debug)]
pub enum CloudEvent {
    Failure(FailureReason),
    TelemetryUploadEnabledChanged(bool),
    Telemetry(Telemetry),
    Alert(String),
    ConnectionChanged(bool),
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

pub struct Cloud {
    last_acked_version: u32,
    date_time_buffer: String,
    azureiot: AzureIoT,
    events: RefCell<Vec<CloudEvent>>,
}

impl IoCallback for Cloud {
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

impl Cloud {
    pub fn new() -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let azureiot = AzureIoT::new(String::from(MODEL_ID))?;

        Ok(Self {
            last_acked_version,
            date_time_buffer,
            azureiot,
            events: RefCell::new(Vec::<CloudEvent>::new()),
        })
    }

    pub fn test(&mut self) {
        azs::debug!("Cloud::test()\n");
        self.azureiot.test();
        let events = self.do_work();
        for event in events.iter() {
            match event {
                CloudEvent::Failure(reason) => {
                    azs::debug!("Cloud::test() - CloudEvent::Failure({:?})\n", reason);
                }
                CloudEvent::TelemetryUploadEnabledChanged(status) => {
                    azs::debug!(
                        "Cloud::test() - CloudEvent::TelemetryUploadEnabledChanged({:?})\n",
                        status
                    );
                }
                CloudEvent::Telemetry(telemetry) => {
                    azs::debug!(
                        "Cloud::test() - CloudEvent::Telemetry({:?})\n",
                        telemetry.temperature
                    );
                }
                CloudEvent::Alert(message) => {
                    azs::debug!("Cloud::test() - CloudEvent::Alert({:?})\n", message);
                }
                CloudEvent::ConnectionChanged(connected) => {
                    azs::debug!(
                        "Cloud::test() - CloudEvent::ConnectionChanged({:?})\n",
                        connected
                    );
                }
            }
        }
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

    pub fn do_work(&self) -> Vec<CloudEvent> {
        let iot_events = self.azureiot.do_work();
        for event in iot_events.iter() {
            // Process each event
            match event {
                AzureIoTEvent::Failure(reason) => {
                    azs::debug!("INFO: Azure IoT Hub failure message received.\n");
                    let event = CloudEvent::Failure(*reason);
                    self.events.borrow_mut().push(event);
                }
                _ => {} // bugbug: finish filling this out
            }
        }

        let empty_vec = Vec::<CloudEvent>::new();
        self.events.replace(empty_vec) // Replace current list with empty, and return current list
    }
}
