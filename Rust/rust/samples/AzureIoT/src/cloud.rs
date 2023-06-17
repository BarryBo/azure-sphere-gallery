use crate::azureiot::{AzureIoT, FailureCallback, FailureReason, IoTResult};
use azs::applibs::eventloop::{IoCallbackList, IoEvents};
use azure_sphere as azs;
use chrono::{DateTime, Datelike, Timelike, Utc};
use std::cell::RefCell;
use std::rc::Rc;
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

pub trait CloudCallbacks {
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

struct CloudData<FC, CB:CloudCallbacks> {
    last_acked_version: u32,
    date_time_buffer: String,
    fc: FC,
    callbacks: CB,
}

impl<FC, CB: CloudCallbacks> CloudData<FC, CB> {
    fn device_method_callback_handler(&mut self, method_name: String, payload: Vec<u8>) -> (i32, Vec<u8>) {
        if method_name == "displayAlert" {
            let payload_string = std::str::from_utf8(&payload).unwrap_or("Failed to convert alert to utf8");
            self.callbacks.display_alert(payload_string);
            let response_string = "\"Alert message displayed successfully.\""; // must be a JSON string (in quotes)
            (200, response_string.as_bytes().to_vec())
        } else {
            // All other method names are ignored
            (-1, "{}".as_bytes().to_vec())
        }
    }

    fn device_twin_callback_handler(&mut self, content: String)
    {
        // bugbug: implement
        azs::debug!("device_twin_callback_handler: {:?}\n", content);
    }

    fn device_twin_report_state_ack_handler(&self, success: bool) {
        if success {
            azs::debug!("INFO: Azure IoT Hub Device Twin update was successfully sent.\n");
        } else {
            azs::debug!("WARNING: Azure IoT Hub Device Twin update FAILED!\n");
        }
   
    }
}

pub struct Cloud<FC, CB:CloudCallbacks> {
    azureiot: AzureIoT<Rc<RefCell<CloudData<FC, CB>>>>,
}

impl<FC: 'static, CB: CloudCallbacks + 'static> IoCallbackList for Cloud<FC, CB> {
    fn event(&mut self, fd: i32, events: IoEvents) {
        self.azureiot.event(fd, events)
    }

    unsafe fn fd_list(&self) -> Vec<i32> {
        self.azureiot.fd_list()
    }
}

impl<FC: FailureCallback, CB: CloudCallbacks> FailureCallback for CloudData<FC, CB> {
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

impl<FC: FailureCallback + 'static, CB: CloudCallbacks + 'static> Cloud<FC, CB> {
    pub fn new(
        failure_callback: FC,
        callbacks: CB,
        hostname: String,
    ) -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let inner = CloudData {
            last_acked_version,
            date_time_buffer,
            fc: failure_callback,
            callbacks,
        };
        let inner = Rc::new(RefCell::new(inner));
        let mut iot_callbacks = crate::azureiot::Callbacks::default();

        // .connectionStatusCallbackFunction = ConnectionChangedCallbackHandler
        let inner_clone = inner.clone();
        iot_callbacks.connection_status = Some(Box::new(move |status| {
            inner_clone
                .as_ref()
                .borrow_mut()
                .callbacks
                .connection_changed(status)
        }));

        // .deviceTwinReceivedCallbackFunction = DeviceTwinCallbackHandler,
        let inner_clone = inner.clone();
        iot_callbacks.device_twin_received =
            Some(Box::new(move |content: String| {
                inner_clone
                    .as_ref()
                    .borrow_mut()
                    .device_twin_callback_handler(content)
            }));

        // .deviceTwinReportStateAckCallbackTypeFunction = DeviceTwinReportStateAckCallbackTypeHandler
        let inner_clone = inner.clone();
        iot_callbacks.device_twin_report_state_ack =
            Some(Box::new(move |success: bool| {
                inner_clone
                    .as_ref()
                    .borrow_mut()
                    .device_twin_report_state_ack_handler(success)
            }));

        // .deviceMethodCallbackFunction = DeviceMethodCallbackHandler};
        let inner_clone = inner.clone();
        iot_callbacks.device_method =
            Some(Box::new(move |method_name: String, payload: Vec<u8>| {
                inner_clone
                    .as_ref()
                    .borrow_mut()
                    .device_method_callback_handler(method_name, payload)
            }));
        let azureiot = AzureIoT::new(String::from(MODEL_ID), inner, iot_callbacks, hostname)?;

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
