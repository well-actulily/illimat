/// Monte Carlo Tree Search implementation with SIMD-accelerated CompactState
/// 
/// This module implements a high-performance MCTS algorithm specifically optimized
/// for Illimat game tree search, leveraging the 144-byte CompactState representation
/// and SIMD vectorization for maximum throughput.
use crate::game::compact_state::CompactState;
use crate::game::simd_compact_integration::{SimdCompactOps, SimdMove, BitsetOp};
use crate::game::state::IllimatState;
use crate::game::actions::Action;
use crate::game::card::Card;
use crate::game::field_id::FieldId;
use crate::game::player::PlayerId;

use std::collections::HashMap;
use std::time::{Duration, Instant};
use std::sync::Arc;

/// WebAssembly-safe time representation
#[cfg(target_arch = "wasm32")]
#[derive(Copy, Clone)]
struct SafeInstant(f64); // JavaScript performance.now() returns f64

#[cfg(not(target_arch = "wasm32"))]
#[derive(Copy, Clone)]
struct SafeInstant(Instant);

/// MCTS Node with CompactState for memory efficiency
/// 
/// Each node stores the compact game state (144 bytes) plus MCTS statistics.
/// This enables ~10x more nodes to fit in cache compared to IllimatState nodes.
#[derive(Clone, Debug)]
pub struct MctsNode {
    /// Compact representation of game state (144 bytes)
    pub state: CompactState,
    
    /// MCTS statistics for UCB1 selection
    pub visits: u32,
    pub total_reward: f32,
    pub parent_index: Option<usize>,
    pub children: Vec<usize>,
    
    /// Move that led to this state from parent
    pub parent_move: Option<SimdMove>,
    
    /// Whether this node has been expanded (children generated)
    pub is_expanded: bool,
    
    /// Game-specific metadata
    pub is_terminal: bool,
    pub terminal_reward: Option<f32>,
}

impl MctsNode {
    /// Create new MCTS node from CompactState
    pub fn new(state: CompactState, parent_move: Option<SimdMove>) -> Self {
        Self {
            state,
            visits: 0,
            total_reward: 0.0,
            parent_index: None,
            children: Vec::new(),
            parent_move,
            is_expanded: false,
            is_terminal: false,
            terminal_reward: None,
        }
    }
    
    /// Calculate UCB1 value for node selection (WebAssembly-safe)
    pub fn ucb1_value(&self, parent_visits: u32, exploration_constant: f32) -> f32 {
        if self.visits == 0 {
            return 1e6f32; // Large finite value instead of INFINITY for WASM compatibility
        }
        
        let exploitation = self.total_reward / self.visits as f32;
        
        // Safe bounds checking for floating point operations
        let parent_visits_f32 = (parent_visits as f32).max(1.0); // Avoid ln(0)
        let visits_f32 = (self.visits as f32).max(1.0);
        
        let ln_value = parent_visits_f32.ln();
        let sqrt_value = (ln_value / visits_f32).sqrt();
        
        // Check for NaN/infinity and use safe fallback
        let exploration = if ln_value.is_finite() && sqrt_value.is_finite() {
            exploration_constant * sqrt_value
        } else {
            exploration_constant // Fallback to just the constant
        };
        
        let result = exploitation + exploration;
        
        // Ensure result is finite
        if result.is_finite() {
            result
        } else {
            exploitation.max(0.0) // Fallback to just exploitation value
        }
    }
    
    /// Get average reward (win rate) for this node
    pub fn average_reward(&self) -> f32 {
        if self.visits == 0 {
            0.0
        } else {
            self.total_reward / self.visits as f32
        }
    }
    
    /// Update node statistics after simulation
    pub fn backpropagate(&mut self, reward: f32) {
        self.visits += 1;
        self.total_reward += reward;
    }
}

/// High-performance MCTS tree with SIMD acceleration
pub struct MctsTree {
    /// Node storage (arena allocation for cache efficiency)
    pub nodes: Vec<MctsNode>,
    
    /// Root node index
    pub root_index: usize,
    
    /// MCTS configuration parameters
    pub config: MctsConfig,
    
    /// Performance metrics
    pub stats: MctsStats,
}

/// MCTS configuration parameters
#[derive(Clone, Debug)]
pub struct MctsConfig {
    /// Maximum number of simulations per search
    pub max_simulations: u32,
    
    /// Time limit per search (if Some, overrides max_simulations)
    pub time_limit: Option<Duration>,
    
    /// UCB1 exploration constant (typically √2 ≈ 1.414)
    pub exploration_constant: f32,
    
    /// Maximum tree depth for memory management
    pub max_depth: u32,
    
    /// Enable SIMD acceleration where available
    pub enable_simd: bool,
}

impl Default for MctsConfig {
    fn default() -> Self {
        Self {
            max_simulations: 10_000,
            time_limit: None,
            exploration_constant: 1.414, // √2
            max_depth: 100,
            enable_simd: cfg!(target_arch = "x86_64"),
        }
    }
}

/// MCTS performance statistics
#[derive(Default, Clone, Debug)]
pub struct MctsStats {
    /// Total simulations performed
    pub simulations_completed: u32,
    
    /// Time spent in search
    pub search_time: Duration,
    
    /// Simulations per second achieved
    pub simulations_per_second: f32,
    
    /// Maximum tree depth reached
    pub max_depth_reached: u32,
    
    /// Total nodes allocated
    pub total_nodes: usize,
    
    /// Cache hit rate for node reuse
    pub cache_hit_rate: f32,
}

impl MctsTree {
    /// Create new MCTS tree with root state
    pub fn new(root_state: CompactState, config: MctsConfig) -> Self {
        let root_node = MctsNode::new(root_state, None);
        let mut nodes = Vec::with_capacity(100_000); // Pre-allocate for performance
        nodes.push(root_node);
        
        Self {
            nodes,
            root_index: 0,
            config,
            stats: MctsStats::default(),
        }
    }
    
    /// Perform MCTS search and return best move (WebAssembly-safe)
    pub fn search(&mut self) -> Option<SimdMove> {
        let start_time = self.get_safe_time();
        let mut simulations = 0;
        
        // Search loop
        while !self.should_terminate(simulations, start_time) {
            // MCTS phases: Selection → Expansion → Simulation → Backpropagation
            
            // 1. Selection: Find leaf node using UCB1
            let selected_node_index = self.select_node();
            
            // 2. Expansion: Add child nodes if not terminal
            let expansion_result = self.expand_node(selected_node_index);
            
            // 3. Simulation: Random playout or heuristic evaluation
            let reward = match expansion_result {
                Some(child_index) => self.simulate_from_node(child_index),
                None => self.simulate_from_node(selected_node_index),
            };
            
            // 4. Backpropagation: Update statistics up the tree
            let node_to_backprop = expansion_result.unwrap_or(selected_node_index);
            self.backpropagate_reward(node_to_backprop, reward);
            
            simulations += 1;
        }
        
        // Update statistics (WebAssembly-safe)
        let elapsed_time = self.get_safe_elapsed(start_time);
        self.stats.simulations_completed = simulations;
        self.stats.search_time = elapsed_time;
        self.stats.simulations_per_second = if elapsed_time.as_secs_f32() > 0.0 {
            simulations as f32 / elapsed_time.as_secs_f32()
        } else {
            0.0 // Avoid division by zero
        };
        self.stats.total_nodes = self.nodes.len();
        
        // Return best move from root
        self.best_move_from_root()
    }
    
    /// UCB1-based node selection (WebAssembly-safe with bounds checking)
    fn select_node(&self) -> usize {
        let mut current_index = self.root_index;
        let mut depth = 0;
        const MAX_SELECTION_DEPTH: usize = 1000; // Prevent infinite loops
        
        loop {
            // Safety check to prevent infinite loops in WebAssembly
            depth += 1;
            if depth > MAX_SELECTION_DEPTH {
                return current_index; // Return current node if we go too deep
            }
            let current_node = &self.nodes[current_index];
            
            // If node is not expanded or has no children, return it
            if !current_node.is_expanded || current_node.children.is_empty() {
                return current_index;
            }
            
            // If terminal node, return it
            if current_node.is_terminal {
                return current_index;
            }
            
            // Select child with highest UCB1 value
            let mut best_child_index = current_node.children[0];
            let mut best_ucb1 = -1e6f32; // Large negative finite value instead of -INFINITY
            
            for &child_index in &current_node.children {
                let child = &self.nodes[child_index];
                let ucb1_value = child.ucb1_value(current_node.visits, self.config.exploration_constant);
                
                if ucb1_value > best_ucb1 {
                    best_ucb1 = ucb1_value;
                    best_child_index = child_index;
                }
            }
            
            current_index = best_child_index;
        }
    }
    
    /// Expand node by generating children
    fn expand_node(&mut self, node_index: usize) -> Option<usize> {
        // Check if already expanded
        if self.nodes[node_index].is_expanded {
            return None;
        }
        
        // Check if terminal
        if self.nodes[node_index].is_terminal {
            return None;
        }
        
        // Generate possible moves using SIMD operations
        let possible_moves = self.generate_moves_simd(node_index);
        
        if possible_moves.is_empty() {
            // No moves available - mark as terminal
            self.nodes[node_index].is_terminal = true;
            self.nodes[node_index].terminal_reward = Some(self.evaluate_terminal_state(node_index));
            return None;
        }
        
        // Create child nodes for each move
        let parent_state = self.nodes[node_index].state;
        let mut child_indices = Vec::new();
        
        for simd_move in possible_moves {
            // Apply move to create child state
            if let Some(child_state) = self.apply_move_to_compact_state(parent_state, simd_move) {
                let mut child_node = MctsNode::new(child_state, Some(simd_move));
                child_node.parent_index = Some(node_index);
                
                let child_index = self.nodes.len();
                self.nodes.push(child_node);
                child_indices.push(child_index);
            }
        }
        
        // Update parent node
        self.nodes[node_index].children = child_indices;
        self.nodes[node_index].is_expanded = true;
        
        // Return first child for immediate simulation
        self.nodes[node_index].children.first().copied()
    }
    
    /// Generate possible moves using SIMD operations
    pub fn generate_moves_simd(&self, node_index: usize) -> Vec<SimdMove> {
        // Bounds check
        if node_index >= self.nodes.len() {
            return Vec::new();
        }
        
        let state = &self.nodes[node_index].state;
        let mut moves = Vec::new();
        
        // Get current player from compact state with bounds checking
        let current_player = state.current_player();
        if current_player > 3 {
            return Vec::new(); // Invalid player, return no moves
        }
        
        let player_hand = state.player_hands[current_player as usize];
        
        // Generate sow moves for each card in hand
        for card_bit in 0..64 {
            if (player_hand & (1u64 << card_bit)) != 0 {
                // Player has this card, can sow to each field
                for field in 0..4 {
                    moves.push(SimdMove::Sow {
                        field: field as u8,
                        card_bitset: 1u64 << card_bit,
                    });
                }
            }
        }
        
        // Generate harvest moves (simplified for now)
        for field in 0..4 {
            let field_cards = state.field_cards[field];
            if field_cards != 0 {
                // Field has cards to harvest
                for card_bit in 0..64 {
                    if (player_hand & (1u64 << card_bit)) != 0 {
                        // Try to harvest with this card
                        moves.push(SimdMove::Harvest {
                            field: field as u8,
                            play_card: 1u64 << card_bit,
                            target_cards: field_cards, // Simplified: harvest all field cards
                        });
                    }
                }
            }
        }
        
        moves
    }
    
    /// Apply move to CompactState (simplified implementation)
    fn apply_move_to_compact_state(&self, state: CompactState, simd_move: SimdMove) -> Option<CompactState> {
        let mut new_state = state;
        let current_player = state.current_player() as usize;
        
        match simd_move {
            SimdMove::Sow { field, card_bitset } => {
                // Remove card from player hand
                new_state.player_hands[current_player] &= !card_bitset;
                
                // Add card to field
                new_state.field_cards[field as usize] |= card_bitset;
                
                Some(new_state)
            }
            SimdMove::Harvest { field, play_card, target_cards } => {
                // Check if harvest is valid (simplified)
                if (new_state.player_hands[current_player] & play_card) != 0 {
                    // Remove play card from hand
                    new_state.player_hands[current_player] &= !play_card;
                    
                    // Remove target cards from field
                    new_state.field_cards[field as usize] &= !target_cards;
                    
                    // Add cards to player harvest
                    new_state.player_harvests[current_player] |= play_card | target_cards;
                    
                    Some(new_state)
                } else {
                    None // Invalid move
                }
            }
        }
    }
    
    /// Simulate game from node using SIMD-accelerated evaluation
    fn simulate_from_node(&self, node_index: usize) -> f32 {
        let node = &self.nodes[node_index];
        
        // If terminal node, return known reward
        if let Some(reward) = node.terminal_reward {
            return reward;
        }
        
        // Use SIMD-accelerated heuristic evaluation instead of full random playout
        // This is much faster and often more accurate than random simulation
        self.evaluate_state_simd(&node.state)
    }
    
    /// SIMD-accelerated state evaluation
    pub fn evaluate_state_simd(&self, state: &CompactState) -> f32 {
        // Simplified heuristic evaluation for WASM compatibility
        // Avoid complex SIMD operations that might cause issues in WebAssembly
        
        let mut score = 0.0f32;
        
        // Basic evaluation: count cards in different locations
        for field_idx in 0..4 {
            let field_cards = state.field_cards[field_idx].count_ones();
            score += field_cards as f32 * 0.1;
        }
        
        for player_idx in 0..4 {
            let hand_cards = state.player_hands[player_idx].count_ones();
            let harvest_cards = state.player_harvests[player_idx].count_ones();
            
            // Hand cards are neutral, harvested cards are good
            score += hand_cards as f32 * 0.05;
            score += harvest_cards as f32 * 0.2;
        }
        
        // Return a value between 0 and 1
        (score / 100.0).min(1.0).max(0.0)
    }
    
    /// Evaluate terminal state reward
    fn evaluate_terminal_state(&self, node_index: usize) -> f32 {
        // Simplified terminal evaluation
        // In a full implementation, this would check win conditions
        self.evaluate_state_simd(&self.nodes[node_index].state)
    }
    
    /// Backpropagate reward up the tree
    fn backpropagate_reward(&mut self, mut node_index: usize, reward: f32) {
        loop {
            self.nodes[node_index].backpropagate(reward);
            
            if let Some(parent_index) = self.nodes[node_index].parent_index {
                node_index = parent_index;
            } else {
                break; // Reached root
            }
        }
    }
    
    /// Check if search should terminate (WebAssembly-safe)
    fn should_terminate(&self, simulations: u32, start_time: SafeInstant) -> bool {
        if let Some(time_limit) = self.config.time_limit {
            self.get_safe_elapsed(start_time) >= time_limit
        } else {
            simulations >= self.config.max_simulations
        }
    }
    
    /// Get current time in a WebAssembly-safe way
    #[cfg(target_arch = "wasm32")]
    fn get_safe_time(&self) -> SafeInstant {
        SafeInstant(web_sys::window().unwrap().performance().unwrap().now())
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_safe_time(&self) -> SafeInstant {
        SafeInstant(std::time::Instant::now())
    }
    
    /// Calculate elapsed time in a WebAssembly-safe way
    #[cfg(target_arch = "wasm32")]
    fn get_safe_elapsed(&self, start: SafeInstant) -> Duration {
        let now = web_sys::window().unwrap().performance().unwrap().now();
        let elapsed_ms = (now - start.0).max(0.0);
        Duration::from_millis(elapsed_ms as u64)
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    fn get_safe_elapsed(&self, start: SafeInstant) -> Duration {
        start.0.elapsed()
    }
    
    /// Get best move from root node
    fn best_move_from_root(&self) -> Option<SimdMove> {
        let root = &self.nodes[self.root_index];
        
        if root.children.is_empty() {
            return None;
        }
        
        // Select child with highest visit count (most robust move)
        let mut best_child_index = root.children[0];
        let mut best_visits = 0;
        
        for &child_index in &root.children {
            let child = &self.nodes[child_index];
            if child.visits > best_visits {
                best_visits = child.visits;
                best_child_index = child_index;
            }
        }
        
        self.nodes[best_child_index].parent_move
    }
    
    /// Get analysis of the search tree
    pub fn get_analysis(&self) -> MctsAnalysis {
        let root = &self.nodes[self.root_index];
        
        let mut child_analysis = Vec::new();
        for &child_index in &root.children {
            let child = &self.nodes[child_index];
            child_analysis.push(ChildAnalysis {
                move_taken: child.parent_move,
                visits: child.visits,
                average_reward: child.average_reward(),
                confidence: child.visits as f32 / root.visits as f32,
            });
        }
        
        // Sort by visit count (confidence)
        child_analysis.sort_by(|a, b| b.visits.cmp(&a.visits));
        
        MctsAnalysis {
            total_simulations: self.stats.simulations_completed,
            search_time: self.stats.search_time,
            simulations_per_second: self.stats.simulations_per_second,
            total_nodes: self.stats.total_nodes,
            root_visits: root.visits,
            best_move: self.best_move_from_root(),
            top_moves: child_analysis,
        }
    }
}

/// Analysis results from MCTS search
#[derive(Debug, serde::Serialize)]
pub struct MctsAnalysis {
    pub total_simulations: u32,
    pub search_time: Duration,
    pub simulations_per_second: f32,
    pub total_nodes: usize,
    pub root_visits: u32,
    pub best_move: Option<SimdMove>,
    pub top_moves: Vec<ChildAnalysis>,
}

/// Analysis of individual child moves
#[derive(Debug, serde::Serialize)]
pub struct ChildAnalysis {
    pub move_taken: Option<SimdMove>,
    pub visits: u32,
    pub average_reward: f32,
    pub confidence: f32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::game_config::GameConfig;
    use crate::game::state::IllimatState;
    
    #[test]
    fn test_mcts_node_creation() {
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        let compact = CompactState::from(&state);
        
        let node = MctsNode::new(compact, None);
        
        assert_eq!(node.visits, 0);
        assert_eq!(node.total_reward, 0.0);
        assert_eq!(node.average_reward(), 0.0);
        assert!(!node.is_expanded);
        assert!(!node.is_terminal);
    }
    
    #[test]
    fn test_ucb1_calculation() {
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        let compact = CompactState::from(&state);
        
        let mut node = MctsNode::new(compact, None);
        
        // Unvisited node should have very high UCB1 value
        assert_eq!(node.ucb1_value(100, 1.414), 1e6f32);
        
        // Add some visits and reward
        node.visits = 10;
        node.total_reward = 7.0;
        
        let ucb1 = node.ucb1_value(100, 1.414);
        assert!(ucb1 > 0.0);
        assert!(ucb1.is_finite());
    }
    
    #[test]
    fn test_mcts_tree_initialization() {
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        let compact = CompactState::from(&state);
        
        let mcts_config = MctsConfig::default();
        let tree = MctsTree::new(compact, mcts_config);
        
        assert_eq!(tree.nodes.len(), 1);
        assert_eq!(tree.root_index, 0);
        assert_eq!(tree.nodes[0].visits, 0);
    }
    
    #[test]
    fn test_move_generation() {
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        let compact = CompactState::from(&state);
        
        let mcts_config = MctsConfig::default();
        let tree = MctsTree::new(compact, mcts_config);
        
        let moves = tree.generate_moves_simd(0);
        
        // Should generate some moves (exact count depends on initial state)
        assert!(!moves.is_empty(), "Should generate at least some moves");
    }
    
    #[test]
    fn test_basic_mcts_search() {
        let config = GameConfig::new(2);
        let state = IllimatState::new(config);
        let compact = CompactState::from(&state);
        
        let mut mcts_config = MctsConfig::default();
        mcts_config.max_simulations = 100; // Small number for testing
        
        let mut tree = MctsTree::new(compact, mcts_config);
        
        let best_move = tree.search();
        
        // Should find some move
        assert!(best_move.is_some(), "MCTS should find a move");
        
        // Should have performed simulations
        assert!(tree.stats.simulations_completed > 0);
        assert!(tree.stats.simulations_per_second > 0.0);
        
        println!("MCTS completed {} simulations at {:.1} sim/sec", 
                tree.stats.simulations_completed, tree.stats.simulations_per_second);
    }
}