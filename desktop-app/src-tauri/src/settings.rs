use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub pi_ip: String,
    pub pi_port: u16,
    pub local_port: u16,
    pub operator_token: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            pi_ip: "127.0.0.1".to_string(),
            pi_port: 8080,
            local_port: 8081,
            operator_token: "DEMO-OPERATOR-TOKEN-2026".to_string(),
        }
    }
}

impl Settings {
    pub fn pi_addr(&self) -> String {
        format!("{}:{}", self.pi_ip, self.pi_port)
    }

    pub fn local_addr(&self) -> String {
        format!("127.0.0.1:{}", self.local_port)
    }

    pub fn load(app_data_dir: &Path) -> Self {
        let path = app_data_dir.join("settings.json");
        match std::fs::read_to_string(&path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
                tracing::warn!("Failed to parse settings.json, using defaults: {}", e);
                Default::default()
            }),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self, app_data_dir: &Path) -> Result<(), String> {
        let path = app_data_dir.join("settings.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_default_settings() {
        let s = Settings::default();
        assert_eq!(s.pi_ip, "127.0.0.1");
        assert_eq!(s.pi_port, 8080);
        assert_eq!(s.local_port, 8081);
        assert_eq!(s.operator_token, "DEMO-OPERATOR-TOKEN-2026");
    }

    #[test]
    fn test_pi_addr_format() {
        let s = Settings::default();
        assert_eq!(s.pi_addr(), "127.0.0.1:8080");
    }

    #[test]
    fn test_local_addr_format() {
        let s = Settings::default();
        assert_eq!(s.local_addr(), "127.0.0.1:8081");
    }

    #[test]
    fn test_save_load_cycle() {
        let dir = std::env::temp_dir().join("manpads-test-settings");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let s = Settings {
            pi_ip: "10.0.0.1".to_string(),
            pi_port: 9090,
            local_port: 9091,
            operator_token: "test-token".to_string(),
        };
        s.save(&dir).unwrap();

        let loaded = Settings::load(&dir);
        assert_eq!(loaded.pi_ip, "10.0.0.1");
        assert_eq!(loaded.pi_port, 9090);
        assert_eq!(loaded.local_port, 9091);
        assert_eq!(loaded.operator_token, "test-token");

        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn test_load_missing_file_returns_default() {
        let dir = std::env::temp_dir().join("manpads-test-settings-missing");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let loaded = Settings::load(&dir);
        assert_eq!(loaded.pi_ip, "127.0.0.1");
        assert_eq!(loaded.pi_port, 8080);

        fs::remove_dir_all(&dir).unwrap();
    }
}
