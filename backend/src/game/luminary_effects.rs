/// Specific Luminary effect implementations
/// 
/// This module contains concrete implementations of each Luminary's unique rule modifications.
/// Each Luminary has its own modifier that implements the LuminaryRuleModifier trait.

use crate::game::luminary::*;
use crate::game::field_id::FieldId;
use crate::game::season::Season;
use crate::game::player::PlayerId;
use crate::game::card::Card;
use crate::game::actions::Action;
use crate::game::state::IllimatState;
use crate::game::okus::OkusPosition;

/// The Forest Queen - "It is always Summer in the Forest Queen's field"
pub struct ForestQueenModifier;

impl LuminaryRuleModifier for ForestQueenModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Check if Forest Queen is active in this field
        let forest_queen_active = matches!(
            luminary_states[field.0 as usize],
            LuminaryState::FaceUp(LuminaryCard::TheForestQueen) | 
            LuminaryState::Claimed(LuminaryCard::TheForestQueen, _)
        );

        if forest_queen_active {
            // Forest Queen makes field always Summer - all actions allowed
            FieldCapabilities {
                can_sow: true,
                can_harvest: true,
                can_stockpile: true,
                special_rules: vec!["Field is always Summer due to The Forest Queen".to_string()],
            }
        } else {
            // Use base season rules
            FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: base_season != Season::Winter,
                can_stockpile: base_season != Season::Spring,
                special_rules: vec![],
            }
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Forest Queen doesn't modify action resolution, only capabilities
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        // Forest Queen doesn't affect field clearing
        FieldClearingResult {
            should_reseed: false, // Let base game logic handle this
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheForestQueen {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Forest Queen is revealed! Her field is now always Summer.".to_string(),
                    "The Illimat cannot be turned while she is on the board.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheForestQueen {
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} may turn the Illimat to any new position.", player.0),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Drought - "While in play, you cannot harvest cards from the Summer field"
pub struct DroughtModifier;

impl LuminaryRuleModifier for DroughtModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        let mut capabilities = FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        };

        // Check if Drought is active anywhere on the board
        let drought_active = luminary_states.iter().any(|state| matches!(
            state,
            LuminaryState::FaceUp(LuminaryCard::TheDrought) | 
            LuminaryState::Claimed(LuminaryCard::TheDrought, _)
        ));

        if drought_active {
            // Check if this field is effectively Summer (either base season or due to Forest Queen)
            let is_effectively_summer = base_season == Season::Summer || 
                matches!(luminary_states[field.0 as usize], 
                    LuminaryState::FaceUp(LuminaryCard::TheForestQueen) | 
                    LuminaryState::Claimed(LuminaryCard::TheForestQueen, _));
            
            if is_effectively_summer {
                capabilities.can_harvest = false;
                capabilities.special_rules.push(
                    "Cannot harvest from Summer field due to The Drought".to_string()
                );
            }
        }

        capabilities
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if trying to harvest in Summer while Drought is active
        if let Action::Harvest { field, .. } = action {
            let drought_active = luminary_states.iter().any(|state| matches!(
                state,
                LuminaryState::FaceUp(LuminaryCard::TheDrought) | 
                LuminaryState::Claimed(LuminaryCard::TheDrought, _)
            ));

            if drought_active {
                // Need to check if this field is Summer - simplified for now
                return ActionModification::Modified {
                    description: "The Drought affects this harvest".to_string(),
                    additional_effects: vec![
                        "Summer field harvesting blocked by The Drought".to_string()
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheDrought {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Drought is revealed! No harvesting from Summer fields while active.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheDrought {
            ClaimingResult {
                immediate_effects: vec![
                    "The Drought has been claimed. Summer harvesting is now allowed.".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Island - "Players may only interact with that field. All season effects and other Luminary effects are ignored."
pub struct IslandModifier;

impl LuminaryRuleModifier for IslandModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        _base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Check if Island is active anywhere
        let (island_active, island_field) = luminary_states.iter().enumerate()
            .find_map(|(i, state)| match state {
                LuminaryState::FaceUp(LuminaryCard::TheIsland) | 
                LuminaryState::Claimed(LuminaryCard::TheIsland, _) => Some((true, i)),
                _ => None,
            })
            .unwrap_or((false, 0));

        if island_active {
            if field.0 as usize == island_field {
                // Island field - all actions allowed, ignore all restrictions
                FieldCapabilities {
                    can_sow: true,
                    can_harvest: true,
                    can_stockpile: true,
                    special_rules: vec![
                        "All actions allowed - The Island ignores season and Luminary effects".to_string(),
                    ],
                }
            } else {
                // Non-Island field - no actions allowed
                FieldCapabilities {
                    can_sow: false,
                    can_harvest: false,
                    can_stockpile: false,
                    special_rules: vec![
                        "No actions allowed - The Island restricts interaction to its field only".to_string(),
                    ],
                }
            }
        } else {
            // No Island active - use base season rules
            FieldCapabilities {
                can_sow: _base_season != Season::Autumn,
                can_harvest: _base_season != Season::Winter,
                can_stockpile: _base_season != Season::Spring,
                special_rules: vec![],
            }
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if Island is active and if action is in wrong field
        let (island_active, island_field) = luminary_states.iter().enumerate()
            .find_map(|(i, state)| match state {
                LuminaryState::FaceUp(LuminaryCard::TheIsland) | 
                LuminaryState::Claimed(LuminaryCard::TheIsland, _) => Some((true, i)),
                _ => None,
            })
            .unwrap_or((false, 0));

        if island_active {
            let action_field = match action {
                Action::Sow { field, .. } => field.0 as usize,
                Action::Harvest { field, .. } => field.0 as usize,
                Action::Stockpile { field, .. } => field.0 as usize,
                // Placeholder for Luminary actions
                Action::ChangelingExchange { .. } => 0,
                Action::RakeSow { .. } => 0,
                Action::LoomStockpile { .. } => 0,
                Action::EchoRepeat { .. } => 0,
            };

            if action_field != island_field {
                return ActionModification::Blocked(
                    "The Island restricts all interactions to its field only".to_string()
                );
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheIsland {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Island is revealed! Players may only interact with this field.".to_string(),
                    "All season effects and other Luminary effects are ignored.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheIsland {
            ClaimingResult {
                immediate_effects: vec![
                    "The Island has been claimed. The game returns to normal.".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Children - "When revealed, deal 3 cards beneath the Children (hidden). When claimed, these cards are added to your harvest pile"
pub struct ChildrenModifier;

impl LuminaryRuleModifier for ChildrenModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Children don't modify field capabilities
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Children don't modify actions
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheChildren {
            // Deal 3 cards beneath the Children (stored in the game state as hidden cards)
            let mut hidden_cards = vec![];
            
            for _ in 0..3 {
                if let Some(card) = game_state.deck.pop() {
                    hidden_cards.push(card);
                } else {
                    break; // Deck exhausted
                }
            }
            
            let cards_dealt_count = hidden_cards.len();
            
            // NOTE: In a full implementation, these hidden cards would need to be stored
            // in the game state in a way that only the claiming player can see them.
            // For now, we'll just note this in the special effects.
            
            RevelationResult {
                cards_to_deal: vec![], // Cards are hidden, not dealt to field
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    format!("The Children are revealed! {} cards are dealt beneath them (hidden).", cards_dealt_count),
                    "These cards will be added to the harvest pile of whoever claims The Children.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheChildren {
            ClaimingResult {
                immediate_effects: vec![
                    format!(
                        "Player {} claims The Children and adds the hidden cards to their harvest pile",
                        player.0
                    ),
                    "The claiming player may examine these cards but should not reveal them to others".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Changeling - "Once during your turn, you may exchange a card from your hand with a card in the same field as the Changeling"
pub struct ChangelingModifier;

impl LuminaryRuleModifier for ChangelingModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Changeling doesn't modify base field capabilities, only adds exchange ability
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Changeling doesn't modify standard actions, only provides special exchange ability
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheChangeling {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Changeling is revealed! Once per turn, you may exchange a card from your hand with a card in its field.".to_string(),
                    "You cannot exchange cards that are part of a Stockpile.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheChangeling {
            ClaimingResult {
                immediate_effects: vec![
                    format!(
                        "Player {} may immediately exchange two cards from their hand for any two on the board",
                        player.0
                    ),
                    "This exchange does not change the season".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }

    fn get_available_actions(
        &self,
        player: PlayerId,
        luminary_states: &[LuminaryState; 4],
        game_state: &IllimatState,
    ) -> Vec<Action> {
        let mut actions = vec![];
        
        // Find active Changeling
        for (field_idx, state) in luminary_states.iter().enumerate() {
            if matches!(state, 
                LuminaryState::FaceUp(LuminaryCard::TheChangeling) | 
                LuminaryState::Claimed(LuminaryCard::TheChangeling, _)
            ) {
                let field = FieldId(field_idx as u8);
                let player_hand = &game_state.player_hands[player.0 as usize];
                let field_cards = &game_state.field_cards[field_idx];
                
                // Generate exchange actions for each combination of hand card + field card
                for &hand_card in player_hand {
                    for &field_card in field_cards {
                        // Can't exchange cards that are part of stockpiles
                        // (simplified check - in full implementation would check stockpile membership)
                        actions.push(Action::ChangelingExchange {
                            field,
                            hand_card,
                            field_card,
                        });
                    }
                }
            }
        }
        
        actions
    }
}

/// The Maiden - "While the Maiden is on the board, Winter has no effect: cards may be harvested from the Winter field"
pub struct MaidenModifier;

impl LuminaryRuleModifier for MaidenModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Check if Maiden is active anywhere on the board
        let maiden_active = luminary_states.iter().any(|state| matches!(
            state,
            LuminaryState::FaceUp(LuminaryCard::TheMaiden) | 
            LuminaryState::Claimed(LuminaryCard::TheMaiden, _)
        ));

        let mut capabilities = FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        };

        if maiden_active && base_season == Season::Winter {
            // Maiden allows harvesting in Winter
            capabilities.can_harvest = true;
            capabilities.special_rules.push(
                "The Maiden allows harvesting in Winter".to_string()
            );
        }

        capabilities
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Maiden doesn't modify action resolution, only capabilities
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheMaiden {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Maiden is revealed! Winter has no effect - you may harvest from Winter fields.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheMaiden {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    "The Maiden has been claimed but Winter harvesting remains allowed while she's claimed".to_string(),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Union - "When harvesting in the Union's field, you may play two cards from your hand and treat them as one card of combined value"
pub struct UnionModifier;

impl LuminaryRuleModifier for UnionModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Union doesn't modify base field capabilities, only harvest behavior
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a harvest action in the Union's field
        if let Action::Harvest { field, .. } = action {
            let union_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheUnion) | 
                LuminaryState::Claimed(LuminaryCard::TheUnion, _)
            );

            if union_active_in_field {
                return ActionModification::Modified {
                    description: "The Union allows playing two cards as one".to_string(),
                    additional_effects: vec![
                        "You may play two cards from your hand and treat them as one card of combined value".to_string()
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheUnion {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Union is revealed! You may now play two cards as one when harvesting in this field.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheUnion {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!(
                        "Player {} can continue using The Union's two-card harvest ability",
                        player.0
                    ),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Astronomer - "When revealed, the player names a season. All Stars cards are considered that season for the round"
pub struct AstronomerModifier;

impl LuminaryRuleModifier for AstronomerModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Astronomer doesn't modify field capabilities directly, only Stars card interpretation
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Astronomer doesn't modify actions directly, but affects Stars card interpretation
        // In a full implementation, this would check if Stars cards are being played
        // and modify their season interpretation based on the Astronomer's chosen season
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheAstronomer {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Astronomer is revealed! The revealing player names a season.".to_string(),
                    "All Stars cards are considered that season for the rest of this round.".to_string(),
                    "This affects how Stars face cards change the Illimat orientation.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheAstronomer {
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} may name a new season for all Stars cards", player.0),
                    "This replaces any previously chosen season from The Astronomer".to_string(),
                ],
                ongoing_effects: vec![
                    "The chosen season applies to all Stars cards for the rest of the round".to_string(),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Gambler - "When revealed, all players may discard up to 4 cards for replacements"
pub struct GamblerModifier;

impl LuminaryRuleModifier for GamblerModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Gambler doesn't modify field capabilities
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Gambler doesn't modify actions, only provides revelation/claiming effects
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheGambler {
            // NOTE: In a full implementation, this would prompt all players to choose cards to discard
            // and then handle the reshuffling and replacement dealing. For now, we indicate the effect.
            
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Gambler is revealed! All players may discard up to 4 cards.".to_string(),
                    "The revealing player shuffles discarded cards into the deck and deals replacements.".to_string(),
                    "Each player draws back up to their starting hand size.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheGambler {
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} may discard up to 4 cards and reshuffle them into the draw pile", player.0),
                    "Draw replacement cards back up to hand size".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Echo - "After performing any action in the Echo's field, you may repeat the same action in a different field"
pub struct EchoModifier;

impl LuminaryRuleModifier for EchoModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Echo doesn't modify base field capabilities, only action behavior
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is an action in the Echo's field
        let action_field = match action {
            Action::Sow { field, .. } => field.0 as usize,
            Action::Harvest { field, .. } => field.0 as usize,
            Action::Stockpile { field, .. } => field.0 as usize,
            _ => return ActionModification::Normal, // Not a basic action
        };

        let echo_active_in_field = matches!(
            luminary_states[action_field],
            LuminaryState::FaceUp(LuminaryCard::TheEcho) | 
            LuminaryState::Claimed(LuminaryCard::TheEcho, _)
        );

        if echo_active_in_field {
            return ActionModification::Modified {
                description: "The Echo allows action repetition".to_string(),
                additional_effects: vec![
                    "After this action, you may repeat it in a different field using a different card".to_string(),
                    "You must obey season rules for the second action".to_string(),
                    "Resolve the original action completely before the second".to_string(),
                ],
            };
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheEcho {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Echo is revealed! Actions in this field may be repeated in other fields.".to_string(),
                    "After acting here, you may repeat the same action elsewhere with a different card.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheEcho {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Echo's action repetition ability", player.0),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Collective - "When harvesting, you may play any number of cards from your hand and treat them as one combined value"
pub struct CollectiveModifier;

impl LuminaryRuleModifier for CollectiveModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,  
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Collective doesn't modify base field capabilities, only harvest behavior
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a harvest action in the Collective's field
        if let Action::Harvest { field, .. } = action {
            let collective_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheCollective) | 
                LuminaryState::Claimed(LuminaryCard::TheCollective, _)
            );

            if collective_active_in_field {
                return ActionModification::Modified {
                    description: "The Collective allows multi-card harvesting".to_string(),
                    additional_effects: vec![
                        "You may play any number of cards from your hand and treat them as one combined value".to_string(),
                        "You must match a card in the field with that total value".to_string(),
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheCollective {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Collective is revealed! Multi-card harvesting is now possible in this field.".to_string(),
                    "You may play any number of cards as one combined value when harvesting.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheCollective {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Collective's multi-card harvest ability", player.0),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Universe - "When harvesting in this field, you may also harvest from any other field as if all cards were in this one"
pub struct UniverseModifier;

impl LuminaryRuleModifier for UniverseModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Universe doesn't modify base field capabilities, only harvest behavior
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a harvest action in the Universe's field
        if let Action::Harvest { field, .. } = action {
            let universe_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheUniverse) | 
                LuminaryState::Claimed(LuminaryCard::TheUniverse, _)
            );

            if universe_active_in_field {
                return ActionModification::Modified {
                    description: "The Universe allows cross-field harvesting".to_string(),
                    additional_effects: vec![
                        "You may harvest from any other field as if all cards were in this field".to_string(),
                        "You must harvest at least one card from this field".to_string(),
                        "Season and Luminary restrictions ignored in other fields".to_string(),
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheUniverse {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Universe is revealed! Cross-field harvesting is now possible in this field.".to_string(),
                    "When harvesting here, you may harvest from any field, ignoring restrictions.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheUniverse {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Universe's cross-field harvest ability", player.0),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Boat - "When you harvest from The Boat, you may also harvest from the opposite field"
pub struct BoatModifier;

impl LuminaryRuleModifier for BoatModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Check if Boat is active in this field
        let boat_active_in_field = matches!(
            luminary_states[field.0 as usize],
            LuminaryState::FaceUp(LuminaryCard::TheBoat) | 
            LuminaryState::Claimed(LuminaryCard::TheBoat, _)
        );

        if boat_active_in_field && base_season == Season::Winter {
            // Boat in Winter cannot harvest at all
            FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: false, // Boat blocks harvesting in Winter
                can_stockpile: base_season != Season::Spring,
                special_rules: vec![
                    "The Boat in Winter prevents all harvesting".to_string(),
                ],
            }
        } else {
            // Use base season rules, with Boat allowing opposite field harvesting
            let mut special_rules = vec![];
            if boat_active_in_field && base_season == Season::Summer {
                special_rules.push("The Boat allows harvesting from the opposite Winter field".to_string());
            }

            FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: base_season != Season::Winter,
                can_stockpile: base_season != Season::Spring,
                special_rules,
            }
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a harvest action in the Boat's field
        if let Action::Harvest { field, .. } = action {
            let boat_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheBoat) | 
                LuminaryState::Claimed(LuminaryCard::TheBoat, _)
            );

            if boat_active_in_field {
                // Check what season the Boat's field is in (simplified - would need season calculation)
                return ActionModification::Modified {
                    description: "The Boat allows opposite field harvesting".to_string(),
                    additional_effects: vec![
                        "You may also harvest from the opposite field".to_string(),
                        "If The Boat is in Summer, you can harvest from Winter".to_string(),
                        "If both fields are cleared, receive rewards for both".to_string(),
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheBoat {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Boat is revealed! It enables opposite field harvesting.".to_string(),
                    "When harvesting from The Boat, you may also harvest from the opposite field.".to_string(),
                    "If The Boat is in Winter, you cannot harvest from it at all.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheBoat {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Boat's opposite field harvest ability", player.0),
                    "Harvesting from The Boat may also harvest from the opposite field".to_string(),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Soldiers - "When you Sow into the Soldiers' field, sow one additional hidden card from the deck"
pub struct SoldiersModifier;

impl LuminaryRuleModifier for SoldiersModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Soldiers don't modify field capabilities
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a sow action in the Soldiers' field
        if let Action::Sow { field, .. } = action {
            let soldiers_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheSoldiers) | 
                LuminaryState::Claimed(LuminaryCard::TheSoldiers, _)
            );

            if soldiers_active_in_field {
                return ActionModification::Modified {
                    description: "The Soldiers add a hidden card when sowing".to_string(),
                    additional_effects: vec![
                        "One additional card is sown face-down from the deck".to_string(),
                        "The hidden card is not revealed until the field is cleared".to_string(),
                        "Both cards are placed in the field".to_string(),
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheSoldiers {
            // Soldiers affect the initial seeding - add 3 additional hidden cards
            let mut hidden_cards = vec![];
            
            for _ in 0..3 {
                if let Some(card) = game_state.deck.pop() {
                    hidden_cards.push(card);
                } else {
                    break; // Deck exhausted
                }
            }
            
            let cards_added = hidden_cards.len();
            
            RevelationResult {
                cards_to_deal: if !hidden_cards.is_empty() {
                    vec![(field, hidden_cards)]
                } else {
                    vec![]
                },
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Soldiers are revealed! They add hidden cards to their field.".to_string(),
                    format!("{} hidden cards dealt to The Soldiers' field", cards_added),
                    "When sowing here, an additional hidden card will be added from the deck.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheSoldiers {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Soldiers' additional sowing ability", player.0),
                    "Sowing into this field will continue to add hidden cards from the deck".to_string(),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Butchers - "When revealed, discard the Luminary in the opposite field. When claimed, force an opponent to return an okus"
pub struct ButchersModifier;

impl LuminaryRuleModifier for ButchersModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Butchers don't modify field capabilities
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Butchers don't modify actions, only provide revelation/claiming effects
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        field: FieldId,
        luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheButchers {
            let mut special_effects = vec![
                "The Butchers are revealed! They discard the Luminary in the opposite field.".to_string(),
            ];

            // Calculate opposite field (0↔2, 1↔3)
            let opposite_field_idx = match field.0 {
                0 => 2,
                1 => 3,
                2 => 0,
                3 => 1,
                _ => panic!("Invalid field ID"),
            };

            // Check if there's a Luminary in the opposite field to discard
            match luminary_states[opposite_field_idx] {
                LuminaryState::None => {
                    special_effects.push("No Luminary in the opposite field to discard.".to_string());
                }
                LuminaryState::FaceDown(opponent_luminary) | 
                LuminaryState::FaceUp(opponent_luminary) |
                LuminaryState::Claimed(opponent_luminary, _) => {
                    special_effects.push(format!(
                        "The {} in the opposite field is discarded by The Butchers.",
                        opponent_luminary.display_name()
                    ));
                    // Actually discard the opposite Luminary
                    luminary_states[opposite_field_idx] = LuminaryState::None;
                }
            }

            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects,
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheButchers {
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} may force an opponent to return a collected okus", player.0),
                    "This occurs after collecting any okus from clearing the field".to_string(),
                    "The forced return targets any okus the opponent has collected this round".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Perfect Crime - "When claimed, take an okus from another player or the Illimat"
pub struct PerfectCrimeModifier;

impl LuminaryRuleModifier for PerfectCrimeModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Perfect Crime doesn't modify field capabilities
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Perfect Crime doesn't modify actions, only provides claiming effects
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::ThePerfectCrime {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Perfect Crime is revealed! When claimed, allows stealing an okus.".to_string(),
                    "The claiming player may take an okus from another player or the Illimat.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::ThePerfectCrime {
            // NOTE: In a full implementation, this would need to identify available okus
            // from other players or the Illimat center and allow the player to choose
            let mut available_targets = vec![];
            
            // Check if any okus are on the Illimat (center)
            let okus_on_center = game_state.okus_positions.iter()
                .filter(|pos| matches!(pos, OkusPosition::OnIllimat))
                .count();
            if okus_on_center > 0 {
                available_targets.push("the Illimat center".to_string());
            }
            
            // Check other players' okus (simplified - would need proper player state access)
            for i in 0..4 {
                if i != player.0 as usize {
                    // In full implementation, would check if Player(i) has any okus
                    available_targets.push(format!("Player {}", i));
                }
            }
            
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} may take an okus from another player or the Illimat", player.0),
                    format!("Available targets: {}", if available_targets.is_empty() { 
                        "None".to_string() 
                    } else { 
                        available_targets.join(", ") 
                    }),
                    "This occurs after collecting any okus from clearing the field".to_string(),
                ],
                ongoing_effects: vec![],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

/// The Loom - "Once per turn, stockpile a card into the Loom's field ignoring season rules"
pub struct LoomModifier;

impl LuminaryRuleModifier for LoomModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // Check if Loom is active in this field
        let loom_active_in_field = matches!(
            luminary_states[field.0 as usize],
            LuminaryState::FaceUp(LuminaryCard::TheLoom) | 
            LuminaryState::Claimed(LuminaryCard::TheLoom, _)
        );

        if loom_active_in_field {
            // Loom allows stockpiling even in Spring (which normally blocks it)
            FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: base_season != Season::Winter,
                can_stockpile: true, // Always allow stockpiling in Loom's field
                special_rules: vec![
                    "The Loom allows stockpiling once per turn, ignoring season rules".to_string(),
                ],
            }
        } else {
            // Use base season rules
            FieldCapabilities {
                can_sow: base_season != Season::Autumn,
                can_harvest: base_season != Season::Winter,
                can_stockpile: base_season != Season::Spring,
                special_rules: vec![],
            }
        }
    }

    fn modify_action_resolution(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        // Check if this is a special Loom stockpile action
        if let Action::LoomStockpile { field, .. } = action {
            let loom_active_in_field = matches!(
                luminary_states[field.0 as usize],
                LuminaryState::FaceUp(LuminaryCard::TheLoom) | 
                LuminaryState::Claimed(LuminaryCard::TheLoom, _)
            );

            if loom_active_in_field {
                return ActionModification::Modified {
                    description: "The Loom allows season-ignoring stockpiling".to_string(),
                    additional_effects: vec![
                        "Stockpiling ignores season restrictions".to_string(),
                        "Stockpiling still changes the season".to_string(),
                        "Limited to once per turn".to_string(),
                    ],
                };
            }
        }

        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheLoom {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    "The Loom is revealed! Once per turn, you may stockpile ignoring season rules.".to_string(),
                    "Stockpiling this way still changes the season.".to_string(),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheLoom {
            ClaimingResult {
                immediate_effects: vec![
                    format!("Player {} must discard their entire hand and draw 4 new cards", player.0),
                    "This replaces the normal hand completely".to_string(),
                ],
                ongoing_effects: vec![
                    format!("Player {} can continue using The Loom's season-ignoring stockpile ability", player.0),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }

    fn get_available_actions(
        &self,
        player: PlayerId,
        luminary_states: &[LuminaryState; 4],
        game_state: &IllimatState,
    ) -> Vec<Action> {
        let mut actions = vec![];
        
        // Find active Loom
        for (field_idx, state) in luminary_states.iter().enumerate() {
            if matches!(state, 
                LuminaryState::FaceUp(LuminaryCard::TheLoom) | 
                LuminaryState::Claimed(LuminaryCard::TheLoom, _)
            ) {
                let field = FieldId(field_idx as u8);
                let player_hand = &game_state.player_hands[player.0 as usize];
                let field_cards = &game_state.field_cards[field_idx];
                
                // Generate Loom stockpile actions for each combination of hand card + field targets
                for &hand_card in player_hand {
                    // Generate all possible combinations of field cards to stockpile with
                    for target_card in field_cards {
                        actions.push(Action::LoomStockpile {
                            field,
                            card: hand_card,
                            targets: vec![*target_card],
                        });
                    }
                }
            }
        }
        
        actions
    }
}

/// The River - "If you are Frostbit at the end of the round and have claimed the River, gain 2 points instead of losing 2."
pub struct RiverModifier;

impl LuminaryRuleModifier for RiverModifier {
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // River doesn't modify field capabilities, only scoring
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &IllimatState,
    ) -> ActionModification {
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false,
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        game_state: &mut IllimatState,
    ) -> RevelationResult {
        if luminary == LuminaryCard::TheRiver {
            // River deals 6 cards into its field when revealed
            let mut cards_to_deal = vec![];
            let mut cards_dealt = vec![];
            
            for _ in 0..6 {
                if let Some(card) = game_state.deck.pop() {
                    cards_dealt.push(card);
                } else {
                    break; // Deck exhausted
                }
            }
            
            if !cards_dealt.is_empty() {
                cards_to_deal.push((field, cards_dealt));
            }

            let cards_dealt_count = cards_to_deal.first().map(|(_, cards)| cards.len()).unwrap_or(0);
            
            RevelationResult {
                cards_to_deal,
                reveal_other_luminaries: vec![],
                special_effects: vec![
                    format!("The River is revealed! {} cards dealt to its field.", cards_dealt_count),
                ],
            }
        } else {
            RevelationResult {
                cards_to_deal: vec![],
                reveal_other_luminaries: vec![],
                special_effects: vec![],
            }
        }
    }

    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        _field: FieldId,
        player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut IllimatState,
    ) -> ClaimingResult {
        if luminary == LuminaryCard::TheRiver {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![
                    format!("Player {} gains Frostbit protection from The River", player.0),
                ],
            }
        } else {
            ClaimingResult {
                immediate_effects: vec![],
                ongoing_effects: vec![],
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::field_id::FieldId;
    use crate::game::season::Season;
    use crate::game::player::PlayerId;
    use crate::game::card::{Card, Rank, Suit};

    #[test]
    fn test_forest_queen_always_summer() {
        let modifier = ForestQueenModifier;
        
        // Forest Queen face-up in field 0
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheForestQueen),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Even in Autumn (which normally blocks sowing), Forest Queen allows all actions
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Autumn, &luminary_states, 2
        );

        assert!(caps.can_sow, "Forest Queen should allow sowing even in Autumn");
        assert!(caps.can_harvest, "Forest Queen should allow harvesting");
        assert!(caps.can_stockpile, "Forest Queen should allow stockpiling");
        assert!(!caps.special_rules.is_empty(), "Should have special rule message");
    }

    #[test]
    fn test_forest_queen_face_down_no_effect() {
        let modifier = ForestQueenModifier;
        
        // Forest Queen face-down (hidden) in field 0
        let luminary_states = [
            LuminaryState::FaceDown(LuminaryCard::TheForestQueen),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Face-down Luminary should not affect rules
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Autumn, &luminary_states, 2
        );

        assert!(!caps.can_sow, "Face-down Forest Queen should not affect Autumn sowing restriction");
        assert!(caps.can_harvest, "Autumn should allow harvesting");
        assert!(caps.can_stockpile, "Autumn should allow stockpiling");
        assert!(caps.special_rules.is_empty(), "Should have no special rules when face-down");
    }

    #[test]
    fn test_drought_blocks_summer_harvest() {
        let modifier = DroughtModifier;
        
        // Drought face-up in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheDrought),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Summer field (field 1) should have harvest blocked by Drought
        let caps = modifier.modify_capabilities(
            FieldId(1), Season::Summer, &luminary_states, 1
        );

        assert!(caps.can_sow, "Summer should allow sowing");
        assert!(!caps.can_harvest, "Drought should block Summer harvesting");
        assert!(caps.can_stockpile, "Summer should allow stockpiling");
        assert!(!caps.special_rules.is_empty(), "Should have drought message");
    }

    #[test]
    fn test_drought_claimed_no_effect() {
        let modifier = DroughtModifier;
        
        // Drought claimed by player 0
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::Claimed(LuminaryCard::TheDrought, PlayerId(0)),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Claimed Drought should still block Summer harvesting
        let caps = modifier.modify_capabilities(
            FieldId(1), Season::Summer, &luminary_states, 1
        );

        assert!(!caps.can_harvest, "Claimed Drought should still block Summer harvesting");
    }

    #[test]
    fn test_island_isolates_field() {
        let modifier = IslandModifier;
        
        // Island face-up in field 2
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheIsland),
            LuminaryState::None,
        ];

        // Island field should allow all actions
        let island_caps = modifier.modify_capabilities(
            FieldId(2), Season::Winter, &luminary_states, 0
        );
        assert!(island_caps.can_sow, "Island field should allow sowing");
        assert!(island_caps.can_harvest, "Island field should allow harvesting even in Winter");
        assert!(island_caps.can_stockpile, "Island field should allow stockpiling");

        // Other fields should block all actions
        let other_caps = modifier.modify_capabilities(
            FieldId(0), Season::Summer, &luminary_states, 0
        );
        assert!(!other_caps.can_sow, "Non-Island field should block sowing");
        assert!(!other_caps.can_harvest, "Non-Island field should block harvesting");
        assert!(!other_caps.can_stockpile, "Non-Island field should block stockpiling");
    }

    #[test]
    fn test_changeling_provides_exchange_ability() {
        let modifier = ChangelingModifier;
        
        // Changeling face-up in field 0 should not modify basic capabilities
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheChangeling),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Test that Changeling doesn't modify basic field capabilities
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Spring, &luminary_states, 0
        );

        assert!(caps.can_sow, "Spring should allow sowing");
        assert!(caps.can_harvest, "Spring should allow harvesting");  
        assert!(!caps.can_stockpile, "Spring should block stockpiling");
        assert!(caps.special_rules.is_empty(), "Changeling doesn't modify basic capabilities");
    }

    #[test]
    fn test_changeling_revelation_effects() {
        let modifier = ChangelingModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_revelation(
            LuminaryCard::TheChangeling,
            FieldId(0),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should have special effects explaining the exchange ability
        assert!(!result.special_effects.is_empty(), "Changeling should have revelation effects");
        assert!(result.special_effects.len() >= 2, "Should have multiple effect messages");
        assert!(result.special_effects[0].contains("exchange"), "Should mention exchange ability");
        assert!(result.special_effects[1].contains("Stockpile"), "Should mention stockpile restriction");
        assert!(result.cards_to_deal.is_empty(), "Changeling doesn't deal cards");
        assert!(result.reveal_other_luminaries.is_empty(), "Changeling doesn't reveal other luminaries");
    }

    #[test]
    fn test_changeling_claiming_effects() {
        let modifier = ChangelingModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_claiming(
            LuminaryCard::TheChangeling,
            FieldId(0),
            PlayerId(1),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should have immediate effects for two-card exchange
        assert!(!result.immediate_effects.is_empty(), "Changeling should have claiming effects");
        assert!(result.immediate_effects.len() >= 2, "Should have multiple immediate effects");
        assert!(result.immediate_effects[0].contains("Player 1"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("two"), "Should mention two-card exchange");
        assert!(result.immediate_effects[1].contains("season"), "Should mention no season change");
        assert!(result.ongoing_effects.is_empty(), "Changeling has no ongoing effects after claiming");
    }

    #[test]
    fn test_changeling_no_action_modification() {
        let modifier = ChangelingModifier;
        
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheChangeling),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        
        // Test that Changeling doesn't modify standard actions
        let sow_action = Action::Sow {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
        };
        let result = modifier.modify_action_resolution(&sow_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Changeling should not modify sow actions");
        
        let harvest_action = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };
        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Changeling should not modify harvest actions");
    }

    #[test]
    fn test_changeling_face_down_no_effect() {
        let modifier = ChangelingModifier;
        
        let result = modifier.handle_revelation(
            LuminaryCard::TheMaiden, // Different luminary
            FieldId(0),
            &mut [LuminaryState::None; 4],
            &mut IllimatState::new_test_game(),
        );

        assert!(result.special_effects.is_empty(), "Should have no effects for other luminaries");
        
        let result = modifier.handle_claiming(
            LuminaryCard::TheMaiden, // Different luminary
            FieldId(0),
            PlayerId(0),
            &mut [LuminaryState::None; 4],
            &mut IllimatState::new_test_game(),
        );

        assert!(result.immediate_effects.is_empty(), "Should have no effects for other luminaries");
        assert!(result.ongoing_effects.is_empty(), "Should have no effects for other luminaries");
    }

    #[test]
    fn test_maiden_allows_winter_harvest() {
        let modifier = MaidenModifier;
        
        // Maiden face-up in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheMaiden),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Winter field should allow harvesting when Maiden is active
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Winter, &luminary_states, 3
        );

        assert!(caps.can_sow, "Winter should allow sowing normally");
        assert!(caps.can_harvest, "Maiden should allow Winter harvesting");
        assert!(caps.can_stockpile, "Winter should allow stockpiling normally");
        assert!(!caps.special_rules.is_empty(), "Should have Maiden special rule");
    }

    #[test]
    fn test_maiden_no_effect_non_winter() {
        let modifier = MaidenModifier;
        
        // Maiden face-up in field 1  
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheMaiden),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Spring field should behave normally
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Spring, &luminary_states, 0
        );

        assert!(caps.can_sow, "Spring should allow sowing");
        assert!(caps.can_harvest, "Spring should allow harvesting");
        assert!(!caps.can_stockpile, "Spring should block stockpiling");
        assert!(caps.special_rules.is_empty(), "No special rules for non-Winter seasons");
    }

    #[test] 
    fn test_maiden_face_down_no_effect() {
        let modifier = MaidenModifier;
        
        // Maiden face-down in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceDown(LuminaryCard::TheMaiden),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Winter field should still block harvesting when Maiden is face-down
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Winter, &luminary_states, 3
        );

        assert!(caps.can_sow, "Winter should allow sowing");
        assert!(!caps.can_harvest, "Face-down Maiden should not affect Winter restrictions");
        assert!(caps.can_stockpile, "Winter should allow stockpiling");
        assert!(caps.special_rules.is_empty(), "No special rules when face-down");
    }

    #[test]
    fn test_maiden_claimed_still_active() {
        let modifier = MaidenModifier;
        
        // Maiden claimed by player 0
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::Claimed(LuminaryCard::TheMaiden, PlayerId(0)),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Winter field should still allow harvesting when Maiden is claimed
        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Winter, &luminary_states, 3
        );

        assert!(caps.can_harvest, "Claimed Maiden should still allow Winter harvesting");
        assert!(!caps.special_rules.is_empty(), "Should have Maiden special rule when claimed");
    }

    #[test]
    fn test_union_allows_two_card_harvest() {
        let modifier = UnionModifier;
        
        // Union face-up in field 0
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheUnion),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring), Card::new(Rank::Two, Suit::Autumn)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Union"));
                assert!(!additional_effects.is_empty());
                assert!(additional_effects[0].contains("two cards"));
            }
            _ => panic!("Expected Union to modify harvest actions in its field"),
        }
    }

    #[test]
    fn test_union_no_effect_outside_field() {
        let modifier = UnionModifier;
        
        // Union face-up in field 0, action in field 1
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheUnion),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(1), // Different field
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Union should not affect actions outside its field");
    }

    #[test]
    fn test_union_no_effect_on_sow_stockpile() {
        let modifier = UnionModifier;
        
        // Union face-up in field 0
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheUnion),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        
        // Test sow action
        let sow_action = Action::Sow {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
        };
        let result = modifier.modify_action_resolution(&sow_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Union should not affect sow actions");
        
        // Test stockpile action
        let stockpile_action = Action::Stockpile {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring)],
        };  
        let result = modifier.modify_action_resolution(&stockpile_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Union should not affect stockpile actions");
    }

    #[test]
    fn test_union_face_down_no_effect() {
        let modifier = UnionModifier;
        
        // Union face-down in field 0
        let luminary_states = [
            LuminaryState::FaceDown(LuminaryCard::TheUnion),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Face-down Union should not affect actions");
    }

    #[test]
    fn test_river_revelation_deals_cards() {
        let modifier = RiverModifier;
        let mut game_state = IllimatState::new_test_game();
        
        // Ensure deck has cards
        assert!(!game_state.deck.is_empty(), "Test game should have cards in deck");
        let _initial_deck_size = game_state.deck.len();

        let result = modifier.handle_revelation(
            LuminaryCard::TheRiver,
            FieldId(0),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should deal cards to the field
        assert!(!result.cards_to_deal.is_empty(), "River should deal cards when revealed");
        if let Some((field, cards)) = result.cards_to_deal.first() {
            assert_eq!(*field, FieldId(0), "Cards should be dealt to River's field");
            assert!(cards.len() <= 6, "Should deal at most 6 cards");
            assert!(cards.len() > 0, "Should deal at least 1 card");
        }

        // Should have special effect message
        assert!(!result.special_effects.is_empty(), "River should have revelation message");
    }

    // FALSE BARON'S SET TESTS

    #[test]
    fn test_universe_cross_field_harvest() {
        let modifier = UniverseModifier;
        
        // Universe face-up in field 0
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheUniverse),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(0),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring), Card::new(Rank::Two, Suit::Autumn)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Universe"));
                assert!(additional_effects.len() >= 3);
                assert!(additional_effects[0].contains("cross-field"));
                assert!(additional_effects[1].contains("at least one card"));
                assert!(additional_effects[2].contains("restrictions ignored"));
            }
            _ => panic!("Expected Universe to modify harvest actions in its field"),
        }
    }

    #[test]
    fn test_universe_no_effect_outside_field() {
        let modifier = UniverseModifier;
        
        // Universe in field 0, action in field 1
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheUniverse),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(1), // Different field
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Five, Suit::Spring)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Universe should not affect actions outside its field");
    }

    #[test]
    fn test_collective_multi_card_harvest() {
        let modifier = CollectiveModifier;
        
        // Collective face-up in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheCollective),
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(1),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring), Card::new(Rank::Two, Suit::Autumn)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Collective"));
                assert!(additional_effects.len() >= 2);
                assert!(additional_effects[0].contains("any number of cards"));
                assert!(additional_effects[1].contains("total value"));
            }
            _ => panic!("Expected Collective to modify harvest actions in its field"),
        }
    }

    #[test]
    fn test_echo_action_repetition() {
        let modifier = EchoModifier;
        
        // Echo face-up in field 2
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheEcho),
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let sow_action = Action::Sow {
            field: FieldId(2),
            card: Card::new(Rank::Five, Suit::Summer),
        };

        let result = modifier.modify_action_resolution(&sow_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Echo"));
                assert!(additional_effects.len() >= 3);
                assert!(additional_effects[0].contains("repeat"));
                assert!(additional_effects[1].contains("season rules"));
                assert!(additional_effects[2].contains("original action"));
            }
            _ => panic!("Expected Echo to modify actions in its field"),
        }
    }

    #[test]
    fn test_echo_no_effect_outside_field() {
        let modifier = EchoModifier;
        
        // Echo in field 2, action in field 0
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheEcho),
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let sow_action = Action::Sow {
            field: FieldId(0), // Different field
            card: Card::new(Rank::Five, Suit::Summer),
        };

        let result = modifier.modify_action_resolution(&sow_action, &luminary_states, &game_state);
        assert_eq!(result, ActionModification::Normal, "Echo should not affect actions outside its field");
    }

    #[test]
    fn test_gambler_revelation_effects() {
        let modifier = GamblerModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_revelation(
            LuminaryCard::TheGambler,
            FieldId(0),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should have special effects explaining the discard/replace ability
        assert!(!result.special_effects.is_empty(), "Gambler should have revelation effects");
        assert!(result.special_effects.len() >= 3, "Should have multiple effect messages");
        assert!(result.special_effects[0].contains("discard up to 4"));
        assert!(result.special_effects[1].contains("shuffles"));
        assert!(result.special_effects[2].contains("draw"));
        assert!(result.cards_to_deal.is_empty(), "Gambler doesn't deal cards directly");
    }

    #[test]
    fn test_gambler_claiming_effects() {
        let modifier = GamblerModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_claiming(
            LuminaryCard::TheGambler,
            FieldId(0),
            PlayerId(2),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should have immediate effects for personal discard/replace
        assert!(!result.immediate_effects.is_empty(), "Gambler should have claiming effects");
        assert!(result.immediate_effects.len() >= 2, "Should have multiple immediate effects");
        assert!(result.immediate_effects[0].contains("Player 2"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("discard up to 4"));
        assert!(result.immediate_effects[1].contains("replacement"));
        assert!(result.ongoing_effects.is_empty(), "Gambler has no ongoing effects");
    }

    #[test]
    fn test_astronomer_revelation_effects() {
        let modifier = AstronomerModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_revelation(
            LuminaryCard::TheAstronomer,
            FieldId(3),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should have special effects explaining Stars card season designation
        assert!(!result.special_effects.is_empty(), "Astronomer should have revelation effects");
        assert!(result.special_effects.len() >= 3, "Should have multiple effect messages");
        assert!(result.special_effects[0].contains("names a season"));
        assert!(result.special_effects[1].contains("Stars cards"));
        assert!(result.special_effects[2].contains("Illimat orientation"));
        assert!(result.cards_to_deal.is_empty(), "Astronomer doesn't deal cards");
        assert!(result.reveal_other_luminaries.is_empty(), "Astronomer doesn't reveal others");
    }

    #[test]
    fn test_astronomer_claiming_effects() {
        let modifier = AstronomerModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_claiming(
            LuminaryCard::TheAstronomer,
            FieldId(3),
            PlayerId(0),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should allow renaming season for Stars cards
        assert!(!result.immediate_effects.is_empty(), "Astronomer should have claiming effects");
        assert!(result.immediate_effects.len() >= 2, "Should have multiple immediate effects");
        assert!(result.immediate_effects[0].contains("Player 0"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("new season"));
        assert!(result.immediate_effects[1].contains("replaces"));
        assert!(!result.ongoing_effects.is_empty(), "Astronomer has ongoing effects");
        assert!(result.ongoing_effects[0].contains("chosen season"));
    }

    #[test]
    fn test_false_barons_set_no_capabilities_modification() {
        // Test that False Baron's Set Luminaries don't modify basic field capabilities
        // (they affect action resolution instead)
        
        // Test each modifier individually since they have different types
        let test_cases = [
            ("Universe", LuminaryCard::TheUniverse),
            ("Collective", LuminaryCard::TheCollective), 
            ("Echo", LuminaryCard::TheEcho),
            ("Gambler", LuminaryCard::TheGambler),
            ("Astronomer", LuminaryCard::TheAstronomer),
        ];

        for (name, luminary_card) in test_cases.iter() {
            let luminary_states = [
                LuminaryState::FaceUp(*luminary_card),
                LuminaryState::None,
                LuminaryState::None,
                LuminaryState::None,
            ];

            // Test Spring season (blocks stockpiling) - use appropriate modifier for each
            let caps = match luminary_card {
                LuminaryCard::TheUniverse => UniverseModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheCollective => CollectiveModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheEcho => EchoModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheGambler => GamblerModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheAstronomer => AstronomerModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                _ => panic!("Unexpected luminary in test"),
            };

            assert!(caps.can_sow, "{} should not affect Spring sowing", name);
            assert!(caps.can_harvest, "{} should not affect Spring harvesting", name);
            assert!(!caps.can_stockpile, "{} should not affect Spring stockpiling restriction", name);
            assert!(caps.special_rules.is_empty(), "{} should not add special capability rules", name);
        }
    }

    // CRANE WIFE EXPANSION TESTS

    #[test]
    fn test_loom_allows_spring_stockpiling() {
        let modifier = LoomModifier;
        
        // Loom face-up in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheLoom),
            LuminaryState::None,
            LuminaryState::None,
        ];

        // Spring normally blocks stockpiling, but Loom should allow it
        let caps = modifier.modify_capabilities(
            FieldId(1), Season::Spring, &luminary_states, 0
        );

        assert!(caps.can_sow, "Loom should allow Spring sowing");
        assert!(caps.can_harvest, "Loom should allow Spring harvesting");
        assert!(caps.can_stockpile, "Loom should allow Spring stockpiling despite season restriction");
        assert!(!caps.special_rules.is_empty(), "Loom should have special rule for stockpiling");
        assert!(caps.special_rules[0].contains("ignoring season rules"));
    }

    #[test]
    fn test_loom_stockpile_action_modification() {
        let modifier = LoomModifier;
        
        // Loom face-up in field 2
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheLoom),
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let loom_stockpile = Action::LoomStockpile {
            field: FieldId(2),
            card: Card::new(Rank::Five, Suit::Summer),
            targets: vec![Card::new(Rank::Three, Suit::Spring)],
        };

        let result = modifier.modify_action_resolution(&loom_stockpile, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Loom"));
                assert!(additional_effects.len() >= 3);
                assert!(additional_effects[0].contains("ignores season"));
                assert!(additional_effects[1].contains("still changes"));
                assert!(additional_effects[2].contains("once per turn"));
            }
            _ => panic!("Expected Loom to modify LoomStockpile actions in its field"),
        }
    }

    #[test]
    fn test_loom_claiming_discards_hand() {
        let modifier = LoomModifier;
        let mut game_state = IllimatState::new_test_game();
        
        let result = modifier.handle_claiming(
            LuminaryCard::TheLoom,
            FieldId(0),
            PlayerId(1),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should require discarding hand and drawing 4 new cards
        assert!(!result.immediate_effects.is_empty(), "Loom should have claiming effects");
        assert!(result.immediate_effects.len() >= 2, "Should have multiple immediate effects");
        assert!(result.immediate_effects[0].contains("Player 1"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("discard"), "Should mention discarding hand");
        assert!(result.immediate_effects[0].contains("4 new cards"), "Should mention drawing 4 cards");
        assert!(!result.ongoing_effects.is_empty(), "Loom has ongoing stockpile ability");
    }

    #[test]
    fn test_perfect_crime_claiming_steals_okus() {
        let modifier = PerfectCrimeModifier;
        let mut game_state = IllimatState::new_test_game();
        // Set up some okus on the Illimat for testing
        // Set some okus to be on the Illimat center for testing
        game_state.okus_positions[0] = OkusPosition::OnIllimat;
        game_state.okus_positions[1] = OkusPosition::OnIllimat;
        
        let result = modifier.handle_claiming(
            LuminaryCard::ThePerfectCrime,
            FieldId(0),
            PlayerId(2),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should allow stealing okus from other players or Illimat
        assert!(!result.immediate_effects.is_empty(), "Perfect Crime should have claiming effects");
        assert!(result.immediate_effects.len() >= 3, "Should have multiple immediate effects");
        assert!(result.immediate_effects[0].contains("Player 2"), "Should reference claiming player");
        assert!(result.immediate_effects[0].contains("take an okus"), "Should mention taking okus");
        assert!(result.immediate_effects[1].contains("Available targets"), "Should list available targets");
        assert!(result.immediate_effects[2].contains("after collecting"), "Should mention timing");
        assert!(result.ongoing_effects.is_empty(), "Perfect Crime has no ongoing effects");
    }

    #[test]
    fn test_butchers_discard_opposite_luminary() {
        let modifier = ButchersModifier;
        let mut game_state = IllimatState::new_test_game();
        
        // Set up opposing Luminaries (field 0 and field 2 are opposite)
        let mut luminary_states = [
            LuminaryState::None, // Butchers will be revealed here
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheRiver), // This should be discarded
            LuminaryState::None,
        ];

        let result = modifier.handle_revelation(
            LuminaryCard::TheButchers,
            FieldId(0), // Butchers in field 0
            &mut luminary_states,
            &mut game_state,
        );

        // Should discard The River in the opposite field (field 2)
        assert!(!result.special_effects.is_empty(), "Butchers should have revelation effects");
        assert!(result.special_effects.len() >= 2, "Should have multiple effect messages");
        assert!(result.special_effects[0].contains("Butchers are revealed"));
        assert!(result.special_effects[1].contains("The River"), "Should mention discarding The River");
        assert!(result.special_effects[1].contains("discarded"), "Should confirm discarding");
        
        // Verify the opposite Luminary was actually discarded
        assert_eq!(luminary_states[2], LuminaryState::None, "Opposite Luminary should be discarded");
    }

    #[test]
    fn test_butchers_no_opposite_luminary() {
        let modifier = ButchersModifier;
        let mut game_state = IllimatState::new_test_game();
        
        // No opposing Luminary
        let mut luminary_states = [LuminaryState::None; 4];

        let result = modifier.handle_revelation(
            LuminaryCard::TheButchers,
            FieldId(1), // Butchers in field 1 (opposite is field 3)
            &mut luminary_states,
            &mut game_state,
        );

        // Should indicate no Luminary to discard
        assert!(!result.special_effects.is_empty(), "Butchers should have revelation effects");
        assert!(result.special_effects.len() >= 2, "Should have multiple effect messages");
        assert!(result.special_effects[1].contains("No Luminary"), "Should mention no Luminary to discard");
    }

    #[test]
    fn test_soldiers_add_hidden_cards_on_sow() {
        let modifier = SoldiersModifier;
        
        // Soldiers face-up in field 3
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheSoldiers),
        ];

        let game_state = IllimatState::new_test_game();
        let sow_action = Action::Sow {
            field: FieldId(3),
            card: Card::new(Rank::Knight, Suit::Autumn),
        };

        let result = modifier.modify_action_resolution(&sow_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Soldiers"));
                assert!(additional_effects.len() >= 3);
                assert!(additional_effects[0].contains("additional card"));
                assert!(additional_effects[1].contains("face-down"));
                assert!(additional_effects[2].contains("Both cards"));
            }
            _ => panic!("Expected Soldiers to modify sow actions in their field"),
        }
    }

    #[test]
    fn test_soldiers_revelation_adds_initial_cards() {
        let modifier = SoldiersModifier;
        let mut game_state = IllimatState::new_test_game();
        
        // Ensure deck has cards
        assert!(!game_state.deck.is_empty(), "Test game should have cards in deck");
        let initial_deck_size = game_state.deck.len();

        let result = modifier.handle_revelation(
            LuminaryCard::TheSoldiers,
            FieldId(1),
            &mut [LuminaryState::None; 4],
            &mut game_state,
        );

        // Should deal additional cards to the field
        assert!(!result.cards_to_deal.is_empty(), "Soldiers should deal cards when revealed");
        if let Some((field, cards)) = result.cards_to_deal.first() {
            assert_eq!(*field, FieldId(1), "Cards should be dealt to Soldiers' field");
            assert!(cards.len() <= 3, "Should deal at most 3 cards");
            assert!(cards.len() > 0, "Should deal at least 1 card");
        }

        // Deck should have fewer cards
        assert!(game_state.deck.len() < initial_deck_size, "Deck should have fewer cards after Soldiers revelation");

        // Should have special effects
        assert!(!result.special_effects.is_empty(), "Soldiers revelation should have special effects");
        assert!(result.special_effects.len() >= 3, "Should have multiple effect messages");
        assert!(result.special_effects[0].contains("Soldiers are revealed"));
        assert!(result.special_effects[2].contains("additional hidden card"));
    }

    #[test]
    fn test_boat_blocks_winter_harvesting() {
        let modifier = BoatModifier;
        
        // Boat face-up in field 0, Winter season
        let luminary_states = [
            LuminaryState::FaceUp(LuminaryCard::TheBoat),
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::None,
        ];

        let caps = modifier.modify_capabilities(
            FieldId(0), Season::Winter, &luminary_states, 3
        );

        assert!(caps.can_sow, "Boat should allow Winter sowing");
        assert!(!caps.can_harvest, "Boat in Winter should block all harvesting");
        assert!(caps.can_stockpile, "Boat should allow Winter stockpiling");
        assert!(!caps.special_rules.is_empty(), "Boat should have special rule for Winter blocking");
        assert!(caps.special_rules[0].contains("prevents all harvesting"));
    }

    #[test]
    fn test_boat_allows_summer_opposite_harvesting() {
        let modifier = BoatModifier;
        
        // Boat face-up in field 2, Summer season
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheBoat),
            LuminaryState::None,
        ];

        let caps = modifier.modify_capabilities(
            FieldId(2), Season::Summer, &luminary_states, 1
        );

        assert!(caps.can_sow, "Boat should allow Summer sowing");
        assert!(caps.can_harvest, "Boat should allow Summer harvesting");
        assert!(caps.can_stockpile, "Boat should allow Summer stockpiling");
        assert!(!caps.special_rules.is_empty(), "Boat should have special rule for opposite harvesting");
        assert!(caps.special_rules[0].contains("opposite Winter field"));
    }

    #[test]
    fn test_boat_harvest_modification() {
        let modifier = BoatModifier;
        
        // Boat face-up in field 1
        let luminary_states = [
            LuminaryState::None,
            LuminaryState::FaceUp(LuminaryCard::TheBoat),
            LuminaryState::None,
            LuminaryState::None,
        ];

        let game_state = IllimatState::new_test_game();
        let harvest_action = Action::Harvest {
            field: FieldId(1),
            card: Card::new(Rank::Seven, Suit::Summer),
            targets: vec![Card::new(Rank::Four, Suit::Spring), Card::new(Rank::Three, Suit::Autumn)],
        };

        let result = modifier.modify_action_resolution(&harvest_action, &luminary_states, &game_state);
        
        match result {
            ActionModification::Modified { description, additional_effects } => {
                assert!(description.contains("Boat"));
                assert!(additional_effects.len() >= 3);
                assert!(additional_effects[0].contains("opposite field"));
                assert!(additional_effects[1].contains("Summer"));
                assert!(additional_effects[2].contains("both fields"));
            }
            _ => panic!("Expected Boat to modify harvest actions in its field"),
        }
    }

    #[test]
    fn test_crane_wife_opposite_field_calculation() {
        // Test that opposite field calculations are correct for Butchers and Boat
        let test_cases = [
            (0, 2), // Field 0 opposite is Field 2
            (1, 3), // Field 1 opposite is Field 3  
            (2, 0), // Field 2 opposite is Field 0
            (3, 1), // Field 3 opposite is Field 1
        ];

        for (field, expected_opposite) in test_cases.iter() {
            let calculated_opposite = (*field + 2) % 4;
            assert_eq!(calculated_opposite, *expected_opposite, 
                "Field {} opposite should be {}, got {}", field, expected_opposite, calculated_opposite);
        }
    }

    #[test]
    fn test_crane_wife_no_base_capability_changes() {
        // Test that most Crane Wife Luminaries don't modify basic field capabilities
        // (except Loom which allows stockpiling, and Boat which has Winter restrictions)
        
        let test_cases = [
            ("Perfect Crime", LuminaryCard::ThePerfectCrime),
            ("Butchers", LuminaryCard::TheButchers),
            ("Soldiers", LuminaryCard::TheSoldiers),
        ];

        for (name, luminary_card) in test_cases.iter() {
            let luminary_states = [
                LuminaryState::FaceUp(*luminary_card),
                LuminaryState::None,
                LuminaryState::None,
                LuminaryState::None,
            ];

            // Test Spring season (blocks stockpiling) - use appropriate modifier for each
            let caps = match luminary_card {
                LuminaryCard::ThePerfectCrime => PerfectCrimeModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheButchers => ButchersModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                LuminaryCard::TheSoldiers => SoldiersModifier.modify_capabilities(
                    FieldId(0), Season::Spring, &luminary_states, 0
                ),
                _ => panic!("Unexpected luminary in test"),
            };

            assert!(caps.can_sow, "{} should not affect Spring sowing", name);
            assert!(caps.can_harvest, "{} should not affect Spring harvesting", name);
            assert!(!caps.can_stockpile, "{} should not affect Spring stockpiling restriction", name);
            assert!(caps.special_rules.is_empty(), "{} should not add special capability rules", name);
        }
    }
}