use crate::network::protocol::SystemState;

pub struct SensorPoller {
    battery_voltage: f64,
    temperature: f64,
    gps_lat: f64,
    gps_lng: f64,
}

impl Default for SensorPoller {
    fn default() -> Self {
        Self::new()
    }
}

impl SensorPoller {
    pub fn new() -> Self {
        Self {
            battery_voltage: 12.6,
            temperature: 38.5,
            gps_lat: 37.774929,
            gps_lng: -122.419416,
        }
    }

    pub fn poll(&mut self, state: SystemState) -> (f64, f64, f64, f64) {
        // 1. Simulate battery drain depending on system state
        match state {
            SystemState::Active => {
                self.battery_voltage -= 0.05; // Quick drain
            }
            SystemState::Armed => {
                self.battery_voltage -= 0.01; // Slow drain
            }
            SystemState::Safe | SystemState::Off => {
                // Self-recovery slightly or slow decay
                self.battery_voltage -= 0.0005;
            }
            SystemState::Emergency => {}
        }
        if self.battery_voltage < 9.5 {
            self.battery_voltage = 12.6; // Recharge simulator
        }

        // 2. Simulate temperature changes depending on state
        match state {
            SystemState::Active => {
                self.temperature += 0.8; // Quick heat
            }
            SystemState::Armed => {
                self.temperature += 0.15;
            }
            SystemState::Safe | SystemState::Off | SystemState::Emergency => {
                // Cool down to baseline
                if self.temperature > 38.5 {
                    self.temperature -= 0.2;
                } else {
                    self.temperature += rand_noise() * 0.1;
                }
            }
        }
        // Limit max temp
        if self.temperature > 85.0 {
            self.temperature = 85.0;
        }

        // 3. Simulate GPS coordinates with small random walks (tactical hover)
        self.gps_lat += rand_noise() * 0.00002;
        self.gps_lng += rand_noise() * 0.00002;

        (
            self.battery_voltage,
            self.temperature,
            self.gps_lat,
            self.gps_lng,
        )
    }
}

fn rand_noise() -> f64 {
    (rand::random::<f64>() * 2.0) - 1.0
}
