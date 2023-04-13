use azs::applibs::eventloop::{IoCallback, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::iothub_device_client_ll;
use azs::applibs::iothub_message;
use azs::applibs::iothub_message::IotHubMessageRef;
use azs::applibs::networking;
use azure_sphere as azs;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
pub type FailureCallback = Box<dyn FnMut(FailureReason)>;
use std::cell::RefMut;
use std::fmt;

pub struct Callbacks {
    pub connection_status: Option<Box<dyn FnMut(bool /* connected */)>>,
    pub device_twin_received: Option<Box<dyn FnMut(String /* json twin content*/)>>,
    pub device_twin_report_state_ack: Option<Box<dyn FnMut(bool /* success */)>>,
    pub send_telemetry: Option<Box<dyn FnMut(bool /* success */)>>,
    pub device_method:
        Option<Box<dyn FnMut(String /* method name */, String /* payload */) -> String>>,
    pub cloud_to_device: Option<Box<dyn FnMut(IotHubMessageRef /* message */)>>,
}

// dyn FnMut doesn't support Debug, so stub out here.
impl fmt::Debug for Callbacks {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Callbacks").finish_non_exhaustive()
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

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    NotStarted,
    Started,
    Complete(Rc<iothub_device_client_ll::IotHubDeviceClient>),
    Failed,
}

/// Connection callback.
pub type ConnectionCallbackHandler = Box<dyn FnMut(ConnectionStatus)>;

struct Connection {
    model_id: String,
    status_callback: RefCell<ConnectionCallbackHandler>,
}

impl Connection {
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            status_callback: RefCell::new(Box::new(Connection::default_connection_callback)),
        }
    }

    // This needs to be &self instead of &mut self so that the caller doesn't need
    // a mutable ref when calling this.  The callback handler needs to have the
    // mutable referenece in the lambda.
    pub fn intialize(&self, status_callback: ConnectionCallbackHandler) {
        let _ = self.status_callback.replace(status_callback);
    }

    pub fn test(&self) {
        azs::debug!("Connection::test {:?}\n", self.model_id);
        (*self.status_callback.borrow_mut())(ConnectionStatus::Failed);
    }

    fn default_connection_callback(_status: ConnectionStatus) {
        azs::debug!("Connection::default_connection_callback\n");
    }
}

/// Mutable state associated with an AzureIoT instance
#[derive(Debug)]
struct AzureIoTState {
    elt: eventloop_timer_utilities::EventLoopTimer,
    connect_period_seconds: u64,
    authentication_state: AuthenticationState,
    client_handle: Option<Rc<iothub_device_client_ll::IotHubDeviceClient>>,
    /// Callback functions
    cb: Callbacks,
}

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

fn cloud_to_device_callback(
    _rc_state: RefMut<AzureIoTState>,
    _message: &iothub_message::IotHubMessageRef,
) -> iothub_device_client_ll::MessageDisposition {
    // bugbug: implement
    iothub_device_client_ll::MessageDisposition::Rejected
}

impl AzureIoTState {
    fn device_twin_callback(
        &mut self,
        _update_state: iothub_device_client_ll::DeviceTwinUpdateState,
        _payload: Vec<u8>,
    ) {
        // bugbug: implement
    }

    fn device_message_callback(
        &mut self,
        _method_name: &std::ffi::CStr,
        _payload: &Vec<u8>,
    ) -> (i32, Vec<u8>) {
        // bugbug: implement
        (0, vec![])
    }

    fn message_status_callback(
        &mut self,
        _result: iothub_device_client_ll::ConnectionStatus,
        _result_reason: iothub_device_client_ll::ConnectionStatusReason,
    ) {
        // bugbug: implement
    }
}

// An AzureIoT object, representing an IoT Hub client
pub struct AzureIoT {
    /// Mutable state.  This is kept separately so that the AzureIoTState can be mutated
    /// without having to take a mutable ref to the connection object.
    state: Rc<RefCell<AzureIoTState>>,
    /// Immutable state, the underlying IoT Hub client connection
    connection: Connection,
    /// Failure callback
    failure_callback: FailureCallback,
}

impl IoCallback for AzureIoT {
    /// Azure timer event:  Check connection status and send telemetry
    fn event(&mut self, _events: IoEvents) {
        self.state.borrow().elt.consume_event().unwrap();

        // bugbug: see AzureIoTConnectTimerEventHandler
    }

    unsafe fn fd(&self) -> i32 {
        self.state.borrow().elt.fd()
    }
}

impl AzureIoT {
    pub fn new(
        model_id: String,
        failure_callback: FailureCallback,
        cb: Callbacks,
    ) -> Result<Self, std::io::Error> {
        let elt = eventloop_timer_utilities::EventLoopTimer::new()?;
        let connect_period = Duration::new(DEFAULT_CONNECT_PERIOD_SECONDS, 0);
        elt.set_period(connect_period)?;
        let connection = Connection::new(model_id);

        let state = AzureIoTState {
            elt,
            connect_period_seconds: DEFAULT_CONNECT_PERIOD_SECONDS,
            authentication_state: AuthenticationState::NotAuthenticated,
            client_handle: None,
            cb,
        };

        Ok(Self {
            state: Rc::new(RefCell::new(state)),
            connection,
            failure_callback,
        })
    }

    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        // Bump the refcount on the AzureIoTState
        let mut captured_state = self.state.clone();

        // Initialize the connection, including a mutable lambda.  It acquires ownership of the clone
        // of self.state, and there are no other mutable references, so it can borrow_mut().
        // Note that checking for borrow_mut() is only done at runtime.
        self.connection.intialize(Box::new(move |status| {
            connection_callback_handler(&mut captured_state, status);
            let mut state = captured_state.borrow_mut();
            state.connect_period_seconds = u64::MAX;
        }));

        // bugbug: the IoCallback trait means the AzureIoT object can only have
        // one EventLoop callback.  We need two here, one for AzureIoTConnectTimerEventHandler
        // and one for AzureIoTDoWorkTimerEventHandler.  Probably need to switch over to
        // Box<dyn fnmut> style.

        Ok(())
    }

    pub fn test(&self) {
        azs::debug!("AzureIoT::test()\n");
        self.connection.test()
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
                (self.failure_callback)(FailureReason::NetworkingIsReadyFailed);
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
        if self.state.borrow().authentication_state != AuthenticationState::Authenticated {
            // AzureIoT client is not authenticated. Log a warning and return.
            azs::debug!("WARNING: Azure IoT Hub is not authenticated. Not sending telemetry.\n");
            return Err(IoTResult::OtherFailure);
        }
        let message = iothub_message::IotHubMessage::from_string(json_message.as_str());
        if message.is_err() {
            azs::debug!("ERROR: unable to create a new IoTHubMessage.\n");
            return Err(IoTResult::OtherFailure);
        }
        let _message = message.unwrap();
        // bugbug: call IoTHubDeviceClient_LL_SendEventAsync

        Ok(())
    }
}
