/// Field identifier (0-3, just board positions)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct FieldId(pub u8);

impl FieldId {
    /// Get the seasonal name for this field based on current Illimat orientation
    pub fn seasonal_name(self, illimat_orientation: u8) -> &'static str {
        use crate::game::season::{Season, SeasonManager};
        let season = SeasonManager::get_base_season(self, illimat_orientation);
        match season {
            Season::Spring => "Spring Field",
            Season::Summer => "Summer Field", 
            Season::Autumn => "Autumn Field",
            Season::Winter => "Winter Field",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seasonal_names() {
        // With default orientation (0), fields should be in order: Spring, Summer, Autumn, Winter
        assert_eq!(FieldId(0).seasonal_name(0), "Spring Field");
        assert_eq!(FieldId(1).seasonal_name(0), "Summer Field");
        assert_eq!(FieldId(2).seasonal_name(0), "Autumn Field");
        assert_eq!(FieldId(3).seasonal_name(0), "Winter Field");
        
        // With orientation 1, should rotate: Winter, Spring, Summer, Autumn  
        assert_eq!(FieldId(0).seasonal_name(1), "Winter Field");
        assert_eq!(FieldId(1).seasonal_name(1), "Spring Field");
        assert_eq!(FieldId(2).seasonal_name(1), "Summer Field");
        assert_eq!(FieldId(3).seasonal_name(1), "Autumn Field");
    }
}