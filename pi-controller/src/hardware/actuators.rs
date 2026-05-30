use crate::network::protocol::SystemState;

pub struct Actuators {
    // Real GPIO relay on Linux target
    #[cfg(target_os = "linux")]
    pin_relay: Option<rppal::gpio::OutputPin>,
}

impl Actuators {
    pub fn new() -> Self {
        #[cfg(target_os = "linux")]
        {
            let gpio = rppal::gpio::Gpio::new().ok();
            let pin_relay = gpio.and_then(|g| g.get(23).ok().map(|p| p.into_output()));
            Self { pin_relay }
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self {}
        }
    }

    pub fn set_state(&mut self, state: SystemState) {
        tracing::info!("ACTUATOR STATE UPDATE: {:?}", state);
        #[cfg(target_os = "linux")]
        {
            if let Some(pin) = &mut self.pin_relay {
                match state {
                    SystemState::Active => pin.set_high(),
                    _ => pin.set_low(),
                }
            }
        }
    }

    pub fn trigger_fire(&mut self) {
        tracing::warn!("🔥 ACTUATOR WARNING: FIRE SIGNAL DETECTED! Relays driven HIGH.");
        #[cfg(target_os = "linux")]
        {
            if let Some(pin) = &mut self.pin_relay {
                pin.set_high();
            }
        }
    }
}
