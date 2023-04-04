//! Azure Sphere specific IoT Hub APIs
//! 
//! Use this function to get an Azure IoT Hub handle when you use the Device
//! Provisioning Service (DPS) with Azure IoT Hub. The function wraps much of
//! the logic with authentication to DPS, creating a device, registering a
//! device, and getting an Azure IoT Hub handle for you.

use azure_sphere_sys::applibs::azure_sphere_provisioning;
use crate::applibs::iothub_device_client_ll;
use azure_sphere_sys::applibs::{iothubtransportmqtt, iothubtransportmqtt_websockets};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProvDeviceResult {
    InvalidArg = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_INVALID_ARG as isize,
    Success = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_SUCCESS as isize,
    Memory = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_MEMORY as isize,
    Parsing = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_PARSING as isize,
    Transport = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_TRANSPORT as isize,
    InvalidState = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_INVALID_STATE as isize,
    DeviceAuth = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_DEV_AUTH_ERROR as isize,
    Timeout = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_TIMEOUT as isize,
    KeyError = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_KEY_ERROR as isize,
    Error = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_ERROR as isize,
    HubNotSpecified = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_HUB_NOT_SPECIFIED as isize,
    Unauthorized = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_UNAUTHORIZED as isize,
    Disabled = azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_DISABLED as isize
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum IotHubClientResult {
    InvalidArg = azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_ARG as isize,
    Error = azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_ERROR as isize,
    InvalidSize = azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_SIZE as isize,
    IndefiniteTime = azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INDEFINITE_TIME as isize
}

/// Error information about what failed
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ProvReturnValue {
    /// One or more parameters were invalid.
    InvalidParam,
    /// Device could not be provisioned as network is not ready.
    NetworkNotReady,
    /// Device could not be provisioned as device authentication is not ready.
    DeviceAuthNotReady,
    /// Provisioning failed
    ProvDeviceError(ProvDeviceResult),
    /// IoT hub device client creation failed
    IotHubClientError(IotHubClientResult),
    /// Device provisioning failed for any other reason.
    GenericError
}

/// IoTHubDeviceClient_LL_CreateWithAzureSphereDeviceAuthProvisioning
/// Provisions the Azure Sphere device using the provisioning service
/// specified by id_scope and creates an IoT hub connection handle.
/// 
/// # Arguments 
/// 
///  * `id_scope` - The Azure IoT Device Provisioning Service scope ID for this device.
///  * `timeout` - Time to wait for provisioning (in milliseconds) before timing out. In the event of a timeout, the result field of the return value will be set ResultTimeout
///
pub fn create_with_device_auth_provisioning(
    id_scope: &str,
    timeout: libc::c_uint
    ) -> Result<iothub_device_client_ll::IotHubDeviceClient, ProvReturnValue>
{
    let id_scope_native = std::ffi::CString::new(id_scope.as_bytes()).unwrap();
    let mut handle:u32 = 0;
    let result = unsafe {
        azure_sphere_provisioning::IoTHubDeviceClient_LL_CreateWithAzureSphereDeviceAuthProvisioning(
            id_scope_native.as_ptr() as *const libc::c_char,
            timeout,
            &mut handle
        )
    };
    match result.result {
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_OK => unsafe {
            Ok(iothub_device_client_ll::IotHubDeviceClient::from_handle(handle))
        },
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_INVALID_PARAM => Err(ProvReturnValue::InvalidParam),
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_NETWORK_NOT_READY => Err(ProvReturnValue::NetworkNotReady),
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_DEVICEAUTH_NOT_READY => Err(ProvReturnValue::DeviceAuthNotReady),
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_PROV_DEVICE_ERROR => {
            let device_err = match result.prov_device_error {
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_INVALID_ARG => ProvDeviceResult::InvalidArg,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_SUCCESS => ProvDeviceResult::Success,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_MEMORY => ProvDeviceResult::Memory,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_PARSING => ProvDeviceResult::Parsing,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_TRANSPORT => ProvDeviceResult::Transport,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_INVALID_STATE => ProvDeviceResult::InvalidState,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_DEV_AUTH_ERROR => ProvDeviceResult::DeviceAuth,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_TIMEOUT => ProvDeviceResult::Timeout,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_KEY_ERROR => ProvDeviceResult::KeyError,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_ERROR => ProvDeviceResult::Error,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_HUB_NOT_SPECIFIED => ProvDeviceResult::HubNotSpecified,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_UNAUTHORIZED => ProvDeviceResult::Unauthorized,
                azure_sphere_provisioning::PROV_DEVICE_RESULT_TAG_PROV_DEVICE_RESULT_DISABLED => ProvDeviceResult::Disabled,
                _ => ProvDeviceResult::Error               
            };
            Err(ProvReturnValue::ProvDeviceError(device_err))
        },
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_IOTHUB_CLIENT_ERROR => {
            let iot_err = match result.iothub_client_error {
                azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_ARG => IotHubClientResult::InvalidArg,
                azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_ERROR => IotHubClientResult::Error,
                azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INVALID_SIZE => IotHubClientResult::InvalidSize,
                azure_sphere_provisioning::IOTHUB_CLIENT_RESULT_TAG_IOTHUB_CLIENT_INDEFINITE_TIME => IotHubClientResult::IndefiniteTime,
                _ => IotHubClientResult::Error
            };
            Err(ProvReturnValue::IotHubClientError(iot_err))
        },
        azure_sphere_provisioning::AZURE_SPHERE_PROV_RESULT_AZURE_SPHERE_PROV_RESULT_GENERIC_ERROR => Err(ProvReturnValue::GenericError),
        _ => Err(ProvReturnValue::GenericError)
    } 
}


/// IoTHubDeviceClient_LL_CreateWithAzureSphereFromDeviceAuth
/// Creates an IoT hub connection handle using Azure Sphere device authentication with
/// the provided IoT Hub URI using the specified protocol.
/// 
/// Returns on success, the IoT Hub connection handle.
/// 
/// # Arguments
/// 
///  * `iothub_uri` - The Azure IoT Hub URI received in the DPS registration process.
///  * `protocol` - Function pointer for protocol implementation
/// 
pub fn create_from_device_auth(iothub_uri: &str, protocol:iothub_device_client_ll::TransportProvider) -> Result<iothub_device_client_ll::IotHubDeviceClient, std::io::Error>
{
    let iothub_uri_native = std::ffi::CString::new(iothub_uri.as_bytes()).unwrap();
    unsafe {
        let protocol_native = match protocol {
            iothub_device_client_ll::TransportProvider::MQTT => iothubtransportmqtt::MQTT_Protocol as *mut libc::c_void as u32,
            iothub_device_client_ll::TransportProvider::MQTTWebSocket => iothubtransportmqtt_websockets::MQTT_WebSocket_Protocol as *mut libc::c_void as u32
        };
        let handle = azure_sphere_provisioning::IoTHubDeviceClient_LL_CreateWithAzureSphereFromDeviceAuth(
            iothub_uri_native.as_ptr() as *const libc::c_char, 
            protocol_native
        );
        if handle == 0 {
            Err(std::io::Error::from_raw_os_error(libc::EINVAL))
        } else {
            Ok(iothub_device_client_ll::IotHubDeviceClient::from_handle(handle))
        }
    }
}