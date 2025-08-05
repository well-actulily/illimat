use crate::game::player::PlayerType;

/// Game configuration
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GameConfig {
    pub player_count: u8,
    pub player_types: [PlayerType; 4],  // Human/Computer for each slot
    pub use_stars_suit: bool,             // true = 65 cards, false = 52 cards
}

impl GameConfig {
    /// Create a new game configuration with all human players
    pub fn new(player_count: u8) -> Self {
        if player_count < 2 || player_count > 4 {
            panic!("Player count must be between 2 and 4");
        }
        
        let mut player_types = [PlayerType::Computer; 4];
        for i in 0..player_count as usize {
            player_types[i] = PlayerType::Human;
        }
        
        GameConfig {
            player_count,
            player_types,
            use_stars_suit: true, // Default to full deck (including Stars)
        }
    }
    
    /// Set whether to use the full 65-card deck or standard 52-card deck
    pub fn with_deck_size(mut self, use_stars_suit: bool) -> Self {
        self.use_stars_suit = use_stars_suit;
        self
    }
    
    /// Set a specific player as computer-controlled
    pub fn with_computer_player(mut self, player_id: u8) -> Self {
        if (player_id as usize) < self.player_count as usize {
            self.player_types[player_id as usize] = PlayerType::Computer;
        }
        self
    }
    
    /// Get the number of human players
    pub fn human_player_count(&self) -> u8 {
        self.player_types[..self.player_count as usize]
            .iter()
            .filter(|&&player_type| player_type == PlayerType::Human)
            .count() as u8
    }
    
    /// Get the number of computer players
    pub fn computer_player_count(&self) -> u8 {
        self.player_types[..self.player_count as usize]
            .iter()
            .filter(|&&player_type| player_type == PlayerType::Computer)
            .count() as u8
    }
    
    /// Check if a player is human-controlled
    pub fn is_human_player(&self, player_id: u8) -> bool {
        if (player_id as usize) >= self.player_count as usize {
            false
        } else {
            self.player_types[player_id as usize] == PlayerType::Human
        }
    }
    
    /// Get the expected deck size based on configuration
    pub fn expected_deck_size(&self) -> usize {
        if self.use_stars_suit {
            65 // 5 suits × 13 cards each
        } else {
            52 // 4 suits × 13 cards each (no Stars)
        }
    }
}

/// Game phase for proper state management
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum GamePhase {
    Setup,      // Game creation/configuration
    Playing,    // Normal turn-by-turn play
    RoundEnd,   // Scoring and cleanup between rounds
    GameEnd,    // Victory achieved
}

impl GamePhase {
    /// Check if the game is in an active playing state
    pub fn is_active(&self) -> bool {
        matches!(self, GamePhase::Playing)
    }
    
    /// Check if the game has ended
    pub fn is_ended(&self) -> bool {
        matches!(self, GamePhase::GameEnd)
    }
    
    /// Check if scoring should be processed
    pub fn should_score(&self) -> bool {
        matches!(self, GamePhase::RoundEnd)
    }
}