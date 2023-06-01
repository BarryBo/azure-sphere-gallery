//! APIs that allow a user (usually a device) to communicate with an Azure IotHub.
//!
//! IotHubDeviceClientLowLevel is a module that allows a user (usually a
//! device) to communicate with an Azure IotHub. It can send events
//! and receive messages. At any given moment in time there can only
//! be at most 1 message callback function.
use crate::applibs::iothub_message;
use azure_sphere_sys::applibs::azure_sphere_provisioning;
use azure_sphere_sys::applibs::iothub_client_options;
use azure_sphere_sys::applibs::iothub_device_client_ll;
use azure_sphere_sys::applibs::prov_device_ll_client;
use azure_sphere_sys::applibs::{iothubtransportmqtt, iothubtransportmqtt_websockets};
use std::slice;

/// The transport provider to be used.  In the C SDK, these are function pointers
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransportProvider {
    /// Use the MQTT provider.  const TRANSPORT_PROVIDER* MQTT_Protocol(void)
    MQTT,
    /// Use the MQTT WebSocket provider.  extern const TRANSPORT_PROVIDER* MQTT_WebSocket_Protocol(void);
    MQTTWebSocket,
}

/// Client result.
///
/// See IOTHUB_CLIENT_RESULT
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClientResult {
    /// IOTHUB_CLIENT_INVALID_ARG
    InvalidArg =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_ARG as isize,
    /// IOTHUB_CLIENT_ERROR
    Error = iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_ERROR as isize,
    /// IOTHUB_CLIENT_INVALID_SIZE
    InvalidSize =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_SIZE as isize,
    /// IOTHUB_CLIENT_INDEFINITE_TIME
    IndefiniteTime =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INDEFINITE_TIME as isize,
}

/// Client retry policy.
///
/// See IOTHUB_CLIENT_RETRY_POLICY
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClientRetryPolicy {
    /// IOTHUB_CLIENT_RETRY_NONE
    None =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_NONE as isize,
    /// IOTHUB_CLIENT_RETRY_IMMEDIATE
    Immediate =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_IMMEDIATE
            as isize,
    /// IOTHUB_CLIENT_RETRY_INTERVAL
    Interval = iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_INTERVAL
        as isize,
    /// IOTHUB_CLIENT_RETRY_LINEAR_BACKOFF
    LinearBackoff =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_LINEAR_BACKOFF
            as isize,
    /// IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF1            
    ExponentialBackoff =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF as isize,
    /// IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF_WITH_JITTER
    ExponentialBackoffWithJitter =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF_WITH_JITTER
            as isize,
    /// IOTHUB_CLIENT_RETRY_RANDOM            
    Random = iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_RANDOM as isize,
}

/// See IOTHUB_CLIENT_DEVICE_TWIN_CALLBACK
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceTwinUpdateState {
    /// DEVICE_TWIN_UPDATE_COMPLETE
    Complete,
    /// DEVICE_TWIN_UPDATE_PARTIAL
    Partial,
}

/// Enumeration passed in by the IoT Hub when the event confirmation
/// callback is invoked to indicate status of the event processing in
/// the hub.
///
/// See IOTHUB_CLIENT_CONNECTION_STATUS
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionStatus {
    /// IOTHUB_CLIENT_CONNECTION_AUTHENTICATED
    Authenticated,
    /// IOTHUB_CLIENT_CONNECTION_UNAUTHENTICATED
    Unauthenticated,
}

/// Enumeration passed in by the IoT Hub when the connection status
/// callback is invoked to indicate status of the connection in
/// the hub.
///
/// See IOTHUB_CLIENT_CONNECTION_STATUS_REASON
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionStatusReason {
    /// IOTHUB_CLIENT_CONNECTION_EXPIRED_SAS_TOKEN
    ExpiredSasToken,
    /// IOTHUB_CLIENT_CONNECTION_DEVICE_DISABLED
    DeviceDisabled,
    /// IOTHUB_CLIENT_CONNECTION_BAD_CREDENTIAL
    BadCredential,
    /// IOTHUB_CLIENT_CONNECTION_RETRY_EXPIRED
    RetryExpired,
    /// IOTHUB_CLIENT_CONNECTION_NO_NETWORK
    NoNetwork,
    /// IOTHUB_CLIENT_CONNECTION_COMMUNICATION_ERROR
    CommunicationError,
    /// IOTHUB_CLIENT_CONNECTION_OK
    Ok,
    /// IOTHUB_CLIENT_CONNECTION_NO_PING_RESPONSE
    NoPingResponse,
    /// Unknown/unexpected error returned from the IoT C SDK
    UnknownError,
}

/// Enumeration passed in by the IoT Hub when the event confirmation
/// callback is invoked to indicate status of the event processing in
/// the hub.
///
/// See IOTHUBMESSAGE_DISPOSITION_RESULT
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageDisposition {
    /// IOTHUBMESSAGE_ACCEPTED
    Accepted,
    /// IOTHUBMESSAGE_REJECTED
    Rejected,
    /// IOTHUBMESSAGE_ABANDONED
    Abandoned,
}

/// Enumeration passed in by the IoT Hub when the event confirmation
/// callback is invoked to indicate status of the event processing in
/// the hub.
///
/// See IOTHUB_CLIENT_CONFIRMATION_RESULT
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConfirmationResult {
    /// IOTHUB_CLIENT_CONFIRMATION_OK
    Ok,
    /// IOTHUB_CLIENT_CONFIRMATION_BECAUSE_DESTROY
    BecauseDestroy,
    /// IOTHUB_CLIENT_CONFIRMATION_MESSAGE_TIMEOUT
    MessageTimeout,
    /// IOTHUB_CLIENT_CONFIRMATION_ERROR
    Error,
}

/// See HTTP_PROXY_OPTIONS
pub struct HttpProxyOptions {
    /// Host address
    host_address: String,
    /// Port
    port: u16,
    /// User name
    username: Option<String>,
    /// Password
    password: Option<Vec<u8>>, // not String, as it doesn't have to be strictly UTF-8
}

/// An IoT Hub device client wrapper.
#[derive(Debug)]
pub struct IotHubDeviceClientLowLevel {
    handle: u32,
}

impl IotHubDeviceClientLowLevel {
    /// Helper to map a client result from C to a Result<>
    fn map_client_result(result: u32) -> Result<(), ClientResult> {
        match result {
            iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_OK => Ok(()),
            iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_ARG => {
                Err(ClientResult::InvalidArg)
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_ERROR => {
                Err(ClientResult::Error)
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_SIZE => {
                Err(ClientResult::InvalidSize)
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INDEFINITE_TIME => {
                Err(ClientResult::IndefiniteTime)
            }
            _ => Err(ClientResult::Error),
        }
    }

    /// Creates a IoT Hub client for communication with an existing
    /// IoT Hub using the specified connection string parameter.
    ///
    /// Sample connection string:
    ///   HostName=[IoT Hub name goes here].[IoT Hub suffix goes here, e.g., private.azure-devices-int.net];DeviceId=[Device ID goes here];SharedAccessKey=[Device key goes here];
    ///
    /// See IotHubDeviceClient_LL_CreateFromConnectionString()
    pub fn from_connection_string(
        connection_string: &str,
        protocol: TransportProvider,
    ) -> Result<Self, std::io::Error> {
        let connection_string_native =
            std::ffi::CString::new(connection_string.as_bytes()).unwrap();
        let protocol_native = match protocol {
            TransportProvider::MQTT => {
                iothubtransportmqtt::MQTT_Protocol as *mut libc::c_void as u32
            }
            TransportProvider::MQTTWebSocket => {
                iothubtransportmqtt_websockets::MQTT_WebSocket_Protocol as *mut libc::c_void as u32
            }
        };
        let handle = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_CreateFromConnectionString(
                connection_string_native.as_ptr() as *const libc::c_char,
                protocol_native,
            )
        };
        if handle == 0 {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        } else {
            Ok(Self { handle })
        }
    }

    /// Creates a IoT Hub client for communication with an existing IoT
    /// Hub using the device auth module.
    ///
    /// See IotHubDeviceClientLL_CreateFromDeviceAuth()
    pub fn from_device_auth(
        iothub_uri: &str,
        device_id: &str,
        protocol: TransportProvider,
    ) -> Result<Self, std::io::Error> {
        let iothub_uri_native = std::ffi::CString::new(iothub_uri.as_bytes()).unwrap();
        let device_id_native = std::ffi::CString::new(device_id.as_bytes()).unwrap();
        let protocol_native = match protocol {
            TransportProvider::MQTT => {
                iothubtransportmqtt::MQTT_Protocol as *mut libc::c_void as u32
            }
            TransportProvider::MQTTWebSocket => {
                iothubtransportmqtt_websockets::MQTT_WebSocket_Protocol as *mut libc::c_void as u32
            }
        };
        let handle = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_CreateFromDeviceAuth(
                iothub_uri_native.as_ptr() as *const libc::c_char,
                device_id_native.as_ptr() as *const libc::c_char,
                protocol_native,
            )
        };
        if handle == 0 {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        } else {
            Ok(Self { handle })
        }
    }

    /// Internal helper called by azure_sphere_provisioning
    pub(crate) unsafe fn from_handle(handle: u32) -> Self {
        Self { handle }
    }

    unsafe extern "C" fn event_confirmation_callback_wrapper<F>(
        result: iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT,
        user_context_callback: *mut libc::c_void,
    ) where
        F: FnMut(ConfirmationResult),
    {
        let callback = &mut *(user_context_callback as *mut F);
        let result = match result {
            iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT_TAG_IOTHUB_CLIENT_CONFIRMATION_OK => {
                ConfirmationResult::Ok
            },
            iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT_TAG_IOTHUB_CLIENT_CONFIRMATION_BECAUSE_DESTROY => {
                ConfirmationResult::BecauseDestroy
            },
            iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT_TAG_IOTHUB_CLIENT_CONFIRMATION_MESSAGE_TIMEOUT => {
                ConfirmationResult::MessageTimeout
            },
            _ => {
                ConfirmationResult::Error
            },
        };
        callback(result)
    }

    /// Asynchronous call to send the message
    ///
    /// See IotHubDeviceClientLL_SendEventAsync()
    pub fn send_event_async<F>(
        &self,
        event_message: iothub_message::IotHubMessage,
        callback: F,
    ) -> Result<(), ClientResult>
    where
        F: FnMut(ConfirmationResult),
    {
        let result = unsafe {
            let event_message_handle = event_message.take_handle();
            let mut context = callback;
            iothub_device_client_ll::IoTHubDeviceClient_LL_SendEventAsync(
                self.handle,
                event_message_handle,
                Some(Self::event_confirmation_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn message_callback_wrapper<F>(
        message: iothub_device_client_ll::IOTHUB_MESSAGE_HANDLE,
        user_context_callback: *mut libc::c_void,
    ) -> iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT
    where
        F: FnMut(iothub_message::IotHubMessage) -> MessageDisposition,
    {
        let callback = &mut *(user_context_callback as *mut F);

        // BUGBUG: it would be simpler if IotHubMessageRef was removed, and IotHubMessage
        // became a true struct again.  Ignore the message ID here completely, trusting that
        // the caller will hold onto the IotHubMessage in its closure.
        let message = iothub_message::IotHubMessage::from_handle(message);
        let result = callback(message);
        match result {
            MessageDisposition::Abandoned => {
                iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT_TAG_IOTHUBMESSAGE_ABANDONED
            },
            MessageDisposition::Accepted => {
                iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT_TAG_IOTHUBMESSAGE_ACCEPTED
            },
            _ => {
                iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT_TAG_IOTHUBMESSAGE_REJECTED
            },
        }
    }

    /// Sets up the message callback to be invoked when IoT Hub issues a
    /// message to the device. This is a blocking call.
    ///
    /// See IotHubDeviceClientLL_SetMessageCallback()
    pub fn set_message_callback<F>(&self, callback: F) -> Result<(), ClientResult>
    where
        F: FnMut(iothub_message::IotHubMessage) -> MessageDisposition,
    {
        let mut context = callback;
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetMessageCallback(
                self.handle,
                Some(Self::message_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    fn map_connection_status(result_reason: u32) -> ConnectionStatusReason {
        match result_reason {
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_EXPIRED_SAS_TOKEN => {
                ConnectionStatusReason::ExpiredSasToken
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_DEVICE_DISABLED => {
                ConnectionStatusReason::DeviceDisabled
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_BAD_CREDENTIAL => {
                ConnectionStatusReason::BadCredential
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_RETRY_EXPIRED => {
                ConnectionStatusReason::RetryExpired
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_NO_NETWORK => {
                ConnectionStatusReason::NoNetwork
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_COMMUNICATION_ERROR => {
                ConnectionStatusReason::CommunicationError
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_OK => {
                ConnectionStatusReason::Ok
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON_TAG_IOTHUB_CLIENT_CONNECTION_NO_PING_RESPONSE => {
                ConnectionStatusReason::NoPingResponse
            }
            _ => ConnectionStatusReason::UnknownError,
        }
    }

    unsafe extern "C" fn connection_status_callback_wrapper<F>(
        result: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS,
        result_reason: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON,
        user_context_callback: *mut libc::c_void,
    ) where
        F: FnMut(ConnectionStatus, ConnectionStatusReason),
    {
        let connection_status = match result {
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_TAG_IOTHUB_CLIENT_CONNECTION_AUTHENTICATED => {
                ConnectionStatus::Authenticated
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_TAG_IOTHUB_CLIENT_CONNECTION_UNAUTHENTICATED => {
                ConnectionStatus::Unauthenticated
            }
            _ => ConnectionStatus::Unauthenticated,
        };

        let result_reason = Self::map_connection_status(result_reason);

        let callback = &mut *(user_context_callback as *mut F);
        callback(connection_status, result_reason)
    }

    /// Sets up the connection status callback to be invoked representing the status of
    /// the connection to IOT Hub. This is a blocking call.
    ///
    /// See IotHubDeviceClientLL_SetConnectionStatusCallback()
    pub fn set_connection_status_callback<F>(&self, callback: F) -> Result<(), ClientResult>
    where
        F: FnMut(ConnectionStatus, ConnectionStatusReason),
    {
        let mut context = callback;
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetConnectionStatusCallback(
                self.handle,
                Some(Self::connection_status_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    /// Set the retry policy
    ///
    /// See IotHubDeviceClientLL_SetRetryPolicy()
    pub fn set_retry_policy(
        &self,
        retry_policy: ClientRetryPolicy,
        retry_timeout_limit_in_seconds: usize,
    ) -> Result<(), ClientResult> {
        let retry_policy_native = match retry_policy {
            ClientRetryPolicy::None => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_NONE
            }
            ClientRetryPolicy::Immediate => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_IMMEDIATE
            }
            ClientRetryPolicy::LinearBackoff => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_LINEAR_BACKOFF
            }
            ClientRetryPolicy::ExponentialBackoff => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF
            }
            ClientRetryPolicy::ExponentialBackoffWithJitter => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF_WITH_JITTER
            }
            ClientRetryPolicy::Random => {
                iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_RANDOM
            }
            _ => return Err(ClientResult::Error),
        };
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetRetryPolicy(
                self.handle,
                retry_policy_native,
                retry_timeout_limit_in_seconds,
            )
        };
        Self::map_client_result(result)
    }

    /// Get the retry policy
    ///
    /// See IotHubDeviceClientLL_GetRetryPolicy()
    pub fn get_retry_policy(&self) -> Result<(ClientRetryPolicy, usize), ClientResult> {
        let mut retry_policy_native: u32 = 0;
        let mut retry_timeout_limit_in_seconds: usize = 0;
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_GetRetryPolicy(
                self.handle,
                &mut retry_policy_native,
                &mut retry_timeout_limit_in_seconds,
            )
        };
        let result = Self::map_client_result(result);
        if result.is_err() {
            return Err(result.unwrap_err());
        }
        let retry_policy = match retry_policy_native {
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_NONE => {
                ClientRetryPolicy::None
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_IMMEDIATE => {
                ClientRetryPolicy::Immediate
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_LINEAR_BACKOFF => {
                ClientRetryPolicy::LinearBackoff
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF => {
                ClientRetryPolicy::ExponentialBackoff
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF_WITH_JITTER => {
                ClientRetryPolicy::ExponentialBackoffWithJitter
            }
            iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_RANDOM => {
                ClientRetryPolicy::Random
            }
            _ => return Err(ClientResult::Error),
        };
        Ok((retry_policy, retry_timeout_limit_in_seconds))
    }

    /// This function MUST be called by the user so work (sending/receiving data on the wire,
    /// computing and enforcing timeout controls, managing the connection to the IoT Hub) can
    /// be done by the IotHubClient.
    /// The recommended call frequency is at least once every 100 milliseconds.
    ///
    /// See IotHubDeviceClientLL_DoWork)_
    pub fn do_work(&self) {
        unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_DoWork(self.handle) };
    }

    /// Helper function on top of IotHubDeviceClientLL_SetOption()
    pub(crate) unsafe fn set_option_internal(
        // unsafe because it consumes unsafe types
        &self,
        option: *const libc::c_char,
        value: *const libc::c_void,
    ) -> Result<(), ClientResult> {
        let result =
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetOption(self.handle, option, value);
        Self::map_client_result(result)
    }

    /// Helper function on top of IotHubDeviceClientLL_SetOption()
    // IotHubDeviceClientLL_SetOption is polymorphic, so we use generics to support many options
    // It is unsafe because the option string and value type must match
    // See https://github.com/Azure/azure-iot-sdk-c/blob/main/doc/Iothub_sdk_options.md
    pub unsafe fn set_option<T>(&self, option: &str, value: T) -> Result<(), ClientResult> {
        let option_name = std::ffi::CString::new(option.as_bytes()).unwrap();
        unsafe {
            self.set_option_internal(
                option_name.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_LOG_TRACE/PROV_OPTION_LOG_TRACE (bool*)
    pub fn set_option_log_trace(&self, value: bool) -> Result<(), ClientResult> {
        unsafe {
            let value = if value { 1u8 } else { 0u8 };
            self.set_option_internal(
                iothub_client_options::OPTION_LOG_TRACE.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for PROV_REGISTRATION_ID (const char*)
    pub fn set_option_registration_id(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                prov_device_ll_client::PROV_REGISTRATION_ID.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for PROV_OPTION_TIMEOUT (long*)
    pub fn set_option_timeout(&self, value: usize) -> Result<(), ClientResult> {
        unsafe {
            self.set_option_internal(
                prov_device_ll_client::PROV_OPTION_TIMEOUT.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_X509_CERT (const char*)
    pub fn set_option_x509_cert(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                iothub_client_options::OPTION_X509_CERT.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_X509_PRIVATE_KEY (const char*)
    pub fn set_option_x509_private_key(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                iothub_client_options::OPTION_X509_PRIVATE_KEY.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_KEEP_ALIVE (int*)
    pub fn set_option_keep_alive(&self, value: i32) -> Result<(), ClientResult> {
        unsafe {
            self.set_option_internal(
                iothub_client_options::OPTION_KEEP_ALIVE.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_CONNECTION_TIMEOUT (int*)
    pub fn set_option_connection_timeout(&self, value: i32) -> Result<(), ClientResult> {
        unsafe {
            self.set_option_internal(
                iothub_client_options::OPTION_CONNECTION_TIMEOUT.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_SAS_TOKEN_LIFETIME (size_t*)
    pub fn set_option_sas_token_lifetime(&self, value: usize) -> Result<(), ClientResult> {
        unsafe {
            self.set_option_internal(
                iothub_client_options::OPTION_SAS_TOKEN_LIFETIME.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    // deprecated: IotHubDeviceClientLL_SetOption for OPTION_SAS_TOKEN_LIFETIME (size_t*)

    /// IotHubDeviceClientLL_SetOption for OPTION_PRODUCT_INFO (const char*)
    pub fn set_option_product_info(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                iothub_client_options::OPTION_PRODUCT_INFO.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_MODEL_ID (const char*)
    pub fn set_option_model_id(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                iothub_client_options::OPTION_MODEL_ID.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_AUTO_URL_ENCODE_DECODE (bool*)
    pub fn set_option_auto_url_encode_decode(&self, value: bool) -> Result<(), ClientResult> {
        unsafe {
            let value = if value { 1u8 } else { 0u8 };
            self.set_option_internal(
                iothub_client_options::OPTION_AUTO_URL_ENCODE_DECODE.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    // deprecated: IotHubDeviceClientLL_SetOption for OPTION_MESSAGE_TIMEOUT

    /// IotHubDeviceClientLL_SetOption for OPTION_HTTP_PROXY (HTTP_PROXY_OPTIONS*)
    pub fn option_set_http_proxy(&self, value: &HttpProxyOptions) -> Result<(), ClientResult> {
        let port = value.port as i32;
        unsafe {
            let host_address = std::ffi::CString::new(value.host_address.as_bytes()).unwrap();
            let username = if let Some(username) = value.username.as_ref() {
                std::ffi::CString::new(username.as_bytes())
                    .unwrap()
                    .as_bytes()
                    .as_ptr()
            } else {
                std::ptr::null() as *const libc::c_char
            };
            let password = if let Some(password) = value.password.as_ref() {
                password.as_ptr()
            } else {
                std::ptr::null() as *const libc::c_char
            };
            let option = azure_sphere_provisioning::HTTP_PROXY_OPTIONS_TAG {
                host_address: host_address.as_ptr(),
                port,
                username,
                password,
            };
            self.set_option_internal(
                azure_sphere_provisioning::OPTION_HTTP_PROXY.as_ptr(),
                &option as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_TRUSTED_CERT (const char*)
    pub fn option_set_trusted_cert(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                azure_sphere_provisioning::OPTION_TRUSTED_CERT.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_X509_ECC_CERT (const char*)
    pub fn option_set_x509_ecc_cert(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                azure_sphere_provisioning::OPTION_X509_ECC_CERT.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_X509_ECC_KEY (const char*)
    pub fn option_set_x509_ecc_key(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.set_option_internal(
                azure_sphere_provisioning::OPTION_X509_ECC_KEY.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_TLS_VERSION (int*)
    pub fn option_tls_version(&self, value: i32) -> Result<(), ClientResult> {
        unsafe {
            self.set_option_internal(
                azure_sphere_provisioning::OPTION_TLS_VERSION.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    unsafe extern "C" fn device_twin_callback_wrapper<F>(
        update_state: iothub_device_client_ll::DEVICE_TWIN_UPDATE_STATE,
        payload: *const libc::c_uchar,
        size: usize,
        user_context_callback: *mut libc::c_void,
    ) where
        F: FnMut(DeviceTwinUpdateState, Vec<u8>),
    {
        let payload = std::slice::from_raw_parts(payload, size);
        let update_state = match update_state {
            iothub_device_client_ll::DEVICE_TWIN_UPDATE_STATE_TAG_DEVICE_TWIN_UPDATE_COMPLETE => {
                DeviceTwinUpdateState::Complete
            }
            _ => DeviceTwinUpdateState::Partial,
        };
        let callback = &mut *(user_context_callback as *mut F);
        callback(update_state, payload.to_vec());
    }

    /// This API specifies a callback to be used when the device receives a desired state update.
    /// See IotHubDeviceClientLL_SetDeviceTwinCallback()
    pub fn set_device_twin_callback<F>(
        &self,
        callback: F, // BUGBUG: this should be Option()
    ) -> Result<(), ClientResult>
    where
        F: FnMut(DeviceTwinUpdateState, Vec<u8>),
    {
        let mut context = callback;

        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetDeviceTwinCallback(
                self.handle,
                Some(Self::device_twin_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn reported_state_callback_wrapper<F>(
        status_code: libc::c_int,
        user_context_callback: *mut libc::c_void,
    ) where
        F: FnMut(ConnectionStatusReason),
    {
        let callback = &mut *(user_context_callback as *mut F);
        let status_code = Self::map_connection_status(status_code as u32);
        (*callback)(status_code);
    }

    /// This API sends a report of the device's properties and their current values.
    ///
    /// See IotHubDeviceClientLL_SendReportedState()
    pub fn send_reported_state<F>(
        &self,
        reported_state: &[u8],
        callback: F, // BUGBUG: this should be Option()
    ) -> Result<(), ClientResult>
    where
        F: FnMut(ConnectionStatusReason),
    {
        let mut context = callback;
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SendReportedState(
                self.handle,
                reported_state.as_ptr(),
                reported_state.len(),
                Some(Self::reported_state_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn device_method_callback_wrapper<F>(
        method_name: *const libc::c_char,
        payload: *const libc::c_uchar,
        size: usize,
        response: *mut *mut libc::c_uchar,
        response_size: *mut usize,
        user_context_callback: *mut libc::c_void,
    ) -> libc::c_int
    where
        F: FnMut(String, Vec<u8>) -> (i32, Vec<u8>),
    {
        let method_name = std::ffi::CStr::from_ptr(method_name);
        let method_name = method_name.to_string_lossy().into_owned();
        let payload = slice::from_raw_parts(payload, size).to_vec();

        let callback = &mut *(user_context_callback as *mut F);
        let (response_code, response_data) = callback(method_name, payload);

        // `response` must be memory allocated via C malloc() and doesn't need to be null-terminated
        let response_native = libc::malloc(response_data.len());
        std::ptr::copy_nonoverlapping(
            response_data.as_ptr(),
            response_native as *mut libc::c_uchar,
            response_data.len(),
        );

        *response = response_native as *mut libc::c_uchar;
        *response_size = response_data.len();
        response_code
    }

    /// This API sets the callback for async cloud to device method calls.
    ///
    /// See IotHubDeviceClientLL_SetDeviceMethodCallback()
    pub fn set_device_method_callback<F>(
        &self,
        callback: F, // BUGBUG: this should be Option()
    ) -> Result<(), ClientResult>
    where
        F: FnMut(String, Vec<u8>) -> (i32, Vec<u8>),
    {
        let mut context = callback;
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetDeviceMethodCallback(
                self.handle,
                Some(Self::device_method_callback_wrapper::<F>),
                &mut context as *mut _ as *mut std::ffi::c_void,
            )
        };
        Self::map_client_result(result)
    }

    // IotHubDeviceClientLL_DeviceMethodResponse() is deprecated.
}

impl Drop for IotHubDeviceClientLowLevel {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_Destroy(self.handle) };
    }
}
