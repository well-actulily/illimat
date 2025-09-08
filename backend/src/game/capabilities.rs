use crate::game::field_id::FieldId;
use crate::game::season::{Season, SeasonManager};
use crate::game::luminary::{LuminaryState};
use crate::game::luminary_manager::LuminaryManager;

/// Field capability checking (what actions are allowed)
pub struct CapabilityManager;

impl CapabilityManager {
    /// Check if sowing is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_sow(field: FieldId, illimat_orientation: u8, luminary_states: &[LuminaryState; 4]) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        let capabilities = LuminaryManager::get_field_capabilities(field, base_season, luminary_states, illimat_orientation);
        capabilities.can_sow
    }
    
    /// Check if harvesting is allowed in a field (considering Illimat + Luminary effects)  
    pub fn can_harvest(field: FieldId, illimat_orientation: u8, luminary_states: &[LuminaryState; 4]) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        let capabilities = LuminaryManager::get_field_capabilities(field, base_season, luminary_states, illimat_orientation);
        capabilities.can_harvest
    }
    
    /// Check if stockpiling is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_stockpile(field: FieldId, illimat_orientation: u8, luminary_states: &[LuminaryState; 4]) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        let capabilities = LuminaryManager::get_field_capabilities(field, base_season, luminary_states, illimat_orientation);
        capabilities.can_stockpile
    }

    /// Get full field capabilities with Luminary effects and special rules
    pub fn get_field_capabilities(field: FieldId, illimat_orientation: u8, luminary_states: &[LuminaryState; 4]) -> crate::game::luminary::FieldCapabilities {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        LuminaryManager::get_field_capabilities(field, base_season, luminary_states, illimat_orientation)
    }

    /// Legacy method for backward compatibility (no Luminaries)
    pub fn can_sow_basic(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        base_season != Season::Autumn
    }
    
    /// Legacy method for backward compatibility (no Luminaries)
    pub fn can_harvest_basic(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        base_season != Season::Winter
    }
    
    /// Legacy method for backward compatibility (no Luminaries)
    pub fn can_stockpile_basic(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        base_season != Season::Spring
    }
}