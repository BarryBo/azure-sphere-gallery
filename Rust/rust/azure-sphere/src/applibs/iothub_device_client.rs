/// A callback-free wrapper on top of iot_device_client.rs
///
/// The do_work() method returns a list of events that were generated.
///
use crate::applibs::iothub_device_client_ll::{
    ClientResult, ClientRetryPolicy, ConfirmationResult, ConnectionStatus, ConnectionStatusReason,
    DeviceTwinUpdateState, IotHubDeviceClientLowLevel, MessageDisposition,
};
use crate::applibs::iothub_message::IotHubMessage;
use std::cell::RefCell;

#[derive(Debug)]
pub enum IotHubEvent {
    EventConfirmation(ConfirmationResult, IotHubMessage),
    Message(IotHubMessage),
    ConnectionStatusChanged(ConnectionStatus, ConnectionStatusReason),
    DeviceTwinChanged(DeviceTwinUpdateState, Vec<u8>),
    ReportedStateSent(ConnectionStatusReason),
    DeviceMethod(std::ffi::CString, Vec<u8>),
}

#[derive(Debug)]
pub struct IotHubDeviceClient {
    // bugbug: make private once the set_option_* are sorted out.
    pub client: IotHubDeviceClientLowLevel,

    // RefCell here, so that immutable "self" can be used in callbacks
    events: RefCell<Vec<IotHubEvent>>,
}

impl IotHubDeviceClient {
    pub fn new(client_ll: IotHubDeviceClientLowLevel) -> Result<Self, ClientResult> {
        let result = IotHubDeviceClient {
            client: client_ll,
            events: RefCell::new(Vec::<IotHubEvent>::new()),
        };
        result.client.set_message_callback(|message| {
            let event = IotHubEvent::Message(message);
            result.events.borrow_mut().push(event);
            // bugbug: no good way for the event-based system to
            // know what disposition to use here.
            MessageDisposition::Accepted
        })?;
        Ok(result)
    }

    // bugbug: bring over the set_option_* methods from iot_device_client.rs

    pub fn send_event(&self, event_message: IotHubMessage) -> Result<(), ClientResult> {
        // bugbug: this is ugly.  The send_event_async() should use a callback that
        // includes the IotHubMessage, but it's too difficult to fold that into the
        // current generic-with-bounds style.
        unsafe {
            let event_message_handle = event_message.get_handle();

            let callback = |confirmation_result| {
                let new_event_message = IotHubMessage::from_handle(event_message_handle);
                let event = IotHubEvent::EventConfirmation(confirmation_result, new_event_message);
                self.events.borrow_mut().push(event);
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
            let event = IotHubEvent::ReportedStateSent(status_code);
            self.events.borrow_mut().push(event);
        };
        self.client.send_reported_state(reported_state, callback)
    }

    pub fn do_work(&self) -> Vec<IotHubEvent> {
        self.client.do_work();
        let empty_vec = Vec::<IotHubEvent>::new();
        self.events.replace(empty_vec) // Replace current list with empty, and return current list
    }
}
