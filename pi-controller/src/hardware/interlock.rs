pub struct HardwareInterlock {
    #[cfg(target_os = "linux")]
    pin_estop: Option<rppal::gpio::InputPin>,
    software_estop: bool,
}

impl Default for HardwareInterlock {
    fn default() -> Self {
        Self::new()
    }
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

    pub fn is_estop_active(&self) -> bool {
        if self.software_estop {
            return true;
        }
        #[cfg(target_os = "linux")]
        {
            if let Some(pin) = &self.pin_estop {
                return pin.is_low();
            }
        }
        false
    }

    pub fn activate_software_estop(&mut self) {
        tracing::warn!("SOFTWARE E-STOP ACTIVATED");
        self.software_estop = true;
    }

    pub fn clear_software_estop(&mut self) {
        tracing::info!("Software E-STOP cleared");
        self.software_estop = false;
    }
}
