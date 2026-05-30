use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

pub const PROTOCOL_VERSION: u8 = 1;

static HMAC_KEY: OnceLock<Vec<u8>> = OnceLock::new();

pub fn init_hmac_key(secret: &str) {
    HMAC_KEY.set(secret.as_bytes().to_vec()).ok();
}

pub fn compute_hmac(payload_str: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    let key = HMAC_KEY.get().expect("HMAC key must be initialized before use");
    let mut mac = Hmac::<Sha256>::new_from_slice(key)
        .expect("HMAC-SHA256 accepts any key length");
    mac.update(payload_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub mod fault_flags {
    pub const OK: u32 = 0;
    pub const WATCHDOG_TIMEOUT: u32 = 1 << 0;
    pub const GPIO_INTERLOCK_ERR: u32 = 1 << 1;
    pub const THERMAL_CRITICAL: u32 = 1 << 2;
    pub const BATTERY_LOW: u32 = 1 << 3;
    pub const GPS_STALE: u32 = 1 << 4;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CommandAction {
    Arm,
    Disarm,
    Fire,
    Estop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandPayload {
    pub protocol_version: u8,
    pub seq: u64,
    pub timestamp_ms: u64,
    pub action: CommandAction,
    pub auth_token: String,
    pub hmac: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SystemState {
    Off,
    Safe,
    Armed,
    Active,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetryFrame {
    pub protocol_version: u8,
    pub seq: u64,
    pub timestamp_ms: u64,
    pub system_state: SystemState,
    pub battery_voltage: f64,
    pub temperature: f64,
    pub gps_latitude: f64,
    pub gps_longitude: f64,
    pub fault_mask: u32,
    pub hmac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AckFrame {
    pub protocol_version: u8,
    pub seq: u64,
    pub timestamp_ms: u64,
    pub command_seq: u64,
    pub success: bool,
    pub error_msg: String,
    pub hmac: String,
}

pub trait SignedMessage: Clone + serde::Serialize {
    fn hmac_ref(&self) -> &str;
    fn set_hmac(&mut self, hmac: String);

    fn sign(&mut self) {
        let hmac = {
            let mut clone = self.clone();
            clone.set_hmac(String::new());
            serde_json::to_string(&clone).map(|s| compute_hmac(&s))
        };
        if let Ok(h) = hmac {
            self.set_hmac(h);
        }
    }

    fn verify_signature(&self) -> bool {
        let mut clone = self.clone();
        clone.set_hmac(String::new());
        serde_json::to_string(&clone)
            .ok()
            .map(|s| self.hmac_ref() == compute_hmac(&s))
            .unwrap_or(false)
    }
}

impl SignedMessage for CommandPayload {
    fn hmac_ref(&self) -> &str {
        &self.hmac
    }
    fn set_hmac(&mut self, hmac: String) {
        self.hmac = hmac;
    }
}

impl SignedMessage for TelemetryFrame {
    fn hmac_ref(&self) -> &str {
        &self.hmac
    }
    fn set_hmac(&mut self, hmac: String) {
        self.hmac = hmac;
    }
}

impl SignedMessage for AckFrame {
    fn hmac_ref(&self) -> &str {
        &self.hmac
    }
    fn set_hmac(&mut self, hmac: String) {
        self.hmac = hmac;
    }
}

macro_rules! impl_json {
    ($t:ty) => {
        impl $t {
            pub fn to_json(&self) -> Result<String, serde_json::Error> {
                serde_json::to_string(self)
            }
            pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
                serde_json::from_str(json)
            }
        }
    };
}

impl_json!(CommandPayload);
impl_json!(TelemetryFrame);
impl_json!(AckFrame);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization_and_signing() {
        let key = "test-hmac-key-32-bytes-long-okay!";
        init_hmac_key(key);
        let mut cmd = CommandPayload {
            protocol_version: PROTOCOL_VERSION,
            seq: 42,
            timestamp_ms: 1717083040000,
            action: CommandAction::Arm,
            auth_token: "test-token".to_string(),
            hmac: String::new(),
        };
        assert!(!cmd.verify_signature());
        cmd.sign();
        assert!(!cmd.hmac.is_empty());
        let json = cmd.to_json().unwrap();
        let parsed = CommandPayload::from_json(&json).unwrap();
        assert!(parsed.verify_signature());
        assert_eq!(parsed.seq, 42);
        assert_eq!(parsed.action, CommandAction::Arm);
    }

    #[test]
    fn test_telemetry_frame_signature() {
        let mut frame = TelemetryFrame {
            protocol_version: PROTOCOL_VERSION,
            seq: 100,
            timestamp_ms: 1717083040100,
            system_state: SystemState::Armed,
            battery_voltage: 12.2,
            temperature: 44.5,
            gps_latitude: 37.774929,
            gps_longitude: -122.419416,
            fault_mask: 0,
            hmac: String::new(),
        };
        assert!(!frame.verify_signature());
        frame.sign();
        assert!(frame.verify_signature());
        frame.battery_voltage = 8.0;
        assert!(!frame.verify_signature());
    }

    #[test]
    fn test_fault_flags_are_powers_of_two() {
        let flags = [
            fault_flags::OK,
            fault_flags::WATCHDOG_TIMEOUT,
            fault_flags::GPIO_INTERLOCK_ERR,
            fault_flags::THERMAL_CRITICAL,
            fault_flags::BATTERY_LOW,
            fault_flags::GPS_STALE,
        ];
        for flag in &flags {
            if *flag == 0 { continue; }
            assert!((flag & (flag - 1)) == 0, "{:b} is not a power of two", flag);
        }
    }
}
