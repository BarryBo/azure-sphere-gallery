use crate::azureiot::{AzureIoT, FailureCallbackFunction, IoTResult};
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

pub struct CloudCallbacks {
    pub telemetry_upload_enabled_changed: Box<dyn FnMut(bool, bool)>,
    pub display_alert: Box<dyn FnMut(&str)>,
    pub connection_changed: Box<dyn FnMut(bool)>,
}

struct CloudData {
    last_acked_version: u64,
    date_time_buffer: String,
    //fc: Box<FailureCallbackFunction>,
    callbacks: CloudCallbacks,
}

impl CloudData {
    fn device_method_callback_handler(
        &mut self,
        method_name: String,
        payload: Vec<u8>,
    ) -> (i32, Vec<u8>) {
        if method_name == "displayAlert" {
            let payload_string =
                std::str::from_utf8(&payload).unwrap_or("Failed to convert alert to utf8");
            self.callbacks.display_alert.as_mut()(payload_string);
            let response_string = "\"Alert message displayed successfully.\""; // must be a JSON string (in quotes)
            (200, response_string.as_bytes().to_vec())
        } else {
            // All other method names are ignored
            (-1, "{}".as_bytes().to_vec())
        }
    }

    fn device_twin_callback_handler(&mut self, content: String) {
        let v = serde_json::from_str(&content);
        if v.is_err() {
            azs::debug!("WARNING: Cannot parse the string as JSON content.\n");
        } else {
            // If we have a desired property for the "thermometerTelemetryUploadEnabled" property, let's
            // process it.
            let v: serde_json::Value = v.unwrap();
            let desired = v.get("desired");
            let desired_properties = match desired {
                Some(value) => value,
                None => &v,
            };
            let thermometer_telemetry_upload_enabled =
                desired_properties.get("thermometerTelemetryUploadEnabled");
            if thermometer_telemetry_upload_enabled.is_some() {
                let thermometer_telemetry_upload_enabled = thermometer_telemetry_upload_enabled
                    .unwrap()
                    .as_bool()
                    .unwrap_or(false);

                // The parson JSON parser returns 0 if json_object_dotget_number() fails to parse.
                let desired_version = desired_properties["$version"].as_u64().unwrap_or(0);

                // If there is a desired property change (including at boot, restart and
                // reconnection), the device should implement the logic that decides whether it has
                // to be applied or not. In this sample, we model this logic as an always-true
                // clause, just as a place holder for an actual logic (if any needed).

                // If accepted, the device must ack the desired version number.
                self.last_acked_version = desired_version;
                self.callbacks.telemetry_upload_enabled_changed.as_mut()(
                    thermometer_telemetry_upload_enabled,
                    true,
                );
            }
        }
    }

    fn device_twin_report_state_ack_handler(&self, success: bool) {
        if success {
            azs::debug!("INFO: Azure IoT Hub Device Twin update was successfully sent.\n");
        } else {
            azs::debug!("WARNING: Azure IoT Hub Device Twin update FAILED!\n");
        }
    }
}

pub struct Cloud {
    azureiot: AzureIoT,
}

impl IoCallbackList for Cloud {
    fn event(&mut self, fd: i32, events: IoEvents) {
        self.azureiot.event(fd, events)
    }

    unsafe fn fd_list(&self) -> Vec<i32> {
        self.azureiot.fd_list()
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
    pub fn new(
        failure_callback: Box<FailureCallbackFunction>,
        callbacks: CloudCallbacks,
        hostname: String,
    ) -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let inner = CloudData {
            last_acked_version,
            date_time_buffer,
            //fc: failure_callback,
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
                .connection_changed
                .as_mut()(status)
        }));

        // .deviceTwinReceivedCallbackFunction = DeviceTwinCallbackHandler,
        let inner_clone = inner.clone();
        iot_callbacks.device_twin_received = Some(Box::new(move |content: String| {
            inner_clone
                .as_ref()
                .borrow_mut()
                .device_twin_callback_handler(content)
        }));

        // .deviceTwinReportStateAckCallbackTypeFunction = DeviceTwinReportStateAckCallbackTypeHandler
        let inner_clone = inner.clone();
        iot_callbacks.device_twin_report_state_ack = Some(Box::new(move |success: bool| {
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
        let azureiot = AzureIoT::new(
            String::from(MODEL_ID),
            failure_callback,
            iot_callbacks,
            hostname,
        )?;

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
