/// Field Clearing and Okus Collection Implementation Test
/// Testing that okus tokens are correctly collected when fields are cleared

use illimat_core::game::state::IllimatState;
use illimat_core::game::actions::Action;
use illimat_core::game::card::{Card, Rank, Suit};
use illimat_core::game::field_id::FieldId;
use illimat_core::game::player::PlayerId;
use illimat_core::game::game_config::GameConfig;
use illimat_core::game::okus::{OkusPosition, OkusManager};

#[test]
fn test_okus_collection_on_field_clearing() {
    let config = GameConfig::new(2);
    let mut state = IllimatState::new(config);
    let player = state.current_player;
    let field = FieldId(0);
    
    // Set up a simple field clearing scenario
    // Player plays a 5, field has exactly one 5 card
    let played_card = Card::new(Rank::Five, Suit::Summer);
    let field_card = Card::new(Rank::Five, Suit::Spring);
    
    // Clear the field and set up our test scenario
    state.field_cards[field.0 as usize].clear();
    state.field_cards[field.0 as usize].push(field_card);
    
    // Ensure player has the card they want to play
    state.player_hands[player.0 as usize].clear();
    state.player_hands[player.0 as usize].push(played_card);
    
    // Verify all okus start on Illimat
    let initial_okus = OkusManager::get_available_okus(&state.okus_positions);
    assert_eq!(initial_okus.len(), 4, "All okus should start on Illimat");
    
    // Verify player has no okus initially
    let player_okus_before = OkusManager::count_player_okus(&state.okus_positions, player);
    assert_eq!(player_okus_before, 0, "Player should have no okus initially");
    
    println!("Before harvest: Field has {} cards, Player has {} okus", 
             state.field_cards[field.0 as usize].len(), player_okus_before);
    
    // Perform harvest that should clear the field (5 harvests 5)
    let harvest_action = Action::Harvest { 
        field, 
        card: played_card, 
        targets: vec![] // Auto-collect the matching 5
    };
    
    let field_cleared = state.apply_action(harvest_action).expect("Harvest should succeed");
    
    // Verify field was cleared
    assert!(field_cleared, "Field should have been cleared");
    assert!(state.field_cards[field.0 as usize].is_empty(), "Field should be empty after clearing");
    
    // Verify okus were collected
    let player_okus_after = OkusManager::count_player_okus(&state.okus_positions, player);
    assert_eq!(player_okus_after, 4, "Player should have collected all 4 okus");
    
    // Verify no okus remain on Illimat
    let remaining_okus = OkusManager::get_available_okus(&state.okus_positions);
    assert_eq!(remaining_okus.len(), 0, "No okus should remain on Illimat");
    
    println!("After harvest: Field cleared = {}, Player has {} okus", 
             field_cleared, player_okus_after);
    
    println!("✅ OKUS COLLECTION: Field clearing correctly awards okus to player");
}

#[test]
fn test_no_okus_collection_when_field_not_cleared() {
    let config = GameConfig::new(2);
    let mut state = IllimatState::new(config);
    let player = state.current_player;
    let field = FieldId(0);
    
    // Set up scenario where field is NOT cleared
    // Player plays a 7, field has a 3 and a 4 (3+4=7) but also has other cards
    let played_card = Card::new(Rank::Seven, Suit::Summer);
    let target1 = Card::new(Rank::Three, Suit::Spring);  
    let target2 = Card::new(Rank::Four, Suit::Winter);
    let remaining_card = Card::new(Rank::Two, Suit::Autumn); // This prevents field clearing
    
    // Set up field with targets and extra card
    state.field_cards[field.0 as usize].clear();
    state.field_cards[field.0 as usize].push(target1);
    state.field_cards[field.0 as usize].push(target2);
    state.field_cards[field.0 as usize].push(remaining_card);
    
    // Ensure player has the card they want to play
    state.player_hands[player.0 as usize].clear();
    state.player_hands[player.0 as usize].push(played_card);
    
    // Verify player has no okus initially
    let player_okus_before = OkusManager::count_player_okus(&state.okus_positions, player);
    assert_eq!(player_okus_before, 0, "Player should have no okus initially");
    
    // Perform harvest that does NOT clear the field (7 harvests 3+4, leaving 2)
    let harvest_action = Action::Harvest { 
        field, 
        card: played_card, 
        targets: vec![target1, target2] // Manual selection of 3+4=7
    };
    
    let field_cleared = state.apply_action(harvest_action).expect("Harvest should succeed");
    
    // Verify field was NOT cleared
    assert!(!field_cleared, "Field should NOT have been cleared");
    assert_eq!(state.field_cards[field.0 as usize].len(), 1, "Field should still have the remaining card");
    
    // Verify no okus were collected
    let player_okus_after = OkusManager::count_player_okus(&state.okus_positions, player);
    assert_eq!(player_okus_after, 0, "Player should have no okus when field not cleared");
    
    // Verify all okus remain on Illimat
    let remaining_okus = OkusManager::get_available_okus(&state.okus_positions);
    assert_eq!(remaining_okus.len(), 4, "All okus should remain on Illimat");
    
    println!("✅ NO OKUS COLLECTION: Field not cleared, no okus awarded");
}

#[test]
fn test_okus_scoring_integration() {
    let config = GameConfig::new(2);
    let mut state = IllimatState::new(config);
    let player = state.current_player;
    let field = FieldId(0);
    
    // Set up field clearing scenario
    let played_card = Card::new(Rank::King, Suit::Summer);
    let field_card = Card::new(Rank::King, Suit::Spring);
    
    state.field_cards[field.0 as usize].clear();
    state.field_cards[field.0 as usize].push(field_card);
    
    state.player_hands[player.0 as usize].clear();
    state.player_hands[player.0 as usize].push(played_card);
    
    // Clear field and collect okus
    let harvest_action = Action::Harvest { 
        field, 
        card: played_card, 
        targets: vec![] 
    };
    
    let field_cleared = state.apply_action(harvest_action).expect("Harvest should succeed");
    assert!(field_cleared, "Field should be cleared");
    
    // Verify okus were collected
    let player_okus = OkusManager::count_player_okus(&state.okus_positions, player);
    assert_eq!(player_okus, 4, "Player should have all 4 okus");
    
    // End the round to trigger scoring
    let scoring = state.end_round();
    
    // Verify okus contribute to player's score
    // Each okus is worth +1 point, so player should get at least 4 points from okus
    let player_score = scoring.individual_scores[player.0 as usize];
    assert!(player_score >= 4, "Player should get at least 4 points from okus");
    
    // Start new round to reset okus
    state.start_new_round();
    
    // Verify okus are reset for next round
    let okus_after_round = OkusManager::get_available_okus(&state.okus_positions);
    assert_eq!(okus_after_round.len(), 4, "All okus should be back on Illimat for next round");
    
    println!("✅ OKUS SCORING: Field clearing okus contribute {} points to player score", 
             player_score);
}