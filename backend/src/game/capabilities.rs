use crate::game::field_id::FieldId;
use crate::game::season::{Season, SeasonManager};

/// Field capability checking (what actions are allowed)
pub struct CapabilityManager;

impl CapabilityManager {
    /// Check if sowing is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_sow(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        
        // Base rule: Autumn forbids sowing
        let base_can_sow = base_season != Season::Autumn;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - Forest Queen makes her field always Summer (allows sowing)
        // - Other Luminaries might modify capabilities
        
        base_can_sow
    }
    
    /// Check if harvesting is allowed in a field (considering Illimat + Luminary effects)  
    pub fn can_harvest(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        
        // Base rule: Winter blocks harvesting
        let base_can_harvest = base_season != Season::Winter;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - The Maiden allows harvesting in Winter
        // - The Drought blocks harvesting in Summer
        
        base_can_harvest
    }
    
    /// Check if stockpiling is allowed in a field (considering Illimat + Luminary effects)
    pub fn can_stockpile(field: FieldId, illimat_orientation: u8) -> bool {
        let base_season = SeasonManager::get_base_season(field, illimat_orientation);
        
        // Base rule: Spring forbids stockpiling  
        let base_can_stockpile = base_season != Season::Spring;
        
        // TODO: Apply Luminary effects here
        // For example:
        // - Forest Queen makes her field always Summer (allows stockpiling)
        // - The Loom allows stockpiling ignoring season rules
        
        base_can_stockpile
    }
}