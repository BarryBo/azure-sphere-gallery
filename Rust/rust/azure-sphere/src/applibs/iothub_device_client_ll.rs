use azure_sphere_sys::applibs::iothub_device_client_ll;
use azure_sphere_sys::applibs::{iothubtransportmqtt, iothubtransportmqtt_websockets};

/// The transport provider to be used
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransportProvider {
    /// Use the MQTT provider
    MQTT,
    /// Use the MQTT WebSocket provider
    MQTTWebSocket
}

#[derive(Debug)]
pub struct IotHubDeviceClient {
    handle: u32,
}

impl IotHubDeviceClient {
    pub fn from_connection_string(connection_string: &str, protocol:TransportProvider) -> Result<Self, std::io::Error> {
        let connection_string_native = std::ffi::CString::new(connection_string.as_bytes()).unwrap();
        let protocol_native = match protocol {
            TransportProvider::MQTT => iothubtransportmqtt::MQTT_Protocol as *mut libc::c_void as u32,
            TransportProvider::MQTTWebSocket => iothubtransportmqtt_websockets::MQTT_WebSocket_Protocol as *mut libc::c_void as u32
        };
        let handle = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_CreateFromConnectionString(
                connection_string_native.as_ptr() as *const libc::c_char,
                protocol_native
            )
        };
        if handle == 0 {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        } else {
            Ok(Self { handle })
        }
    }

    pub fn from_device_auth(iothub_uri: &str, device_id:&str, protocol:TransportProvider) -> Result<Self, std::io::Error> {
        let iothub_uri_native = std::ffi::CString::new(iothub_uri.as_bytes()).unwrap();
        let device_id_native = std::ffi::CString::new(device_id.as_bytes()).unwrap();
        let protocol_native = match protocol {
            TransportProvider::MQTT => iothubtransportmqtt::MQTT_Protocol as *mut libc::c_void as u32,
            TransportProvider::MQTTWebSocket => iothubtransportmqtt_websockets::MQTT_WebSocket_Protocol as *mut libc::c_void as u32
        };
        let handle = unsafe {
            iothub_device_client_ll::IoTHubDeviceClient_LL_CreateFromDeviceAuth(
                iothub_uri_native.as_ptr() as *const libc::c_char,
                device_id_native.as_ptr() as *const libc::c_char,
                protocol_native
            )
        };
        if handle == 0 {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        } else {
            Ok(Self { handle })
        }
    }

    pub(crate) unsafe fn from_handle(handle:u32) -> Self {
        Self { handle }
    }
}

impl Drop for IotHubDeviceClient {
    fn drop(&mut self) {
        let _ = unsafe { iothub_device_client_ll::IoTHubDeviceClient_LL_Destroy(self.handle) };
    }
}
