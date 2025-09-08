/// Simple CPU Algorithm for Illimat
/// 
/// This provides a basic opponent that follows a simple strategy:
/// 1. Harvest if possible (prioritizes highest value combinations)
/// 2. Stockpile if possible (creates stockpiles for future harvesting)
/// 3. Sow otherwise (chooses field randomly)
/// 
/// All choices within categories are made randomly for unpredictability.

use crate::game::state::IllimatState;
use crate::game::actions::Action;
use crate::game::card::Card;
use crate::game::field_id::FieldId;
use crate::game::player::PlayerId;
use crate::game::capabilities::CapabilityManager;
use crate::game::stockpile::Stockpile;
use rand::prelude::*;

pub struct SimpleCpu {
    rng: StdRng,
}

impl SimpleCpu {
    /// Create a new simple CPU with a random seed
    pub fn new() -> Self {
        Self {
            rng: StdRng::from_entropy(),
        }
    }
    
    /// Create a new simple CPU with a specific seed (for testing)
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: StdRng::seed_from_u64(seed),
        }
    }
    
    /// Choose an action for the given player in the given game state
    pub fn choose_action(&mut self, game: &IllimatState, player: PlayerId) -> Action {
        let hand = &game.player_hands[player.0 as usize];
        
        // Strategy 1: Try to harvest (highest priority)
        if let Some(action) = self.try_harvest(game, player, hand) {
            return action;
        }
        
        // Strategy 2: Try to stockpile (medium priority)  
        if let Some(action) = self.try_stockpile(game, player, hand) {
            return action;
        }
        
        // Strategy 3: Sow (lowest priority, always possible)
        self.choose_sow_action(game, player, hand)
    }
    
    /// Try to find a harvest action
    fn try_harvest(&mut self, game: &IllimatState, player: PlayerId, hand: &[Card]) -> Option<Action> {
        let mut harvest_options = Vec::new();
        
        for (field_idx, field_cards) in game.field_cards.iter().enumerate() {
            if field_cards.is_empty() {
                continue;
            }
            
            let field_id = FieldId(field_idx as u8);
            
            // Check if harvesting is allowed in this field
            if !CapabilityManager::can_harvest_basic(field_id, game.illimat_orientation) {
                continue;
            }
            
            // Try each card in hand
            for &hand_card in hand {
                let combinations = self.find_harvest_combinations(
                    field_cards, 
                    &game.field_stockpiles[field_idx], 
                    hand_card
                );
                
                if !combinations.is_empty() {
                    // Prioritize combinations by total value (prefer higher value harvests)
                    let mut combo_options: Vec<_> = combinations.into_iter()
                        .map(|combo| {
                            let total_value: u8 = combo.iter().map(|&c| self.get_card_value(c)).sum();
                            (combo, total_value)
                        })
                        .collect();
                    
                    // Sort by value descending
                    combo_options.sort_by(|a, b| b.1.cmp(&a.1));
                    
                    for (combo, value) in combo_options {
                        harvest_options.push((
                            Action::Harvest {
                                field: field_id,
                                card: hand_card,
                                targets: combo,
                            },
                            value, // Store value for prioritization
                        ));
                    }
                }
            }
        }
        
        if harvest_options.is_empty() {
            return None;
        }
        
        // Sort by value descending and pick randomly from top 3
        harvest_options.sort_by(|a, b| b.1.cmp(&a.1));
        let top_choices = std::cmp::min(3, harvest_options.len());
        let chosen = harvest_options[..top_choices].choose(&mut self.rng)?;
        
        Some(chosen.0.clone())
    }
    
    /// Try to find a stockpile action  
    fn try_stockpile(&mut self, game: &IllimatState, player: PlayerId, hand: &[Card]) -> Option<Action> {
        let mut stockpile_options = Vec::new();
        
        for (field_idx, field_cards) in game.field_cards.iter().enumerate() {
            if field_cards.is_empty() {
                continue;
            }
            
            let field_id = FieldId(field_idx as u8);
            
            // Check if stockpiling is allowed in this field
            if !CapabilityManager::can_stockpile_basic(field_id, game.illimat_orientation) {
                continue;
            }
            
            // Try each card in hand as played card
            for &hand_card in hand {
                let played_value = self.get_card_value(hand_card);
                
                // Look for single cards that could combine with played card value
                for &field_card in field_cards {
                    let field_value = self.get_card_value(field_card);
                    
                    // Check if this creates a valuable stockpile
                    // (Prefer creating stockpiles of 10+ value as they're good harvest targets)
                    let stockpile_value = played_value + field_value;
                    if stockpile_value >= 6 && stockpile_value <= 14 {
                        stockpile_options.push((
                            Action::Stockpile {
                                field: field_id,
                                card: hand_card,
                                targets: vec![field_card],
                            },
                            stockpile_value, // Store for prioritization
                        ));
                    }
                }
            }
        }
        
        if stockpile_options.is_empty() {
            return None;
        }
        
        // Prefer higher value stockpiles (better future harvest targets)
        stockpile_options.sort_by(|a, b| b.1.cmp(&a.1));
        let top_choices = std::cmp::min(3, stockpile_options.len());
        let chosen = stockpile_options[..top_choices].choose(&mut self.rng)?;
        
        Some(chosen.0.clone())
    }
    
    /// Choose a sow action (always available)
    fn choose_sow_action(&mut self, game: &IllimatState, player: PlayerId, hand: &[Card]) -> Action {
        let mut sow_options = Vec::new();
        
        for field_idx in 0..4 {
            let field_id = FieldId(field_idx);
            
            // Check if sowing is allowed in this field  
            if CapabilityManager::can_sow_basic(field_id, game.illimat_orientation) {
                // Try each card in hand
                for &hand_card in hand {
                    sow_options.push(Action::Sow {
                        field: field_id,
                        card: hand_card,
                    });
                }
            }
        }
        
        // If no sowing is possible (shouldn't happen), sow anywhere
        if sow_options.is_empty() {
            let field_id = FieldId(0);
            let hand_card = hand[0]; // Just pick first card
            return Action::Sow { field: field_id, card: hand_card };
        }
        
        // Choose randomly from available sow options
        sow_options.choose(&mut self.rng).unwrap().clone()
    }
    
    /// Get the numeric value of a card (same logic as console)
    fn get_card_value(&self, card: Card) -> u8 {
        use crate::game::card::Rank;
        match card.rank() {
            Rank::Fool => 1, // For CPU, Fool is always 1 (simpler logic)
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
    
    /// Find all valid harvest combinations (simplified version of console logic)
    fn find_harvest_combinations(
        &self,
        field_cards: &[Card],
        field_stockpiles: &[Stockpile], 
        played_card: Card
    ) -> Vec<Vec<Card>> {
        let mut combinations = Vec::new();
        let target_value = self.get_card_value(played_card);
        
        // Single cards that match
        for &card in field_cards {
            if self.get_card_value(card) == target_value {
                combinations.push(vec![card]);
            }
        }
        
        // Stockpiles that match  
        for stockpile in field_stockpiles {
            if stockpile.value == target_value {
                combinations.push(stockpile.cards.clone());
            }
        }
        
        // Simple two-card combinations (most common)
        for (i, &card1) in field_cards.iter().enumerate() {
            for &card2 in &field_cards[i+1..] {
                if self.get_card_value(card1) + self.get_card_value(card2) == target_value {
                    combinations.push(vec![card1, card2]);
                }
            }
        }
        
        combinations
    }
}

impl Default for SimpleCpu {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::game_config::GameConfig;
    use crate::game::luminary::LuminaryConfiguration;
    use crate::game::card::{Card, Rank, Suit};
    
    fn create_test_game() -> IllimatState {
        let config = GameConfig::new(2).with_luminaries(LuminaryConfiguration::none());
        IllimatState::new(config)
    }
    
    #[test]
    fn test_simple_cpu_chooses_harvest_over_sow() {
        let mut game = create_test_game();
        let mut cpu = SimpleCpu::with_seed(42);
        
        // Clear all fields first to ensure clean state
        game.field_cards = [vec![], vec![], vec![], vec![]];
        game.field_stockpiles = [vec![], vec![], vec![], vec![]];
        
        // Set up a harvestable situation
        game.field_cards[0] = vec![Card::new(Rank::Five, Suit::Spring)];
        game.player_hands[0] = vec![Card::new(Rank::Five, Suit::Summer)];
        
        // Ensure field 0 is in summer (allows harvesting)
        game.illimat_orientation = 3; // Makes field 0 Summer season
        use crate::game::season::SeasonManager;
        SeasonManager::update_all_seasons(&mut game.field_seasons, game.illimat_orientation);
        
        let action = cpu.choose_action(&game, PlayerId(0));
        
        // Should choose harvest over sow
        match action {
            Action::Harvest { field, card, targets } => {
                assert_eq!(field, FieldId(0));
                assert_eq!(card, Card::new(Rank::Five, Suit::Summer));
                assert_eq!(targets, vec![Card::new(Rank::Five, Suit::Spring)]);
            },
            _ => panic!("Expected harvest action, got {:?}", action),
        }
    }
    
    #[test]
    fn test_simple_cpu_chooses_stockpile_over_sow() {
        let mut game = create_test_game();
        let mut cpu = SimpleCpu::with_seed(42);
        
        // Clear all fields first to ensure clean state
        game.field_cards = [vec![], vec![], vec![], vec![]];
        game.field_stockpiles = [vec![], vec![], vec![], vec![]];
        
        // Set up a stockpile situation (no harvest possible)  
        game.field_cards[0] = vec![Card::new(Rank::Three, Suit::Spring)];
        game.player_hands[0] = vec![Card::new(Rank::Seven, Suit::Summer)]; // 7+3=10, good stockpile
        
        // Ensure field 0 is in summer (allows stockpiling)
        game.illimat_orientation = 3; // Makes field 0 Summer season
        use crate::game::season::SeasonManager;
        SeasonManager::update_all_seasons(&mut game.field_seasons, game.illimat_orientation);
        
        let action = cpu.choose_action(&game, PlayerId(0));
        
        // Should choose stockpile over sow
        match action {
            Action::Stockpile { field, card, targets } => {
                assert_eq!(field, FieldId(0));
                assert_eq!(card, Card::new(Rank::Seven, Suit::Summer));
                assert_eq!(targets, vec![Card::new(Rank::Three, Suit::Spring)]);
            },
            _ => panic!("Expected stockpile action, got {:?}", action),
        }
    }
    
    #[test]
    fn test_simple_cpu_falls_back_to_sow() {
        let mut game = create_test_game();
        let mut cpu = SimpleCpu::with_seed(42);
        
        // Set up empty fields (no harvest or stockpile possible)
        game.field_cards = [vec![], vec![], vec![], vec![]];
        game.player_hands[0] = vec![Card::new(Rank::Seven, Suit::Summer)];
        
        let action = cpu.choose_action(&game, PlayerId(0));
        
        // Should fall back to sow
        match action {
            Action::Sow { field: _, card } => {
                assert_eq!(card, Card::new(Rank::Seven, Suit::Summer));
            },
            _ => panic!("Expected sow action, got {:?}", action),
        }
    }
}