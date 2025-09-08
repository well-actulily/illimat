/// Turn Management System for Luminary-Enhanced Gameplay
/// 
/// This module handles the complex turn structure when Luminaries add
/// optional and mandatory actions to players' turns.

use crate::game::actions::Action;
use crate::game::player::PlayerId;
use crate::game::state::IllimatState;
use crate::game::luminary::{LuminaryState, ActionTiming};
use crate::game::luminary_manager::LuminaryManager;

/// Represents the current phase of a player's turn
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TurnPhase {
    /// Pre-action phase (mandatory actions before main action)
    PreAction,
    /// Main action phase (standard Sow/Harvest/Stockpile)
    MainAction,
    /// Post-action phase (optional/mandatory actions after main action)
    PostAction,
    /// Turn complete
    Complete,
}

/// Turn state tracking for complex Luminary interactions
#[derive(Clone, Debug)]
pub struct TurnState {
    pub current_player: PlayerId,
    pub phase: TurnPhase,
    pub main_action_taken: Option<Action>,
    pub luminary_actions_taken: Vec<Action>,
    pub luminary_actions_available: Vec<Action>,
    pub luminary_actions_mandatory: Vec<Action>,
}

pub struct TurnManager;

impl TurnManager {
    /// Start a new turn for a player
    pub fn start_turn(player: PlayerId, game_state: &IllimatState) -> TurnState {
        let luminary_actions_available = Self::get_all_available_luminary_actions(player, game_state);
        let luminary_actions_mandatory = Self::get_all_mandatory_luminary_actions(player, game_state);
        
        let phase = if luminary_actions_mandatory.iter()
            .any(|action| Self::get_action_timing(action, &game_state.field_luminaries) == ActionTiming::BeforeMain) 
        {
            TurnPhase::PreAction
        } else {
            TurnPhase::MainAction
        };

        TurnState {
            current_player: player,
            phase,
            main_action_taken: None,
            luminary_actions_taken: vec![],
            luminary_actions_available,
            luminary_actions_mandatory,
        }
    }

    /// Get all available actions for the current turn phase
    pub fn get_available_actions(turn_state: &TurnState, game_state: &IllimatState) -> Vec<Action> {
        match turn_state.phase {
            TurnPhase::PreAction => {
                // Only mandatory pre-action Luminary actions
                turn_state.luminary_actions_mandatory.iter()
                    .filter(|action| Self::get_action_timing(action, &game_state.field_luminaries) == ActionTiming::BeforeMain)
                    .cloned()
                    .collect()
            }
            TurnPhase::MainAction => {
                let mut actions = Self::get_standard_actions(turn_state.current_player, game_state);
                
                // Add optional pre-action Luminary actions
                let optional_pre_actions: Vec<Action> = turn_state.luminary_actions_available.iter()
                    .filter(|action| matches!(
                        Self::get_action_timing(action, &game_state.field_luminaries),
                        ActionTiming::OptionalBefore | ActionTiming::Normal
                    ))
                    .cloned()
                    .collect();
                
                actions.extend(optional_pre_actions);
                actions
            }
            TurnPhase::PostAction => {
                // Mandatory and optional post-action Luminary actions
                turn_state.luminary_actions_available.iter()
                    .chain(turn_state.luminary_actions_mandatory.iter())
                    .filter(|action| matches!(
                        Self::get_action_timing(action, &game_state.field_luminaries),
                        ActionTiming::AfterMain | ActionTiming::OptionalAfter
                    ))
                    .cloned()
                    .collect()
            }
            TurnPhase::Complete => vec![],
        }
    }

    /// Apply an action and update turn state
    pub fn apply_action(
        turn_state: &mut TurnState, 
        action: Action, 
        game_state: &mut IllimatState
    ) -> Result<(), String> {
        // Apply the action to the game state
        game_state.apply_action(action.clone())?;
        
        // Update turn state
        match &action {
            Action::Sow { .. } | Action::Harvest { .. } | Action::Stockpile { .. } => {
                // Main action taken
                turn_state.main_action_taken = Some(action.clone());
                
                // Check for triggered actions (like Echo)
                let triggered = Self::get_triggered_actions(&action, game_state);
                turn_state.luminary_actions_available.extend(triggered);
                
                // Move to post-action phase if needed
                if Self::has_post_action_requirements(turn_state, game_state) {
                    turn_state.phase = TurnPhase::PostAction;
                } else {
                    turn_state.phase = TurnPhase::Complete;
                }
            }
            _ => {
                // Luminary action taken
                turn_state.luminary_actions_taken.push(action.clone());
                
                // Remove from available actions (if it was one-time use)
                turn_state.luminary_actions_available.retain(|a| a != &action);
                turn_state.luminary_actions_mandatory.retain(|a| a != &action);
                
                // Check if we can advance phases
                if turn_state.phase == TurnPhase::PreAction && 
                   !Self::has_mandatory_pre_actions(turn_state, game_state) {
                    turn_state.phase = TurnPhase::MainAction;
                }
            }
        }

        Ok(())
    }

    /// Check if turn is complete
    pub fn is_turn_complete(turn_state: &TurnState, game_state: &IllimatState) -> bool {
        turn_state.phase == TurnPhase::Complete && 
        turn_state.main_action_taken.is_some() &&
        !Self::has_mandatory_actions_remaining(turn_state, game_state)
    }

    // Helper methods

    fn get_all_available_luminary_actions(player: PlayerId, game_state: &IllimatState) -> Vec<Action> {
        // This would collect all available Luminary actions from all active Luminaries
        // For now, simplified implementation
        vec![]
    }

    fn get_all_mandatory_luminary_actions(player: PlayerId, game_state: &IllimatState) -> Vec<Action> {
        // This would collect all mandatory Luminary actions (like Rake's mandatory sow)
        vec![]
    }

    fn get_standard_actions(player: PlayerId, game_state: &IllimatState) -> Vec<Action> {
        // Generate standard Sow/Harvest/Stockpile actions
        // This would be the existing action generation logic
        vec![]
    }

    fn get_action_timing(action: &Action, luminary_states: &[LuminaryState; 4]) -> ActionTiming {
        // Determine timing requirements for an action
        match action {
            Action::RakeSow { .. } => ActionTiming::BeforeMain, // Rake must sow before or after main action
            Action::ChangelingExchange { .. } => ActionTiming::Normal, // Can be done anytime
            Action::EchoRepeat { .. } => ActionTiming::AfterMain, // Echo happens after main action
            _ => ActionTiming::Normal,
        }
    }

    fn get_triggered_actions(action: &Action, game_state: &IllimatState) -> Vec<Action> {
        // Get actions triggered by the main action (like Echo)
        vec![]
    }

    fn has_post_action_requirements(turn_state: &TurnState, game_state: &IllimatState) -> bool {
        // Check if there are mandatory post-action Luminary effects
        false
    }

    fn has_mandatory_pre_actions(turn_state: &TurnState, game_state: &IllimatState) -> bool {
        turn_state.luminary_actions_mandatory.iter()
            .any(|action| Self::get_action_timing(action, &game_state.field_luminaries) == ActionTiming::BeforeMain)
    }

    fn has_mandatory_actions_remaining(turn_state: &TurnState, game_state: &IllimatState) -> bool {
        !turn_state.luminary_actions_mandatory.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::field_id::FieldId;
    use crate::game::card::{Card, Rank, Suit};

    #[test]
    fn test_basic_turn_flow() {
        let game_state = IllimatState::new_test_game();
        let mut turn_state = TurnManager::start_turn(PlayerId(0), &game_state);
        
        // Should start in main action phase if no mandatory pre-actions
        assert_eq!(turn_state.phase, TurnPhase::MainAction);
        
        // Should be able to get available actions
        let actions = TurnManager::get_available_actions(&turn_state, &game_state);
        // In a real implementation, this would have standard actions available
        
        assert!(!TurnManager::is_turn_complete(&turn_state, &game_state));
    }

    #[test]
    fn test_luminary_action_timing() {
        // Test that different Luminary actions have correct timing
        let rake_action = Action::RakeSow {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
        };
        
        let timing = TurnManager::get_action_timing(&rake_action, &[LuminaryState::None; 4]);
        assert_eq!(timing, ActionTiming::BeforeMain);
    }
}