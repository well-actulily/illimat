use wasm_bindgen::prelude::*;
use crate::game::state::IllimatState;
use crate::game::game_config::GameConfig;
use crate::game::actions::Action;
use crate::game::player::PlayerId;
use crate::game::field_id::FieldId;
use crate::game::card::Card;

// Use wee_alloc as the global allocator for smaller WASM bundle size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Initialize panic hook for better debugging in browsers
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WebAssembly-compatible game engine for Illimat
#[wasm_bindgen]
pub struct WasmGameEngine {
    state: IllimatState,
}

/// Move validation and application response
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct MoveResponse {
    success: bool,
    error_message: Option<String>,
}

#[wasm_bindgen]
impl MoveResponse {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
}

#[wasm_bindgen]
impl WasmGameEngine {
    /// Create a new game engine
    #[wasm_bindgen(constructor)]
    pub fn new(player_count: u8) -> Result<WasmGameEngine, JsValue> {
        if player_count < 2 || player_count > 4 {
            return Err(JsValue::from_str("Player count must be between 2 and 4"));
        }
        
        let config = GameConfig::new(player_count);
        
        let state = IllimatState::new(config);
        
        Ok(WasmGameEngine { state })
    }
    
    /// Get the complete game state as JSON string
    #[wasm_bindgen]
    pub fn get_state_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Load game state from JSON string
    #[wasm_bindgen]
    pub fn set_state_json(&mut self, json: &str) -> Result<(), JsValue> {
        let state: IllimatState = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
        self.state = state;
        Ok(())
    }
    
    /// Get the current player ID
    #[wasm_bindgen]
    pub fn get_current_player(&self) -> u8 {
        self.state.current_player.0
    }
    
    /// Get the current season for a field (0=Winter, 1=Spring, 2=Summer, 3=Autumn)
    #[wasm_bindgen]
    pub fn get_field_season(&self, field_id: u8) -> Result<u8, JsValue> {
        if field_id >= 4 {
            return Err(JsValue::from_str("Field ID must be 0-3"));
        }
        
        let field = FieldId(field_id);
        let season = self.state.field_seasons[field.0 as usize];
        Ok(season as u8)
    }
    
    /// Validate a sow move without applying it
    #[wasm_bindgen]
    pub fn validate_sow(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> MoveResponse {
        let result = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a sow move to the game state
    #[wasm_bindgen]
    pub fn apply_sow(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> MoveResponse {
        let result = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit);
        
        match result {
            Ok((_player, field, card)) => {
                let action = Action::Sow { field, card };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply sow: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Validate a harvest move without applying it
    #[wasm_bindgen]
    pub fn validate_harvest(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_harvest(player_id, field_id, card_rank, card_suit, target_cards_json);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a harvest move to the game state
    #[wasm_bindgen]
    pub fn apply_harvest(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_harvest(player_id, field_id, card_rank, card_suit, target_cards_json);
        
        match result {
            Ok((_player, field, card, targets)) => {
                let action = Action::Harvest { field, card, targets };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply harvest: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Validate a stockpile move without applying it
    #[wasm_bindgen]
    pub fn validate_stockpile(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_stockpile(player_id, field_id, card_rank, card_suit, target_cards_json);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a stockpile move to the game state
    #[wasm_bindgen]
    pub fn apply_stockpile(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_stockpile(player_id, field_id, card_rank, card_suit, target_cards_json);
        
        match result {
            Ok((_player, field, card, targets)) => {
                let action = Action::Stockpile { field, card, targets };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply stockpile: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Get legal moves for the current player as JSON
    #[wasm_bindgen]
    pub fn get_legal_moves_json(&self) -> Result<String, JsValue> {
        // TODO: Implement legal moves generation
        let legal_moves: Vec<String> = vec!["Placeholder for legal moves".to_string()];
        serde_json::to_string(&legal_moves)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Check if the game is over
    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        // TODO: Implement game over check
        false
    }
    
    /// Get the winner (if game is over)
    #[wasm_bindgen]
    pub fn get_winner(&self) -> Option<u8> {
        // TODO: Implement winner determination
        None
    }
}

// Helper methods for parsing and validation
impl WasmGameEngine {
    fn parse_and_validate_sow(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> Result<(PlayerId, FieldId, Card), String> {
        let player = PlayerId(player_id);
        
        if field_id >= 4 {
            return Err("Field ID must be 0-3".to_string());
        }
        let field = FieldId(field_id);
        
        let card = self.parse_card(card_rank, card_suit)?;
        
        // Validate player has the card
        if !self.state.player_hands[player.0 as usize].contains(&card) {
            return Err("Player does not have this card".to_string());
        }
        
        // Validate it's the player's turn
        if self.state.current_player != player {
            return Err("Not this player's turn".to_string());
        }
        
        Ok((player, field, card))
    }
    
    fn parse_and_validate_harvest(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> Result<(PlayerId, FieldId, Card, Vec<Card>), String> {
        let (player, field, card) = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit)?;
        
        let targets: Vec<Card> = serde_json::from_str(target_cards_json)
            .map_err(|_| "Invalid target cards JSON".to_string())?;
        
        Ok((player, field, card, targets))
    }
    
    fn parse_and_validate_stockpile(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> Result<(PlayerId, FieldId, Card, Vec<Card>), String> {
        let (player, field, card) = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit)?;
        
        let targets: Vec<Card> = serde_json::from_str(target_cards_json)
            .map_err(|_| "Invalid target cards JSON".to_string())?;
        
        Ok((player, field, card, targets))
    }
    
    fn parse_card(&self, rank: u8, suit: u8) -> Result<Card, String> {
        use crate::game::card::{Rank, Suit};
        
        let rank = match rank {
            0 => Rank::Fool,
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Knight,
            12 => Rank::Queen,
            13 => Rank::King,
            _ => return Err("Invalid rank".to_string()),
        };
        
        let suit = match suit {
            0 => Suit::Spring,
            1 => Suit::Summer,
            2 => Suit::Autumn,
            3 => Suit::Winter,  
            4 => Suit::Stars,
            _ => return Err("Invalid suit".to_string()),
        };
        
        Ok(Card::new(rank, suit))
    }
}

// Export types for TypeScript definitions
#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACE: &'static str = r#"
export interface WasmGameState {
  config: {
    player_count: number;
    use_stars_suit: boolean;
    target_score: number;
  };
  phase: string;
  field_cards: Card[][];
  player_hands: Card[][];
  player_harvests: Card[][];
  deck: Card[];
  field_stockpiles: Stockpile[][];
  field_seasons: Season[];
  okus_positions: OkusPosition[];
  current_player: { 0: number };
  dealer: { 0: number };
  total_scores: number[];
  round_number: number;
  turn_number: number;
  illimat_orientation: number;
}

export interface Card {
  rank: string;
  suit: string;
}

export interface MoveResponseData {
  success: boolean;
  error_message?: string;
}
"#;