use crate::game::card::Card;
use crate::game::field_id::FieldId;
use crate::game::player::PlayerId;
use crate::game::season::Season;
use crate::game::stockpile::Stockpile;
use crate::game::okus::{OkusId, OkusPosition};

/// Display utilities for game state components
pub struct DisplayManager;

impl DisplayManager {
    /// Format a field display with cards and stockpiles
    pub fn format_field(
        field: FieldId,
        field_cards: &[Card],
        field_stockpiles: &[Stockpile],
        season: Season,
        illimat_orientation: u8
    ) -> String {
        let field_name = Self::field_name_with_season_restrictions(field, season, illimat_orientation);
        
        if field_cards.is_empty() && field_stockpiles.is_empty() {
            format!("{}: empty", field_name)
        } else {
            let mut parts = Vec::new();
            
            // Loose cards
            if !field_cards.is_empty() {
                let cards_str = field_cards.iter()
                    .map(|card| format!("{}", card))
                    .collect::<Vec<_>>()
                    .join(" ");
                parts.push(cards_str);
            }
            
            // Stockpiles  
            if !field_stockpiles.is_empty() {
                let stockpiles_str = field_stockpiles.iter()
                    .map(|stockpile| Self::format_stockpile(stockpile))
                    .collect::<Vec<_>>()
                    .join(" ");
                parts.push(format!("(Stockpiles: {})", stockpiles_str));
            }
            
            format!("{}: {}", field_name, parts.join(" "))
        }
    }
    
    /// Format a stockpile display
    pub fn format_stockpile(stockpile: &Stockpile) -> String {
        let cards_str = stockpile.cards.iter()
            .map(|card| format!("{}", card))
            .collect::<Vec<_>>()
            .join(", ");
        format!("[{}: {}]", stockpile.value, cards_str)
    }
    
    /// Format a player's hand with numbered choices
    pub fn format_hand_with_numbers(hand: &[Card]) -> String {
        if hand.is_empty() {
            "empty".to_string()
        } else {
            hand.iter()
                .enumerate()
                .map(|(i, card)| format!("{}. {}", i + 1, card))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
    
    /// Format a player's harvest
    pub fn format_harvest(harvest: &[Card]) -> String {
        if harvest.is_empty() {
            "none".to_string()
        } else {
            harvest.iter()
                .map(|card| format!("{}", card))
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
    
    /// Format harvest combinations with numbers for selection
    pub fn format_harvest_combinations(combinations: &[Vec<Card>]) -> String {
        combinations.iter()
            .enumerate()
            .map(|(i, combo)| {
                let cards_str = combo.iter()
                    .map(|card| format!("{}", card))
                    .collect::<Vec<_>>()
                    .join(" + ");
                let total = combo.iter().map(|&card| Self::get_card_value(card)).sum::<u8>();
                format!("{}. {} (total: {})", i + 1, cards_str, total)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    
    /// Format okus status display
    pub fn format_okus_status(okus_positions: &[OkusPosition; 4]) -> String {
        let mut parts = Vec::new();
        
        // Available okus on Illimat
        let available: Vec<_> = [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
            .iter()
            .filter(|&&okus| okus_positions[okus as usize] == OkusPosition::OnIllimat)
            .collect();
        
        if !available.is_empty() {
            let available_str = available.iter()
                .map(|okus| format!("{}", okus))
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("On Illimat: {}", available_str));
        }
        
        // Player okus
        for player_id in 0..4 {
            let player = PlayerId(player_id);
            let player_okus: Vec<_> = [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
                .iter()
                .filter(|&&okus| okus_positions[okus as usize] == OkusPosition::WithPlayer(player))
                .collect();
            
            if !player_okus.is_empty() {
                let okus_str = player_okus.iter()
                    .map(|okus| format!("{}", okus))
                    .collect::<Vec<_>>()
                    .join(", ");
                parts.push(format!("Player {}: {}", player_id, okus_str));
            }
        }
        
        if parts.is_empty() {
            "All okus distributed".to_string()
        } else {
            parts.join(" | ")
        }
    }
    
    /// Format okus selection options
    pub fn format_okus_selection(available_okus: &[OkusId]) -> String {
        available_okus.iter()
            .enumerate()
            .map(|(i, okus)| format!("{}. {}", i + 1, okus))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Format season with mystical descriptions
    pub fn format_season_mystical(season: Season) -> String {
        match season {
            Season::Spring => "Spring (时节万物复苏, stockpiling forbidden)".to_string(),
            Season::Summer => "Summer (盛夏如火如荼, all actions permitted)".to_string(), 
            Season::Autumn => "Autumn (秋收冬藏之候, sowing forbidden)".to_string(),
            Season::Winter => "Winter (严冬万籁俱寂, harvesting forbidden)".to_string(),
        }
    }
    
    // Helper functions
    
    fn field_name_with_season_restrictions(field: FieldId, season: Season, illimat_orientation: u8) -> String {
        let seasonal_name = field.seasonal_name(illimat_orientation);
        let restrictions = Self::get_season_restrictions(season);
        format!("{}{}", seasonal_name, restrictions)
    }
    
    fn get_season_restrictions(season: Season) -> String {
        match season {
            Season::Winter => " (no harvesting)".to_string(),
            Season::Spring => " (no stockpiling)".to_string(),
            Season::Summer => "".to_string(), // No restrictions
            Season::Autumn => " (no sowing)".to_string(),
        }
    }
    
    fn get_card_value(card: Card) -> u8 {
        match card.rank() {
            crate::game::card::Rank::Fool => 1, // Display as 1 for simplicity
            crate::game::card::Rank::Two => 2,
            crate::game::card::Rank::Three => 3,
            crate::game::card::Rank::Four => 4,
            crate::game::card::Rank::Five => 5,
            crate::game::card::Rank::Six => 6,
            crate::game::card::Rank::Seven => 7,
            crate::game::card::Rank::Eight => 8,
            crate::game::card::Rank::Nine => 9,
            crate::game::card::Rank::Ten => 10,
            crate::game::card::Rank::Knight => 11,
            crate::game::card::Rank::Queen => 12,  
            crate::game::card::Rank::King => 13,
        }
    }
}

