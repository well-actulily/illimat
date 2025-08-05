use crate::game::card::{Card, Suit, Rank};
use rand::seq::SliceRandom;
use rand::Rng;
use std::fmt;

/// Player identifier (0-3)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

/// Field identifier (0-3, just board positions)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FieldId(pub u8);

/// Okus token identifier (A-D)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OkusId {
    A = 0,
    B = 1, 
    C = 2,
    D = 3,
}

impl fmt::Display for OkusId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OkusId::A => write!(f, "A"),
            OkusId::B => write!(f, "B"),
            OkusId::C => write!(f, "C"),
            OkusId::D => write!(f, "D"),
        }
    }
}

/// Okus position - either with a player or on the Illimat
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OkusPosition {
    WithPlayer(PlayerId),
    OnIllimat,
}

/// Season types that restrict actions
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Season {
    Winter = 0,  // No Harvesting
    Spring = 1,  // No Stockpiling  
    Summer = 2,  // No restrictions
    Autumn = 3,  // No Sowing (Stockpiling allowed)
}

/// Player type for future AI support
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    Computer,
}

/// Game configuration
#[derive(Clone, Debug, PartialEq)]
pub struct GameConfig {
    pub player_count: u8,
    pub player_types: [PlayerType; 4],  // Human/Computer for each slot
    pub use_stars_suit: bool,             // true = 65 cards, false = 52 cards
}

/// Game phase for proper state management
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Setup,      // Game creation/configuration
    Playing,    // Normal turn-by-turn play
    RoundEnd,   // Scoring and cleanup between rounds
    GameEnd,    // Victory achieved
}

/// Competitive scoring results for end of round
#[derive(Debug, Clone)]
pub struct RoundScoring {
    pub bumper_crop_winner: Option<PlayerId>,
    pub sunkissed_winner: Option<PlayerId>, 
    pub frostbit_players: Vec<PlayerId>,
    pub individual_scores: [i8; 4], // Fools + okus for each player
}

/// Core game actions
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Sow { field: FieldId, card: Card },
    Harvest { field: FieldId, card: Card, targets: Vec<Card> },
    Stockpile { field: FieldId, card: Card, targets: Vec<Card> },
}

/// Stockpile representation - a set of cards that sum to a harvestable value
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stockpile {
    pub value: u8,
    pub cards: Vec<Card>,
    pub created_turn: u16, // Turn number when this stockpile was created
}

/// Main game state
#[derive(Clone, Debug, PartialEq)]
pub struct IllimatState {
    // Game configuration
    pub config: GameConfig,
    pub phase: GamePhase,
    
    // Card locations
    pub field_cards: [Vec<Card>; 4],        // Loose cards per field
    pub player_hands: [Vec<Card>; 4],       // Player card lists  
    pub player_harvests: [Vec<Card>; 4],    // Harvested cards this round per player
    pub deck: Vec<Card>,                    // Remaining cards to draw
    
    // Field state
    pub field_stockpiles: [Vec<Stockpile>; 4], // Stockpile sets per field
    pub field_seasons: [Season; 4],            // Current season per field
    pub okus_positions: [OkusPosition; 4],     // Position of each okus token (A-D)
    
    // Game metadata
    pub current_player: PlayerId,
    pub dealer: PlayerId,
    pub total_scores: [u8; 4],                 // Cumulative scores across all rounds
    pub round_number: u8,                      // Current round (starts at 1)
    pub turn_number: u16,                      // Global turn counter for stockpile tracking
    pub illimat_orientation: u8,               // Which field (0-3) is currently Spring (others follow clockwise)
}

impl IllimatState {
    /// Get the de jure season of a field based on Illimat orientation (before Luminary effects)
    pub fn get_field_de_jure_season(&self, field: FieldId) -> Season {
        // Calculate field's season based on Illimat orientation
        // If Illimat points to field 0 for Spring, then:
        // Field 0 = Spring, Field 1 = Summer, Field 2 = Autumn, Field 3 = Winter
        let season_offset = (field.0 + 4 - self.illimat_orientation) % 4;
        match season_offset {
            0 => Season::Spring,
            1 => Season::Summer, 
            2 => Season::Autumn,
            3 => Season::Winter,
            _ => unreachable!(),
        }
    }
    
    /// Check if sowing is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_sow_in_field(&self, field: FieldId) -> bool {
        let base_season = self.get_field_de_jure_season(field);
        
        // Base rule: Autumn forbids sowing
        let base_can_sow = base_season != Season::Autumn;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - Forest Queen makes her field always Summer (allows sowing)
        // - Other Luminaries might modify capabilities
        
        base_can_sow
    }
    
    /// Check if harvesting is allowed in a field (considering Illimat + Luminary effects)  
    pub fn can_harvest_in_field(&self, field: FieldId) -> bool {
        let base_season = self.get_field_de_jure_season(field);
        
        // Base rule: Winter blocks harvesting
        let base_can_harvest = base_season != Season::Winter;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - The Maiden allows harvesting in Winter
        // - The Drought blocks harvesting in Summer
        
        base_can_harvest
    }
    
    /// Check if stockpiling is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_stockpile_in_field(&self, field: FieldId) -> bool {
        let base_season = self.get_field_de_jure_season(field);
        
        // Base rule: Spring forbids stockpiling  
        let base_can_stockpile = base_season != Season::Spring;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - Forest Queen makes her field always Summer (allows stockpiling)
        // - The Loom allows stockpiling ignoring season rules
        
        base_can_stockpile
    }
    
    /// Update all field seasons based on current Illimat orientation
    fn update_field_seasons(&mut self) {
        for i in 0..4 {
            self.field_seasons[i] = self.get_field_de_jure_season(FieldId(i as u8));
        }
    }
    
    /// Rotate the Illimat so that the specified field becomes the target season
    pub fn rotate_illimat_to_season(&mut self, field: FieldId, target_season: Season) {
        // Calculate what orientation would make this field have the target season
        let season_offset = match target_season {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Autumn => 2, 
            Season::Winter => 3,
        };
        // Illimat orientation = which field should be Spring
        // If we want field N to be season S, then Illimat should point to (N - S) mod 4
        self.illimat_orientation = (field.0 + 4 - season_offset) % 4;
        
        // Update all field seasons based on new orientation
        self.update_field_seasons();
    }
    
    /// Create a new game with specified configuration
    pub fn new_game(config: GameConfig) -> Self {
        // Validate player count
        if config.player_count < 2 || config.player_count > 4 {
            panic!("Player count must be 2-4");
        }
        
        let mut game_state = Self {
            config,
            phase: GamePhase::Setup,
            field_cards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_hands: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_harvests: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            deck: Vec::new(), // Will be created when dealing
            field_stockpiles: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            field_seasons: [Season::Spring; 4], // Will be set by update_field_seasons()
            okus_positions: [OkusPosition::OnIllimat; 4], // All okus start on Illimat
            current_player: PlayerId(0), // Will be set to dealer's left when dealing
            dealer: PlayerId(0), // Will be randomized when dealing
            total_scores: [0; 4],
            round_number: 1,
            turn_number: 0, // Increments with each action
            illimat_orientation: 0, // Illimat starts pointing to field 0 for Spring
        };
        
        // Initialize field seasons based on Illimat orientation
        game_state.update_field_seasons();
        game_state
    }
    
    /// Initialize and deal a new round
    pub fn deal_new_round<R: rand::Rng>(&mut self, rng: &mut R) {
        // Create appropriate deck
        self.deck = if self.config.use_stars_suit {
            Self::create_full_deck()
        } else {
            Self::create_reduced_deck()
        };
        
        // Shuffle deck
        Self::shuffle_deck(&mut self.deck, rng);
        
        // Choose dealer (random for first round, rotate afterwards)
        if self.round_number == 1 {
            self.dealer = PlayerId((rng.gen::<u8>() % self.config.player_count) as u8);
        } else {
            // Rotate dealer clockwise
            self.dealer = PlayerId((self.dealer.0 + 1) % self.config.player_count);
        }
        
        // Clear previous round state
        self.field_cards = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        self.player_hands = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        self.player_harvests = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        self.field_stockpiles = [Vec::new(), Vec::new(), Vec::new(), Vec::new()];
        self.field_seasons = [Season::Summer; 4];
        self.okus_positions = [OkusPosition::OnIllimat; 4];
        
        // Deal cards to fields (3 per field)
        for field_id in 0..4 {
            for _ in 0..3 {
                if let Some(card) = self.draw_card() {
                    self.field_cards[field_id].push(card);
                }
            }
        }
        
        // Deal cards to players
        for player_id in 0..self.config.player_count {
            let cards_to_deal = if player_id == (self.dealer.0 + 1) % self.config.player_count {
                // Player to dealer's left gets 3 cards and goes first
                3
            } else {
                // Other players get 4 cards
                4
            };
            
            let hand = self.draw_cards(cards_to_deal as usize);
            self.player_hands[player_id as usize] = hand;
        }
        
        // Set first player (dealer's left)
        self.current_player = PlayerId((self.dealer.0 + 1) % self.config.player_count);
        
        // Move to playing phase
        self.phase = GamePhase::Playing;
    }
    
    /// Check if this player slot is active (within player count)
    pub fn is_player_active(&self, player: PlayerId) -> bool {
        player.0 < self.config.player_count
    }
    
    /// Check if this player is human-controlled
    pub fn is_player_human(&self, player: PlayerId) -> bool {
        if !self.is_player_active(player) {
            return false;
        }
        self.config.player_types[player.0 as usize] == PlayerType::Human
    }
    
    /// Create a full 65-card deck (all 5 suits)
    pub fn create_full_deck() -> Vec<Card> {
        let mut deck = Vec::with_capacity(65);
        
        // Create all 5 suits × 13 ranks = 65 cards
        for suit_val in 0..5 {
            let suit = match suit_val {
                0 => Suit::Spring,
                1 => Suit::Summer, 
                2 => Suit::Autumn,
                3 => Suit::Winter,
                4 => Suit::Stars,
                _ => unreachable!(),
            };
            
            for rank_val in 0..13 {
                let rank = match rank_val {
                    0 => Rank::Fool,
                    1 => Rank::Two,
                    2 => Rank::Three,
                    3 => Rank::Four,
                    4 => Rank::Five,
                    5 => Rank::Six,
                    6 => Rank::Seven,
                    7 => Rank::Eight,
                    8 => Rank::Nine,
                    9 => Rank::Ten,
                    10 => Rank::Knight,
                    11 => Rank::Queen,
                    12 => Rank::King,
                    _ => unreachable!(),
                };
                
                deck.push(Card::new(rank, suit));
            }
        }
        
        deck
    }
    
    /// Create a 52-card deck (4 suits, no Stars) for 2-3 player games
    pub fn create_reduced_deck() -> Vec<Card> {
        let mut deck = Vec::with_capacity(52);
        
        // Create 4 suits × 13 ranks = 52 cards (excluding Stars)
        for suit_val in 0..4 {
            let suit = match suit_val {
                0 => Suit::Spring,
                1 => Suit::Summer, 
                2 => Suit::Autumn,
                3 => Suit::Winter,
                _ => unreachable!(),
            };
            
            for rank_val in 0..13 {
                let rank = match rank_val {
                    0 => Rank::Fool,
                    1 => Rank::Two,
                    2 => Rank::Three,
                    3 => Rank::Four,
                    4 => Rank::Five,
                    5 => Rank::Six,
                    6 => Rank::Seven,
                    7 => Rank::Eight,
                    8 => Rank::Nine,
                    9 => Rank::Ten,
                    10 => Rank::Knight,
                    11 => Rank::Queen,
                    12 => Rank::King,
                    _ => unreachable!(),
                };
                
                deck.push(Card::new(rank, suit));
            }
        }
        
        deck
    }
    
    /// Shuffle the deck using provided RNG
    pub fn shuffle_deck<R: Rng>(deck: &mut Vec<Card>, rng: &mut R) {
        deck.shuffle(rng);
    }
    
    /// Draw a card from the deck, returns None if deck is empty
    pub fn draw_card(&mut self) -> Option<Card> {
        self.deck.pop()
    }
    
    /// Draw multiple cards from the deck
    pub fn draw_cards(&mut self, count: usize) -> Vec<Card> {
        let mut cards = Vec::with_capacity(count);
        for _ in 0..count {
            if let Some(card) = self.draw_card() {
                cards.push(card);
            } else {
                break; // Stop if deck is empty
            }
        }
        cards
    }
    
    /// Create a new game state for testing
    #[cfg(test)]
    pub fn new_test_game() -> Self {
        let config = GameConfig {
            player_count: 4,
            player_types: [PlayerType::Human; 4],
            use_stars_suit: true,
        };
        
        Self {
            config,
            phase: GamePhase::Playing,
            field_cards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_hands: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_harvests: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            deck: Vec::new(),
            field_stockpiles: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            field_seasons: [Season::Spring, Season::Summer, Season::Autumn, Season::Winter],
            okus_positions: [OkusPosition::OnIllimat; 4], // All okus start on Illimat
            current_player: PlayerId(0),
            dealer: PlayerId(0),
            total_scores: [0; 4],
            round_number: 1,
            turn_number: 0,
            illimat_orientation: 0,
        }
    }
    
    /// Create a new game with a shuffled full deck
    #[cfg(test)]
    pub fn new_test_game_with_full_deck<R: Rng>(rng: &mut R) -> Self {
        let mut deck = Self::create_full_deck();
        Self::shuffle_deck(&mut deck, rng);
        
        let config = GameConfig {
            player_count: 4,
            player_types: [PlayerType::Human; 4],
            use_stars_suit: true,
        };
        
        Self {
            config,
            phase: GamePhase::Playing,
            field_cards: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_hands: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            player_harvests: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            deck,
            field_stockpiles: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
            field_seasons: [Season::Spring, Season::Summer, Season::Autumn, Season::Winter],
            okus_positions: [OkusPosition::OnIllimat; 4],
            current_player: PlayerId(0),
            dealer: PlayerId(0),
            total_scores: [0; 4],
            round_number: 1,
            turn_number: 0,
            illimat_orientation: 0,
        }
    }
    
    /// Add a card to a specific field (for testing)
    #[cfg(test)]
    pub fn add_card_to_field(&mut self, field: FieldId, card: Card) {
        self.field_cards[field.0 as usize].push(card);
    }
    
    /// Check if a field has a specific card (for testing)
    #[cfg(test)]
    pub fn field_has_card(&self, field: FieldId, card: Card) -> bool {
        self.field_cards[field.0 as usize].contains(&card)
    }
    
    /// Get the position of a specific okus token
    pub fn get_okus_position(&self, okus: OkusId) -> OkusPosition {
        self.okus_positions[okus as usize]
    }
    
    /// Move an okus token to a new position
    pub fn move_okus(&mut self, okus: OkusId, position: OkusPosition) {
        self.okus_positions[okus as usize] = position;
    }
    
    /// Count how many okus tokens are on the Illimat
    pub fn count_okus_on_illimat(&self) -> u8 {
        self.okus_positions.iter()
            .filter(|pos| matches!(pos, OkusPosition::OnIllimat))
            .count() as u8
    }
    
    /// Count how many okus tokens a player has
    pub fn count_player_okus(&self, player: PlayerId) -> u8 {
        self.okus_positions.iter()
            .filter(|pos| matches!(pos, OkusPosition::WithPlayer(p) if *p == player))
            .count() as u8
    }
    
    /// Get all available okus tokens on the Illimat
    pub fn get_available_okus(&self) -> Vec<OkusId> {
        [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
            .iter()
            .filter(|&&okus| self.get_okus_position(okus) == OkusPosition::OnIllimat)
            .cloned()
            .collect()
    }
    
    /// Collect an okus token after clearing a field
    /// Returns true if an okus was collected, false if none available
    pub fn collect_okus(&mut self, player: PlayerId, selected_okus: OkusId) -> Result<bool, String> {
        let available_okus = self.get_available_okus();
        
        // Check if any okus are available
        if available_okus.is_empty() {
            return Ok(false); // No okus available, not an error
        }
        
        // Validate the selected okus is available
        if !available_okus.contains(&selected_okus) {
            return Err(format!("Okus {:?} is not available on the Illimat", selected_okus));
        }
        
        // Collect the selected okus
        self.move_okus(selected_okus, OkusPosition::WithPlayer(player));
        Ok(true)
    }
    
    /// Calculate current round scoring for a player (without updating total scores)
    pub fn calculate_round_score(&self, player: PlayerId) -> i8 {
        let harvest = &self.player_harvests[player.0 as usize];
        let mut score = 0i8;
        
        // Count fools for immediate scoring
        let mut fool_count = 0;
        
        for card in harvest {
            if card.rank() == Rank::Fool {
                fool_count += 1;
            }
        }
        
        // Add okus tokens (+1 each)
        score += self.count_player_okus(player) as i8;
        
        // Add fools (+1 each)  
        score += fool_count;
        
        // TODO: Add luminary scoring when implemented
        
        // Note: Bumper Crop, Sunkissed, Frostbit are calculated at end of round
        // when comparing all players
        
        score
    }
    
    /// Check if a player would win if scores were calculated now (17+ points)
    pub fn would_player_win(&self, player: PlayerId) -> bool {
        // This is just the immediate scoring, not including competitive bonuses
        let current_total = self.total_scores[player.0 as usize] as i8;
        let round_score = self.calculate_round_score(player);
        (current_total + round_score) >= 17
    }
    
    /// Get summary of what a player has harvested this round
    pub fn get_harvest_summary(&self, player: PlayerId) -> (u8, u8, u8, u8) {
        let harvest = &self.player_harvests[player.0 as usize];
        let mut total_cards = 0;
        let mut summer_cards = 0;
        let mut winter_cards = 0;
        let mut fools = 0;
        
        for card in harvest {
            total_cards += 1;
            match card.suit() {
                Suit::Summer => summer_cards += 1,
                Suit::Winter => winter_cards += 1,
                _ => {}
            }
            if card.rank() == Rank::Fool {
                fools += 1;
            }
        }
        
        (total_cards, summer_cards, winter_cards, fools)
    }
    
    /// Check if a card is a face card (Fool, Knight, Queen, King)
    pub fn is_face_card(card: Card) -> bool {
        matches!(card.rank(), Rank::Fool | Rank::Knight | Rank::Queen | Rank::King)
    }
    
    
    /// Check if a player has a specific card in their hand
    pub fn player_has_card(&self, player: PlayerId, card: Card) -> bool {
        self.player_hands[player.0 as usize].contains(&card)
    }
    
    /// Validate a sow action
    pub fn can_sow(&self, player: PlayerId, field: FieldId, card: Card) -> Result<(), String> {
        // Check if it's the player's turn
        if self.current_player != player {
            return Err(format!("It's not player {}'s turn", player.0));
        }
        
        // Check if field exists
        if field.0 >= 4 {
            return Err("Invalid field ID".to_string());
        }
        
        // Check if player has the card
        if !self.player_has_card(player, card) {
            return Err("Player doesn't have this card".to_string());
        }
        
        // Check season restrictions
        if !self.can_sow_in_field(field) {
            return Err(format!("Cannot sow in {} field during Autumn", 
                match field.0 {
                    0 => "Spring",
                    1 => "Summer", 
                    2 => "Autumn",
                    3 => "Winter",
                    _ => "Unknown",
                }));
        }
        
        Ok(())
    }
    
    /// Apply a sow action to the game state
    pub fn apply_sow(&mut self, player: PlayerId, field: FieldId, card: Card) -> Result<(), String> {
        // Validate the action first
        self.can_sow(player, field, card)?;
        
        // Increment turn counter for tracking
        self.turn_number += 1;
        
        // Remove card from player's hand
        let hand = &mut self.player_hands[player.0 as usize];
        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        } else {
            return Err("Card not found in player's hand".to_string());
        }
        
        // Add card to field
        self.field_cards[field.0 as usize].push(card);
        
        // Check if this is a face card and update season
        if Self::is_face_card(card) {
            let target_season = match card.suit() {
                Suit::Spring => Season::Spring,
                Suit::Summer => Season::Summer,
                Suit::Autumn => Season::Autumn,
                Suit::Winter => Season::Winter,
                Suit::Stars => {
                    // For Stars face cards, player chooses season
                    // For now, default to Summer, but this should be a parameter
                    Season::Summer
                }
            };
            
            // Rotate the Illimat so the played field becomes the target season
            self.rotate_illimat_to_season(field, target_season);
        }
        
        // Draw a card back to maintain hand size
        if let Some(drawn_card) = self.draw_card() {
            let hand = &mut self.player_hands[player.0 as usize];
            hand.push(drawn_card);
        }
        
        Ok(())
    }
    
    
    /// Find all possible harvest combinations for a given target value in a field
    pub fn find_harvest_combinations(&self, field: FieldId, target_value: u8) -> Vec<Vec<Card>> {
        let field_cards = &self.field_cards[field.0 as usize];
        let mut combinations = Vec::new();
        
        // Generate all possible combinations of cards that sum to target_value
        self.find_combinations_recursive(field_cards, target_value, Vec::new(), &mut combinations, 0);
        
        combinations
    }
    
    /// Recursive helper to find all combinations that sum to target value
    fn find_combinations_recursive(
        &self,
        available_cards: &[Card],
        remaining_value: u8,
        current_combination: Vec<Card>,
        all_combinations: &mut Vec<Vec<Card>>,
        start_index: usize,
    ) {
        if remaining_value == 0 {
            if !current_combination.is_empty() {
                all_combinations.push(current_combination);
            }
            return;
        }
        
        for i in start_index..available_cards.len() {
            let card = available_cards[i];
            let card_value = card.value();
            
            // Handle Fool cards (can be 1 or 14)
            let possible_values = if card.rank() == Rank::Fool {
                vec![1, 14]
            } else {
                vec![card_value]
            };
            
            for &value in &possible_values {
                if value <= remaining_value {
                    let mut new_combination = current_combination.clone();
                    new_combination.push(card);
                    
                    self.find_combinations_recursive(
                        available_cards,
                        remaining_value - value,
                        new_combination,
                        all_combinations,
                        i + 1, // Avoid using same card twice
                    );
                }
            }
        }
    }
    
    /// Validate a harvest action
    pub fn can_harvest(&self, player: PlayerId, field: FieldId, card: Card, targets: &[Card]) -> Result<(), String> {
        // Check if it's the player's turn
        if self.current_player != player {
            return Err(format!("It's not player {}'s turn", player.0));
        }
        
        // Check if field exists
        if field.0 >= 4 {
            return Err("Invalid field ID".to_string());
        }
        
        // Check if player has the card
        if !self.player_has_card(player, card) {
            return Err("Player doesn't have this card".to_string());
        }
        
        // Check season restrictions
        if !self.can_harvest_in_field(field) {
            return Err(format!("Cannot harvest from {} field during Winter", 
                match field.0 {
                    0 => "Spring",
                    1 => "Summer", 
                    2 => "Autumn",
                    3 => "Winter",
                    _ => "Unknown",
                }));
        }
        
        // Check if targets are empty
        if targets.is_empty() {
            return Err("Must select at least one card to harvest".to_string());
        }
        
        // Check if all target cards are actually in the field
        let field_cards = &self.field_cards[field.0 as usize];
        for &target in targets {
            if !field_cards.contains(&target) {
                return Err(format!("Card {} is not in the field", target));
            }
        }
        
        // Check for duplicates in targets
        let mut seen_cards = std::collections::HashSet::new();
        for &target in targets {
            if !seen_cards.insert(target) {
                return Err("Cannot harvest the same card twice".to_string());
            }
        }
        
        // Calculate the sum of target cards and check if it matches played card value
        let target_sum = self.calculate_harvest_sum(targets, card.value());
        let played_card_values = if card.rank() == Rank::Fool {
            vec![1, 14]
        } else {
            vec![card.value()]
        };
        
        if !played_card_values.contains(&target_sum) {
            return Err(format!("Target cards sum to {}, but played card {} has values {:?}", 
                       target_sum, card, played_card_values));
        }
        
        // Check if any stockpiles are being targeted that were created this turn
        let field_stockpiles = &self.field_stockpiles[field.0 as usize];
        for stockpile in field_stockpiles {
            if stockpile.created_turn == self.turn_number {
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
    
    /// Calculate the optimal sum for harvest targets given the played card's possible values
    fn calculate_harvest_sum(&self, targets: &[Card], preferred_value: u8) -> u8 {
        // For each combination of Fool values, calculate sum and see if it matches preferred_value
        let fool_indices: Vec<usize> = targets.iter()
            .enumerate()
            .filter(|(_, card)| card.rank() == Rank::Fool)
            .map(|(i, _)| i)
            .collect();
        
        if fool_indices.is_empty() {
            // No fools, simple sum
            return targets.iter().map(|c| c.value()).sum();
        }
        
        // Try all combinations of fool values (1 or 14 each)
        let num_fools = fool_indices.len();
        for fool_combo in 0..(1 << num_fools) {
            let mut sum = 0;
            
            for (i, &card) in targets.iter().enumerate() {
                if let Some(fool_pos) = fool_indices.iter().position(|&fi| fi == i) {
                    // This is a fool, use the value from the combination
                    sum += if (fool_combo >> fool_pos) & 1 == 1 { 14 } else { 1 };
                } else {
                    sum += card.value();
                }
            }
            
            if sum == preferred_value {
                return sum;
            }
        }
        
        // If no combination matches preferred value, return the sum with fools as 1
        targets.iter().map(|c| if c.rank() == Rank::Fool { 1 } else { c.value() }).sum::<u8>()
    }
    
    /// Apply a harvest action to the game state
    /// Returns true if the field was cleared, false otherwise
    pub fn apply_harvest(&mut self, player: PlayerId, field: FieldId, card: Card, targets: Vec<Card>) -> Result<bool, String> {
        // Validate the action first
        self.can_harvest(player, field, card, &targets)?;
        
        // Increment turn counter for tracking
        self.turn_number += 1;
        
        // Remove played card from player's hand
        let hand = &mut self.player_hands[player.0 as usize];
        if let Some(pos) = hand.iter().position(|&c| c == card) {
            hand.remove(pos);
        } else {
            return Err("Card not found in player's hand".to_string());
        }
        
        // Remove target cards from field and add to player's harvest
        let field_cards = &mut self.field_cards[field.0 as usize];
        let harvest = &mut self.player_harvests[player.0 as usize];
        
        // Add the played card to harvest first
        harvest.push(card);
        
        // Remove targets from field and add to harvest
        for target in targets {
            if let Some(pos) = field_cards.iter().position(|&c| c == target) {
                field_cards.remove(pos);
                harvest.push(target);
            }
        }
        
        // Check if field was cleared
        let field_cleared = field_cards.is_empty();
        
        // Check if this is a face card and update season
        if Self::is_face_card(card) {
            let target_season = match card.suit() {
                Suit::Spring => Season::Spring,
                Suit::Summer => Season::Summer,
                Suit::Autumn => Season::Autumn,
                Suit::Winter => Season::Winter,
                Suit::Stars => {
                    // For Stars face cards, player chooses season
                    // For now, default to Summer, but this should be a parameter
                    Season::Summer
                }
            };
            
            // Rotate the Illimat so the played field becomes the target season
            self.rotate_illimat_to_season(field, target_season);
        }
        
        // Draw a card back to maintain hand size
        if let Some(drawn_card) = self.draw_card() {
            let hand = &mut self.player_hands[player.0 as usize];
            hand.push(drawn_card);
        }
        
        Ok(field_cleared)
    }
    
    
    /// Find all valid stockpile combinations for an active card and passive card value
    pub fn find_stockpile_combinations(&self, field: FieldId, active_card: Card, passive_value: u8) -> Vec<Vec<Card>> {
        let field_cards = &self.field_cards[field.0 as usize];
        let mut combinations = Vec::new();
        
        // The active card value + field cards must sum to passive_value
        let active_value = active_card.value();
        if active_value >= passive_value {
            // Active card alone is already >= passive value, no valid combination
            return combinations;
        }
        
        let needed_value = passive_value - active_value;
        
        // Find all combinations of field cards that sum to needed_value
        self.find_combinations_recursive(field_cards, needed_value, Vec::new(), &mut combinations, 0);
        
        combinations
    }
    
    /// Validate a stockpile action
    pub fn can_stockpile(&self, player: PlayerId, field: FieldId, active_card: Card, passive_card: Card, targets: &[Card]) -> Result<(), String> {
        // Check if it's the player's turn
        if self.current_player != player {
            return Err(format!("It's not player {}'s turn", player.0));
        }
        
        // Check if field exists
        if field.0 >= 4 {
            return Err("Invalid field ID".to_string());
        }
        
        // Check if player has both cards
        if !self.player_has_card(player, active_card) {
            return Err("Player doesn't have the active card".to_string());
        }
        if !self.player_has_card(player, passive_card) {
            return Err("Player doesn't have the passive card".to_string());
        }
        
        // Check if active and passive cards are different
        if active_card == passive_card {
            return Err("Active and passive cards must be different".to_string());
        }
        
        // Check season restrictions
        if !self.can_stockpile_in_field(field) {
            return Err(format!("Cannot stockpile in {} field during Spring", 
                match field.0 {
                    0 => "Spring",
                    1 => "Summer", 
                    2 => "Autumn",
                    3 => "Winter",
                    _ => "Unknown",
                }));
        }
        
        // Check stockpile value limit (max 14)
        let passive_value = passive_card.value();
        if passive_value > 14 {
            return Err("Stockpile value cannot exceed 14".to_string());
        }
        
        // Check if targets are empty
        if targets.is_empty() {
            return Err("Must select at least one field card for stockpile".to_string());
        }
        
        // Check if all target cards are actually in the field
        let field_cards = &self.field_cards[field.0 as usize];
        for &target in targets {
            if !field_cards.contains(&target) {
                return Err(format!("Card {} is not in the field", target));
            }
        }
        
        // Check for duplicates in targets
        let mut seen_cards = std::collections::HashSet::new();
        for &target in targets {
            if !seen_cards.insert(target) {
                return Err("Cannot use the same field card twice".to_string());
            }
        }
        
        // Calculate if active card + targets equals passive card value
        let active_value = active_card.value();
        let targets_sum = self.calculate_stockpile_sum(targets, passive_value - active_value);
        
        if active_value + targets_sum != passive_value {
            return Err(format!("Active card ({}) + field cards ({}) = {} does not equal passive card value ({})", 
                       active_value, targets_sum, active_value + targets_sum, passive_value));
        }
        
        Ok(())
    }
    
    /// Calculate the optimal sum for stockpile targets, handling Fool cards
    fn calculate_stockpile_sum(&self, targets: &[Card], target_sum: u8) -> u8 {
        // Similar to harvest sum calculation but for stockpiling
        let fool_indices: Vec<usize> = targets.iter()
            .enumerate()
            .filter(|(_, card)| card.rank() == Rank::Fool)
            .map(|(i, _)| i)
            .collect();
        
        if fool_indices.is_empty() {
            // No fools, simple sum
            return targets.iter().map(|c| c.value()).sum();
        }
        
        // Try all combinations of fool values (1 or 14 each)
        let num_fools = fool_indices.len();
        for fool_combo in 0..(1 << num_fools) {
            let mut sum = 0;
            
            for (i, &card) in targets.iter().enumerate() {
                if let Some(fool_pos) = fool_indices.iter().position(|&fi| fi == i) {
                    // This is a fool, use the value from the combination
                    sum += if (fool_combo >> fool_pos) & 1 == 1 { 14 } else { 1 };
                } else {
                    sum += card.value();
                }
            }
            
            if sum == target_sum {
                return sum;
            }
        }
        
        // If no combination matches target, return the sum with fools as 1
        targets.iter().map(|c| if c.rank() == Rank::Fool { 1 } else { c.value() }).sum::<u8>()
    }
    
    /// Check if the round is over (deck exhausted)
    pub fn is_round_over(&self) -> bool {
        self.deck.is_empty()
    }
    
    /// Check if any player has won the game (17+ points)
    pub fn check_victory(&self) -> Option<PlayerId> {
        for player_id in 0..self.config.player_count {
            let player = PlayerId(player_id);
            if self.total_scores[player_id as usize] >= 17 {
                return Some(player);
            }
        }
        None
    }
    
    
    /// Calculate end-of-round scoring
    pub fn calculate_round_scoring(&self) -> RoundScoring {
        let mut total_cards = [0u8; 4];
        let mut summer_cards = [0u8; 4];
        let mut winter_cards = [0u8; 4];
        let mut individual_scores = [0i8; 4];
        
        // Count harvested cards and calculate individual scores
        for player_id in 0..self.config.player_count {
            let player = PlayerId(player_id);
            let harvest = &self.player_harvests[player_id as usize];
            
            for card in harvest {
                total_cards[player_id as usize] += 1;
                match card.suit() {
                    Suit::Summer => summer_cards[player_id as usize] += 1,
                    Suit::Winter => winter_cards[player_id as usize] += 1,
                    _ => {}
                }
                // Count fools
                if card.rank() == Rank::Fool {
                    individual_scores[player_id as usize] += 1;
                }
            }
            
            // Count okus tokens
            individual_scores[player_id as usize] += self.count_player_okus(player) as i8;
        }
        
        // Find competitive winners
        let bumper_crop_winner = self.find_winner_by_count(&total_cards);
        let sunkissed_winner = self.find_winner_by_count(&summer_cards);
        let frostbit_players = self.find_losers_by_count(&winter_cards);
        
        RoundScoring {
            bumper_crop_winner,
            sunkissed_winner,
            frostbit_players,
            individual_scores,
        }
    }
    
    /// Find player with most of something, handling ties
    fn find_winner_by_count(&self, counts: &[u8; 4]) -> Option<PlayerId> {
        let max_count = counts[0..self.config.player_count as usize].iter().max()?;
        if *max_count == 0 {
            return None; // No one has any
        }
        
        // Check for ties
        let winners: Vec<u8> = (0..self.config.player_count)
            .filter(|&i| counts[i as usize] == *max_count)
            .collect();
        
        if winners.len() == 1 {
            Some(PlayerId(winners[0]))
        } else {
            // TODO: Handle ties with Luminary count (not implemented yet)
            // For now, ties mean no one wins
            None
        }
    }
    
    /// Find players with most Winter cards (for Frostbit penalty)
    fn find_losers_by_count(&self, winter_counts: &[u8; 4]) -> Vec<PlayerId> {
        let max_winter = winter_counts[0..self.config.player_count as usize].iter().max().unwrap_or(&0);
        if *max_winter == 0 {
            return Vec::new(); // No one has Winter cards
        }
        
        (0..self.config.player_count)
            .filter(|&i| winter_counts[i as usize] == *max_winter)
            .map(PlayerId)
            .collect()
    }
    
    /// Apply round scoring and advance to next round or end game
    pub fn end_round_and_score(&mut self) -> Result<Option<PlayerId>, String> {
        if !self.is_round_over() {
            return Err("Round is not over yet".to_string());
        }
        
        // Calculate scoring
        let scoring = self.calculate_round_scoring();
        
        // Apply competitive bonuses
        if let Some(winner) = scoring.bumper_crop_winner {
            self.total_scores[winner.0 as usize] += 4;
        }
        if let Some(winner) = scoring.sunkissed_winner {
            self.total_scores[winner.0 as usize] += 2;
        }
        // Frostbit penalty (but only if not tied)
        if scoring.frostbit_players.len() == 1 {
            let loser = scoring.frostbit_players[0];
            self.total_scores[loser.0 as usize] = 
                self.total_scores[loser.0 as usize].saturating_sub(2);
        }
        
        // Apply individual scores (fools + okus)
        for player_id in 0..self.config.player_count {
            let individual_score = scoring.individual_scores[player_id as usize];
            if individual_score > 0 {
                self.total_scores[player_id as usize] += individual_score as u8;
            }
        }
        
        // Return all okus to Illimat
        self.okus_positions = [OkusPosition::OnIllimat; 4];
        
        // Check for victory
        if let Some(winner) = self.check_victory() {
            self.phase = GamePhase::GameEnd;
            return Ok(Some(winner));
        }
        
        // Prepare for next round
        self.round_number += 1;
        self.phase = GamePhase::RoundEnd;
        
        Ok(None)
    }
    
    /// Start a new round (called after end_round_and_score)
    pub fn start_new_round<R: rand::Rng>(&mut self, rng: &mut R) -> Result<(), String> {
        if self.phase != GamePhase::RoundEnd {
            return Err("Can only start new round from RoundEnd phase".to_string());
        }
        
        // Deal new round with rotated dealer
        self.deal_new_round(rng);
        
        Ok(())
    }

    /// Apply a stockpile action to the game state
    pub fn apply_stockpile(&mut self, player: PlayerId, field: FieldId, active_card: Card, passive_card: Card, targets: Vec<Card>) -> Result<(), String> {
        // Validate the action first
        self.can_stockpile(player, field, active_card, passive_card, &targets)?;
        
        // Remove both cards from player's hand
        let hand = &mut self.player_hands[player.0 as usize];
        
        // Remove active card
        if let Some(pos) = hand.iter().position(|&c| c == active_card) {
            hand.remove(pos);
        } else {
            return Err("Active card not found in player's hand".to_string());
        }
        
        // Remove passive card (adjust index since we removed active card)
        if let Some(pos) = hand.iter().position(|&c| c == passive_card) {
            hand.remove(pos);
        } else {
            return Err("Passive card not found in player's hand".to_string());
        }
        
        // Remove target cards from field
        let field_cards = &mut self.field_cards[field.0 as usize];
        let mut stockpile_cards = vec![active_card];
        
        for target in targets {
            if let Some(pos) = field_cards.iter().position(|&c| c == target) {
                field_cards.remove(pos);
                stockpile_cards.push(target);
            }
        }
        
        // Increment turn counter for tracking
        self.turn_number += 1;
        
        // Create the stockpile
        let stockpile = Stockpile {
            value: passive_card.value(),
            cards: stockpile_cards,
            created_turn: self.turn_number,
        };
        
        // Add stockpile to field
        self.field_stockpiles[field.0 as usize].push(stockpile);
        
        // Check if this is a face card (active card) and update season
        if Self::is_face_card(active_card) {
            let target_season = match active_card.suit() {
                Suit::Spring => Season::Spring,
                Suit::Summer => Season::Summer,
                Suit::Autumn => Season::Autumn,
                Suit::Winter => Season::Winter,
                Suit::Stars => {
                    // For Stars face cards, player chooses season
                    // For now, default to Summer, but this should be a parameter
                    Season::Summer
                }
            };
            
            // Rotate the Illimat so the played field becomes the target season
            self.rotate_illimat_to_season(field, target_season);
        }
        
        // Draw cards back to maintain hand size (draw 2 since we played 2)
        for _ in 0..2 {
            if let Some(drawn_card) = self.draw_card() {
                let hand = &mut self.player_hands[player.0 as usize];
                hand.push(drawn_card);
            }
        }
        
        Ok(())
    }
}

impl fmt::Display for Season {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Season::Spring => write!(f, "Spring"),
            Season::Summer => write!(f, "Summer"),
            Season::Autumn => write!(f, "Autumn"),
            Season::Winter => write!(f, "Winter"),
        }
    }
}

impl fmt::Display for IllimatState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "=== ILLIMAT - Round {}, Player {} ===", self.round_number, self.current_player.0)?;
        writeln!(f)?;
        
        // Display each field
        let field_names = ["Spring", "Summer", "Autumn", "Winter"];
        for (i, field_name) in field_names.iter().enumerate() {
            let season = &self.field_seasons[i];
            write!(f, "{} Field ({}): ", field_name, season)?;
            
            // Show loose cards
            if self.field_cards[i].is_empty() {
                write!(f, "empty")?;
            } else {
                for (j, card) in self.field_cards[i].iter().enumerate() {
                    if j > 0 { write!(f, " ")?; }
                    write!(f, "{}", card)?;
                }
            }
            
            // Show stockpiles if any
            if !self.field_stockpiles[i].is_empty() {
                write!(f, " (Stockpiles: ")?;
                for (j, stockpile) in self.field_stockpiles[i].iter().enumerate() {
                    if j > 0 { write!(f, ", ")?; }
                    write!(f, "[{}: ", stockpile.value)?;
                    for (k, card) in stockpile.cards.iter().enumerate() {
                        if k > 0 { write!(f, ", ")?; }
                        write!(f, "{}", card)?;
                    }
                    write!(f, "]")?;
                }
                write!(f, ")")?;
            }
            writeln!(f)?;
        }
        
        writeln!(f)?;
        
        // Display current player's hand
        write!(f, "YOUR HAND: ")?;
        let current_hand = &self.player_hands[self.current_player.0 as usize];
        if current_hand.is_empty() {
            write!(f, "empty")?;
        } else {
            for (i, card) in current_hand.iter().enumerate() {
                if i > 0 { write!(f, " ")?; }
                write!(f, "{}", card)?;
            }
        }
        writeln!(f)?;
        
        writeln!(f)?;
        
        // Display round harvests and scoring
        writeln!(f, "ROUND HARVEST:")?;
        for player_id in 0..self.config.player_count {
            let player = PlayerId(player_id);
            let (total, summer, winter, fools) = self.get_harvest_summary(player);
            let round_score = self.calculate_round_score(player);
            let okus_count = self.count_player_okus(player);
            
            write!(f, "P{}: {} cards", player_id, total)?;
            if summer > 0 { write!(f, ", {}☀", summer)?; }
            if winter > 0 { write!(f, ", {}❄", winter)?; }
            if fools > 0 { write!(f, ", {}F", fools)?; }
            if okus_count > 0 { write!(f, ", {}⚬", okus_count)?; }
            writeln!(f, " = +{} pts", round_score)?;
        }
        
        writeln!(f)?;
        
        // Display okus positions
        write!(f, "OKUS: ")?;
        let okus_on_illimat: Vec<OkusId> = [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
            .iter()
            .filter(|&okus| self.get_okus_position(*okus) == OkusPosition::OnIllimat)
            .cloned()
            .collect();
        
        if okus_on_illimat.is_empty() {
            write!(f, "Illimat: none")?;
        } else {
            write!(f, "Illimat: ")?;
            for (i, okus) in okus_on_illimat.iter().enumerate() {
                if i > 0 { write!(f, ", ")?; }
                write!(f, "{}", okus)?;
            }
        }
        
        // Show player okus
        for player_id in 0..self.config.player_count {
            let player = PlayerId(player_id);
            let player_okus: Vec<OkusId> = [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
                .iter()
                .filter(|&okus| self.get_okus_position(*okus) == OkusPosition::WithPlayer(player))
                .cloned()
                .collect();
            
            if !player_okus.is_empty() {
                write!(f, " | P{}: ", player_id)?;
                for (i, okus) in player_okus.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", okus)?;
                }
            }
        }
        writeln!(f)?;
        
        writeln!(f)?;
        
        // Display total scores
        write!(f, "TOTAL SCORES: ")?;
        for i in 0..self.config.player_count {
            if i > 0 { write!(f, " | ")?; }
            let player = PlayerId(i);
            let current_total = self.total_scores[i as usize];
            let round_score = self.calculate_round_score(player);
            let projected = (current_total as i8 + round_score) as u8;
            write!(f, "P{}: {} (+{} = {})", i, current_total, round_score, projected)?;
            if self.would_player_win(player) {
                write!(f, " WIN!")?;
            }
        }
        writeln!(f)?;
        
        writeln!(f, "Cards in deck: {}", self.deck.len())?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    use std::collections::HashSet;

    #[test]
    fn test_new_test_game_initializes_correctly() {
        let state = IllimatState::new_test_game();
        
        // All fields should be empty
        for field in &state.field_cards {
            assert!(field.is_empty());
        }
        
        // All hands should be empty
        for hand in &state.player_hands {
            assert!(hand.is_empty());
        }
        
        // All okus should be on Illimat initially
        assert_eq!(state.count_okus_on_illimat(), 4);
        for okus in [OkusId::A, OkusId::B, OkusId::C, OkusId::D] {
            assert_eq!(state.get_okus_position(okus), OkusPosition::OnIllimat);
        }
        
        // Should start at Summer season
        for season in &state.field_seasons {
            assert_eq!(*season, Season::Summer);
        }
    }
    
    #[test] 
    fn test_add_and_check_field_cards() {
        let mut state = IllimatState::new_test_game();
        let card = Card::new(Rank::Five, Suit::Spring);
        let field = FieldId(0);
        
        // Initially no card
        assert!(!state.field_has_card(field, card));
        
        // Add card
        state.add_card_to_field(field, card);
        
        // Now should have card
        assert!(state.field_has_card(field, card));
        assert_eq!(state.field_cards[0].len(), 1);
    }
    
    #[test]
    fn test_create_full_deck() {
        let deck = IllimatState::create_full_deck();
        
        // Should have exactly 65 cards
        assert_eq!(deck.len(), 65);
        
        // Should have no duplicates
        let unique_cards: HashSet<Card> = deck.iter().cloned().collect();
        assert_eq!(unique_cards.len(), 65);
        
        // Should have all 5 suits
        let suits: HashSet<Suit> = deck.iter().map(|c| c.suit()).collect();
        assert_eq!(suits.len(), 5);
        assert!(suits.contains(&Suit::Spring));
        assert!(suits.contains(&Suit::Summer));
        assert!(suits.contains(&Suit::Autumn));
        assert!(suits.contains(&Suit::Winter));
        assert!(suits.contains(&Suit::Stars));
        
        // Should have 13 cards of each suit
        for suit in [Suit::Spring, Suit::Summer, Suit::Autumn, Suit::Winter, Suit::Stars] {
            let count = deck.iter().filter(|c| c.suit() == suit).count();
            assert_eq!(count, 13, "Suit {:?} should have 13 cards", suit);
        }
    }
    
    #[test]
    fn test_create_reduced_deck() {
        let deck = IllimatState::create_reduced_deck();
        
        // Should have exactly 52 cards
        assert_eq!(deck.len(), 52);
        
        // Should have no duplicates
        let unique_cards: HashSet<Card> = deck.iter().cloned().collect();
        assert_eq!(unique_cards.len(), 52);
        
        // Should have only 4 suits (no Stars)
        let suits: HashSet<Suit> = deck.iter().map(|c| c.suit()).collect();
        assert_eq!(suits.len(), 4);
        assert!(suits.contains(&Suit::Spring));
        assert!(suits.contains(&Suit::Summer));
        assert!(suits.contains(&Suit::Autumn));
        assert!(suits.contains(&Suit::Winter));
        assert!(!suits.contains(&Suit::Stars));
        
        // Should have 13 cards of each non-Stars suit
        for suit in [Suit::Spring, Suit::Summer, Suit::Autumn, Suit::Winter] {
            let count = deck.iter().filter(|c| c.suit() == suit).count();
            assert_eq!(count, 13, "Suit {:?} should have 13 cards", suit);
        }
    }
    
    #[test]
    fn test_shuffle_deck() {
        let mut rng = StdRng::seed_from_u64(42);
        let original_deck = IllimatState::create_full_deck();
        let mut shuffled_deck = original_deck.clone();
        
        IllimatState::shuffle_deck(&mut shuffled_deck, &mut rng);
        
        // Should still have same cards
        assert_eq!(shuffled_deck.len(), original_deck.len());
        let original_set: HashSet<Card> = original_deck.iter().cloned().collect();
        let shuffled_set: HashSet<Card> = shuffled_deck.iter().cloned().collect();
        assert_eq!(original_set, shuffled_set);
        
        // Should be in different order (with high probability)
        // Note: This could theoretically fail with astronomically low probability
        assert_ne!(original_deck, shuffled_deck);
    }
    
    #[test]
    fn test_draw_card_from_empty_deck() {
        let mut state = IllimatState::new_test_game();
        
        // Empty deck should return None
        assert_eq!(state.draw_card(), None);
        assert_eq!(state.deck.len(), 0);
    }
    
    #[test]
    fn test_draw_cards_from_deck() {
        let mut rng = StdRng::seed_from_u64(42);
        let mut state = IllimatState::new_test_game_with_full_deck(&mut rng);
        
        // Should start with 65 cards
        assert_eq!(state.deck.len(), 65);
        
        // Draw a single card
        let card = state.draw_card();
        assert!(card.is_some());
        assert_eq!(state.deck.len(), 64);
        
        // Draw multiple cards
        let cards = state.draw_cards(5);
        assert_eq!(cards.len(), 5);
        assert_eq!(state.deck.len(), 59);
        
        // All drawn cards should be unique
        let mut all_drawn = cards;
        all_drawn.push(card.unwrap());
        let unique_drawn: HashSet<Card> = all_drawn.iter().cloned().collect();
        assert_eq!(unique_drawn.len(), 6);
    }
    
    #[test]
    fn test_draw_more_cards_than_available() {
        let mut state = IllimatState::new_test_game();
        // Add only 3 cards to deck
        state.deck = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Three, Suit::Summer),
            Card::new(Rank::Four, Suit::Autumn),
        ];
        
        // Try to draw 5 cards, should only get 3
        let cards = state.draw_cards(5);
        assert_eq!(cards.len(), 3);
        assert_eq!(state.deck.len(), 0);
        
        // All cards should be unique
        let unique_cards: HashSet<Card> = cards.iter().cloned().collect();
        assert_eq!(unique_cards.len(), 3);
    }
    
    #[test]
    fn test_draw_entire_deck_no_collisions() {
        let mut rng = StdRng::seed_from_u64(123);
        let mut state = IllimatState::new_test_game_with_full_deck(&mut rng);
        
        let mut drawn_cards = Vec::new();
        
        // Draw all cards one by one
        while let Some(card) = state.draw_card() {
            drawn_cards.push(card);
        }
        
        // Should have drawn exactly 65 cards
        assert_eq!(drawn_cards.len(), 65);
        assert_eq!(state.deck.len(), 0);
        
        // All cards should be unique
        let unique_cards: HashSet<Card> = drawn_cards.iter().cloned().collect();
        assert_eq!(unique_cards.len(), 65);
        
        // Should contain all expected cards
        let expected_deck = IllimatState::create_full_deck();
        let expected_set: HashSet<Card> = expected_deck.iter().cloned().collect();
        assert_eq!(unique_cards, expected_set);
    }
    
    #[test]
    fn test_deal_four_hands_randomness() {
        let mut rng = StdRng::seed_from_u64(456);
        let mut state = IllimatState::new_test_game_with_full_deck(&mut rng);
        
        // Draw 4 hands of 4 cards each
        let mut hands = Vec::new();
        for _ in 0..4 {
            let hand = state.draw_cards(4);
            assert_eq!(hand.len(), 4);
            hands.push(hand);
        }
        
        // Should have drawn 16 cards total
        assert_eq!(state.deck.len(), 65 - 16);
        
        // All cards across all hands should be unique
        let mut all_cards = Vec::new();
        for hand in &hands {
            all_cards.extend(hand.iter());
        }
        
        let unique_cards: HashSet<Card> = all_cards.iter().cloned().collect();
        assert_eq!(unique_cards.len(), 16);
        
        // Each individual hand should have unique cards
        for (i, hand) in hands.iter().enumerate() {
            let hand_set: HashSet<Card> = hand.iter().cloned().collect();
            assert_eq!(hand_set.len(), 4, "Hand {} should have 4 unique cards", i);
        }
    }
    
    #[test]
    fn test_okus_management() {
        let mut state = IllimatState::new_test_game();
        
        // Initially all okus on Illimat
        assert_eq!(state.count_okus_on_illimat(), 4);
        assert_eq!(state.count_player_okus(PlayerId(0)), 0);
        assert_eq!(state.count_player_okus(PlayerId(1)), 0);
        
        // Move okus A to player 0
        state.move_okus(OkusId::A, OkusPosition::WithPlayer(PlayerId(0)));
        assert_eq!(state.get_okus_position(OkusId::A), OkusPosition::WithPlayer(PlayerId(0)));
        assert_eq!(state.count_okus_on_illimat(), 3);
        assert_eq!(state.count_player_okus(PlayerId(0)), 1);
        
        // Move okus B to player 1
        state.move_okus(OkusId::B, OkusPosition::WithPlayer(PlayerId(1)));
        assert_eq!(state.get_okus_position(OkusId::B), OkusPosition::WithPlayer(PlayerId(1)));
        assert_eq!(state.count_okus_on_illimat(), 2);
        assert_eq!(state.count_player_okus(PlayerId(1)), 1);
        
        // Move okus A back to Illimat
        state.move_okus(OkusId::A, OkusPosition::OnIllimat);
        assert_eq!(state.get_okus_position(OkusId::A), OkusPosition::OnIllimat);
        assert_eq!(state.count_okus_on_illimat(), 3);
        assert_eq!(state.count_player_okus(PlayerId(0)), 0);
        
        // Other okus should be unchanged
        assert_eq!(state.get_okus_position(OkusId::C), OkusPosition::OnIllimat);
        assert_eq!(state.get_okus_position(OkusId::D), OkusPosition::OnIllimat);
    }
    
    #[test]
    fn test_same_turn_stockpile_harvesting_restriction() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        
        // Set up stockpile creation scenario
        state.player_hands[0] = vec![
            Card::new(Rank::Five, Suit::Summer), // Active card for stockpile
            Card::new(Rank::Eight, Suit::Winter), // Passive card (value 8)
            Card::new(Rank::Eight, Suit::Spring), // To harvest the stockpile
        ];
        state.field_cards[0] = vec![
            Card::new(Rank::Three, Suit::Autumn), // Will be stockpiled with active card (5+3=8)
        ];
        
        // Create a stockpile (this will increment turn_number to 1)
        let active_card = Card::new(Rank::Five, Suit::Summer);
        let passive_card = Card::new(Rank::Eight, Suit::Winter);
        let targets = vec![Card::new(Rank::Three, Suit::Autumn)];
        state.apply_stockpile(player, field, active_card, passive_card, targets).unwrap();
        
        // Verify stockpile was created with current turn number
        assert_eq!(state.field_stockpiles[0].len(), 1);
        assert_eq!(state.field_stockpiles[0][0].created_turn, 1);
        assert_eq!(state.turn_number, 1);
        
        // Try to harvest cards from the just-created stockpile (should fail)
        let harvest_card = Card::new(Rank::Eight, Suit::Spring);
        let stockpile_targets = vec![Card::new(Rank::Five, Suit::Summer)]; // From the stockpile
        
        let result = state.can_harvest(player, field, harvest_card, &stockpile_targets);
        assert!(result.is_err());
        println!("Actual error: {}", result.as_ref().unwrap_err());
        assert!(result.unwrap_err().contains("Cannot harvest cards from a stockpile created this turn"));
        
        // Simulate advancing to next turn by incrementing turn counter
        state.turn_number += 1;
        
        // Now the same harvest should be allowed (if the target is valid)
        // Note: This would need the full stockpile harvesting logic to work properly
        // For now, just verify the same-turn restriction is lifted
        let result2 = state.can_harvest(player, field, harvest_card, &stockpile_targets);
        // This will fail for other reasons (card not in loose field), but not the same-turn restriction
        assert!(result2.is_err());
        assert!(!result2.unwrap_err().contains("Cannot harvest cards from a stockpile created this turn"));
    }
    
    #[test]
    fn test_okus_collection_after_field_clearing() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        
        // Set up a simple harvest scenario that will clear the field
        state.field_cards[0] = vec![Card::new(Rank::Five, Suit::Spring)];
        state.player_hands[0] = vec![Card::new(Rank::Five, Suit::Summer)]; // Matches field card
        
        // Perform harvest that clears the field
        let played_card = Card::new(Rank::Five, Suit::Summer);
        let targets = vec![Card::new(Rank::Five, Suit::Spring)];
        let field_cleared = state.apply_harvest(player, field, played_card, targets).unwrap();
        
        // Verify field was cleared
        assert!(field_cleared);
        assert!(state.field_cards[0].is_empty());
        
        // Check available okus
        let available_okus = state.get_available_okus();
        assert_eq!(available_okus.len(), 4); // All okus should be available
        assert!(available_okus.contains(&OkusId::A));
        assert!(available_okus.contains(&OkusId::B));
        
        // Collect okus A
        let collected = state.collect_okus(player, OkusId::A).unwrap();
        assert!(collected);
        
        // Verify okus collection
        assert_eq!(state.get_okus_position(OkusId::A), OkusPosition::WithPlayer(player));
        assert_eq!(state.count_player_okus(player), 1);
        assert_eq!(state.count_okus_on_illimat(), 3);
        
        // Try to collect same okus again (should fail)
        let result = state.collect_okus(player, OkusId::A);
        assert!(result.is_err());
        
        // Collect another okus
        let collected2 = state.collect_okus(player, OkusId::C).unwrap();
        assert!(collected2);
        assert_eq!(state.count_player_okus(player), 2);
    }
    
    #[test]
    fn test_sow_basic_functionality() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let card = Card::new(Rank::Five, Suit::Spring);
        
        // Add card to player's hand and some cards to deck
        state.player_hands[0].push(card);
        state.deck.push(Card::new(Rank::Two, Suit::Summer));
        
        // Initial state checks
        assert!(state.player_has_card(player, card));
        assert_eq!(state.field_cards[0].len(), 0);
        assert_eq!(state.player_hands[0].len(), 1);
        
        // Apply sow action
        let result = state.apply_sow(player, field, card);
        assert!(result.is_ok());
        
        // Check final state
        assert!(!state.player_has_card(player, card)); // Card removed from hand
        assert_eq!(state.field_cards[0].len(), 1); // Card added to field
        assert!(state.field_cards[0].contains(&card));
        assert_eq!(state.player_hands[0].len(), 1); // Hand refilled from deck
        assert!(state.player_hands[0].contains(&Card::new(Rank::Two, Suit::Summer)));
    }
    
    #[test]
    fn test_sow_season_restrictions() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let card = Card::new(Rank::Five, Suit::Spring);
        
        // Set field to Autumn (no sowing allowed)
        state.field_seasons[0] = Season::Autumn;
        state.player_hands[0].push(card);
        
        // Should fail due to season restriction
        let result = state.can_sow(player, field, card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Autumn"));
        
        // Change to Summer (sowing allowed)
        state.field_seasons[0] = Season::Summer;
        let result = state.can_sow(player, field, card);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_sow_face_card_changes_season() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let face_card = Card::new(Rank::King, Suit::Winter);
        
        // Add face card to hand and deck card for refill
        state.player_hands[0].push(face_card);
        state.deck.push(Card::new(Rank::Two, Suit::Summer));
        
        // Initial seasons are all Summer
        assert_eq!(state.field_seasons, [Season::Summer; 4]);
        
        // Sow the Winter King
        let result = state.apply_sow(player, field, face_card);
        assert!(result.is_ok());
        
        // All fields should now be Winter
        assert_eq!(state.field_seasons, [Season::Winter; 4]);
    }
    
    #[test]
    fn test_sow_validation_errors() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let wrong_player = PlayerId(1);
        let field = FieldId(0);
        let invalid_field = FieldId(4);
        let card = Card::new(Rank::Five, Suit::Spring);
        let missing_card = Card::new(Rank::Ten, Suit::Autumn);
        
        state.player_hands[0].push(card);
        
        // Wrong player's turn
        let result = state.can_sow(wrong_player, field, card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not player 1's turn"));
        
        // Invalid field
        let result = state.can_sow(player, invalid_field, card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid field"));
        
        // Card not in hand
        let result = state.can_sow(player, field, missing_card);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("doesn't have this card"));
    }
    
    #[test]
    fn test_sow_hand_refill_from_empty_deck() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let card = Card::new(Rank::Five, Suit::Spring);
        
        // Add card to hand but leave deck empty
        state.player_hands[0].push(card);
        assert_eq!(state.deck.len(), 0);
        
        // Apply sow action
        let result = state.apply_sow(player, field, card);
        assert!(result.is_ok());
        
        // Card should be sown but hand not refilled (deck empty)
        assert!(!state.player_has_card(player, card));
        assert_eq!(state.field_cards[0].len(), 1);
        assert_eq!(state.player_hands[0].len(), 0); // No refill from empty deck
    }
    
    #[test]
    fn test_is_face_card() {
        assert!(IllimatState::is_face_card(Card::new(Rank::Fool, Suit::Spring)));
        assert!(IllimatState::is_face_card(Card::new(Rank::Knight, Suit::Summer)));
        assert!(IllimatState::is_face_card(Card::new(Rank::Queen, Suit::Autumn)));
        assert!(IllimatState::is_face_card(Card::new(Rank::King, Suit::Winter)));
        
        assert!(!IllimatState::is_face_card(Card::new(Rank::Two, Suit::Spring)));
        assert!(!IllimatState::is_face_card(Card::new(Rank::Ten, Suit::Summer)));
    }
    
    #[test]
    fn test_harvest_basic_functionality() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let played_card = Card::new(Rank::Five, Suit::Spring);
        let target1 = Card::new(Rank::Two, Suit::Summer);
        let target2 = Card::new(Rank::Three, Suit::Autumn);
        
        // Set up state
        state.player_hands[0].push(played_card);
        state.field_cards[0].push(target1);
        state.field_cards[0].push(target2);
        state.deck.push(Card::new(Rank::Six, Suit::Winter)); // For refill
        
        // Apply harvest (2 + 3 = 5)
        let result = state.apply_harvest(player, field, played_card, vec![target1, target2]);
        assert!(result.is_ok());
        
        // Check final state
        assert!(!state.player_has_card(player, played_card)); // Played card removed from hand
        assert_eq!(state.field_cards[0].len(), 0); // Targets removed from field
        assert_eq!(state.player_harvests[0].len(), 3); // Played card + 2 targets in harvest
        assert!(state.player_harvests[0].contains(&played_card));
        assert!(state.player_harvests[0].contains(&target1));
        assert!(state.player_harvests[0].contains(&target2));
        assert_eq!(state.player_hands[0].len(), 1); // Hand refilled from deck
    }
    
    #[test]
    fn test_harvest_season_restrictions() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let played_card = Card::new(Rank::Five, Suit::Spring);
        let target = Card::new(Rank::Five, Suit::Summer);
        
        // Set field to Winter (no harvesting allowed)
        state.field_seasons[0] = Season::Winter;
        state.player_hands[0].push(played_card);
        state.field_cards[0].push(target);
        
        // Should fail due to season restriction
        let result = state.can_harvest(player, field, played_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Winter"));
        
        // Change to Summer (harvesting allowed)
        state.field_seasons[0] = Season::Summer;
        let result = state.can_harvest(player, field, played_card, &[target]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_harvest_fool_cards() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let fool_played = Card::new(Rank::Fool, Suit::Spring);
        let fool_target = Card::new(Rank::Fool, Suit::Summer);
        let king_target = Card::new(Rank::King, Suit::Autumn); // Value 13
        
        // Set up state: Fool (1) can harvest Fool (14) + King (13) = 14 + 13 = 27? No.
        // Let's try: Fool (14) can harvest Fool (1) + King (13) = 1 + 13 = 14
        state.player_hands[0].push(fool_played);
        state.field_cards[0].push(fool_target);
        state.field_cards[0].push(king_target);
        state.deck.push(Card::new(Rank::Six, Suit::Winter)); // For refill
        
        // Apply harvest: Fool (14) harvests Fool (1) + King (13) = 14
        let result = state.apply_harvest(player, field, fool_played, vec![fool_target, king_target]);
        assert!(result.is_ok());
        
        // Check that cards were harvested
        assert_eq!(state.player_harvests[0].len(), 3); // Played fool + 2 targets
        assert_eq!(state.field_cards[0].len(), 0); // Field cleared
    }
    
    #[test]
    fn test_harvest_validation_errors() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let wrong_player = PlayerId(1);
        let field = FieldId(0);
        let invalid_field = FieldId(4);
        let played_card = Card::new(Rank::Five, Suit::Spring);
        let target = Card::new(Rank::Three, Suit::Summer);
        let missing_target = Card::new(Rank::Seven, Suit::Autumn);
        
        state.player_hands[0].push(played_card);
        state.field_cards[0].push(target);
        
        // Wrong player's turn
        let result = state.can_harvest(wrong_player, field, played_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not player 1's turn"));
        
        // Invalid field
        let result = state.can_harvest(player, invalid_field, played_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid field"));
        
        // Card not in field
        let result = state.can_harvest(player, field, played_card, &[missing_target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the field"));
        
        // Empty targets
        let result = state.can_harvest(player, field, played_card, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least one card"));
        
        // Value mismatch (3 != 5)
        let result = state.can_harvest(player, field, played_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sum to 3"));
    }
    
    #[test]
    fn test_find_harvest_combinations() {
        let mut state = IllimatState::new_test_game();
        let field = FieldId(0);
        
        // Set up field with various cards
        state.field_cards[0] = vec![
            Card::new(Rank::Two, Suit::Spring),   // 2
            Card::new(Rank::Three, Suit::Summer), // 3  
            Card::new(Rank::Five, Suit::Autumn),  // 5
            Card::new(Rank::Fool, Suit::Winter),  // 1 or 14
        ];
        
        // Find combinations for value 5
        let combinations = state.find_harvest_combinations(field, 5);
        
        // Should find: [5], [2+3], [Fool(1)+2+Two] - wait, that's wrong
        // Should find: [5], [2+3], [Fool(1)+Two+Two] - no duplicates
        // Should find: [5], [2+3]  
        
        // Let's check what we actually get
        assert!(!combinations.is_empty());
        
        // Should include the single 5 card
        assert!(combinations.iter().any(|combo| 
            combo.len() == 1 && combo[0] == Card::new(Rank::Five, Suit::Autumn)
        ));
        
        // Should include 2+3 combination  
        assert!(combinations.iter().any(|combo| 
            combo.len() == 2 && 
            combo.contains(&Card::new(Rank::Two, Suit::Spring)) &&
            combo.contains(&Card::new(Rank::Three, Suit::Summer))
        ));
    }
    
    #[test]
    fn test_harvest_face_card_changes_season() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let face_card = Card::new(Rank::Queen, Suit::Winter);
        let target = Card::new(Rank::Queen, Suit::Summer); // Same value (12)
        
        // Set up state
        state.player_hands[0].push(face_card);
        state.field_cards[0].push(target);
        state.deck.push(Card::new(Rank::Two, Suit::Summer));
        
        // Initial seasons are all Summer
        assert_eq!(state.field_seasons, [Season::Summer; 4]);
        
        // Harvest with Winter Queen
        let result = state.apply_harvest(player, field, face_card, vec![target]);
        assert!(result.is_ok());
        
        // All fields should now be Winter
        assert_eq!(state.field_seasons, [Season::Winter; 4]);
    }
    
    #[test]
    fn test_stockpile_basic_functionality() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let active_card = Card::new(Rank::Three, Suit::Spring);   // Value 3
        let passive_card = Card::new(Rank::Seven, Suit::Summer);  // Value 7
        let target = Card::new(Rank::Four, Suit::Autumn);        // Value 4
        
        // Set up state: 3 (active) + 4 (field) = 7 (passive)
        state.player_hands[0].push(active_card);
        state.player_hands[0].push(passive_card);
        state.field_cards[0].push(target);
        state.deck.push(Card::new(Rank::Two, Suit::Winter));
        state.deck.push(Card::new(Rank::Five, Suit::Stars));
        
        // Apply stockpile
        let result = state.apply_stockpile(player, field, active_card, passive_card, vec![target]);
        assert!(result.is_ok());
        
        // Check final state
        assert!(!state.player_has_card(player, active_card));  // Active card removed
        assert!(!state.player_has_card(player, passive_card)); // Passive card removed
        assert_eq!(state.field_cards[0].len(), 0);             // Target removed from field
        assert_eq!(state.field_stockpiles[0].len(), 1);        // Stockpile created
        assert_eq!(state.field_stockpiles[0][0].value, 7);     // Correct value
        assert_eq!(state.field_stockpiles[0][0].cards.len(), 2); // Active card + target
        assert_eq!(state.player_hands[0].len(), 2);            // Hand refilled (drew 2 cards)
    }
    
    #[test]
    fn test_stockpile_season_restrictions() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let active_card = Card::new(Rank::Three, Suit::Spring);
        let passive_card = Card::new(Rank::Seven, Suit::Summer);
        let target = Card::new(Rank::Four, Suit::Autumn);
        
        // Set field to Spring (no stockpiling allowed)
        state.field_seasons[0] = Season::Spring;
        state.player_hands[0].push(active_card);
        state.player_hands[0].push(passive_card);
        state.field_cards[0].push(target);
        
        // Should fail due to season restriction
        let result = state.can_stockpile(player, field, active_card, passive_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Spring"));
        
        // Change to Summer (stockpiling allowed)
        state.field_seasons[0] = Season::Summer;
        let result = state.can_stockpile(player, field, active_card, passive_card, &[target]);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_stockpile_validation_errors() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let wrong_player = PlayerId(1);
        let field = FieldId(0);
        let active_card = Card::new(Rank::Three, Suit::Spring);
        let passive_card = Card::new(Rank::Seven, Suit::Summer);
        let same_card = Card::new(Rank::Five, Suit::Autumn);
        let target = Card::new(Rank::Four, Suit::Winter);
        let missing_card = Card::new(Rank::Eight, Suit::Stars);
        let missing_target = Card::new(Rank::Nine, Suit::Spring);
        
        state.player_hands[0].push(active_card);
        state.player_hands[0].push(passive_card);
        state.player_hands[0].push(same_card);
        state.field_cards[0].push(target);
        
        // Wrong player's turn
        let result = state.can_stockpile(wrong_player, field, active_card, passive_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not player 1's turn"));
        
        // Missing active card
        let result = state.can_stockpile(player, field, missing_card, passive_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("doesn't have the active card"));
        
        // Missing passive card
        let result = state.can_stockpile(player, field, active_card, missing_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("doesn't have the passive card"));
        
        // Same active and passive card
        let result = state.can_stockpile(player, field, same_card, same_card, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("must be different"));
        
        // Missing target card
        let result = state.can_stockpile(player, field, active_card, passive_card, &[missing_target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not in the field"));
        
        // Empty targets
        let result = state.can_stockpile(player, field, active_card, passive_card, &[]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least one field card"));
        
        // Value mismatch (3 + 4 = 7, but we're checking against a different passive)
        let wrong_passive = Card::new(Rank::Ten, Suit::Spring); // Value 10
        state.player_hands[0].push(wrong_passive);
        let result = state.can_stockpile(player, field, active_card, wrong_passive, &[target]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not equal passive card value"));
    }
    
    #[test]
    fn test_stockpile_fool_cards() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let active_fool = Card::new(Rank::Fool, Suit::Spring);    // 1 or 14
        let passive_card = Card::new(Rank::King, Suit::Summer);   // 13
        let target_queen = Card::new(Rank::Queen, Suit::Autumn);  // 12
        
        // Set up state: Fool (1) + Queen (12) = 13 (King)
        state.player_hands[0].push(active_fool);
        state.player_hands[0].push(passive_card);
        state.field_cards[0].push(target_queen);
        state.deck.push(Card::new(Rank::Two, Suit::Winter));
        state.deck.push(Card::new(Rank::Three, Suit::Stars));
        
        // Apply stockpile
        let result = state.apply_stockpile(player, field, active_fool, passive_card, vec![target_queen]);
        assert!(result.is_ok());
        
        // Check stockpile was created
        assert_eq!(state.field_stockpiles[0].len(), 1);
        assert_eq!(state.field_stockpiles[0][0].value, 13);
        assert_eq!(state.field_stockpiles[0][0].cards.len(), 2);
    }
    
    #[test]
    fn test_stockpile_face_card_changes_season() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let active_face = Card::new(Rank::Knight, Suit::Autumn);  // Face card, value 11
        let passive_card = Card::new(Rank::King, Suit::Summer);   // Value 13
        let target = Card::new(Rank::Two, Suit::Winter);          // Value 2
        
        // Set up state: Knight (11) + Two (2) = 13 (King)
        state.player_hands[0].push(active_face);
        state.player_hands[0].push(passive_card);
        state.field_cards[0].push(target);
        state.deck.push(Card::new(Rank::Four, Suit::Spring));
        state.deck.push(Card::new(Rank::Five, Suit::Stars));
        
        // Initial seasons are all Summer
        assert_eq!(state.field_seasons, [Season::Summer; 4]);
        
        // Apply stockpile with Autumn Knight
        let result = state.apply_stockpile(player, field, active_face, passive_card, vec![target]);
        assert!(result.is_ok());
        
        // All fields should now be Autumn
        assert_eq!(state.field_seasons, [Season::Autumn; 4]);
    }
    
    #[test]
    fn test_find_stockpile_combinations() {
        let mut state = IllimatState::new_test_game();
        let field = FieldId(0);
        let active_card = Card::new(Rank::Three, Suit::Spring); // Value 3
        
        // Set up field with various cards
        state.field_cards[0] = vec![
            Card::new(Rank::Two, Suit::Summer),   // 2
            Card::new(Rank::Four, Suit::Autumn),  // 4
            Card::new(Rank::Five, Suit::Winter),  // 5
            Card::new(Rank::Fool, Suit::Stars),   // 1 or 14
        ];
        
        // Find combinations for passive value 8 (need 8 - 3 = 5 from field)
        let combinations = state.find_stockpile_combinations(field, active_card, 8);
        
        // Should find: [5], and possibly [Fool(1)+4] for total 5
        assert!(!combinations.is_empty());
        
        // Should include the single 5 card
        assert!(combinations.iter().any(|combo| 
            combo.len() == 1 && combo[0] == Card::new(Rank::Five, Suit::Winter)
        ));
    }
    
    #[test]
    fn test_stockpile_value_limits() {
        let mut state = IllimatState::new_test_game();
        let player = PlayerId(0);
        let field = FieldId(0);
        let active_card = Card::new(Rank::Two, Suit::Spring);
        let invalid_passive = Card::new(Rank::King, Suit::Summer); // This should be fine (13)
        let target = Card::new(Rank::King, Suit::Autumn); // 13
        
        state.player_hands[0].push(active_card);
        state.player_hands[0].push(invalid_passive);
        state.field_cards[0].push(target);
        
        // 2 + 13 = 15, but max stockpile value is 14, so this should be invalid
        // Wait, actually 13 is valid. Let me create a truly invalid case.
        
        // Actually, the passive card value itself cannot exceed 14, and since
        // King = 13, this is valid. The rule is about the stockpile value matching
        // the passive card value, and passive card values are capped at 14 by card design.
        // So this test needs adjustment.
        
        let result = state.can_stockpile(player, field, active_card, invalid_passive, &[target]);
        // This should actually be valid since 2 + 13 = 15, but passive card is 13
        // Wait, that's wrong math. 2 + 13 != 13. This should fail for value mismatch.
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("does not equal"));
    }
    
    #[test]
    fn test_round_end_detection() {
        let mut state = IllimatState::new_test_game();
        
        // Round should not be over with cards in deck
        state.deck.push(Card::new(Rank::Two, Suit::Spring));
        assert!(!state.is_round_over());
        
        // Round should be over with empty deck
        state.deck.clear();
        assert!(state.is_round_over());
    }
    
    #[test]
    fn test_victory_condition() {
        let mut state = IllimatState::new_test_game();
        
        // No victory initially
        assert_eq!(state.check_victory(), None);
        
        // Player 1 reaches 17 points
        state.total_scores[1] = 17;
        assert_eq!(state.check_victory(), Some(PlayerId(1)));
        
        // Player 0 reaches 20 points (higher)
        state.total_scores[0] = 20;
        assert_eq!(state.check_victory(), Some(PlayerId(0))); // Still returns first player found >= 17
    }
    
    #[test]
    fn test_competitive_scoring() {
        let mut state = IllimatState::new_test_game();
        
        // Set up harvests for different players
        state.player_harvests[0] = vec![
            Card::new(Rank::Two, Suit::Summer),     // Summer card
            Card::new(Rank::Three, Suit::Summer),   // Summer card  
            Card::new(Rank::Four, Suit::Spring),    // Regular card
            Card::new(Rank::Fool, Suit::Winter),    // Fool
        ]; // 4 total, 2 summer, 0 winter, 1 fool
        
        state.player_harvests[1] = vec![
            Card::new(Rank::Five, Suit::Winter),    // Winter card
            Card::new(Rank::Six, Suit::Winter),     // Winter card
            Card::new(Rank::Seven, Suit::Autumn),   // Regular card
        ]; // 3 total, 0 summer, 2 winter, 0 fools
        
        state.player_harvests[2] = vec![
            Card::new(Rank::Eight, Suit::Summer),   // Summer card
            Card::new(Rank::Nine, Suit::Spring),    // Regular card
            Card::new(Rank::Ten, Suit::Autumn),     // Regular card
            Card::new(Rank::Knight, Suit::Stars),   // Regular card
            Card::new(Rank::Queen, Suit::Stars),    // Regular card
        ]; // 5 total, 1 summer, 0 winter, 0 fools
        
        // Give player 0 an okus token
        state.move_okus(OkusId::A, OkusPosition::WithPlayer(PlayerId(0)));
        
        let scoring = state.calculate_round_scoring();
        
        // Player 2 should win Bumper Crop (5 cards)
        assert_eq!(scoring.bumper_crop_winner, Some(PlayerId(2)));
        
        // Player 0 should win Sunkissed (2 Summer cards vs 1 for player 2)
        assert_eq!(scoring.sunkissed_winner, Some(PlayerId(0)));
        
        // Player 1 should get Frostbit (2 Winter cards, most)
        assert_eq!(scoring.frostbit_players, vec![PlayerId(1)]);
        
        // Individual scores: P0 = 1 fool + 1 okus = 2, others = 0
        assert_eq!(scoring.individual_scores[0], 2);
        assert_eq!(scoring.individual_scores[1], 0);
        assert_eq!(scoring.individual_scores[2], 0);
    }
    
    #[test]
    fn test_end_round_scoring() {
        let mut state = IllimatState::new_test_game();
        
        // Set up for round end
        state.deck.clear(); // Empty deck = round over
        
        // Set up harvests
        state.player_harvests[0] = vec![
            Card::new(Rank::Two, Suit::Summer),     // Summer
            Card::new(Rank::Three, Suit::Summer),   // Summer
            Card::new(Rank::Fool, Suit::Spring),    // Fool
        ];
        state.player_harvests[1] = vec![
            Card::new(Rank::Four, Suit::Winter),    // Winter
            Card::new(Rank::Five, Suit::Winter),    // Winter
        ];
        
        // Give player 0 an okus
        state.move_okus(OkusId::A, OkusPosition::WithPlayer(PlayerId(0)));
        
        let initial_scores = state.total_scores;
        
        let winner = state.end_round_and_score().unwrap();
        assert_eq!(winner, None); // No one reached 17 points
        
        // Check score changes
        // P0: No Bumper Crop (P0 has 3 cards, P1 has 2 cards, P0 wins), +2 Sunkissed + 1 Fool + 1 okus = +4 Bumper Crop + 2 + 1 + 1 = +8 total
        // Wait, let me recalculate: P0 has 3 cards, P1 has 2 cards, so P0 should get Bumper Crop too
        assert_eq!(state.total_scores[0], initial_scores[0] + 8); // +4 Bumper Crop + 2 Sunkissed + 1 Fool + 1 okus
        // P1: -2 Frostbit = -2 total (but saturating_sub prevents underflow)
        assert_eq!(state.total_scores[1], initial_scores[1].saturating_sub(2));
        
        // Okus should be returned to Illimat
        assert_eq!(state.get_okus_position(OkusId::A), OkusPosition::OnIllimat);
        
        // Round number should increment
        assert_eq!(state.round_number, 2);
        assert_eq!(state.phase, GamePhase::RoundEnd);
    }
    
    #[test]
    fn test_tie_handling() {
        let mut state = IllimatState::new_test_game();
        
        // Both players have same number of Summer cards (tie)
        state.player_harvests[0] = vec![
            Card::new(Rank::Two, Suit::Summer),     
        ]; // 1 summer
        state.player_harvests[1] = vec![
            Card::new(Rank::Three, Suit::Summer),   
        ]; // 1 summer
        
        let scoring = state.calculate_round_scoring();
        
        // Tie should result in no winner
        assert_eq!(scoring.sunkissed_winner, None);
    }
    
    #[test]
    fn test_display_realistic_game_state() {
        let mut state = IllimatState::new_test_game();
        
        // Set up a realistic mid-game scenario
        state.current_player = PlayerId(2);
        
        // Set field seasons
        state.field_seasons = [Season::Winter, Season::Spring, Season::Summer, Season::Autumn];
        
        // Add some loose cards to fields
        state.field_cards[0] = vec![
            Card::new(Rank::Three, Suit::Spring),
            Card::new(Rank::Seven, Suit::Autumn),
        ];
        state.field_cards[1] = vec![
            Card::new(Rank::Fool, Suit::Summer),
            Card::new(Rank::King, Suit::Winter),
        ];
        state.field_cards[2] = vec![]; // Empty field
        state.field_cards[3] = vec![
            Card::new(Rank::Two, Suit::Winter),
        ];
        
        // Add some stockpiles
        state.field_stockpiles[1].push(Stockpile {
            value: 8,
            cards: vec![
                Card::new(Rank::Eight, Suit::Autumn),
                Card::new(Rank::Eight, Suit::Stars),
            ],
            created_turn: 1, // Test data - old stockpile
        });
        state.field_stockpiles[1].push(Stockpile {
            value: 11,
            cards: vec![
                Card::new(Rank::Five, Suit::Spring),
                Card::new(Rank::Six, Suit::Summer),
            ],
            created_turn: 2, // Test data - old stockpile
        });
        
        // Set up player hands
        state.player_hands[0] = vec![];  // Player 0 has no cards
        state.player_hands[1] = vec![
            Card::new(Rank::Nine, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
        ];
        state.player_hands[2] = vec![  // Current player
            Card::new(Rank::Five, Suit::Summer),
            Card::new(Rank::Queen, Suit::Stars),
            Card::new(Rank::Eight, Suit::Autumn),
            Card::new(Rank::Fool, Suit::Winter),
        ];
        state.player_hands[3] = vec![
            Card::new(Rank::Knight, Suit::Winter),
            Card::new(Rank::Four, Suit::Stars),
            Card::new(Rank::Six, Suit::Autumn),
        ];
        
        // Distribute some okus
        state.move_okus(OkusId::A, OkusPosition::WithPlayer(PlayerId(0)));
        state.move_okus(OkusId::B, OkusPosition::WithPlayer(PlayerId(1)));
        // C and D remain on Illimat
        
        // Set some harvested cards for scoring demonstration  
        state.player_harvests[0] = vec![
            Card::new(Rank::Fool, Suit::Spring),        // +1 fool
            Card::new(Rank::Two, Suit::Summer),         // Summer card  
            Card::new(Rank::Three, Suit::Summer),       // Summer card
        ]; // P0 will have: 3 cards, 2 summer, 1 fool, 1 okus = +2 immediate pts
        
        state.player_harvests[1] = vec![
            Card::new(Rank::Four, Suit::Winter),        // Winter card
            Card::new(Rank::Five, Suit::Winter),        // Winter card  
            Card::new(Rank::Six, Suit::Autumn),         // Regular card
            Card::new(Rank::Seven, Suit::Spring),       // Regular card
            Card::new(Rank::Eight, Suit::Stars),        // Regular card
        ]; // P1 will have: 5 cards, 2 winter, 1 okus = +1 immediate pts
        
        state.player_harvests[2] = vec![]; // Current player has no harvests yet
        
        state.player_harvests[3] = vec![
            Card::new(Rank::Fool, Suit::Winter),        // +1 fool
            Card::new(Rank::Nine, Suit::Summer),        // Summer card
        ]; // P3 will have: 2 cards, 1 summer, 1 fool = +1 immediate pts
        
        // Set some total scores from previous rounds
        state.total_scores = [4, 7, 2, 5];
        state.round_number = 3;
        
        // Set deck size
        state.deck = vec![Card::new(Rank::Two, Suit::Spring); 23]; // 23 cards remaining
        
        // Print the state and verify key elements are present
        let display_output = format!("{}", state);
        println!("{}", display_output);
        
        // Test that key elements are in the new display format
        assert!(display_output.contains("=== ILLIMAT - Round 3, Player 2 ==="));
        assert!(display_output.contains("Spring Field (Winter): [3 Sp] [7 Au]"));
        assert!(display_output.contains("Summer Field (Spring): [F Su] [K Wi] (Stockpiles: [8: [8 Au], [8 St]], [11: [5 Sp], [6 Su]])"));
        assert!(display_output.contains("Autumn Field (Summer): empty"));
        assert!(display_output.contains("Winter Field (Autumn): [2 Wi]"));
        assert!(display_output.contains("YOUR HAND: [5 Su] [Q St] [8 Au] [F Wi]"));
        assert!(display_output.contains("ROUND HARVEST:"));
        assert!(display_output.contains("P0: 3 cards, 2☀, 1F, 1⚬ = +2 pts"));
        assert!(display_output.contains("P1: 5 cards, 2❄, 1⚬ = +1 pts"));
        assert!(display_output.contains("P2: 0 cards = +0 pts"));
        assert!(display_output.contains("P3: 2 cards, 1☀, 1❄, 1F = +1 pts"));
        assert!(display_output.contains("OKUS: Illimat: C, D | P0: A | P1: B"));
        assert!(display_output.contains("TOTAL SCORES: P0: 4 (+2 = 6) | P1: 7 (+1 = 8) | P2: 2 (+0 = 2) | P3: 5 (+1 = 6)"));
        assert!(display_output.contains("Cards in deck: 23"));
    }
}