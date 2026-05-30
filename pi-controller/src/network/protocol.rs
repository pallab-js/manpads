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

/// Returns the HMAC secret key, from MANPADS_HMAC_SECRET env var or default.
pub fn hmac_secret() -> String {
    std::env::var("MANPADS_HMAC_SECRET").unwrap_or_else(|_| "manpads-td-secret-key".to_string())
}

/// Operator authentication token loaded from MANPADS_OPERATOR_TOKEN env var
/// or falls back to the TD demo default.
pub fn operator_token() -> String {
    std::env::var("MANPADS_OPERATOR_TOKEN")
        .unwrap_or_else(|_| "DEMO-OPERATOR-TOKEN-2026".to_string())
}

/// HMAC-SHA256 based message authentication
pub fn compute_hmac(payload_str: &str) -> String {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;

    let mut mac = Hmac::<Sha256>::new_from_slice(hmac_secret().as_bytes())
        .expect("HMAC-SHA256 accepts any key length");
    mac.update(payload_str.as_bytes());
    hex::encode(mac.finalize().into_bytes())
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

        // Verify HMAC signature works on parsed struct
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
