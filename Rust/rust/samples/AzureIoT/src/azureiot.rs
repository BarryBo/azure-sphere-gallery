use crate::connection_iot_hub::{Connection, ConnectionStatus};
use azs::applibs::eventloop::{IoCallbackList, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::iothub_device_client;
use azs::applibs::iothub_device_client_ll::{
    ConnectionStatusReason, DeviceTwinUpdateState, MessageDisposition,
};
use azs::applibs::iothub_message;
use azs::applibs::iothub_message::IotHubMessage;
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
    pub device_method: Option<
        Box<dyn FnMut(String /* method name */, Vec<u8> /* payload */) -> (i32, Vec<u8>) + 'a>,
    >,
    pub cloud_to_device: Option<Box<dyn FnMut(IotHubMessage /* message */) + 'a>>,
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

/// Call IoTHubDeviceClient_LL_DoWork() every 100 ms
const DO_WORK_INTERVAL_MILLISECOND: u32 = 100;

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

// An AzureIoT object, representing an IoT Hub client
pub struct AzureIoT<F> {
    inner: Rc<RefCell<AzureIoTData<F>>>,

    /// Immutable state, the underlying IoT Hub client connection
    connection: Connection,
}

struct AzureIoTData<F> {
    do_work_timer: eventloop_timer_utilities::EventLoopTimer,
    connection_timer: eventloop_timer_utilities::EventLoopTimer,
    connect_period_seconds: u64,
    authentication_state: AuthenticationState,
    connection_status: ConnectionStatus,
    client_handle: Option<iothub_device_client::IotHubDeviceClient>,

    failure_callback: F,
    callbacks: Rc<RefCell<Callbacks<'static>>>,
}

impl<F: 'static> IoCallbackList for AzureIoT<F> {
    /// Azure timer event:  Check connection status and send telemetry
    fn event(&mut self, fd: i32, _events: IoEvents) {
        let inner = self.inner.as_ref().borrow_mut();
        if fd == unsafe { inner.do_work_timer.fd() } {
            inner.do_work_timer.consume_event().unwrap();
            drop(inner);
            self.do_work_event()
        } else if fd == unsafe { inner.connection_timer.fd() } {
            inner.connection_timer.consume_event().unwrap();
            drop(inner);
            self.connection_timer_event()
        }
    }

    unsafe fn fd_list(&self) -> Vec<i32> {
        let inner = self.inner.as_ref().borrow();
        vec![inner.do_work_timer.fd(), inner.connection_timer.fd()]
    }
}

impl<F> AzureIoTData<F> {
    // See ConnectionCallbackHandler
    fn connection_status_callback(&mut self, status: ConnectionStatus) {
        azs::debug!("AzureIoT::connection_status_callback: {:?}\n", status);
        match status {
            ConnectionStatus::NotStarted => {}
            ConnectionStatus::Started => {
                azs::debug!("INFO: Azure IoT Hub connection started.\n");
            }
            ConnectionStatus::Complete(client_handle) => {
                azs::debug!("INFO: Azure IoT Hub connection complete.\n");
                {
                    self.client_handle = Some(client_handle);

                    // Successfully connected, so make sure the polling frequency is back to the default
                    self.connect_period_seconds = DEFAULT_CONNECT_PERIOD_SECONDS;

                    // Set client authentication state to initiated. This is done to indicate that
                    // SetUpAzureIoTHubClient() has been called (and so should not be called again) while the
                    // client is waiting for a response via the ConnectionStatusCallback().
                    self.authentication_state = AuthenticationState::AuthenticationInitiated;
                }

                let client = self.client_handle.as_ref().unwrap();
                let callback_clone = self.callbacks.clone();
                let _ = client.set_message_callback(Box::new(
                    move |message: iothub_message::IotHubMessage| {
                        // See CloudToDeviceCallback
                        azs::debug!("INFO: Azure IoT Hub message received.\n");
                        if let Some(cb) = &mut callback_clone.as_ref().borrow_mut().cloud_to_device
                        {
                            (*cb)(message)
                        };
                        MessageDisposition::Accepted
                    },
                ));

                let callback_clone = self.callbacks.clone();
                let _ = client.set_device_twin_callback(Box::new(
                    move |_update_state: DeviceTwinUpdateState, vec: Vec<u8>| {
                        // See DeviceTwinCallback
                        // vec[] is a non-null terminated JSON string.  In the C version,
                        // it is null-terminated here in a local buffer.  For Rust, just
                        // convert to String.
                        if let Some(cb) =
                            &mut callback_clone.as_ref().borrow_mut().device_twin_received
                        {
                            let json = String::from_utf8(vec);
                            match json {
                                Err(e) => {
                                    azs::debug!(
                                        "AzureIot DeviceTwin callback - invalid JSON: {:?}\n",
                                        e
                                    )
                                    // bugbug: the C version invoked the failureCallbackFunction here, but it isn't in scope in Rust
                                    // may need to package both .callbakcs and .failure_callback_function into a single refcounted object
                                }
                                Ok(json) => cb(json),
                            }
                        }
                    },
                ));

                let callback_clone = self.callbacks.clone();
                let _ = client.set_device_method_callback(Box::new(
                    move |method_name: String, payload: Vec<u8>| {
                        // See DeviceMethodCallback
                        azs::debug!(
                            "Received Device Method callback: Method name {:?}.\n",
                            method_name
                        );
                        if let Some(cb) = &mut callback_clone.as_ref().borrow_mut().device_method {
                            (*cb)(method_name, payload)
                        } else {
                            let empty_vec: Vec<u8> = Vec::new();
                            (-1, empty_vec)
                        }
                    },
                ));

                let callback_clone = self.callbacks.clone();
                let _ = client.set_connection_status_callback(Box::new(
                    move |result: azs::applibs::iothub_device_client_ll::ConnectionStatus,
                          reason: ConnectionStatusReason| {
                        // See ConnectionStatusCallback
                        azs::debug!("Azure IoT connection status: {:?}\n", reason);
                        if result == azs::applibs::iothub_device_client_ll::ConnectionStatus::Unauthenticated {
                            // bugbug: call ConnectionCallbackHandler(Connection_NotStarted, NULL)
                        }
                        if let Some(cb) = &mut callback_clone.as_ref().borrow_mut().connection_status {
                            (*cb)(result == azs::applibs::iothub_device_client_ll::ConnectionStatus::Authenticated)
                        }

                        
                    },
                ));
            }
            ConnectionStatus::Failed => {
                // If we fail to connect, reduce the polling frequency, starting at
                // AzureIoTMinReconnectPeriodSeconds and with a backoff up to
                // AzureIoTMaxReconnectPeriodSeconds
                self.connect_period_seconds =
                    if self.connect_period_seconds == DEFAULT_CONNECT_PERIOD_SECONDS {
                        MIN_CONNECT_PERIOD_SECONDS
                    } else {
                        let new_period = self.connect_period_seconds * 2;
                        if new_period > MAX_CONNECT_PERIOD_SECONDS {
                            MAX_CONNECT_PERIOD_SECONDS
                        } else {
                            new_period
                        }
                    };
                let connect_period = Duration::new(self.connect_period_seconds, 0);
                let _ = self.connection_timer.set_period(connect_period);

                azs::debug!(
                    "ERROR: Azure IoT Hub connection failed - will retry in {} seconds.\n",
                    self.connect_period_seconds
                );
            }
        }
    }
}

impl<'a, F: 'static> AzureIoT<F> {
    pub fn new(
        model_id: String,
        failure_callback: F,
        callbacks: Callbacks<'static>,
        hostname: String,
    ) -> Result<Self, std::io::Error> {
        let do_work_timer = eventloop_timer_utilities::EventLoopTimer::new()?;
        let do_work_poll_period = Duration::new(0, DO_WORK_INTERVAL_MILLISECOND * 1000000);
        do_work_timer.set_period(do_work_poll_period)?;

        let connection_timer = eventloop_timer_utilities::EventLoopTimer::new()?;
        let connect_period = Duration::new(DEFAULT_CONNECT_PERIOD_SECONDS, 0);
        connection_timer.set_period(connect_period)?;

        let connection = Connection::new(model_id, hostname);

        Ok(Self {
            inner: Rc::new(RefCell::new(AzureIoTData {
                do_work_timer,
                connection_timer,
                connect_period_seconds: DEFAULT_CONNECT_PERIOD_SECONDS,
                authentication_state: AuthenticationState::NotAuthenticated,
                connection_status: ConnectionStatus::NotStarted,
                client_handle: None,
                failure_callback,
                callbacks: Rc::new(RefCell::new(callbacks)),
            })),
            connection,
        })
    }

    // See AzureIoTDoWorkTimerEventHandler
    fn do_work_event(&mut self) {
        if let Some(client) = &self.inner.borrow().client_handle {
            client.do_work();
        }
    }

    // See AzureIoTConnectTimerEventHandler
    fn connection_timer_event(&mut self) {
        if self.inner.borrow().authentication_state == AuthenticationState::NotAuthenticated {
            if networking::is_networking_ready().unwrap_or(false) {
                self.set_up_azureiot_client()
            };
        }
    }

    fn set_up_azureiot_client(&mut self) {
        let mut inner = self.inner.borrow_mut();
        inner.client_handle = None;
        match inner.connection_status {
            ConnectionStatus::NotStarted | ConnectionStatus::Failed => {
                let inner_clone = self.inner.clone();
                let cb = Box::new(move |status| {
                    inner_clone.borrow_mut().connection_status_callback(status)
                });
                self.connection.start(cb)
            }
            _ => {
                // nothing to do
            }
        }
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
            .as_ref()
            .borrow_mut()
            .connection_status
            .as_mut()
            .unwrap()
            .as_mut()(true);

        azs::debug!("AzureIoT Calling device_method\n");
        let test_payload: Vec<u8> = vec![65, 68];
        let result = self
            .inner
            .borrow_mut()
            .callbacks
            .as_ref()
            .borrow_mut()
            .device_method
            .as_mut()
            .unwrap()
            .as_mut()(String::from("test"), test_payload);
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
        if self.inner.as_ref().borrow().authentication_state != AuthenticationState::Authenticated {
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
            .as_ref()
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
        if let Some(client_handle) = self.inner.as_ref().borrow().client_handle.as_ref() {
            // There is a lower-level client handle, so invoke it to do work
            client_handle.do_work();
        }
    }
}
