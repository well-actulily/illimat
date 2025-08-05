/// TICKET-BUG-SETUP Verification Test
/// Testing correct hand dealing and turn order implementation

use illimat_core::game::state::IllimatState;
use illimat_core::game::game_config::GameConfig;
use illimat_core::game::player::PlayerId;

#[test]
fn test_correct_dealing_rules_2_players() {
    // Test multiple games to ensure consistent behavior across different dealer assignments
    for _ in 0..20 {  // Test multiple random dealer assignments
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        
        let dealer = state.dealer;
        let first_player = PlayerId((dealer.0 + 1) % 2);
        
        // First player (dealer's left) should have 3 cards
        assert_eq!(state.player_hands[first_player.0 as usize].len(), 3,
                  "First player {} should have 3 cards (dealer={})", first_player.0, dealer.0);
        
        // Dealer should have 4 cards  
        assert_eq!(state.player_hands[dealer.0 as usize].len(), 4,
                  "Dealer {} should have 4 cards", dealer.0);
        
        // First player should be current player
        assert_eq!(state.current_player, first_player,
                  "First player {} should be current player (dealer={})", first_player.0, dealer.0);
        
        // Total cards dealt should be 7 (3 + 4)
        let total_hand_cards = state.player_hands[0].len() + state.player_hands[1].len();
        assert_eq!(total_hand_cards, 7, "Total hand cards should be 7");
    }
}

#[test]
fn test_correct_dealing_rules_4_players() {
    for _ in 0..10 {  // Test multiple scenarios
        let config = GameConfig::new(4);
        let state = IllimatState::new(config);
        
        let dealer = state.dealer;
        let first_player = PlayerId((dealer.0 + 1) % 4);
        
        // Check each player's hand size
        for player_id in 0..4 {
            let expected_cards = if PlayerId(player_id) == first_player { 3 } else { 4 };
            assert_eq!(state.player_hands[player_id as usize].len(), expected_cards,
                      "Player {} should have {} cards (dealer={}, first_player={})", 
                      player_id, expected_cards, dealer.0, first_player.0);
        }
        
        // First player should be current player
        assert_eq!(state.current_player, first_player,
                  "First player {} should be current player", first_player.0);
        
        // Total cards in hands should be 15 (3 + 4 + 4 + 4)
        let total_hand_cards: usize = state.player_hands.iter().map(|h| h.len()).sum();
        assert_eq!(total_hand_cards, 15, "Total hand cards should be 15");
    }
}

#[test]
fn test_turn_order_progression() {
    let config = GameConfig::new(3);
    let state = IllimatState::new(config);
    
    let dealer = state.dealer;
    let first_player = state.current_player;
    
    // Verify first player is dealer's left
    let expected_first = PlayerId((dealer.0 + 1) % 3);
    assert_eq!(first_player, expected_first, "First player should be dealer's left");
    
    // Verify first player has 3 cards
    assert_eq!(state.player_hands[first_player.0 as usize].len(), 3,
              "First player should have 3 cards");
    
    println!("✅ TICKET-BUG-SETUP: Correct dealing and turn order verified");
    println!("   Dealer: {}, First Player: {}, Hand sizes: {:?}", 
             dealer.0, first_player.0, 
             state.player_hands.iter().map(|h| h.len()).collect::<Vec<_>>());
}