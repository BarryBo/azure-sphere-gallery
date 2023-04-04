use azure_sphere_sys::applibs::iothub_device_client_ll;
use azure_sphere_sys::applibs::{iothubtransportmqtt, iothubtransportmqtt_websockets};
use std::slice;

/// The transport provider to be used
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransportProvider {
    /// Use the MQTT provider
    MQTT,
    /// Use the MQTT WebSocket provider
    MQTTWebSocket,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClientResult {
    InvalidArg =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_ARG as isize,
    Error = iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_ERROR as isize,
    InvalidSize =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_SIZE as isize,
    IndefiniteTime =
        iothub_device_client_ll::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INDEFINITE_TIME as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContentType {
    ByteArray =
        iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_BYTEARRAY as isize,
    String = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_STRING as isize,
    Unkown = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_UNKNOWN as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageResult {
    Ok = iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK as isize,
    InvalidArg =
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_ARG as isize,
    InvalidType =
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_TYPE as isize,
    Error = iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_ERROR as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ClientRetryPolicy {
    None =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_NONE as isize,
    Immediate =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_IMMEDIATE
            as isize,
    Interval = iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_INTERVAL
        as isize,
    LinearBackoff =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_LINEAR_BACKOFF
            as isize,
    ExponentialBackoff =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF as isize,
    ExponentialBackoffWithJitter =
        iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_EXPONENTIAL_BACKOFF_WITH_JITTER
            as isize,
    Random = iothub_device_client_ll::IOTHUB_CLIENT_RETRY_POLICY_TAG_IOTHUB_CLIENT_RETRY_RANDOM as isize,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum DeviceTwinUpdateState {
    Complete,
    Partial,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionStatus {
    Authenticated,
    Unauthenticated,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConnectionStatusReason {
    ExpiredSasToken,
    DeviceDisabled,
    BadCredential,
    RetryExpired,
    NoNetwork,
    CommunicationError,
    UnknownError,
    Ok,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageDisposition {
    Accepted,
    Rejected,
    Abandoned,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ConfirmationResult {
    Ok,
    BecauseDestroy,
    MessageTimeout,
    Error,
}

// bugbug: possible Rust compiler bug.  This function is reported as dead code, but
// it is called.  However, it is only called from inside a Trait.  Removing it triggers
// a compiler error about the missing function.
#[allow(dead_code)]
fn map_message_result(result: u32) -> Result<(), MessageResult> {
    match result {
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK => Ok(()),
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_ARG => {
            Err(MessageResult::InvalidArg)
        }
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_TYPE => {
            Err(MessageResult::InvalidType)
        }
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_ERROR => {
            Err(MessageResult::Error)
        }
        _ => Err(MessageResult::Error),
    }
}

trait IotHubMessageBase {
    fn get_handle(&self) -> u32;

    // IoTHubMessage_GetByteArray.
    fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        let mut buffer: *const libc::c_uchar = std::ptr::null_mut();
        let mut size: usize = 0;
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetByteArray(
                self.get_handle(),
                &mut buffer,
                &mut size,
            )
        };
        if result != iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK {
            Err(())
        } else {
            let mut bytes = Vec::with_capacity(size);
            unsafe {
                bytes.set_len(size);
                std::ptr::copy_nonoverlapping(buffer, bytes.as_mut_ptr(), size);
            }
            Ok(bytes)
        }
    }

    // IoTHubMessage_GetString
    fn get_string(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetString(self.get_handle())
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_GetContentType
    fn get_content_type(&self) -> ContentType {
        let result =
            unsafe { iothub_device_client_ll::IoTHubMessage_GetContentType(self.get_handle()) };
        match result {
            iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_BYTEARRAY => {
                ContentType::ByteArray
            }
            iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_STRING => {
                ContentType::String
            }
            _ => ContentType::Unkown,
        }
    }

    // IoTHubMessage_SetContentTypeSystemProperty
    fn set_content_type_system_property(&self, content_type: &str) -> Result<(), MessageResult> {
        let content_type_native = std::ffi::CString::new(content_type.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetContentTypeSystemProperty(
                self.get_handle(),
                content_type_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result(result)
    }

    // IoTHubMessage_GetContentTypeSystemProperty
    fn get_content_type_system_property(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetContentTypeSystemProperty(self.get_handle())
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetContentEncodingSystemProperty
    fn set_content_encoding_system_property(
        &self,
        content_type: &str,
    ) -> Result<(), MessageResult> {
        let content_type_native = std::ffi::CString::new(content_type.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetContentEncodingSystemProperty(
                self.get_handle(),
                content_type_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result(result)
    }

    // IoTHubMessage_GetContentEncodingSystemProperty
    fn get_content_encoding_system_property(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetContentEncodingSystemProperty(
                self.get_handle(),
            )
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetProperty
    fn set_property(&self, name: &str, value: &str) -> Result<(), MessageResult> {
        let name_native = std::ffi::CString::new(name.as_bytes()).unwrap();
        let value_native = std::ffi::CString::new(value.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetProperty(
                self.get_handle(),
                name_native.as_ptr() as *const libc::c_char,
                value_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result(result)
    }

    // IoTHubMessage_GetProperty
    fn get_property(&self, name: &str) -> Result<String, ()> {
        let name_native = std::ffi::CString::new(name.as_bytes()).unwrap();
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetProperty(
                self.get_handle(),
                name_native.as_ptr() as *const libc::c_char,
            )
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_GetMessageId
    fn get_message_id(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetMessageId(self.get_handle())
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetMessageId
    fn set_message_id(&self, message_id: &str) -> Result<(), MessageResult> {
        let message_id_native = std::ffi::CString::new(message_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetMessageId(
                self.get_handle(),
                message_id_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result(result)
    }

    // IoTHubMessage_GetCorrelationId
    fn get_correlation_id(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetCorrelationId(self.get_handle())
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetCorrelationId
    fn set_correlation_id(&self, correlation_id: &str) -> Result<(), MessageResult> {
        let correlation_id_native = std::ffi::CString::new(correlation_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetCorrelationId(
                self.get_handle(),
                correlation_id_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result(result)
    }
}

#[derive(Debug)]
pub struct IotHubMessage {
    handle: u32,
}

impl IotHubMessageBase for IotHubMessage {
    fn get_handle(&self) -> u32 {
        self.handle
    }
}

impl IotHubMessage {
    // IoTHubMessage_CreateFromByteArray
    pub fn from_bytearray(byte_array: &[u8]) -> Result<Self, ()> {
        let handle = unsafe {
            iothub_device_client_ll::IoTHubMessage_CreateFromByteArray(
                byte_array.as_ptr() as *const libc::c_uchar,
                byte_array.len(),
            )
        };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self { handle })
        }
    }

    // IoTHubMessage_CreateFromString
    pub fn from_string(source: &str) -> Result<Self, ()> {
        let source_native = std::ffi::CString::new(source.as_bytes()).unwrap();
        let handle = unsafe {
            iothub_device_client_ll::IoTHubMessage_CreateFromString(
                source_native.as_ptr() as *const libc::c_char
            )
        };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self { handle })
        }
    }

    // IoTHubMessage_Clone
    pub fn clone(&self) -> Result<Self, ()> {
        let handle = unsafe { iothub_device_client_ll::IoTHubMessage_Clone(self.get_handle()) };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self { handle })
        }
    }
}

impl Drop for IotHubMessage {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubMessage_Destroy(self.handle) };
    }
}

// An IotHubMessageRef is same as an IotHubMessage except that it doesn't implement Drop
#[derive(Debug)]
pub struct IotHubMessageRef {
    handle: u32,
}

impl IotHubMessageRef {
    pub(crate) fn from_handle(handle: u32) -> IotHubMessageRef {
        IotHubMessageRef { handle }
    }

    // IoTHubMessage_Clone
    pub fn clone(&self) -> Result<Self, ()> {
        let handle = unsafe { iothub_device_client_ll::IoTHubMessage_Clone(self.get_handle()) };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self { handle })
        }
    }
}

impl IotHubMessageBase for IotHubMessageRef {
    fn get_handle(&self) -> u32 {
        self.handle
    }
}

#[derive(Debug)]
pub struct IotHubDeviceClient {
    handle: u32,
}

pub trait EventConfirmationCallback {
    fn cb(&mut self, result: ConfirmationResult);
}

pub trait MessageCallback {
    fn cb(&mut self, message: &IotHubMessageRef) -> MessageDisposition;
}

pub trait ConnectionStatusCallback {
    fn cb(&mut self, result: ConnectionStatus, result_reason: ConnectionStatusReason);
}
pub trait DeviceTwinCallback {
    fn cb(&mut self, update_state: DeviceTwinUpdateState, payload: Vec<u8>);
}

pub trait ReportedStateCallback {
    fn cb(&mut self, status_code: libc::c_int);
}

pub trait DeviceMethodCallback {
    // Returns an HTTP status code and a response payload
    fn cb(&mut self, method_name: &std::ffi::CStr, payload: &Vec<u8>) -> (libc::c_int, Vec<u8>);
}

impl IotHubDeviceClient {
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

    // IoTHubDeviceClient_LL_CreateFromConnectionString
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

    // IoTHubDeviceClient_LL_CreateFromDeviceAuth
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

    // Internal helper called by azure_sphere_provisioning
    pub(crate) unsafe fn from_handle(handle: u32) -> Self {
        Self { handle }
    }

    unsafe extern "C" fn event_confirmation_callback_wrapper(
        result: iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT,
        user_context_callback: *mut libc::c_void,
    ) {
        let context = (user_context_callback as *mut Box<&mut dyn EventConfirmationCallback>)
            .as_mut()
            .unwrap();
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
        context.cb(result)
    }

    // IoTHubDeviceClient_LL_SendEventAsync
    pub fn send_event_async(
        &self,
        event_message: IotHubMessage,
        callback: &mut dyn EventConfirmationCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(
            Box::new(callback) as Box<&mut dyn EventConfirmationCallback>
        ));
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SendEventAsync(
                self.handle,
                event_message.handle,
                Some(Self::event_confirmation_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn message_callback_wrapper(
        message: iothub_device_client_ll::IOTHUB_MESSAGE_HANDLE,
        user_context_callback: *mut libc::c_void,
    ) -> iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT {
        let context = (user_context_callback as *mut Box<&mut dyn MessageCallback>)
            .as_mut()
            .unwrap();

        // BUGBUG: it would be simpler if IotHubMessageRef was removed, and IotHubMessage
        // became a true struct again.  Ignore the message ID here completely, trusting that
        // the caller will hold onto the IotHubMessage in its closure.
        let message = IotHubMessageRef::from_handle(message);
        let result = context.cb(&message);
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

    // IoTHubDeviceClient_LL_SetMessageCallback(
    pub fn set_message_callback(
        &self,
        callback: &mut dyn MessageCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(Box::new(callback) as Box<&mut dyn MessageCallback>));
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetMessageCallback(
                self.handle,
                Some(Self::message_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn connection_status_callback_wrapper(
        result: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS,
        result_reason: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON,
        user_context_callback: *mut libc::c_void,
    ) {
        let connection_status = match result {
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_TAG_IOTHUB_CLIENT_CONNECTION_AUTHENTICATED => {
                ConnectionStatus::Authenticated
            }
            iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_TAG_IOTHUB_CLIENT_CONNECTION_UNAUTHENTICATED => {
                ConnectionStatus::Unauthenticated
            }
            _ => ConnectionStatus::Unauthenticated,
        };

        let result_reason = match result_reason {
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
            _ => ConnectionStatusReason::UnknownError,
        };

        let context = (user_context_callback as *mut Box<&mut dyn ConnectionStatusCallback>)
            .as_mut()
            .unwrap();
        context.cb(connection_status, result_reason)
    }

    // IoTHubDeviceClient_LL_SetConnectionStatusCallback
    pub fn set_connection_status_callback(
        &self,
        callback: &mut dyn ConnectionStatusCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(
            Box::new(callback) as Box<&mut dyn ConnectionStatusCallback>
        ));
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetConnectionStatusCallback(
                self.handle,
                Some(Self::connection_status_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }

    // IoTHubDeviceClient_LL_SetRetryPolicy
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

    // IoTHubDeviceClient_LL_DoWork
    pub fn do_work(&self) {
        unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_DoWork(self.handle) };
    }

    unsafe fn set_option_internal(
        &self,
        option: &std::ffi::CString,
        value: *const libc::c_void,
    ) -> Result<(), ClientResult> {
        let result = iothub_device_client_ll::IoTHubDeviceClient_LL_SetOption(
            self.handle,
            option.as_ptr(),
            value,
        );
        Self::map_client_result(result)
    }

    // IoTHubDeviceClient_LL_SetOption is polymorphic, so we use generics to support many options
    // It is unsafe because the option string and value type must match
    // See https://github.com/Azure/azure-iot-sdk-c/blob/main/doc/Iothub_sdk_options.md
    pub unsafe fn set_option<T>(&self, option: &str, value: T) -> Result<(), ClientResult> {
        let option_name = std::ffi::CString::new(option.as_bytes()).unwrap();
        unsafe { self.set_option_internal(&option_name, &value as *const _ as *const libc::c_void) }
    }

    unsafe extern "C" fn device_twin_callback_wrapper(
        update_state: iothub_device_client_ll::DEVICE_TWIN_UPDATE_STATE,
        payload: *const libc::c_uchar,
        size: usize,
        user_context_callback: *mut libc::c_void,
    ) {
        let payload = std::slice::from_raw_parts(payload, size);
        let update_state = match update_state {
            iothub_device_client_ll::DEVICE_TWIN_UPDATE_STATE_TAG_DEVICE_TWIN_UPDATE_COMPLETE => {
                DeviceTwinUpdateState::Complete
            }
            _ => DeviceTwinUpdateState::Partial,
        };
        let context = (user_context_callback as *mut Box<&mut dyn DeviceTwinCallback>)
            .as_mut()
            .unwrap();
        context.cb(update_state, payload.to_vec());
    }

    // IoTHubDeviceClient_LL_SetDeviceTwinCallback
    pub fn set_device_twin_callback(
        &self,
        callback: &mut dyn DeviceTwinCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(
            Box::new(callback) as Box<&mut dyn DeviceTwinCallback>
        ));

        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetDeviceTwinCallback(
                self.handle,
                Some(Self::device_twin_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn reported_state_callback_wrapper(
        status_code: libc::c_int,
        user_context_callback: *mut libc::c_void,
    ) {
        let context = (user_context_callback as *mut Box<&mut dyn ReportedStateCallback>)
            .as_mut()
            .unwrap();
        context.cb(status_code);
    }

    // IoTHubDeviceClient_LL_SendReportedState
    pub fn send_reported_state(
        &self,
        reported_state: &[u8],
        callback: &mut dyn ReportedStateCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(
            Box::new(callback) as Box<&mut dyn ReportedStateCallback>
        ));

        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SendReportedState(
                self.handle,
                reported_state.as_ptr(),
                reported_state.len(),
                Some(Self::reported_state_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }

    unsafe extern "C" fn device_method_callback_wrapper(
        method_name: *const libc::c_char,
        payload: *const libc::c_uchar,
        size: usize,
        response: *mut *mut libc::c_uchar,
        response_size: *mut usize,
        user_context_callback: *mut libc::c_void,
    ) -> libc::c_int {
        let method_name = std::ffi::CStr::from_ptr(method_name);
        let payload = slice::from_raw_parts(payload, size).to_vec();

        let context = (user_context_callback as *mut Box<&mut dyn DeviceMethodCallback>)
            .as_mut()
            .unwrap();
        let (response_code, response_data) = context.cb(&method_name, &payload);

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

    // IoTHubDeviceClient_LL_SetDeviceMethodCallback
    pub fn set_device_method_callback(
        &self,
        callback: &mut dyn DeviceMethodCallback,
    ) -> Result<(), ClientResult> {
        let context = Box::into_raw(Box::new(
            Box::new(callback) as Box<&mut dyn DeviceMethodCallback>
        ));
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetDeviceMethodCallback(
                self.handle,
                Some(Self::device_method_callback_wrapper),
                context as _,
            )
        };
        Self::map_client_result(result)
    }
    // IoTHubDeviceClient_LL_DeviceMethodResponse
    pub fn device_method_response(
        &self,
        method_id: *mut std::ffi::c_void, // bugbug: improve this type
        response: &[u8],
        status_code: i32,
    ) -> Result<(), ClientResult> {
        let result = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_DeviceMethodResponse(
                self.handle,
                method_id,
                response.as_ptr(),
                response.len(),
                status_code,
            )
        };
        Self::map_client_result(result)
    }
}

impl Drop for IotHubDeviceClient {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_Destroy(self.handle) };
    }
}
