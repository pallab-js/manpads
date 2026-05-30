use crate::hardware::actuators::Actuators;
use crate::hardware::interlock::HardwareInterlock;
use crate::hardware::sensors::SensorPoller;
use crate::hardware::{HardwareBackend, SensorReading};
use crate::network::protocol::SystemState;

pub struct SimulatedHardware {
    pub sensors: SensorPoller,
    pub actuators: Actuators,
    pub interlock: HardwareInterlock,
    pub packet_drop_rate: f64,
    pub latency_range: std::ops::RangeInclusive<u64>,
}

impl Default for SimulatedHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl SimulatedHardware {
    pub fn new() -> Self {
        Self {
            sensors: SensorPoller::new(),
            actuators: Actuators::new(),
            interlock: HardwareInterlock::new(),
            packet_drop_rate: 0.05,
            latency_range: 50..=250,
        }
    }

    pub fn with_packet_drop_rate(mut self, rate: f64) -> Self {
        self.packet_drop_rate = rate;
        self
    }

    pub fn with_latency_range(mut self, range: std::ops::RangeInclusive<u64>) -> Self {
        self.latency_range = range;
        self
    }
}

impl HardwareBackend for SimulatedHardware {
    fn poll_sensors(&mut self, state: SystemState) -> SensorReading {
        let (battery_voltage, temperature, gps_latitude, gps_longitude) = self.sensors.poll(state);
        SensorReading {
            battery_voltage,
            temperature,
            gps_latitude,
            gps_longitude,
        }
    }

    fn set_actuator_state(&mut self, state: SystemState) {
        self.actuators.set_state(state);
    }

    fn trigger_fire(&mut self) {
        self.actuators.trigger_fire();
    }

    fn is_estop_active(&self) -> bool {
        self.interlock.is_estop_active()
    }

    fn activate_software_estop(&mut self) {
        self.interlock.activate_software_estop();
    }

    fn clear_software_estop(&mut self) {
        self.interlock.clear_software_estop();
    }
}

pub struct RealHardware {
    pub actuators: Actuators,
    pub interlock: HardwareInterlock,
}

impl Default for RealHardware {
    fn default() -> Self {
        Self::new()
    }
}

impl RealHardware {
    pub fn new() -> Self {
        Self {
            actuators: Actuators::new(),
            interlock: HardwareInterlock::new(),
        }
    }
}

impl HardwareBackend for RealHardware {
    fn poll_sensors(&mut self, _state: SystemState) -> SensorReading {
        SensorReading {
            battery_voltage: 12.6,
            temperature: 38.5,
            gps_latitude: 37.774929,
            gps_longitude: -122.419416,
        }
    }

    fn set_actuator_state(&mut self, state: SystemState) {
        self.actuators.set_state(state);
    }

    fn trigger_fire(&mut self) {
        self.actuators.trigger_fire();
    }

    fn is_estop_active(&self) -> bool {
        self.interlock.is_estop_active()
    }

    fn activate_software_estop(&mut self) {
        self.interlock.activate_software_estop();
    }

    fn clear_software_estop(&mut self) {
        self.interlock.clear_software_estop();
    }
}
