use azs::applibs::eventloop::{IoCallback, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::iothub_device_client;
use azs::applibs::iothub_device_client::IotHubEvent;
use azs::applibs::iothub_message;
use azs::applibs::networking;
use azure_sphere as azs;
use std::cell::RefCell;
use std::time::Duration;

//pub struct Callbacks {
//    pub connection_status: Option<Box<dyn FnMut(bool /* connected */)>>,
//    pub device_twin_received: Option<Box<dyn FnMut(String /* json twin content*/)>>,
//    pub device_twin_report_state_ack: Option<Box<dyn FnMut(bool /* success */)>>,
//    pub send_telemetry: Option<Box<dyn FnMut(bool /* success */)>>,
//    pub device_method:
//        Option<Box<dyn FnMut(String /* method name */, String /* payload */) -> String>>,
//    pub cloud_to_device: Option<Box<dyn FnMut(IotHubMessageRef /* message */)>>,
//}

// dyn FnMut doesn't support Debug, so stub out here.
//impl fmt::Debug for Callbacks {
//    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//        f.debug_struct("Callbacks").finish_non_exhaustive()
//    }
//}

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

#[derive(Debug)]
pub enum ConnectionStatus {
    NotStarted,
    Started,
    Complete(iothub_device_client::IotHubDeviceClient),
    Failed,
}

struct Connection {
    model_id: String,
}

impl Connection {
    pub fn new(model_id: String) -> Self {
        Self { model_id }
    }
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

#[derive(Debug)]
pub enum AzureIoTEvent {
    Failure(FailureReason), // bugbug: might need to include more context about what failed
    Bugbug,                 // bugbug: remove
}

// An AzureIoT object, representing an IoT Hub client
pub struct AzureIoT {
    // The formerly mutable state...
    elt: eventloop_timer_utilities::EventLoopTimer,
    connect_period_seconds: u64,
    authentication_state: AuthenticationState,
    client_handle: Option<iothub_device_client::IotHubDeviceClient>,

    events: RefCell<Vec<AzureIoTEvent>>,

    /// Immutable state, the underlying IoT Hub client connection
    connection: Connection,
}

impl IoCallback for AzureIoT {
    /// Azure timer event:  Check connection status and send telemetry
    fn event(&mut self, _events: IoEvents) {
        self.elt.consume_event().unwrap();

        // bugbug: see AzureIoTConnectTimerEventHandler
    }

    unsafe fn fd(&self) -> i32 {
        self.elt.fd()
    }
}

impl AzureIoT {
    pub fn new(model_id: String) -> Result<Self, std::io::Error> {
        let elt = eventloop_timer_utilities::EventLoopTimer::new()?;
        let connect_period = Duration::new(DEFAULT_CONNECT_PERIOD_SECONDS, 0);
        elt.set_period(connect_period)?;
        let connection = Connection::new(model_id);

        Ok(Self {
            elt,
            connect_period_seconds: DEFAULT_CONNECT_PERIOD_SECONDS,
            authentication_state: AuthenticationState::NotAuthenticated,
            client_handle: None,
            events: RefCell::new(Vec::<AzureIoTEvent>::new()),
            connection,
        })
    }

    pub fn test(&self) {
        azs::debug!("AzureIoT::test\n");
        let event = AzureIoTEvent::Failure(FailureReason::NetworkingIsReadyFailed);
        self.events.borrow_mut().push(event);
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
                let event = AzureIoTEvent::Failure(FailureReason::NetworkingIsReadyFailed);
                self.events.borrow_mut().push(event);
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
        if self.authentication_state != AuthenticationState::Authenticated {
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
        let result = self.client_handle.as_ref().unwrap().send_event(message);
        if result.is_err() {
            azs::debug!("ERROR: unable to send telemetry to Azure IoT Hub.\n");
            return Err(IoTResult::OtherFailure);
        }

        Ok(())
    }

    pub fn do_work(&self) -> Vec<AzureIoTEvent> {
        if let Some(client_handle) = self.client_handle.as_ref() {
            // There is a lower-level client handle, so invoke it to do work
            let hub_events = client_handle.do_work();
            for event in hub_events.iter() {
                // Process each event
                match event {
                    IotHubEvent::Message(_message) => {
                        azs::debug!("INFO: Azure IoT Hub message received.\n");
                    }
                    _ => {} // bugbug: finish filling this out
                }
            }
        }

        let empty_vec = Vec::<AzureIoTEvent>::new();
        self.events.replace(empty_vec) // Replace current list with empty, and return current list
    }
}
