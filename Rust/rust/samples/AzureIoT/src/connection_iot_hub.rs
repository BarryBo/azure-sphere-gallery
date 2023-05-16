use azs::applibs::iothub_device_client;
use azure_sphere as azs;

// connection.h public interface:

#[derive(Debug)]
pub enum ConnectionStatus {
    NotStarted,
    Started,
    Complete(iothub_device_client::IotHubDeviceClient),
    Failed,
}

// typedef void (*Connection_StatusCallbackType)(Connection_Status status, IOTHUB_DEVICE_CLIENT_LL_HANDLE iothubDeviceClientHandle);

pub trait ConnectionCallback {
    fn connection_status_callback(status: ConnectionStatus) -> u32 /* ExitCode */ {
        drop(status);
        0
    }
}

pub type ConnectionStatusCallback = Box<dyn FnMut(ConnectionStatus) + 'static>;

pub struct Connection {
    model_id: String,
    connection_status_callback: Option<ConnectionStatusCallback>,
}

impl Connection {
    pub fn new(model_id: String) -> Self {
        Self {
            model_id,
            connection_status_callback: None,
        }
    }

    pub fn start(&mut self, connection_status_callback: ConnectionStatusCallback) {
        self.connection_status_callback = Some(connection_status_callback);
        self.connection_status_callback.as_mut().unwrap().as_mut()(ConnectionStatus::Started);
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        // bugbug: Connection_Cleanup() work here
    }
}
