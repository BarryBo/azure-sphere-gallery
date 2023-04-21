/// A callback-free wrapper on top of iot_device_client.rs
///
/// The do_work() method returns a list of events that were generated.
///
use crate::applibs::iothub_device_client_ll::{
    ClientResult, ClientRetryPolicy, ConfirmationResult, ConnectionStatus, ConnectionStatusReason,
    DeviceTwinUpdateState, IotHubDeviceClientLowLevel,
};
use crate::applibs::iothub_message::IotHubMessage;
use std::cell::RefCell;

#[derive(Debug)]
pub enum IotHubEvent {
    EventConfirmation(/* IotHubMessage, */ ConfirmationResult), // bugbug: what should the payload really be?
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
    pub fn new(client_ll: IotHubDeviceClientLowLevel) -> Self {
        IotHubDeviceClient {
            client: client_ll,
            events: RefCell::new(Vec::<IotHubEvent>::new()),
        }
    }

    // bugbug: bring over the set_option_* methods from iot_device_client.rs

    pub fn send_event(&self, event_message: IotHubMessage) -> Result<(), ClientResult> {
        let callback = |confirmation_result| {
            let event = IotHubEvent::EventConfirmation(confirmation_result);
            self.events.borrow_mut().push(event);
        };
        self.client.send_event_async(&event_message, callback)
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
