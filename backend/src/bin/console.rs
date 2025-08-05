use illimat_core::game::state::IllimatState;
use illimat_core::game::player::{PlayerType, PlayerId};
use illimat_core::game::field_id::FieldId;
use illimat_core::game::game_config::GameConfig;
use illimat_core::game::actions::Action;
use std::io::{self, Write};
use std::fs;
use std::path::Path;
use colored::*;
use chrono;
use serde_json;

/// Enhanced save data structure with metadata and statistics
#[derive(serde::Serialize, serde::Deserialize)]
struct EnhancedSaveData {
    metadata: SaveMetadata,
    game_state: IllimatState,
    statistics: GameStatistics,
}

/// Save file metadata
#[derive(serde::Serialize, serde::Deserialize)]
struct SaveMetadata {
    version: String,
    created: chrono::DateTime<chrono::Utc>,
    description: String,
}

/// Game statistics for tracking
#[derive(serde::Serialize, serde::Deserialize)]
struct GameStatistics {
    total_turns: u16,
    turns_per_round: Vec<u16>,
    average_turns_per_round: f64,
    player_actions: [PlayerActionStats; 4],
}

/// Per-player action statistics
#[derive(Copy, Clone, serde::Serialize, serde::Deserialize)]
struct PlayerActionStats {
    sows: u16,
    harvests: u16,
    stockpiles: u16,
    cards_harvested: u16,
    field_clears: u16,
}

impl Default for PlayerActionStats {
    fn default() -> Self {
        Self {
            sows: 0,
            harvests: 0,
            stockpiles: 0,
            cards_harvested: 0,
            field_clears: 0,
        }
    }
}

impl GameStatistics {
    /// Create statistics from current game state
    fn from_state(game: &IllimatState) -> Self {
        // For now, create basic stats. In a full implementation, we'd track these throughout the game
        let turns_this_round = game.turn_number;
        let turns_per_round = if game.round_number > 1 {
            // Estimate based on current progress
            vec![turns_this_round / game.round_number as u16; game.round_number as usize]
        } else {
            vec![turns_this_round]
        };
        
        let average_turns = if turns_per_round.is_empty() {
            0.0
        } else {
            turns_per_round.iter().sum::<u16>() as f64 / turns_per_round.len() as f64
        };
        
        Self {
            total_turns: game.turn_number,
            turns_per_round,
            average_turns_per_round: average_turns,
            player_actions: [PlayerActionStats::default(); 4],
        }
    }
}

/// Action history entry for undo functionality
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct ActionHistoryEntry {
    state_before: IllimatState,
    action_taken: Action,
    player: PlayerId,
    timestamp: chrono::DateTime<chrono::Utc>,
    description: String,
}

/// Undo history manager
struct UndoManager {
    history: Vec<ActionHistoryEntry>,
    max_history: usize,
}

impl UndoManager {
    fn new() -> Self {
        Self {
            history: Vec::new(),
            max_history: 10, // Keep last 10 actions for undo
        }
    }
    
    fn record_action(&mut self, state_before: &IllimatState, action: &Action, player: PlayerId) {
        let description = match action {
            Action::Sow { field, card } => format!("Player {} sowed {} to {:?}", player.0, card, field),
            Action::Harvest { field, card, targets } => format!("Player {} harvested {} cards with {} from {:?}", player.0, targets.len(), card, field),
            Action::Stockpile { field, card, targets } => format!("Player {} stockpiled {} with {} targets in {:?}", player.0, card, targets.len(), field),
        };
        
        let entry = ActionHistoryEntry {
            state_before: state_before.clone(),
            action_taken: action.clone(),
            player,
            timestamp: chrono::Utc::now(),
            description,
        };
        
        self.history.push(entry);
        
        // Maintain max history size
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }
    
    fn undo(&mut self) -> Option<(IllimatState, String)> {
        if let Some(entry) = self.history.pop() {
            Some((entry.state_before, entry.description))
        } else {
            None
        }
    }
    
    fn show_history(&self) {
        if self.history.is_empty() {
            println!("📜 No action history available");
            return;
        }
        
        println!("📜 Recent actions (most recent first):");
        for (i, entry) in self.history.iter().rev().enumerate() {
            println!("  {}: {} ({})", 
                i + 1, 
                entry.description,
                entry.timestamp.format("%H:%M:%S")
            );
        }
    }
}

fn main() {
    // Check for color support and enable if available
    if std::env::var("NO_COLOR").is_err() {
        colored::control::set_override(true);
    }
    
    // Display occult ASCII art
    print_occult_banner();
    
    // Create game through setup process
    let mut game = create_new_game();
    
    // Initialize undo manager for development aid
    let mut undo_manager = UndoManager::new();
    
    println!("Game initialized with {} players", game.config.player_count);
    println!("Dealer: Player {}", game.dealer.0);
    println!("First player: Player {}", game.current_player.0);
    println!("Deck: {} cards", game.deck.len());
    println!();
    
    // Main game loop
    loop {
        // Check if round is over
        if game.should_end_round() {
            println!("=== ROUND {} COMPLETE ===", game.round_number);
            println!("Deck exhausted. Calculating scores...");
            println!();
            
            let scoring = game.end_round();
            
            if let Some(winner) = game.get_winner() {
                println!("🎉 GAME OVER! Player {} wins with {} points!", 
                         winner.0, game.total_scores[winner.0 as usize]);
                println!();
                println!("Final scores:");
                for i in 0..game.config.player_count {
                    println!("  Player {}: {} points", i, game.total_scores[i as usize]);
                }
                break;
            } else {
                println!("Round complete! Current scores:");
                for i in 0..game.config.player_count {
                    println!("  Player {}: {} points", i, game.total_scores[i as usize]);
                }
                println!();
                
                // Display scoring details
                println!("Round {} scoring:", game.round_number - 1);
                use illimat_core::game::scoring::ScoringManager;
                println!("{}", ScoringManager::format_round_scoring(&scoring));
                println!();
                
                // Start new round
                println!("Starting round {}...", game.round_number + 1);
                game.start_new_round();
                println!("Round {} dealt!", game.round_number);
                println!();
                continue;
            }
        }
        
        // Clear screen option for better UX (optional)
        if should_clear_screen() {
            clear_screen();
        }
        
        // Display current game state
        println!("{}", game);
        println!();
        
        // Enhanced player input prompts
        println!("{}", format!("🎮 Player {}'s turn:", game.current_player.0).bright_yellow().bold());
        show_available_actions_hint(&game);
        print!("{} ", "Choose action >".bright_yellow().bold());
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached, quit gracefully
                println!("EOF reached. Exiting game.");
                break;
            }
            Ok(_) => {}
            Err(_) => {
                println!("Error reading input. Exiting game.");
                break;
            }
        }
        let input = input.trim().to_lowercase();
        
        // Handle empty input with enhanced message
        if input.is_empty() {
            println!("{}", "❓ Empty input detected.".bright_yellow());
            println!("Available commands: {}, {}, {}, {} or {} for help", 
                     "s".bright_green(), "h".bright_blue(), "t".bright_cyan(), "q".bright_red(), "?".bright_magenta());
            continue;
        }
        
        match input.as_str() {
            "q" | "quit" => {
                if confirm_quit() {
                    println!("{}", "Thanks for playing Illimat! 🎴".bright_green().bold());
                    break;
                }
            }
            "?" | "help" => {
                show_help(&game);
            }
            "s" | "sow" => {
                if let Err(e) = handle_sow_action(&mut game, &mut undo_manager) {
                    println!("Sow failed: {}", e);
                }
            }
            "h" | "harvest" => {
                if let Err(e) = handle_harvest_action(&mut game, &mut undo_manager) {
                    println!("Harvest failed: {}", e);
                }
            }
            "t" | "stockpile" => {
                if let Err(e) = handle_stockpile_action(&mut game, &mut undo_manager) {
                    println!("Stockpile failed: {}", e);
                }
            }
            "u" | "undo" => {
                if let Some((restored_state, description)) = undo_manager.undo() {
                    game = restored_state;
                    println!("{}", format!("⏪ Undid: {}", description).bright_yellow().bold());
                } else {
                    println!("❌ No actions to undo");
                }
            }
            "i" | "history" => {
                undo_manager.show_history();
            }
            "e" | "export" => {
                if let Err(e) = handle_export_game_state(&game) {
                    println!("Export failed: {}", e);
                } else {
                    println!("{}", "Game state exported successfully! 📊".bright_green().bold());
                }
            }
            "v" | "save" => {
                if let Err(e) = handle_save_game(&game) {
                    println!("Save failed: {}", e);
                } else {
                    println!("{}", "Game saved successfully! 💾".bright_green().bold());
                }
            }
            "l" | "load" => {
                match handle_load_game() {
                    Ok(loaded_game) => {
                        game = loaded_game;
                        println!("{}", "Game loaded successfully! 📂".bright_green().bold());
                    }
                    Err(e) => {
                        println!("Load failed: {}", e);
                    }
                }
            }
            _ => {
                println!("{} {}", "❌ Invalid command:".bright_red(), input.bright_white());
                show_available_actions_hint(&game);
                println!("💡 Type {} for help or {} to quit", "?".bright_magenta(), "q".bright_red());
            }
        }
        
        println!();
    }
}

/// Show contextual help for game actions
fn show_help(game: &IllimatState) {
    println!("{}", "=== ILLIMAT HELP ===".bright_cyan().bold());
    println!();
    
    // Basic actions
    println!("{}", "📋 AVAILABLE ACTIONS:".bright_white().bold());
    println!("  {} - Play a card to a field", "s/sow".bright_green());
    println!("  {} - Collect cards from a field that sum to your card's value", "h/harvest".bright_blue());
    println!("  {} - Combine active + passive cards into a stockpile", "t/stockpile".bright_cyan());
    println!("  {} - Undo last action (development aid)", "u/undo".bright_cyan());
    println!("  {} - Show recent action history", "i/history".bright_magenta());
    println!("  {} - Export human-readable game state report", "e/export".bright_white());
    println!("  {} - Save current game to file", "v/save".bright_yellow());
    println!("  {} - Load saved game from file", "l/load".bright_white());
    println!("  {} - Show this help text", "?/help".bright_magenta());
    println!("  {} - Exit the game", "q/quit".bright_red());
    println!();
    
    // Current season restrictions with enhanced visuals
    println!("{}", "🌟 CURRENT SEASON RESTRICTIONS:".bright_white().bold());
    for (i, season) in game.field_seasons.iter().enumerate() {
        let field_name = FieldId(i as u8).seasonal_name(game.illimat_orientation);
        print!("  {} ({}): ", field_name.bright_white().bold(), season);
        match season {
            illimat_core::game::season::Season::Spring => println!("{}", "❌ No stockpiling allowed".bright_red()),
            illimat_core::game::season::Season::Summer => println!("{}", "✅ All actions allowed".bright_green()),
            illimat_core::game::season::Season::Autumn => println!("{}", "❌ No sowing allowed".bright_red()),
            illimat_core::game::season::Season::Winter => println!("{}", "❌ No harvesting allowed".bright_red()),
        }
    }
    println!();
    
    // Context-aware current situation
    show_current_situation_help(game);
    
    // Enhanced game tips
    println!("{}", "💡 STRATEGIC TIPS:".bright_white().bold());
    println!("  {} Fools can be played as value 1 or 14 for harvesting", "•".bright_yellow());
    println!("  {} Face cards (Fool, Knight, Queen, King) rotate the Illimat", "•".bright_yellow());
    println!("  {} Clear a field by harvesting all cards to collect okus tokens", "•".bright_yellow());
    println!("  {} Stockpiles created this turn cannot be harvested until next turn", "•".bright_yellow());
    println!("  {} Sowing to crowded fields makes more harvest combinations available", "•".bright_yellow());
    println!("  {} First to 17+ points wins the game!", "•".bright_yellow());
    
    // Victory conditions
    println!();
    println!("{}", "🏆 VICTORY CONDITIONS:".bright_white().bold());
    println!("  {} Reach 17 or more points to win", "•".bright_cyan());
    println!("  {} Points from: Bumper Crop (+4), Sunkissed (+2), Frostbit (-2)", "•".bright_cyan());
    println!("  {} Each Fool, Luminary, and Okus token = +1 point", "•".bright_cyan());
    println!();
}

/// Show context-aware help for current game situation
fn show_current_situation_help(game: &IllimatState) {
    let current_player = game.current_player;
    let hand = &game.player_hands[current_player.0 as usize];
    
    println!("{}", "🎯 CURRENT SITUATION:".bright_white().bold());
    
    // Hand analysis
    if hand.is_empty() {
        println!("  {} You have no cards - round will end soon!", "⚠️".bright_yellow());
        return;
    }
    
    println!("  {} You have {} cards in hand", "👋".bright_green(), hand.len());
    
    // Score analysis
    let current_score = game.total_scores[current_player.0 as usize];
    let leader_score = *game.total_scores.iter().max().unwrap();
    
    if current_score >= 17 {
        println!("  {} You have {} points - you can win this round!", "🎉".bright_green(), current_score);
    } else if current_score == leader_score {
        println!("  {} You're leading with {} points (need {})", "📈".bright_cyan(), current_score, 17 - current_score);
    } else {
        println!("  {} You have {} points (leader has {}, need {})", "📊".bright_yellow(), current_score, leader_score, 17 - current_score);
    }
    
    // Available actions analysis
    let mut recommended_actions = Vec::new();
    
    // Check for immediate harvest opportunities
    for (i, field_cards) in game.field_cards.iter().enumerate() {
        if !field_cards.is_empty() {
            let field_id = FieldId(i as u8);
            let can_harvest = illimat_core::game::capabilities::CapabilityManager::can_harvest(field_id, game.illimat_orientation);
            if can_harvest {
                // Check if any card in hand can harvest from this field
                for &hand_card in hand {
                    let harvest_combos = find_harvest_combinations(field_cards, &game.field_stockpiles[i], hand_card);
                    if !harvest_combos.is_empty() {
                        recommended_actions.push(format!("Harvest from {} field with {}", 
                            field_id.seasonal_name(game.illimat_orientation), hand_card));
                        break;
                    }
                }
            }
        }
    }
    
    // Check for stockpile opportunities
    if hand.len() >= 2 {
        for (i, field_cards) in game.field_cards.iter().enumerate() {
            if !field_cards.is_empty() {
                let field_id = FieldId(i as u8);
                let can_stockpile = illimat_core::game::capabilities::CapabilityManager::can_stockpile(field_id, game.illimat_orientation);
                if can_stockpile {
                    recommended_actions.push(format!("Create stockpile in {} field", 
                        field_id.seasonal_name(game.illimat_orientation)));
                    break;
                }
            }
        }
    }
    
    if !recommended_actions.is_empty() {
        println!("  {} Suggested actions:", "💭".bright_cyan());
        for action in recommended_actions.iter().take(3) { // Show max 3 suggestions
            println!("    - {}", action.bright_white());
        }
    }
    
    println!();
}

/// Show smart action hints based on current game state
fn show_available_actions_hint(game: &IllimatState) {
    let current_player = game.current_player;
    let hand = &game.player_hands[current_player.0 as usize];
    
    if hand.is_empty() {
        println!("{}", "⚠️  No cards remaining - round ending soon!".bright_yellow());
        println!("Actions: {}uit or {}elp", "(q)".bright_red(), "(?h)".bright_magenta());
        return;
    }
    
    let mut available_actions = Vec::new();
    let mut smart_suggestions = Vec::new();
    
    // Always available actions
    available_actions.push(format!("{}ow", "(s)".bright_green()));
    
    // Check harvest opportunities
    let mut can_harvest = false;
    for (i, field_cards) in game.field_cards.iter().enumerate() {
        if !field_cards.is_empty() {
            let field_id = FieldId(i as u8);
            if illimat_core::game::capabilities::CapabilityManager::can_harvest(field_id, game.illimat_orientation) {
                for &hand_card in hand {
                    let harvest_combos = find_harvest_combinations(field_cards, &game.field_stockpiles[i], hand_card);
                    if !harvest_combos.is_empty() {
                        can_harvest = true;
                        smart_suggestions.push(format!("harvest with {}", hand_card));
                        break;
                    }
                }
                if can_harvest { break; }
            }
        }
    }
    
    if can_harvest {
        available_actions.push(format!("{}arvest", "(h)".bright_blue()));
    }
    
    // Check stockpile opportunities
    if hand.len() >= 2 {
        let mut can_stockpile = false;
        for (i, field_cards) in game.field_cards.iter().enumerate() {
            if !field_cards.is_empty() {
                let field_id = FieldId(i as u8);
                if illimat_core::game::capabilities::CapabilityManager::can_stockpile(field_id, game.illimat_orientation) {
                    can_stockpile = true;
                    break;
                }
            }
        }
        if can_stockpile {
            available_actions.push(format!("s{}ockpile", "(t)".bright_cyan()));
        }
    }
    
    // Add utility actions
    available_actions.extend_from_slice(&[
        format!("{}ndo", "(u)".bright_cyan()),
        format!("{}elp", "(?h)".bright_magenta()),
        format!("{}uit", "(q)".bright_red())
    ]);
    
    println!("Actions: {}", available_actions.join(", "));
    
    if !smart_suggestions.is_empty() {
        println!("{} Try: {}", "💡".bright_cyan(), smart_suggestions.join(" or ").bright_white());
    }
}

/// Handle stockpile action with user input
fn handle_stockpile_action(game: &mut IllimatState, undo_manager: &mut UndoManager) -> Result<(), String> {
    let current_player = game.current_player;
    let hand = &game.player_hands[current_player.0 as usize];
    
    // Check if player has at least 2 cards (active + passive)
    if hand.len() < 2 {
        return Err("You need at least 2 cards to stockpile (active + passive)!".to_string());
    }
    
    // Show player's hand with indices
    println!("Your hand:");
    for (i, card) in hand.iter().enumerate() {
        let values = if card.rank() == illimat_core::game::card::Rank::Fool {
            "1 or 14".to_string()
        } else {
            get_card_value(*card).to_string()
        };
        println!("  {}: {} (value: {})", i + 1, card, values);
    }
    
    // Get active card selection
    print!("Select ACTIVE card to play (1-{}): ", hand.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let active_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid card number")?;
    
    if active_index == 0 || active_index > hand.len() {
        return Err("Invalid active card selection".to_string());
    }
    
    let active_card = hand[active_index - 1];
    
    // Get passive card selection (must be different from active)
    println!("\nSelect PASSIVE card (determines stockpile value):");
    for (i, card) in hand.iter().enumerate() {
        if i + 1 == active_index {
            continue; // Skip active card
        }
        let values = if card.rank() == illimat_core::game::card::Rank::Fool {
            "1 or 14".to_string()
        } else {
            get_card_value(*card).to_string()
        };
        println!("  {}: {} (value: {})", i + 1, card, values);
    }
    
    print!("Select PASSIVE card (1-{}, excluding {}): ", hand.len(), active_index);
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let passive_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid card number")?;
    
    if passive_index == 0 || passive_index > hand.len() || passive_index == active_index {
        return Err("Invalid passive card selection (must be different from active)".to_string());
    }
    
    let _passive_card = hand[passive_index - 1];
    
    // Show fields with stockpile possibilities
    println!("\nFields:");
    let field_names = ["Spring", "Summer", "Autumn", "Winter"];
    let mut valid_fields = Vec::new();
    
    for (i, field_name) in field_names.iter().enumerate() {
        let field_id = FieldId(i as u8);
        let season = &game.field_seasons[i];
        let field_cards = &game.field_cards[i];
        let can_stockpile = illimat_core::game::capabilities::CapabilityManager::can_stockpile(field_id, game.illimat_orientation);
        
        print!("  {}: {} Field ({}) - {} cards", 
               i + 1, field_name, season, field_cards.len());
        
        if !can_stockpile {
            print!(" [STOCKPILE BLOCKED]");
        } else if !field_cards.is_empty() {
            // Check if we can make valid stockpile combinations
            let combinations = find_stockpile_combinations(&field_cards, &game.field_stockpiles[i], active_card);
            if !combinations.is_empty() {
                print!(" [CAN STOCKPILE]");
                valid_fields.push(i);
            } else {
                print!(" [NO VALID COMBINATIONS]");
            }
        }
        println!();
        
        // Show field cards
        if !field_cards.is_empty() {
            print!("     Cards: ");
            for (j, card) in field_cards.iter().enumerate() {
                if j > 0 { print!(", "); }
                print!("{}", card);
            }
            println!();
        }
    }
    
    if valid_fields.is_empty() {
        return Err("No valid stockpile combinations available in any field".to_string());
    }
    
    // Get field selection
    print!("Select field to stockpile in (1-4): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let field_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid field number")?;
    
    if field_index == 0 || field_index > 4 {
        return Err("Invalid field selection".to_string());
    }
    
    let selected_field = FieldId((field_index - 1) as u8);
    
    // Show stockpile combinations for selected field
    let combinations = find_stockpile_combinations(
        &game.field_cards[selected_field.0 as usize],
        &game.field_stockpiles[selected_field.0 as usize],
        active_card
    );
    
    if combinations.is_empty() {
        return Err("No valid combinations in selected field".to_string());
    }
    
    println!("\nAvailable passive cards for stockpiling:");
    for (i, passive_card) in combinations.iter().enumerate() {
        let active_value = get_card_value(active_card);
        let passive_value = get_card_value(*passive_card);
        let total_value = active_value + passive_value;
        println!("  {}: {} (active) + {} (passive) = {} total", 
                 i + 1, active_card, passive_card, total_value);
    }
    
    // Get combination selection
    print!("Select combination to stockpile (1-{}): ", combinations.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let combo_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid combination number")?;
    
    if combo_index == 0 || combo_index > combinations.len() {
        return Err("Invalid combination selection".to_string());
    }
    
    let selected_passive_card = combinations[combo_index - 1];
    
    // Attempt the stockpile
    let action = Action::Stockpile {
        field: selected_field,
        card: active_card,
        targets: vec![selected_passive_card],
    };
    
    // Record state before action for undo
    undo_manager.record_action(game, &action, current_player);
    
    game.apply_action(action)?;
    
    // Check if it was a face card that changed seasons
    if is_face_card(active_card) {
        println!("Face card played! Illimat rotated.");
    }
    
    println!("Successfully created stockpile using {} and passive card!", active_card);
    
    Ok(())
}

/// Handle harvest action with user input
fn handle_harvest_action(game: &mut IllimatState, undo_manager: &mut UndoManager) -> Result<(), String> {
    let current_player = game.current_player;
    let hand = &game.player_hands[current_player.0 as usize];
    
    // Check if player has cards
    if hand.is_empty() {
        return Err("You have no cards to harvest with!".to_string());
    }
    
    // Show player's hand with indices
    println!("Your hand:");
    for (i, card) in hand.iter().enumerate() {
        let values = if card.rank() == illimat_core::game::card::Rank::Fool {
            "1 or 14".to_string()
        } else {
            get_card_value(*card).to_string()
        };
        println!("  {}: {} (value: {})", i + 1, card, values);
    }
    
    // Get card selection
    print!("Select card to harvest with (1-{}): ", hand.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let card_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid card number")?;
    
    if card_index == 0 || card_index > hand.len() {
        return Err("Invalid card selection".to_string());
    }
    
    let selected_card = hand[card_index - 1];
    
    // Show fields with harvest possibilities
    println!("\nFields:");
    let field_names = ["Spring", "Summer", "Autumn", "Winter"];
    let mut valid_fields = Vec::new();
    
    for (i, field_name) in field_names.iter().enumerate() {
        let field_id = FieldId(i as u8);
        let season = &game.field_seasons[i];
        let field_cards = &game.field_cards[i];
        let can_harvest = illimat_core::game::capabilities::CapabilityManager::can_harvest(field_id, game.illimat_orientation);
        
        print!("  {}: {} Field ({}) - {} cards", 
               i + 1, field_name, season, field_cards.len());
        
        if !can_harvest {
            print!(" [HARVEST BLOCKED]");
        } else if !field_cards.is_empty() {
            // Show possible harvest combinations
            let _target_values = if selected_card.rank() == illimat_core::game::card::Rank::Fool {
                vec![1, 14]
            } else {
                vec![get_card_value(selected_card)]
            };
            
            // Check if field has any available targets
            let has_targets = !game.field_cards[i].is_empty() || !game.field_stockpiles[i].is_empty();
            
            if has_targets {
                print!(" [CAN HARVEST]");
                valid_fields.push(i);
            } else {
                print!(" [NO VALID COMBINATIONS]");
            }
        }
        println!();
        
        // Show field cards
        if !field_cards.is_empty() {
            print!("     Cards: ");
            for (j, card) in field_cards.iter().enumerate() {
                if j > 0 { print!(", "); }
                print!("{}", card);
            }
            println!();
        }
    }
    
    if valid_fields.is_empty() {
        return Err("No valid harvest combinations available in any field".to_string());
    }
    
    // Get field selection
    print!("Select field to harvest from (1-4): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let field_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid field number")?;
    
    if field_index == 0 || field_index > 4 {
        return Err("Invalid field selection".to_string());
    }
    
    let selected_field = FieldId((field_index - 1) as u8);
    
    // Find harvest combinations and separate direct matches from combinations
    let all_combinations = find_harvest_combinations(
        &game.field_cards[selected_field.0 as usize],
        &game.field_stockpiles[selected_field.0 as usize], 
        selected_card
    );
    
    if all_combinations.is_empty() {
        return Err("No valid combinations in selected field".to_string());
    }
    
    // Get target value(s) for the played card
    let target_values = if selected_card.rank() == illimat_core::game::card::Rank::Fool {
        vec![1, 14]
    } else {
        vec![get_card_value(selected_card)]
    };
    
    // Separate direct matches from combinations for each target value
    let mut direct_matches = Vec::new();
    let mut combination_matches = Vec::new();
    
    for &target_value in &target_values {
        for combination in &all_combinations {
            if combination.len() == 1 && get_card_value(combination[0]) == target_value {
                // Single card with exact value match - direct match
                direct_matches.push(combination[0]);
            } else if combination.iter().map(|c| get_card_value(*c)).sum::<u8>() == target_value {
                // Multi-card combination or stockpile - combination match
                combination_matches.push(combination.clone());
            }
        }
    }
    
    let selected_targets = if !direct_matches.is_empty() && combination_matches.is_empty() {
        // Only direct matches available - auto-collect all matching values
        println!("\n🔄 Auto-collecting all cards matching {} (value {}):", 
                 selected_card, get_card_value(selected_card));
        for card in &direct_matches {
            println!("  • {}", card);
        }
        direct_matches
    } else if direct_matches.is_empty() && combination_matches.len() == 1 {
        // Only one combination match available - auto-select it
        let combo = &combination_matches[0];
        println!("\n🔄 Auto-selecting only available combination:");
        print!("  • ");
        for (j, card) in combo.iter().enumerate() {
            if j > 0 { print!(" + "); }
            print!("{}", card);
        }
        let sum: u8 = combo.iter().map(|c| get_card_value(*c)).sum();
        println!(" = {}", sum);
        combo.clone()
    } else {
        // Multiple options available - prompt for selection
        println!("\nAvailable harvest combinations:");
        
        if !direct_matches.is_empty() {
            println!("  1: Auto-collect all exact matches ({})", direct_matches.len());
            print!("     ");
            for (i, card) in direct_matches.iter().enumerate() {
                if i > 0 { print!(", "); }
                print!("{}", card);
            }
            println!();
        }
        
        let combination_start_index = if direct_matches.is_empty() { 1 } else { 2 };
        for (i, combination) in combination_matches.iter().enumerate() {
            print!("  {}: ", combination_start_index + i);
            for (j, card) in combination.iter().enumerate() {
                if j > 0 { print!(" + "); }
                print!("{}", card);
            }
            let sum: u8 = combination.iter().map(|c| get_card_value(*c)).sum();
            println!(" = {}", sum);
        }
        
        let total_options = if direct_matches.is_empty() { 
            combination_matches.len() 
        } else { 
            1 + combination_matches.len() 
        };
        
        // Get combination selection
        print!("Select combination to harvest (1-{}): ", total_options);
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => return Err("EOF reached".to_string()),
            Ok(_) => {},
            Err(_) => return Err("Failed to read input".to_string()),
        }
        
        let trimmed_input = input.trim();
        if trimmed_input.is_empty() {
            return Err("Empty input".to_string());
        }
        
        let combo_index: usize = trimmed_input.parse()
            .map_err(|_| format!("Invalid combination number: '{}'", trimmed_input))?;
        
        if combo_index == 0 || combo_index > total_options {
            return Err(format!("Invalid combination selection: {} (valid range: 1-{})", combo_index, total_options));
        }
        
        if !direct_matches.is_empty() && combo_index == 1 {
            // User selected auto-collect option
            direct_matches
        } else {
            // User selected a specific combination
            let combination_index = if direct_matches.is_empty() { 
                combo_index - 1 
            } else { 
                combo_index - 2 
            };
            combination_matches[combination_index].clone()
        }
    };
    
    // Attempt the harvest
    let action = Action::Harvest {
        field: selected_field,
        card: selected_card,
        targets: selected_targets.clone(),
    };
    
    // Record state before action for undo
    undo_manager.record_action(game, &action, current_player);
    
    let field_cleared = game.apply_action(action)?;
    
    // Handle okus collection if field was cleared
    if field_cleared {
        println!("\n🎯 Field cleared! Checking for okus collection...");
        let available_okus = game.get_available_okus();
        
        if !available_okus.is_empty() {
            println!("Available okus tokens:");
            for (i, okus) in available_okus.iter().enumerate() {
                println!("  {}: Okus {}", i + 1, okus);
            }
            
            print!("Select okus to collect (1-{}): ", available_okus.len());
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
            let trimmed_input = input.trim();
            if trimmed_input.is_empty() {
                return Err("Empty input".to_string());
            }
            let okus_index: usize = trimmed_input.parse()
                .map_err(|_| "Invalid okus number")?;
            
            if okus_index == 0 || okus_index > available_okus.len() {
                return Err("Invalid okus selection".to_string());
            }
            
            let selected_okus = available_okus[okus_index - 1];
            
            match game.collect_okus(current_player, vec![selected_okus]) {
                Ok(()) => println!("✨ Collected okus {}!", selected_okus),
                Err(e) => println!("Failed to collect okus: {}", e),
            }
        } else {
            println!("No okus tokens available on the Illimat.");
        }
    }
    
    // Check if it was a face card that changed seasons
    if is_face_card(selected_card) {
        println!("Face card played! Illimat rotated.");
    }
    
    println!("Successfully harvested {} cards with {}!", 
             selected_targets.len(), selected_card);
    
    Ok(())
}

/// Handle sow action with user input
fn handle_sow_action(game: &mut IllimatState, undo_manager: &mut UndoManager) -> Result<(), String> {
    let current_player = game.current_player;
    let hand = &game.player_hands[current_player.0 as usize];
    
    // Check if player has cards
    if hand.is_empty() {
        return Err("You have no cards to sow!".to_string());
    }
    
    // Show player's hand with indices
    println!("Your hand:");
    for (i, card) in hand.iter().enumerate() {
        println!("  {}: {}", i + 1, card);
    }
    
    // Get card selection
    print!("Select card to sow (1-{}): ", hand.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let card_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid card number")?;
    
    if card_index == 0 || card_index > hand.len() {
        return Err("Invalid card selection".to_string());
    }
    
    let selected_card = hand[card_index - 1];
    
    // Show fields
    println!("\nFields:");
    let field_names = ["Spring", "Summer", "Autumn", "Winter"];
    for (i, field_name) in field_names.iter().enumerate() {
        let field_id = FieldId(i as u8);
        let season = &game.field_seasons[i];
        let card_count = game.field_cards[i].len();
        let can_sow = illimat_core::game::capabilities::CapabilityManager::can_sow(field_id, game.illimat_orientation);
        
        print!("  {}: {} Field ({}) - {} cards", 
               i + 1, field_name, season, card_count);
        if !can_sow {
            print!(" [SOWING BLOCKED]");
        }
        println!();
    }
    
    // Get field selection
    print!("Select field to sow into (1-4): ");
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    io::stdin().read_line(&mut input).map_err(|_| "Failed to read input")?;
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    let field_index: usize = trimmed_input.parse()
        .map_err(|_| "Invalid field number")?;
    
    if field_index == 0 || field_index > 4 {
        return Err("Invalid field selection".to_string());
    }
    
    let selected_field = FieldId((field_index - 1) as u8);
    
    // Attempt the sow
    let action = Action::Sow {
        field: selected_field,
        card: selected_card,
    };
    
    // Record state before action for undo
    undo_manager.record_action(game, &action, current_player);
    
    game.apply_action(action)?;
    
    // Check if it was a face card that changed seasons
    if is_face_card(selected_card) {
        println!("Face card played! Illimat rotated.");
    }
    
    println!("Successfully sowed {} into {} field!", 
             selected_card, field_names[(selected_field.0) as usize]);
    
    Ok(())
}

/// Display mystical ASCII art banner
fn print_occult_banner() {
    println!(r#"
    ╔═══════════════════════════════════════════════════════════════╗
    ║                                                               ║
    ║                     I L L I M A T                             ║
    ║                                                               ║
    ║    ═══════════════════════════════════════════════════════    ║
    ║                                                               ║
    ║    No one knows exactly when Illimat first appeared, but      ║
    ║    by the 8th century CE there is evidence of a game          ║
    ║    called "Ullamat" played by "contestants divers and         ║
    ║    stranje, in far-flunged playces."                          ║
    ║                                                               ║
    ║    ═══════════════════════════════════════════════════════    ║
    ║                                                               ║
    ║              Spring: The season of frolic spurns hoarding     ║
    ║              Summer: The height of power permits all rites   ║
    ║              Autumn: The time of gathering forbids new seed  ║
    ║              Winter: The barren months yield no harvest      ║
    ║              Stars: The celestial wheel turns at thy command ║
    ║                                                               ║
    ║              "Four fields, four seasons,                     ║
    ║               countless possibilities..."                     ║
    ║                                                               ║
    ╚═══════════════════════════════════════════════════════════════╝
"#);
}

/// Create a new game through interactive setup
fn create_new_game() -> IllimatState {
    println!("🎴 Welcome to the Illimat table!");
    println!();
    
    // Get player count
    loop {
        print!("How many players? (2-4): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => {
                // EOF reached, exit gracefully
                println!("EOF reached. Exiting game.");
                std::process::exit(0);
            }
            Ok(_) => {
                let trimmed_input = input.trim();
                if trimmed_input.is_empty() {
                    println!("Empty input. Please enter a number from 2 to 4.");
                    continue;
                }
                
                if let Ok(count) = trimmed_input.parse::<u8>() {
                    if count >= 2 && count <= 4 {
                        println!();
                        
                        // For now, all players are human
                        // TODO: Future feature - prompt for CPU players
                        let mut player_types = [PlayerType::Human; 4];
                        for i in count as usize..4 {
                            player_types[i] = PlayerType::Human; // Inactive slots, doesn't matter
                        }
                        
                        // Determine deck size (2-3 players use reduced deck)
                        let use_stars_suit = count == 4;
                        let deck_description = if use_stars_suit {
                            "65 cards (all 5 suits including Stars)"
                        } else {
                            "52 cards (4 suits, Stars removed)"
                        };
                        
                        println!("🂠 Using {} player deck: {}", count, deck_description);
                        println!();
                        
                        let config = GameConfig {
                            player_count: count,
                            player_types,
                            use_stars_suit,
                        };
                        
                        return IllimatState::new(config);
                    } else {
                        println!("Invalid number. Please enter a number from 2 to 4.");
                    }
                } else {
                    println!("Invalid input '{}'. Please enter a number from 2 to 4.", trimmed_input);
                }
            }
            Err(e) => {
                println!("Error reading input: {}. Exiting game.", e);
                std::process::exit(1);
            }
        }
    }
}


/// Get the numeric value of a card
fn get_card_value(card: illimat_core::game::card::Card) -> u8 {
    use illimat_core::game::card::Rank;
    match card.rank() {
        Rank::Fool => 1,
        Rank::Two => 2,
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Knight => 11,
        Rank::Queen => 12,
        Rank::King => 13,
    }
}

/// Check if a card is a face card (affects Illimat rotation)
fn is_face_card(card: illimat_core::game::card::Card) -> bool {
    use illimat_core::game::card::Rank;
    matches!(card.rank(), Rank::Fool | Rank::Knight | Rank::Queen | Rank::King)
}

/// Find all valid harvest combinations for a played card
fn find_harvest_combinations(
    field_cards: &[illimat_core::game::card::Card],
    field_stockpiles: &[illimat_core::game::stockpile::Stockpile],
    played_card: illimat_core::game::card::Card,
) -> Vec<Vec<illimat_core::game::card::Card>> {
    use illimat_core::game::card::Rank;
    
    let mut combinations = Vec::new();
    
    // Get possible values for the played card (Fool can be 1 or 14)
    let played_values = if played_card.rank() == Rank::Fool {
        vec![1, 14]
    } else {
        vec![get_card_value(played_card)]
    };
    
    for &target_value in &played_values {
        // Single cards that match
        for &card in field_cards {
            if get_card_value(card) == target_value {
                combinations.push(vec![card]);
            }
        }
        
        // Stockpiles that match
        for stockpile in field_stockpiles {
            if stockpile.value == target_value {
                combinations.push(stockpile.cards.clone());
            }
        }
        
        // Card combinations that sum to target value
        find_card_combinations_recursive(field_cards, target_value, &mut combinations);
    }
    
    combinations
}

/// Find card combinations that sum to target value (recursive helper)
fn find_card_combinations_recursive(
    available_cards: &[illimat_core::game::card::Card], 
    target_sum: u8,
    combinations: &mut Vec<Vec<illimat_core::game::card::Card>>
) {
    fn backtrack(
        cards: &[illimat_core::game::card::Card],
        target: u8,
        current_sum: u8,
        current_combo: &mut Vec<illimat_core::game::card::Card>,
        start_idx: usize,
        results: &mut Vec<Vec<illimat_core::game::card::Card>>
    ) {
        if current_sum == target && current_combo.len() >= 2 {
            results.push(current_combo.clone());
            return;
        }
        
        if current_sum > target || start_idx >= cards.len() {
            return;
        }
        
        for i in start_idx..cards.len() {
            let card = cards[i];
            let card_value = get_card_value(card);
            
            if current_sum + card_value <= target {
                current_combo.push(card);
                backtrack(cards, target, current_sum + card_value, current_combo, i + 1, results);
                current_combo.pop();
            }
        }
    }
    
    let mut current_combo = Vec::new();
    backtrack(available_cards, target_sum, 0, &mut current_combo, 0, combinations);
}

/// Find valid stockpile combinations (active + passive card pairs)
fn find_stockpile_combinations(
    field_cards: &[illimat_core::game::card::Card],
    field_stockpiles: &[illimat_core::game::stockpile::Stockpile],
    _active_card: illimat_core::game::card::Card,
) -> Vec<illimat_core::game::card::Card> {
    let mut available_targets = Vec::new();
    
    // Add loose cards as potential passive targets
    available_targets.extend_from_slice(field_cards);
    
    // Add cards from stockpiles as potential passive targets
    for stockpile in field_stockpiles {
        available_targets.extend_from_slice(&stockpile.cards);
    }
    
    available_targets
}

/// Check if screen should be cleared between turns (user preference)
fn should_clear_screen() -> bool {
    // Simple static option - could be made configurable
    std::env::var("ILLIMAT_CLEAR_SCREEN").unwrap_or_default() == "1"
}

/// Clear the terminal screen
fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

/// Confirm quit with user
fn confirm_quit() -> bool {
    print!("{}", "Are you sure you want to quit? (y/N): ".bright_red());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
    } else {
        false
    }
}

/// Enhanced save game state with full preservation
fn handle_save_game(game: &IllimatState) -> Result<(), String> {
    // Create saves directory if it doesn't exist
    let saves_dir = "saves";
    fs::create_dir_all(saves_dir)
        .map_err(|e| format!("Failed to create saves directory: {}", e))?;
    
    // Create timestamped save file
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let save_file = format!("{}/illimat_save_{}.json", saves_dir, timestamp);
    
    // Create enhanced save data structure
    let save_data = EnhancedSaveData {
        metadata: SaveMetadata {
            version: "1.0".to_string(),
            created: chrono::Utc::now(),
            description: format!("Round {}, Player {}'s turn", game.round_number, game.current_player.0),
        },
        game_state: game.clone(),
        statistics: GameStatistics::from_state(game),
    };
    
    // Serialize to JSON with pretty printing
    let json_data = serde_json::to_string_pretty(&save_data)
        .map_err(|e| format!("Failed to serialize game state: {}", e))?;
    
    // Write to file
    fs::write(&save_file, json_data)
        .map_err(|e| format!("Failed to write save file '{}': {}", save_file, e))?;
    
    println!("💾 Game saved to: {}", save_file);
    Ok(())
}

/// Enhanced load game state with save file selection
fn handle_load_game() -> Result<IllimatState, String> {
    let saves_dir = "saves";
    
    // Check if saves directory exists
    if !Path::new(saves_dir).exists() {
        return Err("No saves directory found. Use 'v' or 'save' to create a save file first.".to_string());
    }
    
    // Find all save files
    let save_files = fs::read_dir(saves_dir)
        .map_err(|e| format!("Failed to read saves directory: {}", e))?
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.extension()? == "json" && 
               path.file_name()?.to_str()?.starts_with("illimat_save_") {
                Some(path)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    
    if save_files.is_empty() {
        return Err("No save files found. Use 'v' or 'save' to create one.".to_string());
    }
    
    // Show available save files with metadata
    println!("\n📂 Available save files:");
    let mut save_previews = Vec::new();
    
    for (i, save_path) in save_files.iter().enumerate() {
        let file_content = fs::read_to_string(save_path)
            .map_err(|e| format!("Failed to read save file: {}", e))?;
            
        match serde_json::from_str::<EnhancedSaveData>(&file_content) {
            Ok(save_data) => {
                let filename = save_path.file_name().unwrap().to_str().unwrap();
                println!("  {}: {} - {}", 
                    i + 1, 
                    filename,
                    save_data.metadata.description
                );
                println!("     Created: {} UTC", 
                    save_data.metadata.created.format("%Y-%m-%d %H:%M:%S"));
                save_previews.push(save_data);
            }
            Err(_) => {
                println!("  {}: {} - [Invalid save file]", 
                    i + 1, 
                    save_path.file_name().unwrap().to_str().unwrap()
                );
                save_previews.push(EnhancedSaveData {
                    metadata: SaveMetadata {
                        version: "unknown".to_string(),
                        created: chrono::Utc::now(),
                        description: "Invalid save file".to_string(),
                    },
                    game_state: IllimatState::new(GameConfig::new(2)),
                    statistics: GameStatistics::from_state(&IllimatState::new(GameConfig::new(2))),
                });
            }
        }
    }
    
    // Get user selection
    print!("\nSelect save file to load (1-{}): ", save_files.len());
    io::stdout().flush().unwrap();
    
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(0) => return Err("EOF reached".to_string()),
        Ok(_) => {},
        Err(_) => return Err("Failed to read input".to_string()),
    }
    
    let trimmed_input = input.trim();
    if trimmed_input.is_empty() {
        return Err("Empty input".to_string());
    }
    
    let selection: usize = trimmed_input.parse()
        .map_err(|_| format!("Invalid selection: '{}'", trimmed_input))?;
        
    if selection == 0 || selection > save_files.len() {
        return Err(format!("Invalid selection: {} (valid range: 1-{})", selection, save_files.len()));
    }
    
    // Load the selected save file
    let selected_path = &save_files[selection - 1];
    let file_content = fs::read_to_string(selected_path)
        .map_err(|e| format!("Failed to read save file: {}", e))?;
        
    let save_data: EnhancedSaveData = serde_json::from_str(&file_content)
        .map_err(|e| format!("Failed to parse save file: {}", e))?;
    
    println!("📂 Loaded: {}", save_data.metadata.description);
    println!("   Statistics: {} total turns, {:.1} avg turns per round", 
        save_data.statistics.total_turns,
        save_data.statistics.average_turns_per_round
    );
    
    Ok(save_data.game_state)
}

/// Export human-readable game state report
fn handle_export_game_state(game: &IllimatState) -> Result<(), String> {
    // Create exports directory if it doesn't exist
    let exports_dir = "exports";
    fs::create_dir_all(exports_dir)
        .map_err(|e| format!("Failed to create exports directory: {}", e))?;
    
    // Create timestamped export file
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let export_file = format!("{}/illimat_report_{}.txt", exports_dir, timestamp);
    
    // Generate comprehensive human-readable report
    let mut report = String::new();
    
    // Header
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    report.push_str("                    ILLIMAT GAME STATE REPORT                   \n");
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    report.push_str(&format!("Generated: {} UTC\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S")));
    report.push_str(&format!("Round: {} | Turn: {} | Current Player: {}\n", 
        game.round_number, game.turn_number, game.current_player.0));
    report.push_str(&format!("Game Phase: {:?} | Players: {}\n", 
        game.phase, game.config.player_count));
    report.push_str("═══════════════════════════════════════════════════════════════\n\n");
    
    // Game Configuration
    report.push_str("🎮 GAME CONFIGURATION:\n");
    report.push_str(&format!("  • Players: {}\n", game.config.player_count));
    report.push_str(&format!("  • Deck Type: {} ({})\n", 
        if game.config.use_stars_suit { "Full 5-suit" } else { "4-suit" },
        if game.config.use_stars_suit { "65 cards" } else { "52 cards" }
    ));
    report.push_str(&format!("  • Dealer: Player {}\n", game.dealer.0));
    report.push_str(&format!("  • Remaining Cards: {}\n\n", game.deck.len()));
    
    // Current Scores
    report.push_str("🏆 CURRENT SCORES:\n");
    for i in 0..game.config.player_count {
        let player_idx = i as usize;
        report.push_str(&format!("  • Player {}: {} points\n", i, game.total_scores[player_idx]));
    }
    report.push_str("\n");
    
    // Field State
    report.push_str("🌟 FIELD STATE:\n");
    use illimat_core::game::field_id::FieldId;
    let field_names = ["Spring", "Summer", "Autumn", "Winter"];
    
    for (i, &_field_name) in field_names.iter().enumerate() {
        let field_id = FieldId(i as u8);
        let season = &game.field_seasons[i];
        let cards = &game.field_cards[i];
        let stockpiles = &game.field_stockpiles[i];
        let seasonal_name = field_id.seasonal_name(game.illimat_orientation);
        
        report.push_str(&format!("  📍 {} ({}):\n", seasonal_name, season));
        
        // Field capabilities
        use illimat_core::game::capabilities::CapabilityManager;
        let can_sow = CapabilityManager::can_sow(field_id, game.illimat_orientation);
        let can_harvest = CapabilityManager::can_harvest(field_id, game.illimat_orientation);
        let can_stockpile = CapabilityManager::can_stockpile(field_id, game.illimat_orientation);
        
        report.push_str("     Allowed Actions: ");
        let mut actions = Vec::new();
        if can_sow { actions.push("Sow"); }
        if can_harvest { actions.push("Harvest"); }
        if can_stockpile { actions.push("Stockpile"); }
        if actions.is_empty() { actions.push("None"); }
        report.push_str(&actions.join(", "));
        report.push_str("\n");
        
        // Loose cards
        if cards.is_empty() {
            report.push_str("     Loose Cards: None\n");
        } else {
            report.push_str(&format!("     Loose Cards ({}): ", cards.len()));
            for (j, card) in cards.iter().enumerate() {
                if j > 0 { report.push_str(", "); }
                report.push_str(&format!("{}", card));
            }
            report.push_str("\n");
        }
        
        // Stockpiles
        if stockpiles.is_empty() {
            report.push_str("     Stockpiles: None\n");
        } else {
            report.push_str(&format!("     Stockpiles ({}):\n", stockpiles.len()));
            for (j, stockpile) in stockpiles.iter().enumerate() {
                report.push_str(&format!("       #{} (value {}): ", j + 1, stockpile.value));
                for (k, card) in stockpile.cards.iter().enumerate() {
                    if k > 0 { report.push_str(" + "); }
                    report.push_str(&format!("{}", card));
                }
                report.push_str(&format!(" (created turn {})\n", stockpile.created_turn));
            }
        }
        report.push_str("\n");
    }
    
    // Player Hands
    report.push_str("🃏 PLAYER HANDS:\n");
    for i in 0..game.config.player_count {
        let player_idx = i as usize;
        let hand = &game.player_hands[player_idx];
        let harvest = &game.player_harvests[player_idx];
        
        report.push_str(&format!("  👤 Player {} ({} cards in hand, {} harvested):\n", 
            i, hand.len(), harvest.len()));
        
        if hand.is_empty() {
            report.push_str("     Hand: Empty\n");
        } else {
            report.push_str("     Hand: ");
            for (j, card) in hand.iter().enumerate() {
                if j > 0 { report.push_str(", "); }
                report.push_str(&format!("{}", card));
            }
            report.push_str("\n");
        }
        
        if harvest.is_empty() {
            report.push_str("     Harvested: None this round\n");
        } else {
            report.push_str(&format!("     Harvested ({}): ", harvest.len()));
            for (j, card) in harvest.iter().enumerate() {
                if j > 0 { report.push_str(", "); }
                report.push_str(&format!("{}", card));
            }
            report.push_str("\n");
        }
        report.push_str("\n");
    }
    
    // Okus Tokens
    report.push_str("🎯 OKUS TOKENS:\n");
    use illimat_core::game::okus::{OkusId, OkusPosition};
    let okus_ids = [OkusId::A, OkusId::B, OkusId::C, OkusId::D];
    
    for (i, okus_id) in okus_ids.iter().enumerate() {
        let position = &game.okus_positions[i];
        report.push_str(&format!("  • Okus {}: ", okus_id));
        match position {
            OkusPosition::OnIllimat => report.push_str("On Illimat (available)\n"),
            OkusPosition::WithPlayer(player) => report.push_str(&format!("With Player {} (+1 point)\n", player.0)),
        }
    }
    report.push_str("\n");
    
    // Game Statistics
    report.push_str("📊 GAME STATISTICS:\n");
    let stats = GameStatistics::from_state(game);
    report.push_str(&format!("  • Total Turns: {}\n", stats.total_turns));
    report.push_str(&format!("  • Current Round: {}\n", game.round_number));
    report.push_str(&format!("  • Average Turns per Round: {:.1}\n", stats.average_turns_per_round));
    report.push_str(&format!("  • Illimat Orientation: {} (Spring at Field {})\n", 
        game.illimat_orientation, game.illimat_orientation));
    
    // Victory Analysis
    report.push_str("\n🏁 VICTORY ANALYSIS:\n");
    let leader_score = *game.total_scores.iter().max().unwrap();
    let leaders: Vec<_> = (0..game.config.player_count)
        .filter(|&i| game.total_scores[i as usize] == leader_score)
        .collect();
    
    if leader_score >= 17 {
        if leaders.len() == 1 {
            report.push_str(&format!("  🎉 VICTORY! Player {} wins with {} points!\n", leaders[0], leader_score));
        } else {
            report.push_str(&format!("  🤝 TIE! Players {} tied with {} points!\n", 
                leaders.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "), 
                leader_score));
        }
    } else {
        if leaders.len() == 1 {
            report.push_str(&format!("  📈 Leader: Player {} with {} points (need {} more)\n", 
                leaders[0], leader_score, 17 - leader_score));
        } else {
            report.push_str(&format!("  📈 Tied Leaders: Players {} with {} points (need {} more)\n", 
                leaders.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(", "), 
                leader_score, 17 - leader_score));
        }
    }
    
    report.push_str("\n═══════════════════════════════════════════════════════════════\n");
    report.push_str("                          END OF REPORT                        \n");
    report.push_str("═══════════════════════════════════════════════════════════════\n");
    
    // Write to file
    fs::write(&export_file, report)
        .map_err(|e| format!("Failed to write export file '{}': {}", export_file, e))?;
    
    println!("📊 Game state exported to: {}", export_file);
    Ok(())
}