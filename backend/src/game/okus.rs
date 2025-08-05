use std::fmt;
use crate::game::player::PlayerId;

/// Okus token identifier (A-D)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
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
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OkusPosition {
    WithPlayer(PlayerId),
    OnIllimat,
}

/// Okus management utilities
pub struct OkusManager;

impl OkusManager {
    /// Get all available okus tokens on the Illimat
    pub fn get_available_okus(okus_positions: &[OkusPosition; 4]) -> Vec<OkusId> {
        [OkusId::A, OkusId::B, OkusId::C, OkusId::D]
            .iter()
            .filter(|&&okus| okus_positions[okus as usize] == OkusPosition::OnIllimat)
            .cloned()
            .collect()
    }
    
    /// Count how many okus tokens are on the Illimat
    pub fn count_on_illimat(okus_positions: &[OkusPosition; 4]) -> u8 {
        okus_positions.iter()
            .filter(|pos| matches!(pos, OkusPosition::OnIllimat))
            .count() as u8
    }
    
    /// Count how many okus tokens a player has
    pub fn count_player_okus(okus_positions: &[OkusPosition; 4], player: PlayerId) -> u8 {
        okus_positions.iter()
            .filter(|pos| matches!(pos, OkusPosition::WithPlayer(p) if *p == player))
            .count() as u8
    }
}