use azs::applibs::gpio;
use azs::applibs::gpio::{InputPin, OutputPin, Value};
use azure_sphere as azs;

pub struct Button {
    input_pin: InputPin,
    old_value: Value,
}

impl Button {
    fn new(input_pin: InputPin) -> Result<Self, std::io::Error> {
        Ok(Self {
            input_pin,
            old_value: Value::High,
        })
    }

    pub fn is_pressed(&mut self) -> bool {
        let result = self.input_pin.value();
        if let Ok(new_state) = result {
            let is_button_pressed = {
                let is_pressed = new_state != self.old_value && new_state == gpio::Value::Low;
                self.old_value = new_state;
                is_pressed
            };
            is_button_pressed
        } else {
            azs::debug!("ERROR: Could not read button GPIO {:?}\n", result.err());
            false
        }
    }
}

pub struct UserInterface {
    pub button_a: Button,
    pub button_b: Button,
    pub status_led: OutputPin,
}

impl UserInterface {
    pub fn new() -> Result<Self, std::io::Error> {
        let status_led = OutputPin::new(
            hardware::sample_appliance::SAMPLE_LED,
            gpio::OutputMode::PushPull,
            gpio::Value::High,
        )?;

        let button_a = InputPin::new(hardware::sample_appliance::SAMPLE_BUTTON_1)?;
        let button_a = Button::new(button_a)?;

        let button_b = InputPin::new(hardware::sample_appliance::SAMPLE_BUTTON_2)?;
        let button_b = Button::new(button_b)?;

        Ok(Self {
            button_a,
            button_b,
            status_led,
        })
    }

    pub fn set_status(&self, status: bool) {
        let value = if status {
            gpio::Value::High
        } else {
            gpio::Value::Low
        };
        let _ = self.status_led.set_value(value);
    }
}
