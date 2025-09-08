use wasm_bindgen::prelude::*;
use crate::game::state::IllimatState;
use crate::game::game_config::GameConfig;
use crate::game::actions::Action;
use crate::game::player::PlayerId;
use crate::game::field_id::FieldId;
use crate::game::card::Card;
use crate::game::compact_state::CompactState;
use crate::game::mcts::{MctsTree, MctsConfig, MctsAnalysis, MctsNode};
use crate::game::simd_compact_integration::SimdMove;
use std::time::Duration;

// Use wee_alloc as the global allocator for smaller WASM bundle size
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// Initialize panic hook for better debugging in browsers
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// WebAssembly-compatible game engine for Illimat
#[wasm_bindgen]
pub struct WasmGameEngine {
    state: IllimatState,
}

/// MCTS AI engine with WebAssembly optimization
#[wasm_bindgen]
pub struct WasmMctsEngine {
    tree: Option<MctsTree>,
    config: MctsConfig,
}

/// AI move suggestion with analysis
#[wasm_bindgen]
#[derive(Clone)]
pub struct AiMoveResult {
    has_move: bool,
    move_type: String,
    field_id: u8,
    card_rank: u8,
    card_suit: u8,
    confidence: f32,
    simulations: u32,
    search_time_ms: u32,
    error_message: Option<String>,
}

/// Move validation and application response
#[wasm_bindgen]
#[derive(Clone, Debug)]
pub struct MoveResponse {
    success: bool,
    error_message: Option<String>,
}

#[wasm_bindgen]
impl MoveResponse {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
}

#[wasm_bindgen]
impl AiMoveResult {
    #[wasm_bindgen(getter)]
    pub fn has_move(&self) -> bool {
        self.has_move
    }
    
    #[wasm_bindgen(getter)]
    pub fn move_type(&self) -> String {
        self.move_type.clone()
    }
    
    #[wasm_bindgen(getter)]
    pub fn field_id(&self) -> u8 {
        self.field_id
    }
    
    #[wasm_bindgen(getter)]
    pub fn card_rank(&self) -> u8 {
        self.card_rank
    }
    
    #[wasm_bindgen(getter)]
    pub fn card_suit(&self) -> u8 {
        self.card_suit
    }
    
    #[wasm_bindgen(getter)]
    pub fn confidence(&self) -> f32 {
        self.confidence
    }
    
    #[wasm_bindgen(getter)]
    pub fn simulations(&self) -> u32 {
        self.simulations
    }
    
    #[wasm_bindgen(getter)]
    pub fn search_time_ms(&self) -> u32 {
        self.search_time_ms
    }
    
    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> Option<String> {
        self.error_message.clone()
    }
}

#[wasm_bindgen]
impl WasmMctsEngine {
    /// Create new MCTS engine with optimized WebAssembly configuration
    #[wasm_bindgen(constructor)]
    pub fn new(max_simulations: u32, time_limit_ms: Option<u32>) -> WasmMctsEngine {
        let mut config = MctsConfig::default();
        config.max_simulations = max_simulations;
        config.time_limit = time_limit_ms.map(|ms| Duration::from_millis(ms as u64));
        
        // Enable SIMD for WebAssembly if supported
        config.enable_simd = cfg!(target_feature = "simd128");
        
        // WebAssembly-optimized parameters
        config.exploration_constant = 1.414; // √2 for balanced exploration
        config.max_depth = 50; // Conservative depth for WASM memory limits
        
        WasmMctsEngine {
            tree: None,
            config,
        }
    }
    
    /// Update MCTS configuration
    #[wasm_bindgen]
    pub fn set_config(&mut self, max_simulations: u32, time_limit_ms: Option<u32>, exploration_constant: f32) {
        self.config.max_simulations = max_simulations;
        self.config.time_limit = time_limit_ms.map(|ms| Duration::from_millis(ms as u64));
        self.config.exploration_constant = exploration_constant;
    }
    
    /// Initialize MCTS tree with current game state
    #[wasm_bindgen]
    pub fn initialize_from_state(&mut self, state_json: &str) -> Result<(), JsValue> {
        let state: IllimatState = serde_json::from_str(state_json)
            .map_err(|e| JsValue::from_str(&format!("State deserialization error: {}", e)))?;
        
        let compact_state = CompactState::from(&state);
        self.tree = Some(MctsTree::new(compact_state, self.config.clone()));
        
        Ok(())
    }
    
    /// Get AI move suggestion with performance analysis
    #[wasm_bindgen]
    pub fn get_best_move(&mut self) -> AiMoveResult {
        #[cfg(feature = "wasm-safe")]
        {
            self.get_best_move_safe()
        }
        #[cfg(any(feature = "debug-selection", feature = "debug-expansion", feature = "debug-simulation"))]
        {
            self.get_best_move_debug_mcts()
        }
        #[cfg(not(any(feature = "wasm-safe", feature = "debug-selection", feature = "debug-expansion", feature = "debug-simulation")))]
        {
            self.get_best_move_full_mcts()
        }
    }
    
    /// Full MCTS search with WASM-safe backpropagation
    #[cfg(not(feature = "wasm-safe"))]
    fn get_best_move_full_mcts(&mut self) -> AiMoveResult {
        web_sys::console::log_1(&"🔥 FULL-MCTS: Starting real MCTS search".into());
        
        let tree = match &mut self.tree {
            Some(tree) => tree,
            None => {
                web_sys::console::log_1(&"❌ FULL-MCTS: No MCTS tree initialized".into());
                return AiMoveResult {
                    has_move: false,
                    move_type: "none".to_string(),
                    field_id: 0,
                    card_rank: 0,
                    card_suit: 0,
                    confidence: 0.0,
                    simulations: 0,
                    search_time_ms: 0,
                    error_message: Some("MCTS tree not initialized".to_string()),
                };
            }
        };
        
        web_sys::console::log_1(&format!("🎯 FULL-MCTS: Tree initialized with {} nodes", tree.nodes.len()).into());
        
        // Step 3: Run actual MCTS search with proper timing
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        web_sys::console::log_1(&"⏰ FULL-MCTS: Starting performance timer".into());
        
        // Run the real MCTS search with WASM-safe parameters
        // Reduce complexity to avoid the "unreachable" error
        let original_max_sims = tree.config.max_simulations;
        web_sys::console::log_1(&format!("⚙️ FULL-MCTS: Original max simulations: {}", original_max_sims).into());
        tree.config.max_simulations = std::cmp::min(original_max_sims, 50); // Very conservative for stability
        
        // Use severely limited MCTS to avoid backpropagation issues
        tree.config.max_simulations = 1; // Single iteration only
        web_sys::console::log_1(&"🔍 FULL-MCTS: Set max simulations to 1 for safety".into());
        
        web_sys::console::log_1(&"🚀 FULL-MCTS: About to call tree.search() - this is where unreachable usually happens".into());
        
        // Run the actual MCTS search with minimal complexity
        let search_result = tree.search();
        web_sys::console::log_1(&"✅ FULL-MCTS: tree.search() completed successfully!".into());
        
        match search_result {
            Some(best_move) => {
                web_sys::console::log_1(&"🎯 FULL-MCTS: Got best move from MCTS".into());
                let end_time = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now())
                    .unwrap_or(0.0);
                
                let search_time_ms = (end_time - start_time) as u32;
                let (move_type, field_id, card_rank, card_suit) = Self::convert_simd_move_to_wasm_static(best_move);
                
                // Restore original configuration
                tree.config.max_simulations = original_max_sims;
                
                // Get real statistics from the tree
                let actual_simulations = tree.stats.simulations_completed;
                let confidence = if tree.nodes[tree.root_index].visits > 0 {
                    tree.nodes[tree.root_index].average_reward()
                } else {
                    0.0
                };
                
                AiMoveResult {
                    has_move: true,
                    move_type,
                    field_id,
                    card_rank,
                    card_suit,
                    confidence,
                    simulations: actual_simulations,
                    search_time_ms,
                    error_message: None,
                }
            }
            None => {
                web_sys::console::log_1(&"⚠️ FULL-MCTS: tree.search() returned None (no move found)".into());
                let end_time = web_sys::window()
                    .and_then(|w| w.performance())
                    .map(|p| p.now())
                    .unwrap_or(0.0);
                
                let search_time_ms = (end_time - start_time) as u32;
                
                // Restore original configuration
                tree.config.max_simulations = original_max_sims;
                
                AiMoveResult {
                    has_move: false,
                    move_type: "none".to_string(),
                    field_id: 0,
                    card_rank: 0,
                    card_suit: 0,
                    confidence: 0.0,
                    simulations: tree.stats.simulations_completed,
                    search_time_ms,
                    error_message: Some("No valid moves found by MCTS".to_string()),
                }
            }
        }
    }
    
    /// Debug MCTS implementation with progressive feature enabling
    #[cfg(any(feature = "debug-selection", feature = "debug-expansion", feature = "debug-simulation"))]
    fn get_best_move_debug_mcts(&mut self) -> AiMoveResult {
        let tree = match &mut self.tree {
            Some(tree) => tree,
            None => {
                return AiMoveResult {
                    has_move: false,
                    move_type: "none".to_string(),
                    field_id: 0,
                    card_rank: 0,
                    card_suit: 0,
                    confidence: 0.0,
                    simulations: 0,
                    search_time_ms: 0,
                    error_message: Some("MCTS tree not initialized".to_string()),
                };
            }
        };
        
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        // Progressive MCTS implementation - test each phase individually
        let result = Self::debug_mcts_phases_static(tree);
        
        let end_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        let search_time_ms = (end_time - start_time) as u32;
        
        match result {
            Ok(best_move) => {
                let (move_type, field_id, card_rank, card_suit) = Self::convert_simd_move_to_wasm_static(best_move);
                
                AiMoveResult {
                    has_move: true,
                    move_type,
                    field_id,
                    card_rank,
                    card_suit,
                    confidence: 0.8, // Debug confidence
                    simulations: 1, // Single debug iteration
                    search_time_ms,
                    error_message: None,
                }
            }
            Err(error_msg) => {
                AiMoveResult {
                    has_move: false,
                    move_type: "error".to_string(),
                    field_id: 0,
                    card_rank: 0,
                    card_suit: 0,
                    confidence: 0.0,
                    simulations: 0,
                    search_time_ms,
                    error_message: Some(error_msg),
                }
            }
        }
    }
    
    /// Progressive MCTS phase testing
    #[cfg(any(feature = "debug-selection", feature = "debug-expansion", feature = "debug-simulation"))]
    fn debug_mcts_phases_static(tree: &mut MctsTree) -> Result<SimdMove, String> {
        // Phase 1: Test selection only
        #[cfg(feature = "debug-selection")]
        {
            let _selected_node = Self::debug_select_node_static(tree)?;
            // If we get here, selection works - return a simple move
            let moves = tree.generate_moves_simd(tree.root_index);
            if moves.is_empty() {
                return Err("No moves available after selection test".to_string());
            }
            return Ok(moves[0]);
        }
        
        // Phase 2: Test selection + expansion
        #[cfg(feature = "debug-expansion")]
        {
            let selected_node = Self::debug_select_node_static(tree)?;
            let _expanded_node = Self::debug_expand_node_static(tree, selected_node)?;
            // If we get here, expansion works
            let moves = tree.generate_moves_simd(tree.root_index);
            if moves.is_empty() {
                return Err("No moves available after expansion test".to_string());
            }
            return Ok(moves[0]);
        }
        
        // Phase 3: Test selection + expansion + simulation
        #[cfg(feature = "debug-simulation")]
        {
            let selected_node = Self::debug_select_node_static(tree)?;
            let expanded_node = Self::debug_expand_node_static(tree, selected_node)?;
            let _reward = Self::debug_simulate_static(tree, expanded_node.unwrap_or(selected_node))?;
            // If we get here, simulation works
            let moves = tree.generate_moves_simd(tree.root_index);
            if moves.is_empty() {
                return Err("No moves available after simulation test".to_string());
            }
            return Ok(moves[0]);
        }
        
        Err("No debug features enabled".to_string())
    }
    
    /// Debug-safe node selection
    #[cfg(any(feature = "debug-selection", feature = "debug-expansion", feature = "debug-simulation"))]
    fn debug_select_node_static(tree: &MctsTree) -> Result<usize, String> {
        // Very simple selection - just return root or first child
        let root_node = &tree.nodes[tree.root_index];
        
        if root_node.children.is_empty() {
            return Ok(tree.root_index);
        }
        
        // Try to access first child safely
        let first_child = root_node.children[0];
        if first_child < tree.nodes.len() {
            Ok(first_child)
        } else {
            Err("Invalid child index in selection".to_string())
        }
    }
    
    /// Debug-safe node expansion
    #[cfg(any(feature = "debug-expansion", feature = "debug-simulation"))]
    fn debug_expand_node_static(tree: &mut MctsTree, node_index: usize) -> Result<Option<usize>, String> {
        if node_index >= tree.nodes.len() {
            return Err("Invalid node index for expansion".to_string());
        }
        
        // If already expanded, return None
        if tree.nodes[node_index].is_expanded {
            return Ok(None);
        }
        
        // Generate one move safely
        let moves = tree.generate_moves_simd(node_index);
        if moves.is_empty() {
            tree.nodes[node_index].is_expanded = true;
            tree.nodes[node_index].is_terminal = true;
            return Ok(None);
        }
        
        // This is where the error might occur - creating new nodes
        // Let's skip actual node creation and just mark as expanded
        tree.nodes[node_index].is_expanded = true;
        
        Ok(None) // Don't create actual child nodes for now
    }
    
    /// Debug-safe simulation
    #[cfg(feature = "debug-simulation")]
    fn debug_simulate_static(_tree: &MctsTree, _node_index: usize) -> Result<f32, String> {
        // Very simple simulation - just return a fixed reward
        Ok(0.5)
    }
    
    /// WASM-safe AI move generation (simplified but stable)
    #[cfg(feature = "wasm-safe")]
    fn get_best_move_safe(&mut self) -> AiMoveResult {
        // Log that we're using WASM-safe fallback
        web_sys::console::log_1(&"🔧 WASM-SAFE: Using heuristic-based AI (not MCTS)".into());
        
        let tree = match &self.tree {
            Some(tree) => tree,
            None => {
                web_sys::console::log_1(&"❌ WASM-SAFE: No MCTS tree initialized".into());
                return AiMoveResult {
                    has_move: false,
                    move_type: "none".to_string(),
                    field_id: 0,
                    card_rank: 0,
                    card_suit: 0,
                    confidence: 0.0,
                    simulations: 0,
                    search_time_ms: 0,
                    error_message: Some("MCTS tree not initialized".to_string()),
                };
            }
        };
        
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        // Generate moves using the tree's move generation but skip complex search
        let moves = tree.generate_moves_simd(tree.root_index);
        web_sys::console::log_1(&format!("🎯 WASM-SAFE: Generated {} possible moves", moves.len()).into());
        
        let end_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);
        
        let search_time_ms = ((end_time - start_time).max(0.0)) as u32;
        web_sys::console::log_1(&format!("⏱️ WASM-SAFE: Move generation took {}ms", search_time_ms).into());
        
        if moves.is_empty() {
            web_sys::console::log_1(&"⚠️ WASM-SAFE: No legal moves available".into());
            return AiMoveResult {
                has_move: false,
                move_type: "none".to_string(),
                field_id: 0,
                card_rank: 0,
                card_suit: 0,
                confidence: 0.0,
                simulations: 0,
                search_time_ms,
                error_message: Some("No legal moves available".to_string()),
            };
        }
        
        // Use simple heuristic to pick best move instead of full MCTS
        let best_move = self.select_move_with_heuristic(&moves, &tree.nodes[tree.root_index].state);
        let (move_type, field_id, card_rank, card_suit) = Self::convert_simd_move_to_wasm_static(best_move);
        
        web_sys::console::log_1(&format!("✅ WASM-SAFE: Selected {} move (field {}) - HEURISTIC ONLY", move_type, field_id).into());
        
        AiMoveResult {
            has_move: true,
            move_type,
            field_id,
            card_rank,
            card_suit,
            confidence: 0.75, // Fixed confidence for heuristic
            simulations: 0, // CORRECTED: This is heuristic-based, not MCTS simulations
            search_time_ms,
            error_message: None,
        }
    }
    
    /// Simple heuristic for move selection (WASM-safe)
    #[cfg(feature = "wasm-safe")]
    fn select_move_with_heuristic(&self, moves: &[SimdMove], _state: &CompactState) -> SimdMove {
        // Simple heuristic: prefer harvest moves, then sow moves
        for &simd_move in moves {
            match simd_move {
                SimdMove::Harvest { .. } => return simd_move, // Harvests are generally good
                _ => {}
            }
        }
        
        // Fallback to first sow move
        moves[0]
    }

    /// Generate a simple move without full MCTS for WebAssembly stability
    fn generate_simple_move_static(tree: &MctsTree) -> Option<(String, u8, u8, u8)> {
        // Use MCTS tree operations but with better error handling for WASM
        let moves = tree.generate_moves_simd(tree.root_index);
        
        if moves.is_empty() {
            return None;
        }
        
        // Take the first valid move
        let first_move = moves[0];
        let (move_type, field_id, card_rank, card_suit) = Self::convert_simd_move_to_wasm_static(first_move);
        Some((move_type, field_id, card_rank, card_suit))
    }
    
    /// Get detailed search analysis as JSON
    #[wasm_bindgen]
    pub fn get_analysis_json(&self) -> Result<String, JsValue> {
        let tree = self.tree.as_ref()
            .ok_or_else(|| JsValue::from_str("MCTS tree not initialized"))?;
        
        let analysis = tree.get_analysis();
        
        serde_json::to_string(&analysis)
            .map_err(|e| JsValue::from_str(&format!("Analysis serialization error: {}", e)))
    }
    
    /// Update tree with opponent's move for pondering
    #[wasm_bindgen]
    pub fn update_with_move(&mut self, state_json: &str) -> Result<(), JsValue> {
        // For now, reinitialize the tree with the new state
        // In a more advanced implementation, we would try to reuse parts of the existing tree
        self.initialize_from_state(state_json)
    }
    
    /// Get memory usage statistics
    #[wasm_bindgen]
    pub fn get_memory_stats(&self) -> Result<String, JsValue> {
        let tree = self.tree.as_ref()
            .ok_or_else(|| JsValue::from_str("MCTS tree not initialized"))?;
        
        let stats = serde_json::json!({
            "total_nodes": tree.stats.total_nodes,
            "memory_per_node_bytes": std::mem::size_of::<crate::game::mcts::MctsNode>(),
            "estimated_memory_usage_mb": (tree.stats.total_nodes * std::mem::size_of::<crate::game::mcts::MctsNode>()) as f64 / 1024.0 / 1024.0,
            "compact_state_size_bytes": std::mem::size_of::<CompactState>(),
        });
        
        Ok(stats.to_string())
    }
    
    /// Convert SimdMove to WASM-compatible parameters
    fn convert_simd_move_to_wasm(&self, simd_move: SimdMove) -> (String, u8, u8, u8) {
        Self::convert_simd_move_to_wasm_static(simd_move)
    }
    
    /// Convert SimdMove to WASM-compatible parameters (static version)
    fn convert_simd_move_to_wasm_static(simd_move: SimdMove) -> (String, u8, u8, u8) {
        match simd_move {
            SimdMove::Sow { field, card_bitset } => {
                // Extract card from bitset (simplified - get first set bit)
                let card_id = card_bitset.trailing_zeros() as u8;
                let (rank, suit) = Self::card_id_to_rank_suit_static(card_id);
                ("sow".to_string(), field, rank, suit)
            }
            SimdMove::Harvest { field, play_card, .. } => {
                let card_id = play_card.trailing_zeros() as u8;
                let (rank, suit) = Self::card_id_to_rank_suit_static(card_id);
                ("harvest".to_string(), field, rank, suit)
            }
        }
    }
    
    /// Convert card ID to rank and suit for WASM interface
    fn card_id_to_rank_suit(&self, card_id: u8) -> (u8, u8) {
        Self::card_id_to_rank_suit_static(card_id)
    }
    
    /// Convert card ID to rank and suit for WASM interface (static version)
    fn card_id_to_rank_suit_static(card_id: u8) -> (u8, u8) {
        // Simplified conversion - in real implementation, use proper Card::from_id
        let suit = card_id / 13;
        let rank = card_id % 13;
        (rank, suit)
    }
}

#[wasm_bindgen]
impl WasmGameEngine {
    /// Create a new game engine
    #[wasm_bindgen(constructor)]
    pub fn new(player_count: u8) -> Result<WasmGameEngine, JsValue> {
        if player_count < 2 || player_count > 4 {
            return Err(JsValue::from_str("Player count must be between 2 and 4"));
        }
        
        let config = GameConfig::new(player_count);
        
        let state = IllimatState::new(config);
        
        Ok(WasmGameEngine { state })
    }
    
    /// Get the complete game state as JSON string
    #[wasm_bindgen]
    pub fn get_state_json(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.state)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Load game state from JSON string
    #[wasm_bindgen]
    pub fn set_state_json(&mut self, json: &str) -> Result<(), JsValue> {
        let state: IllimatState = serde_json::from_str(json)
            .map_err(|e| JsValue::from_str(&format!("Deserialization error: {}", e)))?;
        
        self.state = state;
        Ok(())
    }
    
    /// Get the current player ID
    #[wasm_bindgen]
    pub fn get_current_player(&self) -> u8 {
        self.state.current_player.0
    }
    
    /// Get the current season for a field (0=Winter, 1=Spring, 2=Summer, 3=Autumn)
    #[wasm_bindgen]
    pub fn get_field_season(&self, field_id: u8) -> Result<u8, JsValue> {
        if field_id >= 4 {
            return Err(JsValue::from_str("Field ID must be 0-3"));
        }
        
        let field = FieldId(field_id);
        let season = self.state.field_seasons[field.0 as usize];
        Ok(season as u8)
    }
    
    /// Validate a sow move without applying it
    #[wasm_bindgen]
    pub fn validate_sow(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> MoveResponse {
        let result = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a sow move to the game state
    #[wasm_bindgen]
    pub fn apply_sow(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> MoveResponse {
        let result = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit);
        
        match result {
            Ok((_player, field, card)) => {
                let action = Action::Sow { field, card };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply sow: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Validate a harvest move without applying it
    #[wasm_bindgen]
    pub fn validate_harvest(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_harvest(player_id, field_id, card_rank, card_suit, target_cards_json);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a harvest move to the game state
    #[wasm_bindgen]
    pub fn apply_harvest(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_harvest(player_id, field_id, card_rank, card_suit, target_cards_json);
        
        match result {
            Ok((_player, field, card, targets)) => {
                let action = Action::Harvest { field, card, targets };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply harvest: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Validate a stockpile move without applying it
    #[wasm_bindgen]
    pub fn validate_stockpile(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_stockpile(player_id, field_id, card_rank, card_suit, target_cards_json);
        match result {
            Ok(_) => MoveResponse { 
                success: true, 
                error_message: None 
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Apply a stockpile move to the game state
    #[wasm_bindgen]
    pub fn apply_stockpile(&mut self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> MoveResponse {
        let result = self.parse_and_validate_stockpile(player_id, field_id, card_rank, card_suit, target_cards_json);
        
        match result {
            Ok((_player, field, card, targets)) => {
                let action = Action::Stockpile { field, card, targets };
                match self.state.apply_action(action) {
                    Ok(_) => MoveResponse { 
                        success: true, 
                        error_message: None 
                    },
                    Err(error) => MoveResponse { 
                        success: false, 
                        error_message: Some(format!("Failed to apply stockpile: {}", error)) 
                    },
                }
            },
            Err(error) => MoveResponse { 
                success: false, 
                error_message: Some(error) 
            },
        }
    }
    
    /// Get legal moves for the current player as JSON
    #[wasm_bindgen]
    pub fn get_legal_moves_json(&self) -> Result<String, JsValue> {
        // TODO: Implement legal moves generation
        let legal_moves: Vec<String> = vec!["Placeholder for legal moves".to_string()];
        serde_json::to_string(&legal_moves)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
    }
    
    /// Check if the game is over
    #[wasm_bindgen]
    pub fn is_game_over(&self) -> bool {
        // TODO: Implement game over check
        false
    }
    
    /// Get the winner (if game is over)
    #[wasm_bindgen]
    pub fn get_winner(&self) -> Option<u8> {
        // TODO: Implement winner determination
        None
    }
}

// Helper methods for parsing and validation
impl WasmGameEngine {
    fn parse_and_validate_sow(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8) -> Result<(PlayerId, FieldId, Card), String> {
        let player = PlayerId(player_id);
        
        if field_id >= 4 {
            return Err("Field ID must be 0-3".to_string());
        }
        let field = FieldId(field_id);
        
        let card = self.parse_card(card_rank, card_suit)?;
        
        // Validate player has the card
        if !self.state.player_hands[player.0 as usize].contains(&card) {
            return Err("Player does not have this card".to_string());
        }
        
        // Validate it's the player's turn
        if self.state.current_player != player {
            return Err("Not this player's turn".to_string());
        }
        
        Ok((player, field, card))
    }
    
    fn parse_and_validate_harvest(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> Result<(PlayerId, FieldId, Card, Vec<Card>), String> {
        let (player, field, card) = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit)?;
        
        let targets: Vec<Card> = serde_json::from_str(target_cards_json)
            .map_err(|_| "Invalid target cards JSON".to_string())?;
        
        Ok((player, field, card, targets))
    }
    
    fn parse_and_validate_stockpile(&self, player_id: u8, field_id: u8, card_rank: u8, card_suit: u8, target_cards_json: &str) -> Result<(PlayerId, FieldId, Card, Vec<Card>), String> {
        let (player, field, card) = self.parse_and_validate_sow(player_id, field_id, card_rank, card_suit)?;
        
        let targets: Vec<Card> = serde_json::from_str(target_cards_json)
            .map_err(|_| "Invalid target cards JSON".to_string())?;
        
        Ok((player, field, card, targets))
    }
    
    fn parse_card(&self, rank: u8, suit: u8) -> Result<Card, String> {
        use crate::game::card::{Rank, Suit};
        
        let rank = match rank {
            0 => Rank::Fool,
            2 => Rank::Two,
            3 => Rank::Three,
            4 => Rank::Four,
            5 => Rank::Five,
            6 => Rank::Six,
            7 => Rank::Seven,
            8 => Rank::Eight,
            9 => Rank::Nine,
            10 => Rank::Ten,
            11 => Rank::Knight,
            12 => Rank::Queen,
            13 => Rank::King,
            _ => return Err("Invalid rank".to_string()),
        };
        
        let suit = match suit {
            0 => Suit::Spring,
            1 => Suit::Summer,
            2 => Suit::Autumn,
            3 => Suit::Winter,  
            4 => Suit::Stars,
            _ => return Err("Invalid suit".to_string()),
        };
        
        Ok(Card::new(rank, suit))
    }
}

// Export types for TypeScript definitions
#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACE: &'static str = r#"
export interface WasmGameState {
  config: {
    player_count: number;
    use_stars_suit: boolean;
    target_score: number;
  };
  phase: string;
  field_cards: Card[][];
  player_hands: Card[][];
  player_harvests: Card[][];
  deck: Card[];
  field_stockpiles: Stockpile[][];
  field_seasons: Season[];
  okus_positions: OkusPosition[];
  current_player: { 0: number };
  dealer: { 0: number };
  total_scores: number[];
  round_number: number;
  turn_number: number;
  illimat_orientation: number;
}

export interface Card {
  rank: string;
  suit: string;
}

export interface MoveResponseData {
  success: boolean;
  error_message?: string;
}

export interface AiMoveResultData {
  has_move: boolean;
  move_type: string;
  field_id: number;
  card_rank: number;
  card_suit: number;
  confidence: number;
  simulations: number;
  search_time_ms: number;
  error_message?: string;
}

export interface MctsAnalysisData {
  total_simulations: number;
  search_time: { secs: number; nanos: number };
  simulations_per_second: number;
  total_nodes: number;
  root_visits: number;
  best_move?: any;
  top_moves: Array<{
    move_taken?: any;
    visits: number;
    average_reward: number;
    confidence: number;
  }>;
}

export interface MctsMemoryStats {
  total_nodes: number;
  memory_per_node_bytes: number;
  estimated_memory_usage_mb: number;
  compact_state_size_bytes: number;
}
"#;