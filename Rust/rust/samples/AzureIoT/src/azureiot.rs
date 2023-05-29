use crate::connection_iot_hub::{Connection, ConnectionStatus};
use azs::applibs::eventloop::{IoCallback, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::iothub_device_client;
use azs::applibs::iothub_message;
use azs::applibs::networking;
use azure_sphere as azs;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

#[derive(Default)]
pub struct Callbacks<'a> {
    pub connection_status: Option<Box<dyn FnMut(bool /* connected */) + 'a>>,
    pub device_twin_received: Option<Box<dyn FnMut(String /* json twin content*/) + 'a>>,
    pub device_twin_report_state_ack: Option<Box<dyn FnMut(bool /* success */) + 'a>>,
    pub send_telemetry: Option<Box<dyn FnMut(bool /* success */) + 'a>>,
    pub device_method:
        Option<Box<dyn FnMut(String /* method name */, String /* payload */) -> String + 'a>>,
    //bugbug: type of IotHubMessage... pub cloud_to_device: Option<Box<dyn FnMut(&IotHubMessage /* message */) + 'a>>,
}

pub trait FailureCallback {
    fn failure_callback(&mut self, reason: FailureReason) {
        drop(reason)
    }
}

/// check if device is connected to the internet and Azure client is setup every second
const DEFAULT_CONNECT_PERIOD_SECONDS: u64 = 1;
/// back off when reconnecting
const MIN_CONNECT_PERIOD_SECONDS: u64 = 10;
/// back off limit
const MAX_CONNECT_PERIOD_SECONDS: u64 = 10 * 60;

// This is equivalent to some of the ExitCode_* constants in the C sample
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FailureReason {
    /// ExitCode_IsNetworkingReady_Failed
    NetworkingIsReadyFailed,
}

/// An enum indicating possible result codes when performing Azure IoT-related operations
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IoTResult {
    /// The operation could not be performed as no network connection was available
    NoNetwork,
    /// The operation failed for another reason not explicitly listed
    OtherFailure,
}

/// Authentication state of the client with respect to the Azure IoT Hub.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum AuthenticationState {
    /// Client is not authenticated by the Azure IoT Hub.
    NotAuthenticated,
    /// Client has initiated authentication to the Azure IoT Hub
    AuthenticationInitiated,
    /// Client is authenticated by the Azure IoT Hub.
    Authenticated,
}

/*
fn connection_callback_handler(rc_state: &Rc<RefCell<AzureIoTState>>, status: ConnectionStatus) {
    azs::debug!("AzureIotState::connection_callback_handler\n");
    match status {
        ConnectionStatus::NotStarted => {}
        ConnectionStatus::Started => {
            azs::debug!("INFO: Azure IoT Hub connection started.\n");
        }
        ConnectionStatus::Complete(client_handle) => {
            // bugbug: implement
            azs::debug!("INFO: Azure IoT Hub connection complete.\n");
            {
                let mut rc_state_borrowed = rc_state.borrow_mut();
                rc_state_borrowed.client_handle = Some(client_handle.clone());

                // Successfully connected, so make sure the polling frequency is back to the default
                rc_state_borrowed.connect_period_seconds = DEFAULT_CONNECT_PERIOD_SECONDS;

                // Set client authentication state to initiated. This is done to indicate that
                // SetUpAzureIoTHubClient() has been called (and so should not be called again) while the
                // client is waiting for a response via the ConnectionStatusCallback().
                rc_state_borrowed.authentication_state =
                    AuthenticationState::AuthenticationInitiated;
            }

            // bugbug: set callbacks
            let captured_state = Rc::clone(&rc_state);
            let _ = client_handle.set_message_callback(Box::new(
                move |message: &iothub_message::IotHubMessageRef| {
                    azs::debug!("INFO: Azure IoT Hub message received.\n");
                    let state = captured_state.borrow_mut();
                    cloud_to_device_callback(state, message);
                    iothub_device_client_ll::MessageDisposition::Abandoned
                },
            ));
        }
        ConnectionStatus::Failed => {
            let mut rc_state_borrowed = rc_state.borrow_mut();
            // If we fail to connect, reduce the polling frequency, starting at
            // AzureIoTMinReconnectPeriodSeconds and with a backoff up to
            // AzureIoTMaxReconnectPeriodSeconds
            rc_state_borrowed.connect_period_seconds =
                if rc_state_borrowed.connect_period_seconds == DEFAULT_CONNECT_PERIOD_SECONDS {
                    MIN_CONNECT_PERIOD_SECONDS
                } else {
                    let new_period = rc_state_borrowed.connect_period_seconds * 2;
                    if new_period > MAX_CONNECT_PERIOD_SECONDS {
                        MAX_CONNECT_PERIOD_SECONDS
                    } else {
                        new_period
                    }
                };
            let connect_period = Duration::new(rc_state_borrowed.connect_period_seconds, 0);
            let _ = rc_state_borrowed.elt.set_period(connect_period);

            azs::debug!(
                "ERROR: Azure IoT Hub connection failed - will retry in {} seconds.\n",
                rc_state_borrowed.connect_period_seconds
            );
        }
    }
}
*/

// An AzureIoT object, representing an IoT Hub client
pub struct AzureIoT<F> {
    inner: Rc<RefCell<AzureIoTData<F>>>,

    /// Immutable state, the underlying IoT Hub client connection
    connection: Connection,
}

struct AzureIoTData<F> {
    elt: eventloop_timer_utilities::EventLoopTimer,
    connect_period_seconds: u64,
    authentication_state: AuthenticationState,
    client_handle: Option<iothub_device_client::IotHubDeviceClient>,

    failure_callback: F,
    callbacks: Callbacks<'static>,
}

impl<F> IoCallback for AzureIoT<F> {
    /// Azure timer event:  Check connection status and send telemetry
    fn event(&mut self, _events: IoEvents) {
        self.inner.as_ref().borrow().elt.consume_event().unwrap();

        // bugbug: see AzureIoTConnectTimerEventHandler
    }

    unsafe fn fd(&self) -> i32 {
        self.inner.borrow().elt.fd()
    }
}

impl<F> AzureIoTData<F> {
    fn connection_status_callback(&mut self, status: ConnectionStatus) {
        azs::debug!("AzureIoT::connection_status_callback: {:?}\n", status);
    }
}

impl<'a, F: 'static> AzureIoT<F> {
    pub fn new(
        model_id: String,
        failure_callback: F,
        callbacks: Callbacks<'static>,
        hostname: String,
    ) -> Result<Self, std::io::Error> {
        let elt = eventloop_timer_utilities::EventLoopTimer::new()?;
        let connect_period = Duration::new(DEFAULT_CONNECT_PERIOD_SECONDS, 0);
        elt.set_period(connect_period)?;
        let connection = Connection::new(model_id, hostname);

        // bugbug: call Connection_Initialize(ConnectionCallbackHandler) in connection_iot_hub or other implementations
        // bugbug: need a second EventLoopTimer for azureIoTConnectionTimer, separate from the DoWork timer.

        Ok(Self {
            inner: Rc::new(RefCell::new(AzureIoTData {
                elt,
                connect_period_seconds: DEFAULT_CONNECT_PERIOD_SECONDS,
                authentication_state: AuthenticationState::NotAuthenticated,
                client_handle: None,
                failure_callback,
                callbacks,
            })),
            connection,
        })
    }

    pub fn test(&mut self) {
        azs::debug!("AzureIoT::test\n");

        let inner_clone = self.inner.clone();
        let cb =
            Box::new(move |status| inner_clone.borrow_mut().connection_status_callback(status));

        self.connection.start(cb);

        azs::debug!("AzureIoT Calling connection_status\n");
        self.inner
            .borrow_mut()
            .callbacks
            .connection_status
            .as_mut()
            .unwrap()
            .as_mut()(true);

        azs::debug!("AzureIoT Calling device_method\n");
        let result = self
            .inner
            .borrow_mut()
            .callbacks
            .device_method
            .as_mut()
            .unwrap()
            .as_mut()(String::from("test"), String::from("payload"));
        azs::debug!("AzureIoT::test: device_method returned {:?}\n", result)
    }

    fn is_connection_ready_to_send_telemetry(&mut self) -> bool {
        let is_ready = networking::is_networking_ready();
        match is_ready {
            Ok(true) => true,
            Ok(false) => {
                azs::debug!(
                    "WARNING: Cannot send Azure IoT Hub telemetry because the network is not up.\n"
                );
                false
            }
            Err(err) => {
                azs::debug!(
                    "WARNING: Cannot send Azure IoT Hub telemetry because the network is not up: {}.\n",
                err);
                // bugbug: call the failure callback
                false
            }
        }
    }

    pub fn send_telemetry(
        &mut self,
        json_message: String,
        _iso8601_datetime: Option<String>,
    ) -> Result<(), IoTResult> {
        if !self.is_connection_ready_to_send_telemetry() {
            return Err(IoTResult::NoNetwork);
        }
        if self.inner.borrow().authentication_state != AuthenticationState::Authenticated {
            // AzureIoT client is not authenticated. Log a warning and return.
            azs::debug!("WARNING: Azure IoT Hub is not authenticated. Not sending telemetry.\n");
            return Err(IoTResult::OtherFailure);
        }
        let message = iothub_message::IotHubMessage::from_string(json_message.as_str());
        if message.is_err() {
            azs::debug!("ERROR: unable to create a new IoTHubMessage.\n");
            return Err(IoTResult::OtherFailure);
        }
        let message = message.unwrap();
        let result = self
            .inner
            .borrow()
            .client_handle
            .as_ref()
            .unwrap()
            .send_event(message);
        if result.is_err() {
            azs::debug!("ERROR: unable to send telemetry to Azure IoT Hub.\n");
            return Err(IoTResult::OtherFailure);
        }

        Ok(())
    }

    pub fn do_work(&self) {
        if let Some(client_handle) = self.inner.borrow().client_handle.as_ref() {
            // There is a lower-level client handle, so invoke it to do work
            client_handle.do_work();
        }
    }
}
