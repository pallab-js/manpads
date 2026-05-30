pub struct HardwareInterlock {
    #[cfg(target_os = "linux")]
    pin_estop: Option<rppal::gpio::InputPin>,
    pub software_estop: bool,
}

impl HardwareInterlock {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            let gpio = rppal::gpio::Gpio::new().ok();
            let pin_estop = gpio.and_then(|g| g.get(26).ok().map(|p| p.into_input_pullup()));
            Self {
                pin_estop,
                software_estop: false,
            }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self {
                software_estop: false,
            }
        }
    }

    /// Evaluates if an Emergency Stop condition is met (either software flag or hardware GPIO Pin 26 grounded).
    pub fn is_estop_active(&self) -> bool {
        if self.software_estop {
            return true;
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(pin) = &self.pin_estop {
                // Grounded pin indicates physical switch is closed (pressed)
                return pin.is_low();
            }
        }
        false
    }
}
