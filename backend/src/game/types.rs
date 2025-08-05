use std::fmt;
use crate::game::card::Card;

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

/// Player type for future AI support
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PlayerType {
    Human,
    // TODO: Add CPU variants when implementing AI
}

/// Game phase tracking
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GamePhase {
    Setup,      // Before first round
    Playing,    // During rounds
    RoundEnd,   // Between rounds
    GameEnd,    // Game finished
}

/// Game configuration
#[derive(Clone, Debug, PartialEq)]
pub struct GameConfig {
    pub player_count: u8,
    pub player_types: [PlayerType; 4],
    pub use_stars_suit: bool,  // false = remove Stars suit for 2-3 players
}

/// Stockpile representation - a set of cards that sum to a harvestable value
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Stockpile {
    pub value: u8,
    pub cards: Vec<Card>,
    pub created_turn: u16, // Turn number when this stockpile was created
}

/// Core game actions
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Sow { field: FieldId, card: Card },
    Harvest { field: FieldId, card: Card, targets: Vec<Card> },
    Stockpile { field: FieldId, card: Card, targets: Vec<Card> },
}

/// Round scoring breakdown
#[derive(Debug, Clone)]
pub struct RoundScoring {
    pub bumper_crop_winner: Option<PlayerId>,
    pub sunkissed_winner: Option<PlayerId>, 
    pub frostbit_players: Vec<PlayerId>,
    pub individual_scores: [i8; 4], // Fools + okus for each player
}