use illimat_core::game::state::IllimatState;
use illimat_core::game::actions::Action;
use illimat_core::game::card::{Card, Rank, Suit};
use illimat_core::game::field_id::FieldId;
use illimat_core::game::player::PlayerId;
use illimat_core::game::game_config::GameConfig;

/// Simple test for TICKET-BUG-001 fix: Harvest Auto-Collection
#[test]
fn test_harvest_auto_collection_fix_simple() {
    let config = GameConfig::new(2);
    let mut state = IllimatState::new(config);
    let player = state.current_player;
    let field = FieldId(0);
    
    // Use 5s for this test - easy to understand
    let played_5 = Card::new(Rank::Five, Suit::Summer);
    let field_5_spring = Card::new(Rank::Five, Suit::Spring);
    let field_5_winter = Card::new(Rank::Five, Suit::Winter);
    let field_3 = Card::new(Rank::Three, Suit::Autumn);
    
    // Set up the exact scenario we want to test
    state.player_hands[player.0 as usize].clear();
    state.player_hands[player.0 as usize].push(played_5);
    
    state.field_cards[field.0 as usize].clear();
    state.field_cards[field.0 as usize].push(field_5_spring);
    state.field_cards[field.0 as usize].push(field_5_winter);
    state.field_cards[field.0 as usize].push(field_3);
    
    println!("SIMPLE AUTO-COLLECTION TEST");
    println!("Player plays: {:?} (value 5)", played_5);
    println!("Field has: {:?} (values: 5, 5, 3)", state.field_cards[field.0 as usize]);
    
    // Test auto-collection (empty targets)
    let auto_harvest = Action::Harvest { 
        field, 
        card: played_5, 
        targets: vec![] // Should auto-collect both 5s
    };
    
    let initial_hand_size = state.player_hands[player.0 as usize].len();
    let initial_field_size = state.field_cards[field.0 as usize].len();
    
    match state.apply_action(auto_harvest) {
        Ok(_) => {
            println!("✅ AUTO-COLLECTION SUCCESS!");
            println!("Player harvest: {:?}", state.player_harvests[player.0 as usize]);
            println!("Remaining field: {:?}", state.field_cards[field.0 as usize]);
            
            // Verify auto-collection worked correctly
            let harvested = &state.player_harvests[player.0 as usize];
            
            // Should have: played card + both field 5s = 3 cards total
            assert_eq!(harvested.len(), 3, "Should have harvested 3 cards (played + 2 field)");
            assert!(harvested.contains(&played_5), "Should have played card");
            assert!(harvested.contains(&field_5_spring), "Should have field 5 Spring");
            assert!(harvested.contains(&field_5_winter), "Should have field 5 Winter");
            
            // Field should only have the 3 (non-matching card)
            assert_eq!(state.field_cards[field.0 as usize].len(), 1, "Field should have 1 card left");
            assert!(state.field_cards[field.0 as usize].contains(&field_3), "Field should have the 3");
            
            // With draw-back-to-4 implemented, player should have 4 cards (played 1, drew 1 back)
            let expected_hand_size = if state.deck.len() > 0 { 4 } else { initial_hand_size - 1 };
            assert_eq!(state.player_hands[player.0 as usize].len(), expected_hand_size, 
                      "Player should have drawn back to 4 cards (or fewer if deck exhausted)");
            
            println!("✅ All assertions passed! Auto-collection is working correctly.");
            
        },
        Err(e) => {
            panic!("❌ HARVEST AUTO-COLLECTION FAILED: {}", e);
        }
    }
}

/// Test that manual selection still works
#[test] 
fn test_manual_selection_still_works() {
    let config = GameConfig::new(2);
    let mut state = IllimatState::new(config);
    let player = state.current_player;
    let field = FieldId(0);
    
    // Set up manual selection scenario - playing 7 to collect 3+4
    let played_7 = Card::new(Rank::Seven, Suit::Summer);
    let field_3 = Card::new(Rank::Three, Suit::Spring);
    let field_4 = Card::new(Rank::Four, Suit::Winter);
    let field_2 = Card::new(Rank::Two, Suit::Autumn);
    
    state.player_hands[player.0 as usize].clear(); 
    state.player_hands[player.0 as usize].push(played_7);
    
    state.field_cards[field.0 as usize].clear();
    state.field_cards[field.0 as usize].push(field_3);
    state.field_cards[field.0 as usize].push(field_4);
    state.field_cards[field.0 as usize].push(field_2);
    
    println!("MANUAL SELECTION TEST");
    println!("Player plays: {:?} (value 7)", played_7);
    println!("Field has: 3, 4, 2 - multiple ways to sum to 7");
    
    // Manual selection: 3+4=7
    let manual_harvest = Action::Harvest { 
        field, 
        card: played_7, 
        targets: vec![field_3, field_4] 
    };
    
    match state.apply_action(manual_harvest) {
        Ok(_) => {
            println!("✅ MANUAL SELECTION SUCCESS!");
            println!("Player harvest: {:?}", state.player_harvests[player.0 as usize]);
            println!("Remaining field: {:?}", state.field_cards[field.0 as usize]);
            
            // Should have harvested: played 7 + 3 + 4 = 3 cards
            assert_eq!(state.player_harvests[player.0 as usize].len(), 3);
            assert!(state.player_harvests[player.0 as usize].contains(&played_7));
            assert!(state.player_harvests[player.0 as usize].contains(&field_3));
            assert!(state.player_harvests[player.0 as usize].contains(&field_4));
            
            // Field should only have the 2
            assert_eq!(state.field_cards[field.0 as usize].len(), 1);
            assert!(state.field_cards[field.0 as usize].contains(&field_2));
            
            println!("✅ Manual selection working correctly!");
            
        },
        Err(e) => {
            panic!("❌ MANUAL SELECTION FAILED: {}", e);
        }
    }
}