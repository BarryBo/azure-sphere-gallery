/// A wrapper on top of iot_device_client.rs
///
use crate::applibs::iothub_device_client_ll::{
    ClientResult, ClientRetryPolicy, ConnectionStatus, ConnectionStatusReason,
    DeviceTwinUpdateState, IotHubDeviceClientLowLevel, MessageDisposition,
};
use crate::applibs::iothub_message;
use crate::applibs::iothub_message::IotHubMessage;
use azure_sphere_sys::applibs::iothub_client_options;
use std::ffi::CString;

#[derive(Debug)]
pub struct IotHubDeviceClient {
    // bugbug: make private once the set_option_* are sorted out.
    pub client: IotHubDeviceClientLowLevel,
}

impl IotHubDeviceClient {
    pub fn new(client_ll: IotHubDeviceClientLowLevel) -> Result<Self, ClientResult> {
        let result = IotHubDeviceClient { client: client_ll };
        result.client.set_message_callback(|message| {
            crate::debug!(
                "IotHubDeviceClient message callback invoked {:?}\n",
                message
            );
            // bugbug: implement
            MessageDisposition::Accepted
        })?;
        result
            .client
            .set_device_twin_callback(|update_state, vec| {
                crate::debug!(
                    "IotHubDeviceClient device twin callback invoked {:?} {:?}\n",
                    update_state,
                    vec
                );
                // bugbug: implement
            })?;
        //result.client.set_device_method_callback(|method, vec| {
        //    let method = std::ffi::CString::new(Box::new(method));
        //    let event = IotHubEvent::DeviceMethod(method, vec.clone());
        //    result.events.borrow_mut().push(event);
        //
        //          (200, vec![0]) // bugbug: handle the return value
        //})?;
        Ok(result)
    }

    pub fn set_device_method_callback<F>(&self, callback: F) -> Result<(), ClientResult>
    where
        F: FnMut(String, Vec<u8>) -> (i32, Vec<u8>),
    {
        self.client.set_device_method_callback(callback)
    }

    pub fn set_device_twin_callback<F>(
        &self,
        callback: F, // BUGBUG: this should be Option()
    ) -> Result<(), ClientResult>
    where
        F: FnMut(DeviceTwinUpdateState, Vec<u8>),
    {
        self.client.set_device_twin_callback(callback)
    }

    pub fn set_connection_status_callback<F>(&self, callback: F) -> Result<(), ClientResult>
    where
        F: FnMut(ConnectionStatus, ConnectionStatusReason),
    {
        self.client.set_connection_status_callback(callback)
    }

    // bugbug: bring over the set_option_* methods from iot_device_client.rs

    pub fn set_option_deviceid(&self, device_id_for_cert_usage: i32) -> Result<(), ClientResult> {
        let option_name = CString::new("SetDeviceId").unwrap();
        unsafe {
            self.client.set_option_internal(
                option_name.as_ptr(),
                &device_id_for_cert_usage as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_AUTO_URL_ENCODE_DECODE (bool*)
    pub fn set_option_auto_url_encode_decode(&self, value: bool) -> Result<(), ClientResult> {
        unsafe {
            let value = if value { 1u8 } else { 0u8 };
            self.client.set_option_internal(
                iothub_client_options::OPTION_AUTO_URL_ENCODE_DECODE.as_ptr(),
                &value as *const _ as *const libc::c_void,
            )
        }
    }

    /// IotHubDeviceClientLL_SetOption for OPTION_MODEL_ID (const char*)
    pub fn set_option_model_id(&self, value: &str) -> Result<(), ClientResult> {
        unsafe {
            let value = std::ffi::CString::new(value.as_bytes()).unwrap();
            self.client.set_option_internal(
                iothub_client_options::OPTION_MODEL_ID.as_ptr(),
                value.as_ptr() as *const libc::c_void,
            )
        }
    }

    pub fn send_event(&self, event_message: IotHubMessage) -> Result<(), ClientResult> {
        // bugbug: this is ugly.  The send_event_async() should use a callback that
        // includes the IotHubMessage, but it's too difficult to fold that into the
        // current generic-with-bounds style.
        unsafe {
            let _event_message_handle = event_message.get_handle();

            let callback = |_confirmation_result| {
                crate::debug!("IotDeviceClient send_event_async callback\n");
                // bugbug: implement
            };
            self.client.send_event_async(event_message, callback)
        }
    }

    pub fn set_retry_policy(
        &self,
        retry_policy: ClientRetryPolicy,
        retry_timeout_limit_in_seconds: usize,
    ) -> Result<(), ClientResult> {
        self.client
            .set_retry_policy(retry_policy, retry_timeout_limit_in_seconds)
    }

    pub fn get_retry_policy(&self) -> Result<(ClientRetryPolicy, usize), ClientResult> {
        self.client.get_retry_policy()
    }

    pub fn send_reported_state(&self, reported_state: &[u8]) -> Result<(), ClientResult> {
        let callback = |status_code| {
            crate::debug!(
                "IotDeviceClient send_reported_state callback {:?}\n",
                status_code
            );
            // bugbug: implement
        };
        self.client.send_reported_state(reported_state, callback)
    }

    /// Sets up the message callback to be invoked when IoT Hub issues a
    /// message to the device. This is a blocking call.
    ///
    /// See IotHubDeviceClientLL_SetMessageCallback()
    pub fn set_message_callback<F>(&self, callback: F) -> Result<(), ClientResult>
    where
        F: FnMut(iothub_message::IotHubMessage) -> MessageDisposition,
    {
        self.client.set_message_callback(callback)
    }

    pub fn do_work(&self) {
        self.client.do_work();
    }
}
