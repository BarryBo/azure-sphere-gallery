use azure_sphere_sys::applibs::iothub_device_client_ll;
use azure_sphere_sys::applibs::{iothubtransportmqtt, iothubtransportmqtt_websockets};

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

#[derive(Debug)]
pub struct IotHubMessage {
    handle: u32,
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
        let handle = unsafe { iothub_device_client_ll::IoTHubMessage_Clone(self.handle) };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self { handle })
        }
    }

    // IoTHubMessage_GetByteArray.
    pub fn get_bytes(&self) -> Result<Vec<u8>, ()> {
        let mut buffer: *const libc::c_uchar = std::ptr::null_mut();
        let mut size: usize = 0;
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetByteArray(self.handle, &mut buffer, &mut size)
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
    pub fn get_string(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetString(self.handle)
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_GetContentType
    pub fn get_content_type(&self) -> ContentType {
        let result = unsafe { iothub_device_client_ll::IoTHubMessage_GetContentType(self.handle) };
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
    // IoTHubMessage_SetContentTypeSystemProperty
    pub fn set_content_type_system_property(
        &self,
        content_type: &str,
    ) -> Result<(), MessageResult> {
        let content_type_native = std::ffi::CString::new(content_type.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetContentTypeSystemProperty(
                self.handle,
                content_type_native.as_ptr() as *const libc::c_char,
            )
        };
        Self::map_message_result(result)
    }

    // IoTHubMessage_GetContentTypeSystemProperty
    pub fn get_content_type_system_property(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetContentTypeSystemProperty(self.handle)
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetContentEncodingSystemProperty
    pub fn set_content_encoding_system_property(
        &self,
        content_type: &str,
    ) -> Result<(), MessageResult> {
        let content_type_native = std::ffi::CString::new(content_type.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetContentEncodingSystemProperty(
                self.handle,
                content_type_native.as_ptr() as *const libc::c_char,
            )
        };
        Self::map_message_result(result)
    }

    // IoTHubMessage_GetContentEncodingSystemProperty
    pub fn get_content_encoding_system_property(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetContentEncodingSystemProperty(self.handle)
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetProperty
    pub fn set_property(&self, name: &str, value: &str) -> Result<(), MessageResult> {
        let name_native = std::ffi::CString::new(name.as_bytes()).unwrap();
        let value_native = std::ffi::CString::new(value.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetProperty(
                self.handle,
                name_native.as_ptr() as *const libc::c_char,
                value_native.as_ptr() as *const libc::c_char,
            )
        };
        Self::map_message_result(result)
    }

    // IoTHubMessage_GetProperty
    pub fn get_property(&self, name: &str) -> Result<String, ()> {
        let name_native = std::ffi::CString::new(name.as_bytes()).unwrap();
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetProperty(
                self.handle,
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
    pub fn get_message_id(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetMessageId(self.handle)
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetMessageId
    pub fn set_message_id(&self, message_id: &str) -> Result<(), MessageResult> {
        let message_id_native = std::ffi::CString::new(message_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetMessageId(
                self.handle,
                message_id_native.as_ptr() as *const libc::c_char,
            )
        };
        Self::map_message_result(result)
    }
    // IoTHubMessage_GetCorrelationId
    pub fn get_correlation_id(&self) -> Result<String, ()> {
        let result = unsafe {
            // Returns a shallow copy of the data.  Do not free it.
            iothub_device_client_ll::IoTHubMessage_GetCorrelationId(self.handle)
        };
        if result != std::ptr::null() {
            Err(())
        } else {
            let string = unsafe { std::ffi::CStr::from_ptr(result) };
            Ok(string.to_string_lossy().into_owned())
        }
    }

    // IoTHubMessage_SetCorrelationId
    pub fn set_correlation_id(&self, correlation_id: &str) -> Result<(), MessageResult> {
        let correlation_id_native = std::ffi::CString::new(correlation_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetCorrelationId(
                self.handle,
                correlation_id_native.as_ptr() as *const libc::c_char,
            )
        };
        Self::map_message_result(result)
    }
}

impl Drop for IotHubMessage {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubMessage_Destroy(self.handle) };
    }
}

#[derive(Debug)]
pub struct IotHubDeviceClient {
    handle: u32,
}

// bugbug: temp placeholder for the build
unsafe extern "C" fn temp_event_confirmation_callback(
    result: iothub_device_client_ll::IOTHUB_CLIENT_CONFIRMATION_RESULT,
    user_context_callback: *mut libc::c_void,
) {
    println!(
        "temp_event_confirmation_callback: result={:?} user_context_callback={:?}",
        result, user_context_callback
    );
}

// bugbug: temp placeholder for the build
unsafe extern "C" fn temp_message_callback(
    message: iothub_device_client_ll::IOTHUB_MESSAGE_HANDLE,
    user_context_callback: *mut libc::c_void,
) -> iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT {
    println!(
        "temp_message_callback: message={:?} user_context_callback={:?}",
        message, user_context_callback
    );
    iothub_device_client_ll::IOTHUBMESSAGE_DISPOSITION_RESULT_TAG_IOTHUBMESSAGE_ACCEPTED
}

// bugbug: temp placeholder for the build
unsafe extern "C" fn temp_connection_status_callback(
    result: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS,
    result_reason: iothub_device_client_ll::IOTHUB_CLIENT_CONNECTION_STATUS_REASON,
    user_context_callback: *mut libc::c_void,
) {
    println!(
        "temp_connection_status_callback: result={:?} result_reason={:?} user_context_callback={:?}",
        result, result_reason, user_context_callback
    );
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

    // IoTHubDeviceClient_LL_SendEventAsync
    pub fn send_event_async(
        &self,
        event_message: IotHubMessage,
        _event_confirmation_callback: u32,
        _user_context_callback: u32,
    ) -> Result<(), ClientResult> {
        let result = unsafe {
            // bugbug: fill in the args
            iothub_device_client_ll::IoTHubDeviceClient_LL_SendEventAsync(
                self.handle,
                event_message.handle,
                Some(temp_event_confirmation_callback),
                std::ptr::null_mut(),
            )
        };
        Self::map_client_result(result)
    }

    // IoTHubDeviceClient_LL_SetMessageCallback(
    pub fn set_message_callback(
        &self,
        _message_callback: u32,
        _user_contenxt_callback: u32,
    ) -> Result<(), ClientResult> {
        let result = unsafe {
            // bugbug: fill in the args
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetMessageCallback(
                self.handle,
                Some(temp_message_callback),
                std::ptr::null_mut(),
            )
        };
        Self::map_client_result(result)
    }

    // IoTHubDeviceClient_LL_SetConnectionStatusCallback
    pub fn set_connection_status_callback(
        &self,
        _connection_status_callback: u32,
        _user_context_callback: u32,
    ) -> Result<(), ClientResult> {
        let result = unsafe {
            // bugbug: fill in the args
            iothub_device_client_ll::IoTHubDeviceClient_LL_SetConnectionStatusCallback(
                self.handle,
                Some(temp_connection_status_callback),
                std::ptr::null_mut(),
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
    // IoTHubDeviceClient_LL_SetOption
    // IoTHubDeviceClient_LL_SetDeviceTwinCallback
    // IoTHubDeviceClient_LL_SendReportedState
    // IoTHubDeviceClient_LL_SetDeviceMethodCallback
    // IoTHubDeviceClient_LL_DeviceMethodResponse
}

impl Drop for IotHubDeviceClient {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_Destroy(self.handle) };
    }
}
