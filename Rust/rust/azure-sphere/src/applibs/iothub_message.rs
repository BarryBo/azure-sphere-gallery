//! The IotHub_Message component encapsulates one message that
//! can be transferred by an IoT hub client.

use azure_sphere_sys::applibs::iothub_device_client_ll;
use std::cell::RefCell;

/// Enumeration specifying the status of calls to various APIs in this module
/// See IOTHUB_MESSAGE_RESULT enum
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum MessageResult {
    /// IOTHUB_MESSAGE_OK
    Ok = iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK as isize,
    /// OTHUB_MESSAGE_INVALID_ARG
    InvalidArg =
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_ARG as isize,
    /// IOTHUB_MESSAGE_INVALID_TYPE
    InvalidType =
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_TYPE as isize,
    /// IOTHUB_MESSAGE_ERROR
    Error = iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_ERROR as isize,
}

/// Enumeration specifying the content type of the a given message.
/// See IOTHUBMESSAGE_CONTENT_TYPE
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ContentType {
    ByteArray =
        iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_BYTEARRAY as isize,
    String = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_STRING as isize,
    Unkown = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_UNKNOWN as isize,
}

/// Helper to map the C enum to the Rust enum
fn map_message_result(result: u32) -> MessageResult {
    match result {
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK => MessageResult::Ok,
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_ARG => {
            MessageResult::InvalidArg
        }
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_INVALID_TYPE => {
            MessageResult::InvalidType
        }
        iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_ERROR => {
            MessageResult::Error
        }
        _ => MessageResult::Error,
    }
}

/// Helper to map the C enum to a Result<(), MessageResult> type.
fn map_message_result_to_result(result: u32) -> Result<(), MessageResult> {
    if result == iothub_device_client_ll::IOTHUB_MESSAGE_RESULT_TAG_IOTHUB_MESSAGE_OK {
        Ok(())
    } else {
        Err(map_message_result(result))
    }
}

/// An IOT_HUB_MESSAGE wrapper
#[derive(Debug)]
pub struct IotHubMessage {
    handle: RefCell<u32>,
}

/// Wrapper for IotHubMessage_* functions.
impl IotHubMessage {
    /// Fetches a pointer and size for the data associated with the IoT
    /// hub message handle. If the content type of the message is not
    /// ContentType::ByteArray then the error is MessageResult::InvalidArg
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetByteArray().
    pub fn get_bytes(&self) -> Result<Vec<u8>, MessageResult> {
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
            Err(map_message_result(result))
        } else {
            let mut bytes = Vec::with_capacity(size);
            unsafe {
                bytes.set_len(size);
                std::ptr::copy_nonoverlapping(buffer, bytes.as_mut_ptr(), size);
            }
            Ok(bytes)
        }
    }

    /// Returns the string stored in the message.
    /// If the content type of the message is not ContentType::String then the function returns Err(()
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetString()
    pub fn get_string(&self) -> Result<String, ()> {
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

    /// Returns the content type of the message
    ///
    /// See IotHubMessage_GetContentType()
    pub fn get_content_type(&self) -> ContentType {
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

    /// Sets the content-type of the message payload, as per supported values on RFC 2046.
    ///
    /// See IotHubMessage_SetContentTypeSystemProperty()
    pub fn set_content_type_system_property(
        &self,
        content_type: &str,
    ) -> Result<(), MessageResult> {
        let content_type_native = std::ffi::CString::new(content_type.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetContentTypeSystemProperty(
                self.get_handle(),
                content_type_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result_to_result(result)
    }

    /// Returns the content-type of the message payload, if defined.
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetContentTypeSystemProperty()
    pub fn get_content_type_system_property(&self) -> Result<String, ()> {
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

    /// Sets the content-encoding of the message payload, as per supported values on RFC 2616.
    ///
    /// See IotHubMessage_SetContentEncodingSystemProperty()
    pub fn set_content_encoding_system_property(
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
        map_message_result_to_result(result)
    }

    /// Returns the content-encoding of the message payload, if defined
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetContentEncodingSystemProperty()
    pub fn get_content_encoding_system_property(&self) -> Result<String, ()> {
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

    /// Sets a property on a Iothub Message.
    ///
    /// See IotHubMessage_SetProperty()
    pub fn set_property(&self, name: &str, value: &str) -> Result<(), MessageResult> {
        let name_native = std::ffi::CString::new(name.as_bytes()).unwrap();
        let value_native = std::ffi::CString::new(value.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetProperty(
                self.get_handle(),
                name_native.as_ptr() as *const libc::c_char,
                value_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result_to_result(result)
    }

    /// Gets a IotHub Message's properties item.
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetProperty()
    pub fn get_property(&self, name: &str) -> Result<String, ()> {
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

    /// Gets the MessageId from the message.
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetMessageId()
    pub fn get_message_id(&self) -> Result<String, ()> {
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

    /// Sets the MessageId for the message.
    ///
    /// See IotHubMessage_SetMessageId()
    pub fn set_message_id(&self, message_id: &str) -> Result<(), MessageResult> {
        let message_id_native = std::ffi::CString::new(message_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetMessageId(
                self.get_handle(),
                message_id_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result_to_result(result)
    }

    /// Gets the CorrelationId from the message.
    ///
    /// Unlike the C SDK, this creates a copy of the underlying buffer.
    ///
    /// See IotHubMessage_GetCorrelationId()
    pub fn get_correlation_id(&self) -> Result<String, ()> {
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

    /// Sets the CorrelationId for the message
    ///
    /// See IotHubMessage_SetCorrelationId()
    pub fn set_correlation_id(&self, correlation_id: &str) -> Result<(), MessageResult> {
        let correlation_id_native = std::ffi::CString::new(correlation_id.as_bytes()).unwrap();
        let result = unsafe {
            iothub_device_client_ll::IoTHubMessage_SetCorrelationId(
                self.get_handle(),
                correlation_id_native.as_ptr() as *const libc::c_char,
            )
        };
        map_message_result_to_result(result)
    }

    pub unsafe fn get_handle(&self) -> u32 {
        *self.handle.as_ptr()
    }

    pub(crate) unsafe fn from_handle(handle: u32) -> IotHubMessage {
        IotHubMessage {
            handle: RefCell::new(handle),
        }
    }

    /// Creates a new IoT hub message from a byte array.  The type will be
    /// ContentType::ByteArray.
    ///
    /// See IotHubMessage_CreateFromByteArray()
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
            Ok(Self {
                handle: RefCell::new(handle),
            })
        }
    }

    /// Creates a new IoT hub message from a string.  The type will be
    /// ContentType::String.
    ///
    /// See IotHubMessage_CreateFromString()
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
            Ok(Self {
                handle: RefCell::new(handle),
            })
        }
    }

    /// Creates a new IoT hub message with the content identical to the current message.
    ///
    /// See IotHubMessage_Clone()
    pub fn clone(&self) -> Result<Self, ()> {
        let handle = unsafe { iothub_device_client_ll::IoTHubMessage_Clone(self.get_handle()) };
        if handle == 0 {
            Err(())
        } else {
            Ok(Self {
                handle: RefCell::new(handle),
            })
        }
    }

    pub(crate) unsafe fn take_handle(&self) -> u32 {
        self.handle.take()
    }
}

impl Drop for IotHubMessage {
    /// Free the underlying message
    fn drop(&mut self) {
        // This is safe to call even if self.handle == 0
        let handle = self.handle.take();
        let _ = unsafe { iothub_device_client_ll::IoTHubMessage_Destroy(handle) };
    }
}
