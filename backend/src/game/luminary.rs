use crate::game::card::Card;
use crate::game::field_id::FieldId;
use crate::game::player::PlayerId;
use crate::game::season::Season;
use crate::game::actions::Action;
use serde::{Deserialize, Serialize};

/// All Luminary cards across all expansions
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LuminaryCard {
    // Core Set (8 cards)
    TheMaiden,
    TheChangeling,
    TheRiver,
    TheChildren,
    TheForestQueen,
    TheRake,
    TheUnion,
    TheNewborn,
    
    // False Baron's Set (6 cards)
    TheAstronomer,
    TheCollective,
    TheDrought,
    TheEcho,
    TheGambler,
    TheUniverse,
    
    // Crane Wife Expansion (6 cards)
    TheLoom,
    TheIsland,
    ThePerfectCrime,
    TheButchers,
    TheSoldiers,
    TheBoat,
    
    // Other Released (2 cards)
    TheAudience,
    TheRusalka,
}

impl LuminaryCard {
    /// Get all Luminaries in a specific expansion
    pub fn expansion_cards(expansion: LuminaryExpansion) -> Vec<LuminaryCard> {
        match expansion {
            LuminaryExpansion::Core => vec![
                LuminaryCard::TheMaiden,
                LuminaryCard::TheChangeling,
                LuminaryCard::TheRiver,
                LuminaryCard::TheChildren,
                LuminaryCard::TheForestQueen,
                LuminaryCard::TheRake,
                LuminaryCard::TheUnion,
                LuminaryCard::TheNewborn,
            ],
            LuminaryExpansion::FalseBarons => vec![
                LuminaryCard::TheAstronomer,
                LuminaryCard::TheCollective,
                LuminaryCard::TheDrought,
                LuminaryCard::TheEcho,
                LuminaryCard::TheGambler,
                LuminaryCard::TheUniverse,
            ],
            LuminaryExpansion::CraneWife => vec![
                LuminaryCard::TheLoom,
                LuminaryCard::TheIsland,
                LuminaryCard::ThePerfectCrime,
                LuminaryCard::TheButchers,
                LuminaryCard::TheSoldiers,
                LuminaryCard::TheBoat,
            ],
            LuminaryExpansion::Other => vec![
                LuminaryCard::TheAudience,
                LuminaryCard::TheRusalka,
            ],
        }
    }

    /// Get human-readable name for display
    pub fn display_name(self) -> &'static str {
        match self {
            // Core Set
            LuminaryCard::TheMaiden => "The Maiden",
            LuminaryCard::TheChangeling => "The Changeling",
            LuminaryCard::TheRiver => "The River",
            LuminaryCard::TheChildren => "The Children",
            LuminaryCard::TheForestQueen => "The Forest Queen",
            LuminaryCard::TheRake => "The Rake",
            LuminaryCard::TheUnion => "The Union",
            LuminaryCard::TheNewborn => "The Newborn",
            
            // False Baron's Set
            LuminaryCard::TheAstronomer => "The Astronomer",
            LuminaryCard::TheCollective => "The Collective",
            LuminaryCard::TheDrought => "The Drought",
            LuminaryCard::TheEcho => "The Echo",
            LuminaryCard::TheGambler => "The Gambler",
            LuminaryCard::TheUniverse => "The Universe",
            
            // Crane Wife Expansion
            LuminaryCard::TheLoom => "The Loom",
            LuminaryCard::TheIsland => "The Island",
            LuminaryCard::ThePerfectCrime => "The Perfect Crime",
            LuminaryCard::TheButchers => "The Butchers",
            LuminaryCard::TheSoldiers => "The Soldiers",
            LuminaryCard::TheBoat => "The Boat",
            
            // Other Released
            LuminaryCard::TheAudience => "The Audience",
            LuminaryCard::TheRusalka => "The Rusalka",
        }
    }

    /// Get all Luminaries as a complete list
    pub fn all_luminaries() -> Vec<LuminaryCard> {
        let mut all = Vec::new();
        all.extend(Self::expansion_cards(LuminaryExpansion::Core));
        all.extend(Self::expansion_cards(LuminaryExpansion::FalseBarons));
        all.extend(Self::expansion_cards(LuminaryExpansion::CraneWife));
        all.extend(Self::expansion_cards(LuminaryExpansion::Other));
        all
    }
}

/// Luminary expansion sets for configuration
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LuminaryExpansion {
    Core,
    FalseBarons,
    CraneWife,
    Other,
}

/// Configuration for which Luminaries are included in a game
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LuminaryConfiguration {
    /// Which expansions to include
    pub enabled_expansions: Vec<LuminaryExpansion>,
    /// Specific Luminaries to exclude (even if their expansion is enabled)
    pub excluded_luminaries: Vec<LuminaryCard>,
    /// Specific Luminaries to include (even if their expansion is disabled)
    pub included_luminaries: Vec<LuminaryCard>,
}

impl LuminaryConfiguration {
    /// Create a configuration with core set only
    pub fn core_only() -> Self {
        Self {
            enabled_expansions: vec![LuminaryExpansion::Core],
            excluded_luminaries: vec![],
            included_luminaries: vec![],
        }
    }

    /// Create a configuration with all expansions
    pub fn all_expansions() -> Self {
        Self {
            enabled_expansions: vec![
                LuminaryExpansion::Core,
                LuminaryExpansion::FalseBarons,
                LuminaryExpansion::CraneWife,
                LuminaryExpansion::Other,
            ],
            excluded_luminaries: vec![],
            included_luminaries: vec![],
        }
    }

    /// Create configuration with no Luminaries (beginner mode)
    pub fn none() -> Self {
        Self {
            enabled_expansions: vec![],
            excluded_luminaries: vec![],
            included_luminaries: vec![],
        }
    }

    /// Get the final list of Luminaries to use in the game
    pub fn get_active_luminaries(&self) -> Vec<LuminaryCard> {
        let mut active = Vec::new();

        // Add all cards from enabled expansions
        for expansion in &self.enabled_expansions {
            active.extend(LuminaryCard::expansion_cards(*expansion));
        }

        // Remove excluded cards
        active.retain(|card| !self.excluded_luminaries.contains(card));

        // Add specifically included cards
        for card in &self.included_luminaries {
            if !active.contains(card) {
                active.push(*card);
            }
        }

        active
    }
}

/// State of a Luminary in a field
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LuminaryState {
    /// No Luminary in this field
    None,
    /// Luminary is face-down (not yet revealed)
    FaceDown(LuminaryCard),
    /// Luminary is face-up (revealed but not claimed)
    FaceUp(LuminaryCard),
    /// Luminary has been claimed by a player
    Claimed(LuminaryCard, PlayerId),
}

impl LuminaryState {
    /// Get the Luminary card if one exists
    pub fn card(self) -> Option<LuminaryCard> {
        match self {
            LuminaryState::None => None,
            LuminaryState::FaceDown(card) => Some(card),
            LuminaryState::FaceUp(card) => Some(card),
            LuminaryState::Claimed(card, _) => Some(card),
        }
    }

    /// Check if this Luminary is active (face-up or claimed)
    pub fn is_active(self) -> bool {
        matches!(self, LuminaryState::FaceUp(_) | LuminaryState::Claimed(_, _))
    }

    /// Check if this Luminary can be claimed
    pub fn can_be_claimed(self) -> bool {
        matches!(self, LuminaryState::FaceUp(_))
    }

    /// Check if this Luminary is claimed by a specific player
    pub fn is_claimed_by(self, player: PlayerId) -> Option<LuminaryCard> {
        match self {
            LuminaryState::Claimed(card, p) if p == player => Some(card),
            _ => None,
        }
    }
}

/// Flexible rule modification system for Luminaries
pub trait LuminaryRuleModifier {
    /// Modify field capabilities (sow/harvest/stockpile permissions)
    fn modify_capabilities(
        &self,
        field: FieldId,
        base_season: Season,
        luminary_states: &[LuminaryState; 4],
        illimat_orientation: u8,
    ) -> FieldCapabilities;

    /// Modify action resolution (before action is applied)
    fn modify_action_resolution(
        &self,
        action: &crate::game::actions::Action,
        luminary_states: &[LuminaryState; 4],
        game_state: &crate::game::state::IllimatState,
    ) -> ActionModification;

    /// Handle field clearing events
    fn handle_field_cleared(
        &self,
        field: FieldId,
        player: PlayerId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut crate::game::state::IllimatState,
    ) -> FieldClearingResult;

    /// Handle Luminary revelation (when face-down becomes face-up)
    fn handle_revelation(
        &self,
        luminary: LuminaryCard,
        field: FieldId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut crate::game::state::IllimatState,
    ) -> RevelationResult;

    /// Handle Luminary claiming (when face-up is claimed by player)
    fn handle_claiming(
        &self,
        luminary: LuminaryCard,
        field: FieldId,
        player: PlayerId,
        luminary_states: &mut [LuminaryState; 4],
        game_state: &mut crate::game::state::IllimatState,
    ) -> ClaimingResult;

    /// Get available Luminary-specific actions for a player's turn
    fn get_available_actions(
        &self,
        player: PlayerId,
        luminary_states: &[LuminaryState; 4],
        game_state: &crate::game::state::IllimatState,
    ) -> Vec<Action> {
        // Default: no additional actions
        vec![]
    }

    /// Check if a Luminary requires mandatory actions this turn  
    fn get_mandatory_actions(
        &self,
        player: PlayerId,
        luminary_states: &[LuminaryState; 4],
        game_state: &crate::game::state::IllimatState,
    ) -> Vec<Action> {
        // Default: no mandatory actions
        vec![]
    }

    /// Check if actions can be performed before/after the main action
    fn get_action_timing(
        &self,
        action: &Action,
        luminary_states: &[LuminaryState; 4],
    ) -> ActionTiming {
        // Default: no timing restrictions
        ActionTiming::Normal
    }
}

/// Field capability modification results
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldCapabilities {
    pub can_sow: bool,
    pub can_harvest: bool,
    pub can_stockpile: bool,
    /// Additional restrictions or requirements
    pub special_rules: Vec<String>,
}

/// Action modification results
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionModification {
    /// Action proceeds normally
    Normal,
    /// Action is blocked
    Blocked(String),
    /// Action is modified (e.g., play multiple cards, repeat in other field)
    Modified {
        description: String,
        additional_effects: Vec<String>,
    },
}

/// Action timing for Luminary effects
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionTiming {
    /// No special timing requirements
    Normal,
    /// Must be performed before main action
    BeforeMain,
    /// May be performed before main action  
    OptionalBefore,
    /// Must be performed after main action
    AfterMain,
    /// May be performed after main action
    OptionalAfter,
    /// Can trigger additional actions (like Echo)
    TriggersAdditional(Vec<Action>),
}

/// Field clearing results
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FieldClearingResult {
    /// Whether field should be reseeded
    pub should_reseed: bool,
    /// Additional cards to deal to specific locations
    pub additional_cards: Vec<(FieldId, Vec<Card>)>,
    /// Special effects triggered by clearing
    pub special_effects: Vec<String>,
}

/// Luminary revelation results
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RevelationResult {
    /// Cards to deal to fields
    pub cards_to_deal: Vec<(FieldId, Vec<Card>)>,
    /// Other Luminaries to reveal
    pub reveal_other_luminaries: Vec<FieldId>,
    /// Special effects
    pub special_effects: Vec<String>,
}

/// Luminary claiming results
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimingResult {
    /// Immediate effects when claimed
    pub immediate_effects: Vec<String>,
    /// Ongoing effects while held
    pub ongoing_effects: Vec<String>,
}

/// Default implementation of rule modifier (no Luminaries active)
pub struct DefaultLuminaryModifier;

impl LuminaryRuleModifier for DefaultLuminaryModifier {
    fn modify_capabilities(
        &self,
        _field: FieldId,
        base_season: Season,
        _luminary_states: &[LuminaryState; 4],
        _illimat_orientation: u8,
    ) -> FieldCapabilities {
        // No Luminaries active - use base season rules
        FieldCapabilities {
            can_sow: base_season != Season::Autumn,
            can_harvest: base_season != Season::Winter,
            can_stockpile: base_season != Season::Spring,
            special_rules: vec![],
        }
    }

    fn modify_action_resolution(
        &self,
        _action: &crate::game::actions::Action,
        _luminary_states: &[LuminaryState; 4],
        _game_state: &crate::game::state::IllimatState,
    ) -> ActionModification {
        ActionModification::Normal
    }

    fn handle_field_cleared(
        &self,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut crate::game::state::IllimatState,
    ) -> FieldClearingResult {
        FieldClearingResult {
            should_reseed: false, // Base game: fields stay fallow unless okus were available
            additional_cards: vec![],
            special_effects: vec![],
        }
    }

    fn handle_revelation(
        &self,
        _luminary: LuminaryCard,
        _field: FieldId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut crate::game::state::IllimatState,
    ) -> RevelationResult {
        RevelationResult {
            cards_to_deal: vec![],
            reveal_other_luminaries: vec![],
            special_effects: vec![],
        }
    }

    fn handle_claiming(
        &self,
        _luminary: LuminaryCard,
        _field: FieldId,
        _player: PlayerId,
        _luminary_states: &mut [LuminaryState; 4],
        _game_state: &mut crate::game::state::IllimatState,
    ) -> ClaimingResult {
        ClaimingResult {
            immediate_effects: vec![],
            ongoing_effects: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_luminary_card_enumeration() {
        let all_luminaries = LuminaryCard::all_luminaries();
        assert_eq!(all_luminaries.len(), 22, "Should have 22 total Luminaries");

        // Test expansion counts
        assert_eq!(LuminaryCard::expansion_cards(LuminaryExpansion::Core).len(), 8);
        assert_eq!(LuminaryCard::expansion_cards(LuminaryExpansion::FalseBarons).len(), 6);
        assert_eq!(LuminaryCard::expansion_cards(LuminaryExpansion::CraneWife).len(), 6);
        assert_eq!(LuminaryCard::expansion_cards(LuminaryExpansion::Other).len(), 2);
    }

    #[test]
    fn test_luminary_configuration() {
        // Test core only
        let core_config = LuminaryConfiguration::core_only();
        let core_luminaries = core_config.get_active_luminaries();
        assert_eq!(core_luminaries.len(), 8);
        assert!(core_luminaries.contains(&LuminaryCard::TheForestQueen));
        assert!(!core_luminaries.contains(&LuminaryCard::TheDrought));

        // Test all expansions
        let all_config = LuminaryConfiguration::all_expansions();
        let all_luminaries = all_config.get_active_luminaries();
        assert_eq!(all_luminaries.len(), 22);

        // Test exclusions
        let mut excluded_config = LuminaryConfiguration::all_expansions();
        excluded_config.excluded_luminaries.push(LuminaryCard::TheIsland);
        let filtered_luminaries = excluded_config.get_active_luminaries();
        assert_eq!(filtered_luminaries.len(), 21);
        assert!(!filtered_luminaries.contains(&LuminaryCard::TheIsland));

        // Test none (beginner mode)
        let none_config = LuminaryConfiguration::none();
        let no_luminaries = none_config.get_active_luminaries();
        assert_eq!(no_luminaries.len(), 0);
    }

    #[test]
    fn test_luminary_state_queries() {
        let face_down = LuminaryState::FaceDown(LuminaryCard::TheForestQueen);
        let face_up = LuminaryState::FaceUp(LuminaryCard::TheRiver);
        let claimed = LuminaryState::Claimed(LuminaryCard::TheChildren, PlayerId(1));
        let none = LuminaryState::None;

        // Test card extraction
        assert_eq!(face_down.card(), Some(LuminaryCard::TheForestQueen));
        assert_eq!(face_up.card(), Some(LuminaryCard::TheRiver));
        assert_eq!(claimed.card(), Some(LuminaryCard::TheChildren));
        assert_eq!(none.card(), None);

        // Test activity
        assert!(!face_down.is_active());
        assert!(face_up.is_active());
        assert!(claimed.is_active());
        assert!(!none.is_active());

        // Test claimability
        assert!(!face_down.can_be_claimed());
        assert!(face_up.can_be_claimed());
        assert!(!claimed.can_be_claimed());
        assert!(!none.can_be_claimed());

        // Test player ownership
        assert_eq!(claimed.is_claimed_by(PlayerId(1)), Some(LuminaryCard::TheChildren));
        assert_eq!(claimed.is_claimed_by(PlayerId(0)), None);
        assert_eq!(face_up.is_claimed_by(PlayerId(1)), None);
    }

    #[test]
    fn test_default_modifier_behavior() {
        let modifier = DefaultLuminaryModifier;
        let empty_states = [LuminaryState::None; 4];

        // Test base season capabilities (no Luminary modifications)
        let spring_caps = modifier.modify_capabilities(
            FieldId(0), 
            Season::Spring, 
            &empty_states, 
            0
        );
        assert!(spring_caps.can_sow);
        assert!(spring_caps.can_harvest);
        assert!(!spring_caps.can_stockpile); // Spring blocks stockpiling

        let winter_caps = modifier.modify_capabilities(
            FieldId(0), 
            Season::Winter, 
            &empty_states, 
            0
        );
        assert!(winter_caps.can_sow);
        assert!(!winter_caps.can_harvest); // Winter blocks harvesting  
        assert!(winter_caps.can_stockpile);

        // Test no action modifications
        let sow_action = crate::game::actions::Action::Sow { 
            field: FieldId(0), 
            card: Card::new(crate::game::card::Rank::Five, crate::game::card::Suit::Spring)
        };
        assert_eq!(
            modifier.modify_action_resolution(&sow_action, &empty_states, &crate::game::state::IllimatState::new_test_game()),
            ActionModification::Normal
        );
    }

    #[test]
    fn test_display_names() {
        assert_eq!(LuminaryCard::TheForestQueen.display_name(), "The Forest Queen");
        assert_eq!(LuminaryCard::TheDrought.display_name(), "The Drought");
        assert_eq!(LuminaryCard::TheIsland.display_name(), "The Island");
        assert_eq!(LuminaryCard::TheRusalka.display_name(), "The Rusalka");
    }

    #[test]
    fn test_serde_serialization() {
        // Test LuminaryCard serialization
        let card = LuminaryCard::TheForestQueen;
        let serialized = serde_json::to_string(&card).expect("Should serialize");
        let deserialized: LuminaryCard = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(card, deserialized);

        // Test LuminaryState serialization
        let state = LuminaryState::Claimed(LuminaryCard::TheRiver, PlayerId(2));
        let serialized = serde_json::to_string(&state).expect("Should serialize");
        let deserialized: LuminaryState = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(state, deserialized);

        // Test LuminaryConfiguration serialization
        let config = LuminaryConfiguration::all_expansions();
        let serialized = serde_json::to_string(&config).expect("Should serialize");
        let deserialized: LuminaryConfiguration = serde_json::from_str(&serialized).expect("Should deserialize");
        assert_eq!(config, deserialized);
    }
}