use serde::{Deserialize, Serialize};

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
    pub seq: u64,
    pub timestamp_ms: u64,
    pub command_seq: u64,
    pub success: bool,
    pub error_msg: String,
    pub hmac: String,
}

// TD-ONLY: Pre-shared key for the HMAC signature stub validation
pub const HMAC_SECRET: &str = "manpads-td-secret-key";

/// Deterministic, zero-dependency TD-only HMAC signature generator
pub fn compute_hmac_stub(payload_str: &str) -> String {
    let mut sum: u32 = 5381;
    for b in payload_str.bytes() {
        sum = sum.wrapping_mul(33).wrapping_add(b as u32);
    }
    for b in HMAC_SECRET.bytes() {
        sum = sum.wrapping_mul(33).wrapping_add(b as u32);
    }
    format!("{:08x}-td-stub", sum)
}

impl CommandPayload {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn verify_signature(&self) -> bool {
        // Build payload without signature to verify it
        let mut clone = self.clone();
        clone.hmac = String::new();
        if let Ok(serialized) = clone.to_json() {
            let expected = compute_hmac_stub(&serialized);
            return self.hmac == expected;
        }
        false
    }

    pub fn sign(&mut self) {
        self.hmac = String::new();
        if let Ok(serialized) = self.to_json() {
            self.hmac = compute_hmac_stub(&serialized);
        }
    }
}

impl TelemetryFrame {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn sign(&mut self) {
        self.hmac = String::new();
        if let Ok(serialized) = self.to_json() {
            self.hmac = compute_hmac_stub(&serialized);
        }
    }

    pub fn verify_signature(&self) -> bool {
        let mut clone = self.clone();
        clone.hmac = String::new();
        if let Ok(serialized) = clone.to_json() {
            let expected = compute_hmac_stub(&serialized);
            return self.hmac == expected;
        }
        false
    }
}

impl AckFrame {
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn sign(&mut self) {
        self.hmac = String::new();
        if let Ok(serialized) = self.to_json() {
            self.hmac = compute_hmac_stub(&serialized);
        }
    }

    pub fn verify_signature(&self) -> bool {
        let mut clone = self.clone();
        clone.hmac = String::new();
        if let Ok(serialized) = clone.to_json() {
            let expected = compute_hmac_stub(&serialized);
            return self.hmac == expected;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_serialization_and_signing() {
        let mut cmd = CommandPayload {
            seq: 42,
            timestamp_ms: 1717083040000,
            action: CommandAction::Arm,
            auth_token: "DEMO-OPERATOR-TOKEN-2026".to_string(),
            hmac: String::new(),
        };

        // Assert unsign verifies to false
        assert!(!cmd.verify_signature());

        // Sign the frame
        cmd.sign();
        assert!(!cmd.hmac.is_empty());

        // Serialize and deserialize
        let json = cmd.to_json().unwrap();
        let parsed = CommandPayload::from_json(&json).unwrap();

        // Verify signature stub works on parsed struct
        assert!(parsed.verify_signature());
        assert_eq!(parsed.seq, 42);
        assert_eq!(parsed.action, CommandAction::Arm);
    }

    #[test]
    fn test_telemetry_frame_signature() {
        let mut frame = TelemetryFrame {
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

        // Change values to simulate tampering
        frame.battery_voltage = 8.0;
        assert!(!frame.verify_signature());
    }
}

