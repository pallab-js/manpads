pub mod actuators;
pub mod interlock;
pub mod sensors;
pub mod simulated;

use crate::network::protocol::SystemState;

pub trait HardwareBackend: Send + Sync + 'static {
    fn poll_sensors(&mut self, state: SystemState) -> SensorReading;
    fn set_actuator_state(&mut self, state: SystemState);
    fn trigger_fire(&mut self);
    fn is_estop_active(&self) -> bool;
    fn activate_software_estop(&mut self);
    fn clear_software_estop(&mut self);
}

pub struct SensorReading {
    pub battery_voltage: f64,
    pub temperature: f64,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
}
