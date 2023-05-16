/* Copyright (c) Microsoft Corporation. All rights reserved.
Licensed under the MIT License. */

// This sample Rust application demonstrates how to use Azure Sphere devices with Azure IoT
// services, using the Azure IoT C SDK.
//
// It implements a simulated thermometer device, with the following features:
// - Telemetry upload (simulated temperature, device moved events) using Azure IoT Hub events.
// - Reporting device state (serial number) using device twin/read-only properties.
// - Mutable device state (telemetry upload enabled) using device twin/writeable properties.
// - Alert messages invoked from the cloud using device methods.
//
// It can be configured using the top-level Cargo.toml to connect either directly to an
// Azure IoT Hub, to an Azure IoT Edge device, or to use the Azure Device Provisioning service to
// connect to either an Azure IoT Hub, or an Azure IoT Central application. All connection types
// make use of the device certificate issued by the Azure Sphere security service to authenticate,
// and supply an Azure IoT PnP model ID on connection.
//
// It uses the following Azure Sphere modules:
// - eventloop (system invokes handlers for timer events)
// - gpio (digital input for button, digital output for LED)
// - log (displays messages in the Device Output window during debugging)
// - networking (network interface connection status)
//
// You will need to provide information in the application manifest to use this application. Please
// see README.md and the other linked documentation for full details.

// Porting notes from C:
//  Rust doesn't have mutable global variables.  So...
//    exitCode:  replaced by STEP enumeration, of what is about to be attempted, via the get_step and set_step macros.  And std::io::Error(), which wraps errno nicely, and supports custom extensions
//    eventLoop: moved into main() as a local variable
//    callback:  moved into a lambda
#![allow(dead_code)] // bugbug: remove when code complete

use azs::applibs::eventloop::{EventLoop, IoCallback, IoEvents};
use azs::applibs::eventloop_timer_utilities;
use azs::applibs::networking;
use azure_sphere as azs;
use std::cell::RefCell;
use std::env::args;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
pub mod cloud;
use crate::cloud::Cloud;
pub mod azureiot;
pub mod user_interface;
use crate::azureiot::FailureReason;
use crate::user_interface::UserInterface;
use signal_hook_registry;

const STEP_SUCCESS: i32 = 0;
const STEP_SIGNAL_REGISTRATION: i32 = 1;
const STEP_TERMHANDLER_SIGTERM: i32 = 2;
const STEP_IS_NETWORK_READY: i32 = 3;
const STEP_MISSING_HOSTNAME: i32 = 4;
//const STEP_INIT_TELEMETRY_TIMER: i32 = 5;
const STEP_INIT_UI: i32 = 6;
const STEP_EVENTLOOP: i32 = 7;
const STEP_CLOUD_INIT: i32 = 8;
const STEP_FAILURE_CALLBACK: i32 = 9;
const STEP_NETWORK_IS_READY_FAILED: i32 = 10;

/// Currently executing program step
static STEP: AtomicI32 = AtomicI32::new(STEP_SUCCESS);

/// Macro to assign a new value to STEP.
macro_rules! set_step {
    ($i:ident) => {
        STEP.store($i, Ordering::Relaxed);
    };
}
/// Macro to read the current STEP value
macro_rules! get_step {
    () => {
        STEP.load(Ordering::Relaxed)
    };
}

/// Hook SIGTERM so that it modifies the returned AtomicBool if signalled
fn hook_sigterm() -> Result<Arc<AtomicBool>, std::io::Error> {
    set_step!(STEP_SIGNAL_REGISTRATION);
    let term = Arc::new(AtomicBool::new(false));

    let term_clone = Arc::clone(&term);
    let _ = unsafe {
        signal_hook_registry::register(15 /* SIGTERM */, move || {
            set_step!(STEP_TERMHANDLER_SIGTERM);
            term_clone.store(true, Ordering::Relaxed)
        })
    }?;

    Ok(term)
}

struct FailureHandler {
    term: Arc<AtomicBool>,
}

impl crate::azureiot::FailureCallback for FailureHandler {
    fn failure_callback(&mut self, reason: FailureReason) {
        azs::debug!(
            "exit_code_callback_handler in main(): reason={:?}\n",
            reason
        );
        match reason {
            FailureReason::NetworkingIsReadyFailed => {
                set_step!(STEP_NETWORK_IS_READY_FAILED);
            }
        };
        self.term.store(true, Ordering::SeqCst);
    }
}

struct UserInterfaceContainer {
    ui: UserInterface,
    elt: eventloop_timer_utilities::EventLoopTimer,
    // Fields accessed within callback closures must use RefCell<T> so that
    // the closure borrows as immutable.  Otherwise only one closure may
    // borrow mutable, and the others won't compile.
    is_connected: RefCell<bool>,
    telemetry_upload_enabled: RefCell<bool>,
}

impl IoCallback for UserInterfaceContainer {
    fn event(&mut self, _events: IoEvents) {
        self.elt.consume_event().unwrap();

        if self.ui.button_a.is_pressed() {
            let telemetry_upload_enabled = self.telemetry_upload_enabled.borrow_mut();
            let new_telemetry_upload_enabled = !*telemetry_upload_enabled;
            azs::debug!(
                "INFO: Telemetry upload enabled state changed (via button press):{:?}\n",
                new_telemetry_upload_enabled
            );
            self.set_thermometer_telemetry_upload_enabled(new_telemetry_upload_enabled, false);
        }
        if self.ui.button_b.is_pressed() {
            self.device_moved();
        }
    }

    unsafe fn fd(&self) -> i32 {
        self.elt.fd()
    }
}

impl UserInterfaceContainer {
    fn device_moved(&self) {
        azs::debug!("INFO: Device moved.\n");

        let _now = SystemTime::now();

        // bugbug: call Cloud_SendThermometerMovedEvent
    }

    fn set_thermometer_telemetry_upload_enabled(&self, upload_enabled: bool, _from_cloud: bool) {
        *self.telemetry_upload_enabled.borrow_mut() = upload_enabled;
        self.ui.set_status(upload_enabled);
        // bugbug: call Cloud_SendThermometerTelemetryUploadEnabledChangedEvent
    }
}

impl crate::cloud::Callbacks for UserInterfaceContainer {
    fn connection_changed(&mut self, connected: bool) {
        azs::debug!(
            "connection_changed_callback_handler in main(): connected={:?}\n",
            connected
        );
        *self.telemetry_upload_enabled.borrow_mut() = connected;

        if connected {
            // bugbug: call Cloud_SendDeviceDetails(serialNumber)
            azs::debug!("Main connection-changed callback!\n");
        }
    }

    fn telemetry_upload_enabled_changed(&mut self, upload_enabled: bool, from_cloud: bool) {
        azs::debug!(
            "INFO: Thermometer telemetry upload enabled state changed (via cloud): {:?}\n",
            upload_enabled
        );
        self.set_thermometer_telemetry_upload_enabled(upload_enabled, from_cloud);
    }

    fn display_alert(&mut self, alert_message: &str) {
        azs::debug!("ALERT: {:?}\n", alert_message);
    }
}

// A main(), except that it returns a Result<T,E>, making it easy to invoke functions using the '?' operator.
fn actual_main(_hostname: &String) -> Result<(), std::io::Error> {
    let term = hook_sigterm()?;

    azs::debug!("Azure IoT Application starting.\n");

    set_step!(STEP_IS_NETWORK_READY);
    if !networking::is_networking_ready().unwrap_or(false) {
        azs::debug!(
            "WARNING: Network is not ready. Device cannot connect until network is ready.\n"
        );
    };

    //
    // InitPeripheralsAndHandlers() inlined into main() as the created objects must all remain live
    //
    set_step!(STEP_INIT_UI);
    let mut event_loop = EventLoop::new()?;

    let ui = UserInterface::new()?;
    let elt = eventloop_timer_utilities::EventLoopTimer::new()?;
    let button_check_period = Duration::new(0, 1000 * 1000);
    elt.set_period(button_check_period)?;

    let failure_handler = FailureHandler { term: term.clone() };

    let mut ui_container = UserInterfaceContainer {
        ui,
        elt,
        is_connected: RefCell::new(false),
        telemetry_upload_enabled: RefCell::new(false),
    };
    event_loop.register_io(IoEvents::Input, &mut ui_container)?;

    set_step!(STEP_CLOUD_INIT);
    let mut cloud = Cloud::new(failure_handler, ui_container)?;
    azs::debug!("Calling cloud.test()\n");
    cloud.test();
    let reading = cloud::Telemetry { temperature: 28.3 };
    let _ = cloud.send_telemetry(&reading, Some(SystemTime::now()));
    event_loop.register_io(IoEvents::Input, &mut cloud)?;

    //
    // Main loop
    //
    while !term.load(Ordering::Relaxed) {
        let result = event_loop.run(-1, true);
        if let Err(e) = result {
            if e.kind() != std::io::ErrorKind::Interrupted {
                std::process::exit(STEP_EVENTLOOP);
            }
        }
    }

    Ok(())
}

pub fn main() -> ! {
    if let Some(hostname) = args().nth(1) {
        let result = actual_main(&hostname);
        if result.is_err() {
            azs::debug!("Failed at step {:?} with {:?}\n", get_step!(), result.err());
            std::process::exit(get_step!());
        }
    } else {
        set_step!(STEP_MISSING_HOSTNAME);
        azs::debug!("Expected one argument, the Azure IoT Hostname.\n");
    };
    azs::debug!("Application exiting\n");
    std::process::exit(get_step!());
}
