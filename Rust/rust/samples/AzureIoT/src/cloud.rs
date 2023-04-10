use crate::azureiot::AzureIoT;
use crate::azureiot::{Callbacks, FailureCallback};
use azs::applibs::eventloop::{IoCallback, IoEvents};
use azure_sphere as azs;

const MODEL_ID: &str = "dtmi:com:example:azuresphere:thermometer;1";

pub type CloudTelemetryUploadEnabledChangeCallback =
    Box<dyn FnMut(bool /* status */, bool /* from_cloud */)>;
pub type CloudDisplayAlertCallback = Box<dyn FnMut(String)>;
pub type CloudConnectionChangedCallback = Box<dyn FnMut(bool /* connected */)>;

pub struct Cloud {
    last_acked_version: u32,
    date_time_buffer: String,
    telemetry_upload_enabled_change_callback: CloudTelemetryUploadEnabledChangeCallback,
    display_alert_callback: CloudDisplayAlertCallback,
    azureiot: AzureIoT,
}

impl IoCallback for Cloud {
    fn event(&mut self, events: IoEvents) {
        self.azureiot.event(events)
    }

    unsafe fn fd(&self) -> i32 {
        self.azureiot.fd()
    }
}

impl Cloud {
    pub fn initialize(
        failure_callback: FailureCallback,
        telemetry_upload_enabled_change_callback: Option<CloudTelemetryUploadEnabledChangeCallback>,
        display_alert_callback: Option<CloudDisplayAlertCallback>,
        connection_changed_callback: Option<CloudConnectionChangedCallback>,
    ) -> Result<Self, std::io::Error> {
        let last_acked_version = 0;
        let date_time_buffer = String::new();

        let telemetry_upload_enabled_change_callback = telemetry_upload_enabled_change_callback
            .unwrap_or(Box::new(
                Self::default_thermometer_telemetry_upload_enabled_change_callback,
            ));

        let display_alert_callback =
            display_alert_callback.unwrap_or(Box::new(Self::default_display_alert_callback));

        let connection_changed_callback = connection_changed_callback
            .unwrap_or(Box::new(Self::default_connection_change_callback));

        let callbacks = Callbacks {
            connection_status: Some(connection_changed_callback),
            device_twin_received: None,
            device_twin_report_state_ack: None,
            send_telemetry: None,
            device_method: None,
            cloud_to_device: None,
        };

        let mut azureiot = AzureIoT::new(String::from(MODEL_ID), callbacks)?;
        azureiot.initialize(failure_callback)?;

        Ok(Self {
            last_acked_version,
            date_time_buffer,
            telemetry_upload_enabled_change_callback,
            display_alert_callback,
            azureiot,
        })
    }

    pub fn test(&mut self) {
        azs::debug!("Cloud::test()\n");
        self.azureiot.test()
    }

    fn default_thermometer_telemetry_upload_enabled_change_callback(
        status: bool,
        from_cloud: bool,
    ) {
        azs::debug!("WARNING: Cloud - no handler registered for TelemetryUploadEnabled - status {:?} from_cloud {:?}\n", status, from_cloud);
    }

    fn default_display_alert_callback(message: String) {
        azs::debug!(
            "WARNING: Cloud - no handler registered for DisplayAlert - message {:?}\n",
            message
        );
    }

    fn default_connection_change_callback(connected: bool) {
        azs::debug!(
            "WARNING: Cloud - no handler registered for ConnectionChanged - connected {:?}\n",
            connected
        );
    }
}
