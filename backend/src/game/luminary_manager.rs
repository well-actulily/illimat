/// Unified Luminary Manager System
/// 
/// This module provides a single entry point for all Luminary rule modifications.
/// It coordinates between different Luminary effect implementations and integrates
/// with the existing game systems.

use crate::game::luminary::*;
use crate::game::luminary_effects::*;
use crate::game::field_id::FieldId;
use crate::game::season::Season;
use crate::game::player::PlayerId;
use crate::game::actions::Action;
use crate::game::state::IllimatState;

/// Manager that coordinates all active Luminary effects
pub struct LuminaryManager;

impl LuminaryManager {
    /// Get the appropriate modifier for a given Luminary
    fn get_modifier_for_luminary(luminary: LuminaryCard) -> Box<dyn LuminaryRuleModifier> {
        match luminary {
            // Core Set
            LuminaryCard::TheForestQueen => Box::new(ForestQueenModifier),
            LuminaryCard::TheMaiden => Box::new(MaidenModifier),
            LuminaryCard::TheChangeling => Box::new(ChangelingModifier),
            LuminaryCard::TheChildren => Box::new(ChildrenModifier),
            LuminaryCard::TheUnion => Box::new(UnionModifier),
            LuminaryCard::TheRiver => Box::new(RiverModifier),
            
            // False Baron's Set
            LuminaryCard::TheAstronomer => Box::new(AstronomerModifier),
            LuminaryCard::TheCollective => Box::new(CollectiveModifier),
            LuminaryCard::TheDrought => Box::new(DroughtModifier),
            LuminaryCard::TheEcho => Box::new(EchoModifier),
            LuminaryCard::TheGambler => Box::new(GamblerModifier),
            LuminaryCard::TheUniverse => Box::new(UniverseModifier),
            
            // Crane Wife Expansion
            LuminaryCard::TheLoom => Box::new(LoomModifier),
            LuminaryCard::TheIsland => Box::new(IslandModifier),
            LuminaryCard::ThePerfectCrime => Box::new(PerfectCrimeModifier),
            LuminaryCard::TheButchers => Box::new(ButchersModifier),
            LuminaryCard::TheSoldiers => Box::new(SoldiersModifier),
            LuminaryCard::TheBoat => Box::new(BoatModifier),
            
            // Other Released
            // TODO: Implement remaining Luminaries (TheAudience, TheRusalka)
            _ => Box::new(DefaultLuminaryModifier),
        }
    }

    /// Collect all active Luminaries and their modifiers
    fn get_active_modifiers(luminary_states: &[LuminaryState; 4]) -> Vec<(LuminaryCard, Box<dyn LuminaryRuleModifier>)> {
        let mut modifiers = Vec::new();
        
        for state in luminary_states {
            if let Some(luminary) = state.card() {
                if state.is_active() {
                    modifiers.push((luminary, Self::get_modifier_for_luminary(luminary)));
                }
            }
        }
        
        modifiers
    }

    /// Compute combined field capabilities considering all active Luminaries
    pub fn get_field_capabilities(
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        illimat_orientation: u8,
    ) -> FieldCapabilities {
        let active_modifiers = Self::get_active_modifiers(luminary_states);
        
        if active_modifiers.is_empty() {
            // No active Luminaries - use base game rules
            return FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: base_season != Season::Winter,
                can_stockpile: base_season != Season::Spring,
                special_rules: vec![],
            };
        }

        // Apply Luminary effects in priority order
        // Island has highest priority (affects all other Luminaries)
        let island_modifier = active_modifiers.iter()
            .find(|(card, _)| *card == LuminaryCard::TheIsland);
        
        if let Some((_, modifier)) = island_modifier {
            // Island active - only it affects capabilities
            return modifier.modify_capabilities(field, base_season, luminary_states, illimat_orientation);
        }

        // Apply other Luminary effects, starting with base capabilities
        let mut capabilities = FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        };

        // Apply Luminary effects in proper priority order
        
        // Step 1: Apply field-specific effects (like Forest Queen)
        for (luminary, modifier) in &active_modifiers {
            if matches!(luminary, LuminaryCard::TheForestQueen) {
                // Forest Queen overrides season restrictions for her field only
                if matches!(luminary_states[field.0 as usize], 
                    LuminaryState::FaceUp(LuminaryCard::TheForestQueen) | 
                    LuminaryState::Claimed(LuminaryCard::TheForestQueen, _)) {
                    let luminary_caps = modifier.modify_capabilities(field, base_season, luminary_states, illimat_orientation);
                    capabilities = luminary_caps;
                }
            }
        }
        
        // Step 2: Apply global effects (like Drought and Maiden) that can override field-specific effects
        for (luminary, modifier) in &active_modifiers {
            match luminary {
                LuminaryCard::TheDrought => {
                    let luminary_caps = modifier.modify_capabilities(field, base_season, luminary_states, illimat_orientation);
                    // Drought blocks Summer harvesting globally, regardless of other effects
                    if !luminary_caps.can_harvest {
                        capabilities.can_harvest = false;
                    }
                    capabilities.special_rules.extend(luminary_caps.special_rules);
                }
                LuminaryCard::TheMaiden => {
                    let luminary_caps = modifier.modify_capabilities(field, base_season, luminary_states, illimat_orientation);
                    // Maiden allows Winter harvesting globally, overriding season restrictions
                    if luminary_caps.can_harvest && base_season == Season::Winter {
                        capabilities.can_harvest = true;
                    }
                    capabilities.special_rules.extend(luminary_caps.special_rules);
                }
                _ => {
                    // Other Luminaries apply standard logic
                    if !matches!(luminary, LuminaryCard::TheForestQueen) {
                        let luminary_caps = modifier.modify_capabilities(field, base_season, luminary_states, illimat_orientation);
                        capabilities.can_sow = capabilities.can_sow && luminary_caps.can_sow;
                        capabilities.can_harvest = capabilities.can_harvest && luminary_caps.can_harvest;  
                        capabilities.can_stockpile = capabilities.can_stockpile && luminary_caps.can_stockpile;
                        capabilities.special_rules.extend(luminary_caps.special_rules);
                    }
                }
            }
        }

        capabilities
    }

    /// Check if an action should be modified by active Luminaries
    pub fn modify_action(
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        game_state: &IllimatState,
    ) -> ActionModification {
        let active_modifiers = Self::get_active_modifiers(luminary_states);
        
        for (_, modifier) in &active_modifiers {
            let modification = modifier.modify_action_resolution(action, luminary_states, game_state);
            
            match modification {
                ActionModification::Blocked(_) => return modification,
                ActionModification::Modified { .. } => return modification,
                ActionModification::Normal => continue,
            }
        }
        
        ActionModification::Normal
    }

    /// Handle field clearing with all active Luminary effects
    pub fn handle_field_cleared(
        field: FieldId,
        player: PlayerId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        let mut combined_result = FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        };

        // Handle the specific Luminary in the cleared field first
        if let Some(luminary) = luminary_states[field.0 as usize].card() {
            let modifier = Self::get_modifier_for_luminary(luminary);
            let result = modifier.handle_field_cleared(field, player, luminary_states, game_state);
            
            combined_result.should_reseed = combined_result.should_reseed || result.should_reseed;
            combined_result.additional_cards.extend(result.additional_cards);
            combined_result.special_effects.extend(result.special_effects);
        }

        // Handle other active Luminaries that might affect field clearing
        let other_luminaries: Vec<(usize, LuminaryCard)> = luminary_states.iter().enumerate()
            .filter_map(|(i, state)| {
                if i != field.0 as usize && state.is_active() {
                    state.card().map(|luminary| (i, luminary))
                } else {
                    None
                }
            })
            .collect();
        
        for (_i, luminary) in other_luminaries {
            let modifier = Self::get_modifier_for_luminary(luminary);
            let result = modifier.handle_field_cleared(field, player, luminary_states, game_state);
            
            combined_result.should_reseed = combined_result.should_reseed || result.should_reseed;
            combined_result.additional_cards.extend(result.additional_cards);
            combined_result.special_effects.extend(result.special_effects);
        }

        combined_result
    }

    /// Handle Luminary revelation
    pub fn handle_luminary_revelation(
        luminary: LuminaryCard,
        field: FieldId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        let modifier = Self::get_modifier_for_luminary(luminary);
        let mut result = modifier.handle_revelation(luminary, field, luminary_states, game_state);
        
        // Apply cards to deal to the game state
        for (target_field, cards) in &result.cards_to_deal {
            game_state.field_cards[target_field.0 as usize].extend(cards.clone());
        }
        
        // Handle revealing other Luminaries (e.g., The Newborn)
        for &other_field in &result.reveal_other_luminaries {
            if let LuminaryState::FaceDown(other_luminary) = luminary_states[other_field.0 as usize] {
                luminary_states[other_field.0 as usize] = LuminaryState::FaceUp(other_luminary);
                
                // Recursively handle the newly revealed Luminary
                let other_result = Self::handle_luminary_revelation(
                    other_luminary, other_field, luminary_states, game_state
                );
                result.special_effects.extend(other_result.special_effects);
            }
        }
        
        result
    }

    /// Handle Luminary claiming
    pub fn handle_luminary_claiming(
        luminary: LuminaryCard,
        field: FieldId,
        player: PlayerId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> ClaimingResult {
        let modifier = Self::get_modifier_for_luminary(luminary);
        modifier.handle_claiming(luminary, field, player, luminary_states, game_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::season::Season;
    use crate::game::field_id::FieldId;
    use crate::game::player::PlayerId;
    use crate::game::card::{Card, Rank, Suit};

    #[test]
    fn test_no_luminaries_active() {
        let luminary_states = [LuminaryState::None; 4];
        
        let caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Spring, &luminary_states, 0
        );
        
        // Should use base season rules
        assert!(caps.can_sow);
        assert!(caps.can_harvest);
        assert!(!caps.can_stockpile); // Spring blocks stockpiling
        assert!(caps.special_rules.is_empty());
    }

    #[test]
    fn test_forest_queen_overrides_season() {
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheForestQueen),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];
        
        let caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Autumn, &luminary_states, 2
        );
        
        // Forest Queen should override Autumn restrictions
        assert!(caps.can_sow, "Forest Queen should allow sowing in Autumn");
        assert!(caps.can_harvest);
        assert!(caps.can_stockpile);
        assert!(!caps.special_rules.is_empty());
    }

    #[test]
    fn test_drought_blocks_summer_harvest() {
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheDrought),
            LuminaryState::None,
            LuminaryState::None,
        ];
        
        let caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Summer, &luminary_states, 1
        );
        
        // Drought should block Summer harvesting
        assert!(caps.can_sow);
        assert!(!caps.can_harvest, "Drought should block Summer harvesting");
        assert!(caps.can_stockpile);
        assert!(!caps.special_rules.is_empty());
    }

    #[test]
    fn test_island_overrides_all_other_effects() {
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheForestQueen), // Would normally allow all
            LuminaryState::FaceUp(LuminaryCard::TheDrought),     // Would normally block harvest
            LuminaryState::FaceUp(LuminaryCard::TheIsland),      // Should override everything
            LuminaryState::None,
        ];
        
        // Island field should allow everything despite other Luminaries
        let island_caps = LuminaryManager::get_field_capabilities(
            FieldId(2), Season::Winter, &luminary_states, 0
        );
        assert!(island_caps.can_sow);
        assert!(island_caps.can_harvest, "Island should allow harvest despite Drought and Winter");
        assert!(island_caps.can_stockpile);
        
        // Non-Island fields should be blocked
        let blocked_caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Summer, &luminary_states, 1
        );
        assert!(!blocked_caps.can_sow, "Island should block other fields");
        assert!(!blocked_caps.can_harvest);
        assert!(!blocked_caps.can_stockpile);
    }

    #[test]
    fn test_multiple_luminary_effects_combine() {
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheForestQueen), // Makes field 0 always Summer
            LuminaryState::FaceUp(LuminaryCard::TheDrought),     // Blocks Summer harvesting globally
            LuminaryState::None,
            LuminaryState::None,
        ];
        
        // Field 0: Forest Queen vs Drought
        let caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Autumn, &luminary_states, 2
        );
        
        // Forest Queen makes field Summer, but Drought blocks Summer harvesting
        assert!(caps.can_sow, "Forest Queen should allow sowing");
        assert!(!caps.can_harvest, "Drought should block harvesting even in Forest Queen field");
        assert!(caps.can_stockpile, "Forest Queen should allow stockpiling");
        
        // Should have special rules from both
        assert!(caps.special_rules.len() >= 1, "Should have special rules from active Luminaries");
    }

    #[test]
    fn test_action_modification() {
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheIsland),
            LuminaryState::None,
        ];
        
        let game_state = IllimatState::new_test_game();
        
        // Action in Island field should be allowed
        let island_action = Action::Sow {
            field: FieldId(2),
            card: Card::new(Rank::Five, Suit::Spring),
        };
        let result = LuminaryManager::modify_action(&island_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal);
        
        // Action in non-Island field should be blocked
        let blocked_action = Action::Sow {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Spring),
        };
        let result = LuminaryManager::modify_action(&blocked_action, &luminary_states, &game_state);
        match result {
            ActionModification::Blocked(_) => {}, // Expected
            _ => panic!("Expected action to be blocked by Island"),
        }
    }

    #[test]
    fn test_changeling_revelation_and_claiming() {
        let mut luminary_states = [LuminaryState::None; 4];
        let mut game_state = IllimatState::new_test_game();
        
        // Test revelation
        let result = LuminaryManager::handle_luminary_revelation(
            LuminaryCard::TheChangeling,
            FieldId(0),
            &mut luminary_states,
            &mut game_state,
        );
        
        // Should have special effects explaining exchange ability
        assert!(!result.special_effects.is_empty(), "Changeling should have revelation effects");
        assert!(result.special_effects[0].contains("exchange"), "Should mention exchange ability");
        
        // Test claiming
        let result = LuminaryManager::handle_luminary_claiming(
            LuminaryCard::TheChangeling,
            FieldId(0),
            PlayerId(1),
            &mut luminary_states,
            &mut game_state,
        );
        
        // Should have immediate effects for claiming
        assert!(!result.immediate_effects.is_empty(), "Changeling should have claiming effects");
        assert!(result.immediate_effects[0].contains("Player 1"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("two"), "Should mention two-card exchange");
    }

    #[test]
    fn test_maiden_allows_winter_harvest() {
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheMaiden),
            LuminaryState::None,
            LuminaryState::None,
        ];
        
        // Winter field should allow harvesting when Maiden is active
        let caps = LuminaryManager::get_field_capabilities(
            FieldId(0), Season::Winter, &luminary_states, 3
        );
        
        assert!(caps.can_sow, "Winter should allow sowing");
        assert!(caps.can_harvest, "Maiden should allow Winter harvesting");
        assert!(caps.can_stockpile, "Winter should allow stockpiling");
        assert!(!caps.special_rules.is_empty(), "Should have Maiden special rule");
    }

    #[test]
    fn test_union_action_modification() {
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheUnion),
            LuminaryState::None,
            LuminaryState::None,
        ];
        
        let game_state = IllimatState::new_test_game();
        
        // Union should modify harvest actions in its field
        let union_harvest = Action::Harvest {
            field: FieldId(1),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring), Card::new(Rank::Two, Suit::Autumn)],
        };
        let result = LuminaryManager::modify_action(&union_harvest, &luminary_states, &game_state);
        match result {
            ActionModification::Modified { .. } => {}, // Expected
            _ => panic!("Expected Union to modify harvest actions in its field"),
        }
        
        // Union should not modify actions outside its field
        let other_harvest = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };
        let result = LuminaryManager::modify_action(&other_harvest, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Union should not affect actions outside its field");
    }

    #[test]
    fn test_river_revelation_effect() {
        let mut luminary_states = [LuminaryState::None; 4];
        let mut game_state = IllimatState::new_test_game();
        
        let initial_field_size = game_state.field_cards[0].len();
        let initial_deck_size = game_state.deck.len();
        
        let result = LuminaryManager::handle_luminary_revelation(
            LuminaryCard::TheRiver,
            FieldId(0),
            &mut luminary_states,
            &mut game_state,
        );
        
        // River should deal cards to its field
        let final_field_size = game_state.field_cards[0].len();
        assert!(final_field_size > initial_field_size, "River should deal cards to field");
        
        // Deck should have fewer cards
        assert!(game_state.deck.len() < initial_deck_size, "Deck should have fewer cards after River revelation");
        
        // Should have special effects
        assert!(!result.special_effects.is_empty(), "River revelation should have special effects");
    }
}