use crate::game::card::Card;

/// Stockpile representation - a set of cards that sum to a harvestable value
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Stockpile {
    pub value: u8,
    pub cards: Vec<Card>,
    pub created_turn: u16, // Turn number when this stockpile was created
}

/// Stockpile management utilities
pub struct StockpileManager;

impl StockpileManager {
    /// Check if any cards in the targets are from a same-turn stockpile
    pub fn validate_same_turn_restriction(
        field_stockpiles: &[Stockpile],
        targets: &[Card],
        current_turn: u16
    ) -> Result<(), String> {
        for stockpile in field_stockpiles {
            if stockpile.created_turn == current_turn {
                // Check if any cards in this stockpile are being targeted
                for &target in targets {
                    if stockpile.cards.contains(&target) {
                        return Err("Cannot harvest cards from a stockpile created this turn".to_string());
                    }
                }
            }
        }
        Ok(())
    }
}