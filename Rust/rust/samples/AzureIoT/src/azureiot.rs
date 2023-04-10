use azs::applibs::eventloop::{IoCallback, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::iothub_message;
use azs::applibs::iothub_message::IotHubMessageRef;
use azs::applibs::networking;
use azure_sphere as azs;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
pub type FailureCallback = Box<dyn FnMut(FailureReason)>;

pub struct Callbacks {
    pub connection_status: Option<Box<dyn FnMut(bool /* connected */)>>,
    pub device_twin_received: Option<Box<dyn FnMut(String /* json twin content*/)>>,
    pub device_twin_report_state_ack: Option<Box<dyn FnMut(bool /* success */)>>,
    pub send_telemetry: Option<Box<dyn FnMut(bool /* success */)>>,
    pub device_method:
        Option<Box<dyn FnMut(String /* method name */, String /* payload */) -> String>>,
    pub cloud_to_device: Option<Box<dyn FnMut(IotHubMessageRef /* message */)>>,
}

/// check if device is connected to the internet and Azure client is setup every second
const DEFAULT_CONNECT_PERIOD_SECONDS: u64 = 1;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionStatus {
    NotStarted,
    Started,
    Complete,
    Failed,
}

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
enum ConnectionState {
    /// Client is not authenticated by the Azure IoT Hub.
    NotAuthenticated,
    /// Client has initiated authentication to the Azure IoT Hub
    AuthenticationInitiated,
    /// Client is authenticated by the Azure IoT Hub.
    Authenticated,
}

pub type ConnectionCallbackHandler = Box<dyn FnMut(ConnectionStatus, i32)>;

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
        (*self.status_callback.borrow_mut())(ConnectionStatus::Complete, 1);
    }

    fn default_connection_callback(_status: ConnectionStatus, _client_handle: i32) {
        azs::debug!("Connection::default_connection_callback\n");
    }
}

/// Mutable state associated with an AzureIoT instance
#[derive(Debug)]
struct AzureIoTState {
    elt: eventloop_timer_utilities::EventLoopTimer,
    connect_period_seconds: u64,
    connection_state: ConnectionState,
}

impl AzureIoTState {
    fn connection_callback_handler(&mut self, status: ConnectionStatus, _client_handle: i32) {
        azs::debug!("AzureIotState::connection_callback_handler\n");
        match status {
            ConnectionStatus::NotStarted => {}
            ConnectionStatus::Started => {
                azs::debug!("INFO: Azure IoT Hub connection started.\n");
            }
            ConnectionStatus::Complete => {
                // bugbug: implement
                azs::debug!("ConnectionStatus::Complete\n");
            }
            ConnectionStatus::Failed => {
                // bugbug: implement
                azs::debug!("ConnectionStatus::Failed\n");
            }
        }
    }
}

// An AzureIoT object, representing an IoT Hub client
pub struct AzureIoT {
    /// Mutable state.  This is kept separately so that the AzureIoTState can be mutated
    /// without having to take a mutable ref to the connection object.
    state: Rc<RefCell<AzureIoTState>>,
    /// Immutable state, the underlying IoT Hub client connection
    connection: Connection,
    /// Callback functions
    cb: Callbacks,
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
            connection_state: ConnectionState::NotAuthenticated,
        };

        Ok(Self {
            state: Rc::new(RefCell::new(state)),
            connection,
            cb,
            failure_callback,
        })
    }

    pub fn initialize(&mut self) -> Result<(), std::io::Error> {
        // Bump the refcount on the AzureIoTState
        let state_clone = self.state.clone();

        // Initialize the connection, including a mutable lambda.  It acquires ownership of the clone
        // of self.state, and there are no other mutable references, so it can borrow_mut().
        // Note that checking for borrow_mut() is only done at runtime.
        self.connection
            .intialize(Box::new(move |status, client_handle| {
                let mut state = state_clone.borrow_mut();
                state.connection_callback_handler(status, client_handle);
                state.connect_period_seconds = 2;
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
        if self.state.borrow().connection_state != ConnectionState::Authenticated {
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
