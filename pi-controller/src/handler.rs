use crate::network::protocol::{CommandAction, SystemState};

/// Pure state transition logic shared between real daemon and simulator.
/// Returns (new_state, error_message) - error_message is empty on success.
pub fn apply_state_transition(state: SystemState, action: CommandAction) -> (SystemState, String) {
    match action {
        CommandAction::Arm => {
            if state == SystemState::Safe {
                (SystemState::Armed, String::new())
            } else {
                (state, "Invalid transition: Must be SAFE to ARM".to_string())
            }
        }
        CommandAction::Disarm => (SystemState::Safe, String::new()),
        CommandAction::Fire => {
            if state == SystemState::Armed {
                (SystemState::Active, String::new())
            } else {
                (state, "CRITICAL: System must be ARMED to Fire".to_string())
            }
        }
        CommandAction::Estop => (SystemState::Emergency, String::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_to_armed() {
        let (new_state, err) = apply_state_transition(SystemState::Safe, CommandAction::Arm);
        assert_eq!(new_state, SystemState::Armed);
        assert!(err.is_empty());
    }

    #[test]
    fn test_armed_to_active() {
        let (new_state, err) = apply_state_transition(SystemState::Armed, CommandAction::Fire);
        assert_eq!(new_state, SystemState::Active);
        assert!(err.is_empty());
    }

    #[test]
    fn test_any_to_safe() {
        for state in [
            SystemState::Armed,
            SystemState::Active,
            SystemState::Emergency,
            SystemState::Off,
        ] {
            let (new_state, err) = apply_state_transition(state, CommandAction::Disarm);
            assert_eq!(new_state, SystemState::Safe);
            assert!(err.is_empty());
        }
    }

    #[test]
    fn test_any_to_emergency() {
        for state in [
            SystemState::Safe,
            SystemState::Armed,
            SystemState::Active,
            SystemState::Off,
        ] {
            let (new_state, err) = apply_state_transition(state, CommandAction::Estop);
            assert_eq!(new_state, SystemState::Emergency);
            assert!(err.is_empty());
        }
    }

    #[test]
    fn test_armed_cannot_arm() {
        let (new_state, err) = apply_state_transition(SystemState::Armed, CommandAction::Arm);
        assert_eq!(new_state, SystemState::Armed);
        assert!(!err.is_empty());
    }

    #[test]
    fn test_active_cannot_fire() {
        let (new_state, err) = apply_state_transition(SystemState::Active, CommandAction::Fire);
        assert_eq!(new_state, SystemState::Active);
        assert!(!err.is_empty());
    }

    #[test]
    fn test_off_cannot_arm() {
        let (new_state, err) = apply_state_transition(SystemState::Off, CommandAction::Arm);
        assert_eq!(new_state, SystemState::Off);
        assert!(!err.is_empty());
    }
}
