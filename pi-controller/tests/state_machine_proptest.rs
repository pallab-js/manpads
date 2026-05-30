use pi_controller::handler::apply_state_transition;
use pi_controller::network::protocol::{CommandAction, SystemState};
use proptest::prelude::*;

fn any_state() -> impl Strategy<Value = SystemState> {
    prop_oneof![
        Just(SystemState::Off),
        Just(SystemState::Safe),
        Just(SystemState::Armed),
        Just(SystemState::Active),
        Just(SystemState::Emergency),
    ]
}

proptest! {
    #[test]
    fn estop_always_transitions_to_emergency(state in any_state()) {
        let (new_state, err) = apply_state_transition(state, CommandAction::Estop);
        prop_assert_eq!(new_state, SystemState::Emergency);
        prop_assert!(err.is_empty());
    }

    #[test]
    fn disarm_always_transitions_to_safe(state in any_state()) {
        let (new_state, err) = apply_state_transition(state, CommandAction::Disarm);
        prop_assert_eq!(new_state, SystemState::Safe);
        prop_assert!(err.is_empty());
    }

    #[test]
    fn fire_only_succeeds_from_armed(state in any_state()) {
        let (new_state, err) = apply_state_transition(state, CommandAction::Fire);
        if state == SystemState::Armed {
            prop_assert_eq!(new_state, SystemState::Active);
            prop_assert!(err.is_empty());
        } else {
            prop_assert_eq!(new_state, state);
            prop_assert!(!err.is_empty());
        }
    }

    #[test]
    fn arm_only_succeeds_from_safe(state in any_state()) {
        let (new_state, err) = apply_state_transition(state, CommandAction::Arm);
        if state == SystemState::Safe {
            prop_assert_eq!(new_state, SystemState::Armed);
            prop_assert!(err.is_empty());
        } else {
            prop_assert_eq!(new_state, state);
            prop_assert!(!err.is_empty());
        }
    }
}
