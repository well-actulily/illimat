use std::fmt;
use crate::game::field_id::FieldId;

/// Season types that restrict actions
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
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

/// Season management utilities
pub struct SeasonManager;

impl SeasonManager {
    /// Get the base season of a field based on Illimat orientation
    pub fn get_base_season(field: FieldId, illimat_orientation: u8) -> Season {
        let season_offset = (field.0 + 4 - illimat_orientation) % 4;
        match season_offset {
            0 => Season::Spring,
            1 => Season::Summer, 
            2 => Season::Autumn,
            3 => Season::Winter,
            _ => unreachable!(),
        }
    }
    
    /// Calculate what Illimat orientation would make a field have the target season
    pub fn calculate_illimat_orientation(field: FieldId, target_season: Season) -> u8 {
        let season_offset = match target_season {
            Season::Spring => 0,
            Season::Summer => 1,
            Season::Autumn => 2, 
            Season::Winter => 3,
        };
        (field.0 + 4 - season_offset) % 4
    }
    
    /// Update all field seasons based on current Illimat orientation
    pub fn update_all_seasons(field_seasons: &mut [Season; 4], illimat_orientation: u8) {
        for i in 0..4 {
            field_seasons[i] = Self::get_base_season(FieldId(i as u8), illimat_orientation);
        }
    }
}