use azure_sphere_sys::applibs::iothub_device_client_ll;

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
pub enum ContentType {
    ByteArray =
        iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_BYTEARRAY as isize,
    String = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_STRING as isize,
    Unkown = iothub_device_client_ll::IOTHUBMESSAGE_CONTENT_TYPE_TAG_IOTHUBMESSAGE_UNKNOWN as isize,
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

pub trait IotHubMessageBase {
    unsafe fn get_handle(&self) -> u32;

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
    unsafe fn get_handle(&self) -> u32 {
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
    unsafe fn get_handle(&self) -> u32 {
        self.handle
    }
}
