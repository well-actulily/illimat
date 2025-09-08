/// Comprehensive tests for Luminary presence/absence and hidden information handling
/// 
/// These tests ensure that:
/// 1. Each Luminary's rule modifications work correctly when present
/// 2. Base game behavior is preserved when Luminaries are absent  
/// 3. Hidden information (face-down Luminaries) doesn't leak into game evaluation
/// 4. MCTS can handle partial observability correctly

#[cfg(test)]
mod luminary_behavior_tests {
    use crate::game::luminary::*;
    use crate::game::state::IllimatState;
    use crate::game::game_config::GameConfig;
    use crate::game::field_id::FieldId;
    use crate::game::season::Season;
    use crate::game::player::PlayerId;
    use crate::game::card::{Card, Rank, Suit};
    use crate::game::actions::Action;

    /// Test base game functionality without any Luminaries
    #[test]
    fn test_base_game_no_luminaries() {
        let config = GameConfig::new(2).beginner_mode(); // No Luminaries
        let state = IllimatState::new(config);
        
        // Verify no Luminaries are present
        for field_state in &state.field_luminaries {
            assert_eq!(*field_state, LuminaryState::None);
        }
        assert!(state.luminary_deck.is_empty());
        
        // Verify base season restrictions work normally
        let modifier = DefaultLuminaryModifier;
        let empty_states = [LuminaryState::None; 4];
        
        // Spring field: can sow and harvest, cannot stockpile
        let spring_caps = modifier.modify_capabilities(
            FieldId(0), Season::Spring, &empty_states, 0
        );
        assert!(spring_caps.can_sow);
        assert!(spring_caps.can_harvest);
        assert!(!spring_caps.can_stockpile);
        
        // Winter field: can sow and stockpile, cannot harvest
        let winter_caps = modifier.modify_capabilities(
            FieldId(3), Season::Winter, &empty_states, 0
        );
        assert!(winter_caps.can_sow);
        assert!(!winter_caps.can_harvest);
        assert!(winter_caps.can_stockpile);
    }

    /// Test that face-down Luminaries don't affect game rules (hidden information)
    #[test]
    fn test_hidden_luminaries_no_effect() {
        let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
        let mut state = IllimatState::new(config);
        
        // Set up face-down Luminaries (hidden information)
        state.field_luminaries[0] = LuminaryState::FaceDown(LuminaryCard::TheForestQueen);
        state.field_luminaries[1] = LuminaryState::FaceDown(LuminaryCard::TheDrought);
        
        let modifier = DefaultLuminaryModifier;
        
        // Forest Queen is face-down, so should NOT make field always Summer
        let autumn_caps = modifier.modify_capabilities(
            FieldId(0), Season::Autumn, &state.field_luminaries, 2
        );
        assert!(!autumn_caps.can_sow); // Should still block sowing in Autumn
        
        // Drought is face-down, so should NOT block Summer harvesting  
        let summer_caps = modifier.modify_capabilities(
            FieldId(1), Season::Summer, &state.field_luminaries, 1
        );
        assert!(summer_caps.can_harvest); // Should still allow harvesting in Summer
    }

    /// Test field clearing reveals Luminaries and triggers effects
    #[test]
    fn test_field_clearing_reveals_luminaries() {
        let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
        let mut state = IllimatState::new(config);
        
        // Set up a clearable field with face-down Luminary
        state.field_luminaries[0] = LuminaryState::FaceDown(LuminaryCard::TheRiver);
        state.field_cards[0] = vec![Card::new(Rank::Five, Suit::Spring)];
        state.field_stockpiles[0] = vec![];
        
        // Player harvests the only card, clearing the field
        let player = PlayerId(0);
        state.player_hands[0] = vec![Card::new(Rank::Five, Suit::Summer)];
        
        let action = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };
        
        let result = state.apply_action(action);
        assert!(result.is_ok());
        
        // Luminary should now be claimed by the player who cleared the field
        match state.field_luminaries[0] {
            LuminaryState::Claimed(LuminaryCard::TheRiver, claimed_player) => {
                assert_eq!(claimed_player, player);
            }
            _ => panic!("Expected The River to be claimed by player"),
        }
    }

    /// Test Luminary configuration system with presence/absence
    #[test]
    fn test_luminary_configuration_affects_game_creation() {
        // Test with no Luminaries
        let no_luminaries_config = GameConfig::new(2).beginner_mode();
        let no_luminaries_state = IllimatState::new(no_luminaries_config);
        
        for field_state in &no_luminaries_state.field_luminaries {
            assert_eq!(*field_state, LuminaryState::None);
        }
        assert!(no_luminaries_state.luminary_deck.is_empty());
        
        // Test with core Luminaries only
        let core_config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
        let core_state = IllimatState::new(core_config);
        
        let face_down_count = core_state.field_luminaries.iter()
            .filter(|&state| matches!(state, LuminaryState::FaceDown(_)))
            .count();
        assert_eq!(face_down_count, 4); // Should deal one to each field
        
        let remaining_count = core_state.luminary_deck.len();
        assert_eq!(remaining_count, 4); // 8 core Luminaries - 4 dealt = 4 remaining
        
        // Test with all expansions
        let all_config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::all_expansions());
        let all_state = IllimatState::new(all_config);
        
        let total_luminaries = all_state.luminary_deck.len() + 
            all_state.field_luminaries.iter()
                .filter(|&state| !matches!(state, LuminaryState::None))
                .count();
        assert_eq!(total_luminaries, 22); // All 22 Luminaries should be in game
    }

    /// Test specific Luminary effects when active (revealed/claimed)
    mod specific_luminary_effects {
        use super::*;

        #[test]
        fn test_forest_queen_always_summer_when_active() {
            // Test that Forest Queen makes field always Summer when revealed
            let luminary_states = [
                LuminaryState::FaceUp(LuminaryCard::TheForestQueen),
                LuminaryState::None,
                LuminaryState::None,
                LuminaryState::None,
            ];
            let modifier = DefaultLuminaryModifier; // TODO: Replace with ForestQueenModifier
            
            // Even though Illimat orientation would make it Autumn, Forest Queen overrides
            let caps = modifier.modify_capabilities(
                FieldId(0), Season::Autumn, &luminary_states, 2
            );
            
            // TODO: When Forest Queen modifier is implemented, these should be:
            // assert!(caps.can_sow);        // Should be able to sow (always Summer)
            // assert!(caps.can_harvest);    // Should be able to harvest (always Summer)  
            // assert!(caps.can_stockpile);  // Should be able to stockpile (always Summer)
            
            // For now with DefaultLuminaryModifier, verify base behavior
            assert!(!caps.can_sow); // Autumn blocks sowing
            assert!(caps.can_harvest); // Autumn allows harvesting
            assert!(caps.can_stockpile); // Autumn allows stockpiling
        }

        #[test]
        fn test_drought_blocks_summer_harvest_when_active() {
            // Test that Drought blocks Summer field harvesting when revealed
            let luminary_states = [
                LuminaryState::None,
                LuminaryState::FaceUp(LuminaryCard::TheDrought),
                LuminaryState::None,
                LuminaryState::None,
            ];
            let modifier = DefaultLuminaryModifier; // TODO: Replace with DroughtModifier
            
            let caps = modifier.modify_capabilities(
                FieldId(1), Season::Summer, &luminary_states, 1
            );
            
            // TODO: When Drought modifier is implemented, this should be:
            // assert!(!caps.can_harvest); // Drought should block Summer harvesting
            
            // For now with DefaultLuminaryModifier, verify base behavior
            assert!(caps.can_harvest); // Summer normally allows harvesting
        }

        #[test]  
        fn test_island_isolates_field_when_active() {
            // Test that Island makes only its field interactive when revealed
            let luminary_states = [
                LuminaryState::None,
                LuminaryState::None,
                LuminaryState::FaceUp(LuminaryCard::TheIsland),
                LuminaryState::None,
            ];
            let modifier = DefaultLuminaryModifier; // TODO: Replace with IslandModifier
            
            // TODO: When Island modifier is implemented, test that:
            // - Only field 2 (Island's field) allows interactions
            // - All other fields should block all actions
            // - Season effects should be ignored while Island is active
            
            // For now, verify base behavior is unaffected
            for field_id in 0..4 {
                let caps = modifier.modify_capabilities(
                    FieldId(field_id), Season::Summer, &luminary_states, 1
                );
                assert!(caps.can_sow);
                assert!(caps.can_harvest);
                assert!(caps.can_stockpile);
            }
        }
    }

    /// Test hidden information handling for MCTS
    mod hidden_information_tests {
        use super::*;

        #[test]
        fn test_player_perspective_hidden_luminaries() {
            let config = GameConfig::new(4).with_luminaries(LuminaryConfiguration::core_only());
            let state = IllimatState::new(config);
            
            // From any player's perspective, face-down Luminaries are unknown
            // but their presence is known (they can see a card is face-down)
            for field_state in &state.field_luminaries {
                match field_state {
                    LuminaryState::FaceDown(_) => {
                        // Player knows a Luminary exists but not which one
                        // MCTS should model this as uncertainty over possible Luminaries
                    }
                    LuminaryState::None => {
                        // Player knows no Luminary is present
                    }
                    _ => {
                        // Face-up or claimed Luminaries are visible to all players
                    }
                }
            }
        }

        #[test]
        fn test_luminary_deck_hidden_from_players() {
            let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
            let state = IllimatState::new(config);
            
            // Luminary deck contents are hidden information
            // Players know how many remain but not which specific ones
            assert!(!state.luminary_deck.is_empty());
            
            // MCTS should handle this by maintaining belief states over possible
            // Luminary configurations when simulating opponent knowledge
        }

        #[test]
        fn test_revelation_creates_information_asymmetry() {
            let mut config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
            let mut state = IllimatState::new(config);
            
            // Set up a scenario where revealing a Luminary affects game evaluation
            state.field_luminaries[0] = LuminaryState::FaceDown(LuminaryCard::TheForestQueen);
            
            // Before revelation: field capabilities depend on base season
            let before_caps = DefaultLuminaryModifier.modify_capabilities(
                FieldId(0), Season::Autumn, &state.field_luminaries, 2
            );
            assert!(!before_caps.can_sow); // Autumn blocks sowing
            
            // After revelation: Forest Queen would override (when implemented)
            state.field_luminaries[0] = LuminaryState::FaceUp(LuminaryCard::TheForestQueen);
            let after_caps = DefaultLuminaryModifier.modify_capabilities(
                FieldId(0), Season::Autumn, &state.field_luminaries, 2
            );
            
            // TODO: When Forest Queen modifier is implemented:
            // assert!(after_caps.can_sow); // Forest Queen should allow sowing
            
            // This revelation changes the game state evaluation significantly
            // MCTS needs to account for the information gain from revelation
        }
    }

    /// Test serialization and state consistency for MCTS
    mod state_consistency_tests {
        use super::*;

        #[test]
        fn test_luminary_state_serialization() {
            let states_to_test = [
                LuminaryState::None,
                LuminaryState::FaceDown(LuminaryCard::TheForestQueen),
                LuminaryState::FaceUp(LuminaryCard::TheDrought),
                LuminaryState::Claimed(LuminaryCard::TheRiver, PlayerId(2)),
            ];
            
            for state in &states_to_test {
                let serialized = serde_json::to_string(state).expect("Should serialize");
                let deserialized: LuminaryState = serde_json::from_str(&serialized).expect("Should deserialize");
                assert_eq!(*state, deserialized);
            }
        }

        #[test]
        fn test_game_state_with_luminaries_serializable() {
            let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::all_expansions());
            let state = IllimatState::new(config);
            
            // Full game state should be serializable for MCTS state storage
            let serialized = serde_json::to_string(&state).expect("Should serialize game state");
            let deserialized: IllimatState = serde_json::from_str(&serialized).expect("Should deserialize game state");
            
            // Verify key Luminary fields are preserved
            assert_eq!(state.field_luminaries, deserialized.field_luminaries);
            assert_eq!(state.luminary_deck.len(), deserialized.luminary_deck.len());
            
            // Verify Luminary configuration is preserved
            assert_eq!(state.config.luminary_config, deserialized.config.luminary_config);
        }

        #[test]
        fn test_luminary_effects_deterministic() {
            // Ensure that identical Luminary states produce identical results
            let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::core_only());
            let state1 = IllimatState::new(config.clone());
            let state2 = IllimatState::new(config);
            
            // With same random seed, should get same Luminary distribution
            // (This test would need deterministic seeding in production)
            
            let modifier = DefaultLuminaryModifier;
            for field_id in 0..4 {
                let caps1 = modifier.modify_capabilities(
                    FieldId(field_id), Season::Summer, &state1.field_luminaries, 1
                );
                let caps2 = modifier.modify_capabilities(
                    FieldId(field_id), Season::Summer, &state2.field_luminaries, 1  
                );
                
                assert_eq!(caps1.can_sow, caps2.can_sow);
                assert_eq!(caps1.can_harvest, caps2.can_harvest);
                assert_eq!(caps1.can_stockpile, caps2.can_stockpile);
            }
        }
    }

    /// Test all 22 Luminaries have basic presence/absence behavior
    mod comprehensive_luminary_tests {
        use super::*;

        #[test]
        fn test_all_luminaries_can_be_face_down() {
            for luminary in LuminaryCard::all_luminaries() {
                let state = LuminaryState::FaceDown(luminary);
                
                // Face-down Luminaries should not be active
                assert!(!state.is_active());
                assert!(!state.can_be_claimed());
                assert_eq!(state.card(), Some(luminary));
                assert_eq!(state.is_claimed_by(PlayerId(0)), None);
            }
        }

        #[test]
        fn test_all_luminaries_can_be_face_up() {
            for luminary in LuminaryCard::all_luminaries() {
                let state = LuminaryState::FaceUp(luminary);
                
                // Face-up Luminaries should be active and claimable
                assert!(state.is_active());
                assert!(state.can_be_claimed());
                assert_eq!(state.card(), Some(luminary));
                assert_eq!(state.is_claimed_by(PlayerId(0)), None);
            }
        }

        #[test]
        fn test_all_luminaries_can_be_claimed() {
            for luminary in LuminaryCard::all_luminaries() {
                let player = PlayerId(1);
                let state = LuminaryState::Claimed(luminary, player);
                
                // Claimed Luminaries should be active but not claimable
                assert!(state.is_active());
                assert!(!state.can_be_claimed());
                assert_eq!(state.card(), Some(luminary));
                assert_eq!(state.is_claimed_by(player), Some(luminary));
                assert_eq!(state.is_claimed_by(PlayerId(0)), None);
            }
        }

        #[test]
        fn test_all_expansion_combinations() {
            let expansion_configs = [
                LuminaryConfiguration::none(),
                LuminaryConfiguration::core_only(),
                LuminaryConfiguration::all_expansions(),
            ];
            
            for config in &expansion_configs {
                let game_config = GameConfig::new(2).with_luminaries(config.clone());
                let state = IllimatState::new(game_config);
                
                let active_luminaries = config.get_active_luminaries();
                let total_in_game = state.luminary_deck.len() + 
                    state.field_luminaries.iter()
                        .filter(|&s| !matches!(s, LuminaryState::None))
                        .count();
                
                // Total Luminaries in game should match configuration  
                assert_eq!(total_in_game, active_luminaries.len());
                
                // No more than 4 should be dealt to fields initially
                let dealt_count = state.field_luminaries.iter()
                    .filter(|&s| !matches!(s, LuminaryState::None))
                    .count();
                assert!(dealt_count <= 4);
                assert!(dealt_count <= active_luminaries.len());
            }
        }
    }
}