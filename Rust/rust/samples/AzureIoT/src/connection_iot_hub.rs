use crate::connection_iot_hub::iothub_device_client::IotHubDeviceClient;
use azs::applibs::application;
use azs::applibs::azure_sphere_provisioning;
use azs::applibs::iothub_device_client;
use azs::applibs::iothub_device_client_ll::TransportProvider;
use azs::applibs::iothub_security_factory::{IotHubSecurityFactory, SecurityType};
use azs::applibs::networking;
use azure_sphere as azs;

// connection.h public interface:

#[derive(Debug)]
pub enum ConnectionStatus {
    NotStarted,
    Started,
    Complete(iothub_device_client::IotHubDeviceClient),
    Failed,
}

pub type ConnectionStatusCallback = Box<dyn FnMut(ConnectionStatus) + 'static>;

pub struct Connection {
    model_id: String,
    hostname: String,
    connection_status_callback: Option<ConnectionStatusCallback>,
}

impl Connection {
    pub fn new(model_id: String, hub_host_name: String) -> Self {
        Self {
            model_id,
            hostname: hub_host_name,
            connection_status_callback: None,
        }
    }

    pub fn start(&mut self, connection_status_callback: ConnectionStatusCallback) {
        self.connection_status_callback = Some(connection_status_callback);
        self.connection_status_callback.as_mut().unwrap().as_mut()(ConnectionStatus::Started);

        let result = self.setup_azureiot_hub_client_with_daa();
        self.connection_status_callback.as_mut().unwrap().as_mut()(result);
    }

    fn setup_azureiot_hub_client_with_daa(&self) -> ConnectionStatus {
        // If network/DAA are not ready, fail out (which will trigger a retry)
        if !is_ready_to_connect() {
            return ConnectionStatus::Failed;
        }

        // Set up auth type
        let connection = IotHubSecurityFactory::init(SecurityType::X509);
        if let Err(err) = connection {
            azs::debug!("ERROR: iothub_security_init failed with error {:?}.\n", err);
            return ConnectionStatus::Failed;
        }
        // bugbug: hold onto 'connection' for its lifetime

        // Create Azure Iot Hub client handle
        let client_handle_ll = azure_sphere_provisioning::create_from_device_auth(
            self.hostname.as_str(),
            TransportProvider::MQTT,
        );
        if let Err(err) = client_handle_ll {
            azs::debug!(
                "ERROR: IoTHubDeviceClient_LL_CreateFromDeviceAuth failed {:?}.\n",
                err
            );
            return ConnectionStatus::Failed;
        }
        let client_handle_ll = client_handle_ll.unwrap();
        let client_handle = IotHubDeviceClient::new(client_handle_ll);
        if let Err(err) = client_handle {
            azs::debug!("Failed to create IotHubDeviceClient {:?}\n", err);
            return ConnectionStatus::Failed;
        }
        let client_handle = client_handle.unwrap();

        // Use DAA cert when connecting - requires the SetDeviceId option to be set on the
        // IoT Hub client.
        let device_id_for_cert_usage: i32 = 1;
        if let Err(err) = client_handle.set_option_deviceid(device_id_for_cert_usage) {
            azs::debug!(
                "ERROR: Failure setting Azure IoT Hub client option \"SetDeviceId\": {:?}\n",
                err
            );
            return ConnectionStatus::Failed;
        }

        // Sets auto URL encoding on IoT Hub Client
        if let Err(err) = client_handle.set_option_auto_url_encode_decode(true) {
            azs::debug!(
                "ERROR: Failed to set auto Url encode option on IoT Hub Client: {:?}\n",
                err
            );
            return ConnectionStatus::Failed;
        }

        // Sets model ID on IoT Hub Client
        if let Err(err) = client_handle.set_option_model_id(&self.model_id) {
            azs::debug!(
                "ERROR: Failed to set the Model ID on IoT Hub Client: {:?}\n",
                err
            );
            return ConnectionStatus::Failed;
        }

        return ConnectionStatus::Complete(client_handle);
    }
}

fn is_ready_to_connect() -> bool {
    let ready = networking::is_networking_ready();
    match ready {
        Err(err) => {
            azs::debug!("ERROR: Networking_IsNetworkingReady: {:?}\n", err);
            return false;
        }
        Ok(ready) => {
            if !ready {
                azs::debug!("ERROR: IoT Hub connection - networking not ready.\n");
                return false;
            }
        }
    }

    let ready = application::is_device_auth_ready();
    match ready {
        Err(err) => {
            azs::debug!("ERROR: Application_IsDeviceAuthReady: {:?}\n", err);
            return false;
        }
        Ok(ready) => {
            if !ready {
                azs::debug!("ERROR: IoT Hub connection - device auth not ready.\n");
                return false;
            }
        }
    }

    true
}

impl Drop for Connection {
    fn drop(&mut self) {
        // bugbug: Connection_Cleanup() work here
    }
}
