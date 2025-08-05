use crate::game::card::{Card, Rank, Suit};
use crate::game::field_id::FieldId;
use crate::game::player::PlayerId;
use crate::game::season::Season;
use crate::game::stockpile::{Stockpile, StockpileManager};
use crate::game::capabilities::CapabilityManager;

/// Enhanced error types for better user experience
#[derive(Debug, Clone)]
pub enum ActionError {
    SeasonRestriction { action: &'static str, season: Season, suggestion: String },
    CardNotInHand { player: PlayerId, card: Card, available_cards: Vec<Card> },
    InvalidCombination { reason: String, valid_options: Vec<String> },
    NoValidTargets { action: &'static str, field_name: String, help: String },
    InvalidField { field: FieldId, reason: String },
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::SeasonRestriction { action, season, suggestion } => {
                write!(f, "Cannot {} during {} season. {}", action, season, suggestion)
            },
            ActionError::CardNotInHand { player, card, available_cards } => {
                write!(f, "Player {} doesn't have {}. Available cards: {}", 
                    player.0, card, 
                    available_cards.iter()
                        .map(|c| format!("{}", c))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            },
            ActionError::InvalidCombination { reason, valid_options } => {
                if valid_options.is_empty() {
                    write!(f, "{}", reason)
                } else {
                    write!(f, "{}. Try: {}", reason, valid_options.join(" or "))
                }
            },
            ActionError::NoValidTargets { action, field_name, help } => {
                write!(f, "No valid targets to {} in {}. {}", action, field_name, help)
            },
            ActionError::InvalidField { field, reason } => {
                write!(f, "Field {} is invalid: {}", field.0, reason)
            },
        }
    }
}

// Enable conversion between ActionError and String for compatibility
impl From<ActionError> for String {
    fn from(error: ActionError) -> Self {
        error.to_string()
    }
}

impl From<String> for ActionError {
    fn from(error: String) -> Self {
        ActionError::InvalidCombination { 
            reason: error, 
            valid_options: vec![] 
        }
    }
}

/// Core game actions
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Action {
    Sow { field: FieldId, card: Card },
    Harvest { field: FieldId, card: Card, targets: Vec<Card> },
    Stockpile { field: FieldId, card: Card, targets: Vec<Card> },
}

/// Action application utilities
pub struct ActionManager;

impl ActionManager {
    /// Apply a sowing action to the game state
    pub fn apply_sow(
        field_cards: &mut [Vec<Card>; 4],
        player_hands: &mut [Vec<Card>; 4],
        field_seasons: &mut [Season; 4],
        illimat_orientation: &mut u8,
        turn_number: &mut u16,
        player: PlayerId,
        field: FieldId,
        card: Card,
    ) -> Result<(), ActionError> {
        // Check if sowing is allowed in this field
        if !CapabilityManager::can_sow(field, *illimat_orientation) {
            let current_season = field_seasons[field.0 as usize];
            let suggestion = match current_season {
                Season::Autumn => "Try harvesting or stockpiling instead. Sowing resumes in Winter.".to_string(),
                _ => "This season blocks sowing.".to_string(),
            };
            return Err(ActionError::SeasonRestriction { 
                action: "sow", 
                season: current_season, 
                suggestion 
            });
        }
        
        // Check if player has the card
        let player_hand = &player_hands[player.0 as usize];
        if !player_hand.contains(&card) {
            return Err(ActionError::CardNotInHand { 
                player, 
                card, 
                available_cards: player_hand.clone() 
            });
        }
        
        // Remove card from player's hand
        player_hands[player.0 as usize].retain(|&c| c != card);
        
        // Add card to field
        field_cards[field.0 as usize].push(card);
        
        // Face cards rotate the Illimat  
        if Self::is_face_card(card) {
            Self::rotate_illimat_to_season(field, card.suit(), illimat_orientation, field_seasons);
        }
        
        // Advance turn
        *turn_number += 1;
        
        Ok(())
    }
    
    /// Apply a harvest action to the game state
    pub fn apply_harvest(
        field_cards: &mut [Vec<Card>; 4],
        field_stockpiles: &mut [Vec<Stockpile>; 4],
        player_hands: &mut [Vec<Card>; 4],
        player_harvests: &mut [Vec<Card>; 4],
        field_seasons: &mut [Season; 4],
        illimat_orientation: &mut u8,
        turn_number: &mut u16,
        player: PlayerId,
        field: FieldId,
        card: Card,
        targets: Vec<Card>,
    ) -> Result<bool, ActionError> {
        // Handle auto-collection for exact matches when no targets specified
        let actual_targets = if targets.is_empty() {
            // Auto-collect all exact value matches
            Self::find_auto_collection_targets(
                &field_cards[field.0 as usize],
                &field_stockpiles[field.0 as usize],
                card
            )
        } else {
            // Use manually specified targets
            targets
        };
        
        // Validate the harvest action
        Self::validate_harvest_action(
            field_cards, field_stockpiles, player_hands, field_seasons,
            *illimat_orientation, *turn_number, player, field, card, &actual_targets
        )?;
        
        // Execute the harvest
        Self::execute_harvest(
            field_cards, field_stockpiles, player_hands, player_harvests,
            field_seasons, illimat_orientation, turn_number, player, field, card, actual_targets
        )
    }
    
    /// Validate a harvest action before execution
    fn validate_harvest_action(
        field_cards: &[Vec<Card>; 4],
        field_stockpiles: &[Vec<Stockpile>; 4],
        player_hands: &[Vec<Card>; 4],
        field_seasons: &[Season; 4],
        illimat_orientation: u8,
        turn_number: u16,
        player: PlayerId,
        field: FieldId,
        card: Card,
        targets: &[Card],
    ) -> Result<(), ActionError> {
        // Check if harvesting is allowed in this field
        if !CapabilityManager::can_harvest(field, illimat_orientation) {
            let current_season = field_seasons[field.0 as usize];
            let suggestion = match current_season {
                Season::Winter => "Try sowing or stockpiling instead. Harvesting resumes in Spring.".to_string(),
                _ => "This season blocks harvesting.".to_string(),
            };
            return Err(ActionError::SeasonRestriction { 
                action: "harvest", 
                season: current_season, 
                suggestion 
            });
        }
        
        // Check if player has the card
        let player_hand = &player_hands[player.0 as usize];
        if !player_hand.contains(&card) {
            return Err(ActionError::CardNotInHand { 
                player, 
                card, 
                available_cards: player_hand.clone() 
            });
        }
        
        // Validate same-turn stockpile restriction
        StockpileManager::validate_same_turn_restriction(
            &field_stockpiles[field.0 as usize], 
            targets, 
            turn_number
        )?;
        
        // Validate the targets match a valid combination
        Self::validate_harvest_targets(
            &field_cards[field.0 as usize],
            &field_stockpiles[field.0 as usize],
            card,
            targets
        )
    }
    
    /// Validate that harvest targets form a valid combination  
    fn validate_harvest_targets(
        field_cards: &[Card],
        field_stockpiles: &[Stockpile],
        played_card: Card,
        targets: &[Card],
    ) -> Result<(), ActionError> {
        let combinations = Self::find_harvest_combinations(
            field_cards, field_stockpiles, played_card
        );
        
        if combinations.is_empty() {
            let field_name = "this field".to_string(); // TODO: Pass actual field name
            let help = match played_card.rank() {
                Rank::Fool => "Try using Fool as value 1 or 14 to match cards or stockpiles.".to_string(),
                _ => format!("Look for cards or stockpiles that add up to {}.", Self::get_card_value(played_card)),
            };
            return Err(ActionError::NoValidTargets { 
                action: "harvest", 
                field_name, 
                help 
            });
        }
        
        // Check if targets form a valid combination
        let is_valid = if Self::is_auto_collection_targets(targets, &combinations) {
            // Auto-collection: targets should be all exact matches combined
            true
        } else {
            // Manual selection: targets should match one of the individual combinations
            combinations.iter().any(|combo| Self::cards_match_exactly(combo, targets))
        };
        
        if !is_valid {
            let valid_options: Vec<String> = combinations.iter().take(3).map(|combo| {
                let cards_str = combo.iter()
                    .map(|c| format!("{}", c))
                    .collect::<Vec<_>>()
                    .join(" + ");
                let total: u8 = combo.iter().map(|&c| Self::get_card_value(c)).sum();
                format!("{} (sum: {})", cards_str, total)
            }).collect();
            
            return Err(ActionError::InvalidCombination { 
                reason: "Selected cards don't form a valid harvest combination".to_string(),
                valid_options 
            });
        }
        
        Ok(())
    }
    
    /// Execute a validated harvest action
    fn execute_harvest(
        field_cards: &mut [Vec<Card>; 4],
        field_stockpiles: &mut [Vec<Stockpile>; 4],
        player_hands: &mut [Vec<Card>; 4],
        player_harvests: &mut [Vec<Card>; 4],
        field_seasons: &mut [Season; 4],
        illimat_orientation: &mut u8,
        turn_number: &mut u16,
        player: PlayerId,
        field: FieldId,
        card: Card,
        targets: Vec<Card>,
    ) -> Result<bool, ActionError> {
        // Remove card from player's hand
        player_hands[player.0 as usize].retain(|&c| c != card);
        
        // Add played card to player's harvest
        player_harvests[player.0 as usize].push(card);
        
        // Remove target cards from field and add to player's harvest
        Self::remove_targets_from_field(field_cards, field_stockpiles, player_harvests, field, player, &targets);
        
        // Handle face card rotation
        if Self::is_face_card(card) {
            Self::rotate_illimat_to_season(field, card.suit(), illimat_orientation, field_seasons);
        }
        
        // Check if field was cleared and advance turn
        let field_cleared = Self::is_field_cleared(field_cards, field_stockpiles, field);
        *turn_number += 1;
        
        Ok(field_cleared)
    }
    
    /// Remove target cards from field and add to player's harvest
    fn remove_targets_from_field(
        field_cards: &mut [Vec<Card>; 4],
        field_stockpiles: &mut [Vec<Stockpile>; 4],
        player_harvests: &mut [Vec<Card>; 4],
        field: FieldId,
        player: PlayerId,
        targets: &[Card],
    ) {
        for &target in targets {
            // Remove from loose cards
            field_cards[field.0 as usize].retain(|&c| c != target);
            
            // Remove from stockpiles (and remove empty stockpiles)
            field_stockpiles[field.0 as usize].retain_mut(|stockpile| {
                stockpile.cards.retain(|&c| c != target);
                !stockpile.cards.is_empty()
            });
            
            player_harvests[player.0 as usize].push(target);
        }
    }
    
    /// Check if field is cleared (no loose cards or stockpiles)
    fn is_field_cleared(
        field_cards: &[Vec<Card>; 4],
        field_stockpiles: &[Vec<Stockpile>; 4],
        field: FieldId,
    ) -> bool {
        field_cards[field.0 as usize].is_empty() && 
        field_stockpiles[field.0 as usize].is_empty()
    }
    
    /// Apply a stockpile action to the game state
    pub fn apply_stockpile(
        field_cards: &mut [Vec<Card>; 4],
        field_stockpiles: &mut [Vec<Stockpile>; 4],
        player_hands: &mut [Vec<Card>; 4],
        field_seasons: &mut [Season; 4],
        illimat_orientation: &mut u8,
        turn_number: &mut u16,
        player: PlayerId,
        field: FieldId,
        card: Card,
        targets: Vec<Card>,
    ) -> Result<(), ActionError> {
        // Check if stockpiling is allowed in this field
        if !CapabilityManager::can_stockpile(field, *illimat_orientation) {
            let current_season = field_seasons[field.0 as usize];
            let suggestion = match current_season {
                Season::Spring => "Try sowing or harvesting instead. Stockpiling resumes in Summer.".to_string(),
                _ => "This season blocks stockpiling.".to_string(),
            };
            return Err(ActionError::SeasonRestriction { 
                action: "stockpile", 
                season: current_season, 
                suggestion 
            });
        }
        
        // Check if player has the card
        let player_hand = &player_hands[player.0 as usize];
        if !player_hand.contains(&card) {
            return Err(ActionError::CardNotInHand { 
                player, 
                card, 
                available_cards: player_hand.clone() 
            });
        }
        
        // Must have exactly one target (the passive card)
        if targets.len() != 1 {
            let available_cards: Vec<String> = field_cards[field.0 as usize].iter().chain(
                field_stockpiles[field.0 as usize].iter().flat_map(|s| s.cards.iter())
            ).map(|c| format!("{}", c)).collect();
            
            return Err(ActionError::InvalidCombination {
                reason: "Stockpiling requires exactly one passive card".to_string(),
                valid_options: if available_cards.is_empty() { 
                    vec![] 
                } else { 
                    vec![format!("Available cards: {}", available_cards.join(", "))] 
                }
            });
        }
        
        let passive_card = targets[0];
        
        // Check that the passive card is available in the field
        let passive_available = field_cards[field.0 as usize].contains(&passive_card) ||
            field_stockpiles[field.0 as usize].iter()
                .any(|stockpile| stockpile.cards.contains(&passive_card));
        
        if !passive_available {
            let available_cards: Vec<String> = field_cards[field.0 as usize].iter().chain(
                field_stockpiles[field.0 as usize].iter().flat_map(|s| s.cards.iter())
            ).map(|c| format!("{}", c)).collect();
            
            return Err(ActionError::InvalidCombination {
                reason: format!("Card {} is not available in this field", passive_card),
                valid_options: if available_cards.is_empty() {
                    vec!["No cards available for stockpiling".to_string()]
                } else {
                    vec![format!("Available: {}", available_cards.join(", "))]
                }
            });
        }
        
        // Calculate stockpile value (sum of active + passive)
        let active_value = Self::get_card_value(card);
        let passive_value = Self::get_card_value(passive_card);
        let stockpile_value = active_value + passive_value;
        
        // Remove active card from player's hand
        player_hands[player.0 as usize].retain(|&c| c != card);
        
        // Remove passive card from field
        field_cards[field.0 as usize].retain(|&c| c != passive_card);
        
        // Remove passive card from existing stockpiles (and clean up empty stockpiles)
        field_stockpiles[field.0 as usize].retain_mut(|stockpile| {
            stockpile.cards.retain(|&c| c != passive_card);
            !stockpile.cards.is_empty()
        });
        
        // Create new stockpile
        let new_stockpile = Stockpile {
            value: stockpile_value,
            cards: vec![card, passive_card],
            created_turn: *turn_number,
        };
        
        field_stockpiles[field.0 as usize].push(new_stockpile);
        
        // Face cards rotate the Illimat
        if Self::is_face_card(card) {
            Self::rotate_illimat_to_season(field, card.suit(), illimat_orientation, field_seasons);
        }
        
        // Advance turn
        *turn_number += 1;
        
        Ok(())
    }
    
    // Helper functions
    
    fn is_face_card(card: Card) -> bool {
        matches!(card.rank(), Rank::Fool | Rank::Knight | Rank::Queen | Rank::King)
    }
    
    pub fn get_card_value(card: Card) -> u8 {
        match card.rank() {
            Rank::Fool => 1, // Always use 1 for stockpiling (player can't choose)
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Knight => 11,
            Rank::Queen => 12,  
            Rank::King => 13,
        }
    }
    
    
    fn rotate_illimat_to_season(
        field: FieldId, 
        suit: Suit, 
        illimat_orientation: &mut u8,
        field_seasons: &mut [Season; 4]
    ) {
        let target_season = match suit {
            Suit::Spring => Season::Spring,
            Suit::Summer => Season::Summer,
            Suit::Autumn => Season::Autumn,
            Suit::Winter => Season::Winter,
            Suit::Stars => return, // Stars don't rotate the Illimat
        };
        
        // Calculate what orientation would make this field have the target season
        let season_offset = match target_season {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Autumn => 2,
            Season::Winter => 3,
        };
        *illimat_orientation = (field.0 + 4 - season_offset) % 4;
        
        // Update all field seasons
        for i in 0..4 {
            let field_season_offset = (i + 4 - (*illimat_orientation as usize)) % 4;
            field_seasons[i] = match field_season_offset {
                0 => Season::Spring,
                1 => Season::Summer,
                2 => Season::Autumn,
                3 => Season::Winter,
                _ => unreachable!(),
            };
        }
    }
    
    /// Find targets for auto-collection (exact value matches only)
    fn find_auto_collection_targets(
        field_cards: &[Card],
        field_stockpiles: &[Stockpile],
        played_card: Card
    ) -> Vec<Card> {
        let mut targets = Vec::new();
        
        // Get possible values for the played card (Fool can be 1 or 14)
        let played_values = if played_card.rank() == Rank::Fool {
            vec![1, 14]
        } else {
            vec![Self::get_card_value(played_card)]
        };
        
        for &target_value in &played_values {
            // Collect exact value matches from loose cards
            for &card in field_cards {
                if Self::get_card_value(card) == target_value {
                    targets.push(card);
                }
            }
            
            // Collect exact value matches from stockpiles
            for stockpile in field_stockpiles {
                if stockpile.value == target_value {
                    targets.extend(stockpile.cards.clone());
                }
            }
        }
        
        targets
    }
    
    fn find_harvest_combinations(
        field_cards: &[Card],
        field_stockpiles: &[Stockpile],
        played_card: Card
    ) -> Vec<Vec<Card>> {
        let mut combinations = Vec::new();
        
        // Get possible values for the played card (Fool can be 1 or 14)
        let played_values = if played_card.rank() == Rank::Fool {
            vec![1, 14]
        } else {
            vec![Self::get_card_value(played_card)]
        };
        
        for &target_value in &played_values {
            // Single cards that match
            for &card in field_cards {
                if Self::get_card_value(card) == target_value {
                    combinations.push(vec![card]);
                }
            }
            
            // Stockpiles that match
            for stockpile in field_stockpiles {
                if stockpile.value == target_value {
                    combinations.push(stockpile.cards.clone());
                }
            }
            
            // Card combinations that sum to target value
            Self::find_card_combinations(field_cards, target_value, &mut combinations);
        }
        
        combinations
    }
    
    fn find_card_combinations(
        available_cards: &[Card],
        target_sum: u8,
        combinations: &mut Vec<Vec<Card>>
    ) {
        fn backtrack(
            cards: &[Card],
            target: u8,
            current_sum: u8,
            current_combo: &mut Vec<Card>,
            start_idx: usize,
            results: &mut Vec<Vec<Card>>
        ) {
            if current_sum == target && current_combo.len() >= 2 {
                results.push(current_combo.clone());
                return;
            }
            
            if current_sum > target || start_idx >= cards.len() {
                return;
            }
            
            for i in start_idx..cards.len() {
                let card = cards[i];
                let card_value = ActionManager::get_card_value(card);
                
                if current_sum + card_value <= target {
                    current_combo.push(card);
                    backtrack(cards, target, current_sum + card_value, current_combo, i + 1, results);
                    current_combo.pop();
                }
            }
        }
        
        let mut current_combo = Vec::new();
        backtrack(available_cards, target_sum, 0, &mut current_combo, 0, combinations);
    }
    
    /// Check if targets represent auto-collection (all exact value matches combined)
    fn is_auto_collection_targets(targets: &[Card], combinations: &[Vec<Card>]) -> bool {
        if targets.is_empty() {
            return false;
        }
        
        // Get all single-card exact matches from combinations
        let exact_matches: Vec<Card> = combinations.iter()
            .filter(|combo| combo.len() == 1) // Only single cards (exact matches)
            .map(|combo| combo[0])
            .collect();
        
        if exact_matches.is_empty() {
            return false;
        }
        
        // Check if targets contains exactly all the exact matches
        let mut targets_sorted = targets.to_vec();
        let mut matches_sorted = exact_matches;
        targets_sorted.sort();
        matches_sorted.sort();
        
        targets_sorted == matches_sorted
    }
    
    fn cards_match_exactly(combo: &[Card], targets: &[Card]) -> bool {
        if combo.len() != targets.len() {
            return false;
        }
        
        let mut combo_sorted = combo.to_vec();
        let mut targets_sorted = targets.to_vec();
        combo_sorted.sort();
        targets_sorted.sort();
        
        combo_sorted == targets_sorted
    }
}