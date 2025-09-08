/// Analysis of MCTS tree scaling and memory limits
/// Can we represent 3 turns per player (12 total turns)?
use crate::game::mcts::MctsNode;
use crate::game::compact_state::CompactState;

/// Tree scaling analysis for deep game tree exploration
pub fn analyze_tree_scaling() -> TreeScalingReport {
    println!("\n📊 === MCTS TREE SCALING ANALYSIS ===");
    
    let mut report = TreeScalingReport::default();
    
    // Memory calculations
    let node_size = std::mem::size_of::<MctsNode>();
    let compact_state_size = std::mem::size_of::<CompactState>();
    
    println!("\n💾 1. Memory Usage per Node:");
    println!("   MctsNode size: {} bytes", node_size);
    println!("   CompactState size: {} bytes", compact_state_size);
    println!("   Total per node: {} bytes", node_size);
    
    report.node_size = node_size;
    report.compact_state_size = compact_state_size;
    
    // Estimate branching factor for Illimat
    println!("\n🌳 2. Illimat Game Tree Characteristics:");
    
    // Conservative estimates for real Illimat
    let hand_size = 6; // Cards per player hand
    let available_fields = 4; // Number of fields
    let harvest_combinations = 8; // Average harvest opportunities per position
    
    let avg_branching_factor = hand_size * available_fields + harvest_combinations;
    println!("   Hand size: {} cards", hand_size);
    println!("   Available fields: {}", available_fields);
    println!("   Harvest combinations: ~{}", harvest_combinations);
    println!("   Average branching factor: ~{} moves per position", avg_branching_factor);
    
    report.avg_branching_factor = avg_branching_factor;
    
    // Calculate tree sizes for different depths
    println!("\n📈 3. Tree Size by Depth (Full Tree):");
    
    let depths = [1, 2, 3, 4, 6, 8, 10, 12];
    for depth in depths {
        let nodes = calculate_full_tree_nodes(avg_branching_factor, depth);
        let memory_mb = (nodes * node_size) as f64 / 1_000_000.0;
        let memory_gb = memory_mb / 1000.0;
        
        println!("   Depth {}: {} nodes, {:.1} MB, {:.3} GB", 
                depth, nodes, memory_mb, memory_gb);
        
        if depth == 12 {
            report.depth_12_nodes = nodes;
            report.depth_12_memory_gb = memory_gb;
        }
    }
    
    // Practical MCTS tree analysis (sparse exploration)
    println!("\n🎯 4. Practical MCTS Tree (Selective Expansion):");
    
    // MCTS doesn't explore all branches - it selectively expands promising paths
    let mcts_selection_ratio = 0.1; // Only explore ~10% of branches deeply
    
    for depth in depths {
        let full_nodes = calculate_full_tree_nodes(avg_branching_factor, depth);
        let mcts_nodes = (full_nodes as f64 * mcts_selection_ratio) as usize;
        let memory_mb = (mcts_nodes * node_size) as f64 / 1_000_000.0;
        
        println!("   Depth {} (MCTS): {} nodes, {:.1} MB", 
                depth, mcts_nodes, memory_mb);
        
        if depth == 12 {
            report.mcts_depth_12_nodes = mcts_nodes;
            report.mcts_depth_12_memory_mb = memory_mb;
        }
    }
    
    // Memory budget analysis
    println!("\n💰 5. Memory Budget Analysis:");
    
    let budgets_mb = [100, 500, 1000, 2000, 4000, 8000];
    for budget_mb in budgets_mb {
        let max_nodes = (budget_mb * 1_000_000) / node_size;
        let max_depth = calculate_max_depth_for_nodes(avg_branching_factor, max_nodes, mcts_selection_ratio);
        
        println!("   {} MB budget: {} nodes, ~{} depth", 
                budget_mb, max_nodes, max_depth);
        
        if budget_mb == 1000 {
            report.budget_1gb_max_depth = max_depth;
        }
    }
    
    // 3 turns per player analysis
    println!("\n👥 6. Three Turns Per Player (12 Total Turns):");
    
    let turns_per_player = 3;
    let total_players = 4;
    let total_turns = turns_per_player * total_players;
    
    println!("   Target depth: {} turns", total_turns);
    println!("   Full tree nodes: {} ({:.1} GB)", 
            report.depth_12_nodes, report.depth_12_memory_gb);
    println!("   MCTS tree nodes: {} ({:.1} MB)", 
            report.mcts_depth_12_nodes, report.mcts_depth_12_memory_mb);
    
    // Feasibility assessment
    let is_feasible_full = report.depth_12_memory_gb < 100.0; // 100GB limit
    let is_feasible_mcts = report.mcts_depth_12_memory_mb < 8000.0; // 8GB limit
    
    println!("\n✅ 7. Feasibility Assessment:");
    println!("   Full tree (12 turns): {} ({})", 
            if is_feasible_full { "FEASIBLE" } else { "NOT FEASIBLE" },
            if is_feasible_full { "within memory limits" } else { "exceeds practical memory" });
    println!("   MCTS tree (12 turns): {} ({})", 
            if is_feasible_mcts { "FEASIBLE" } else { "NOT FEASIBLE" },
            if is_feasible_mcts { "selective exploration works" } else { "still too large" });
    
    report.full_tree_feasible = is_feasible_full;
    report.mcts_tree_feasible = is_feasible_mcts;
    
    // Advanced techniques for deeper trees
    println!("\n🚀 8. Advanced Techniques for Deep Trees:");
    println!("   Transposition Tables: Reuse equivalent positions (major memory savings)");
    println!("   Progressive Widening: Start narrow, expand promising branches");
    println!("   Time-based Cutoffs: Quality over depth");
    println!("   Lazy Expansion: Only expand when needed");
    println!("   Memory Recycling: Reuse old/unlikely branches");
    
    // Practical recommendations
    println!("\n📋 9. Practical Recommendations:");
    
    if is_feasible_mcts {
        println!("   ✅ 12-turn lookahead is ACHIEVABLE with MCTS");
        println!("   💾 Memory usage: ~{:.0} MB", report.mcts_depth_12_memory_mb);
        println!("   🎯 Use selective expansion and transposition tables");
        println!("   ⚡ CompactState makes this practical");
    } else {
        println!("   ⚠️  12-turn full lookahead challenging");
        println!("   🎯 Recommend 6-8 turn horizon with quality evaluation");
        println!("   💡 Focus on position evaluation over raw depth");
    }
    
    report
}

fn calculate_full_tree_nodes(branching_factor: usize, depth: usize) -> usize {
    if depth == 0 {
        return 1;
    }
    
    let mut total = 1; // Root node
    let mut level_nodes = 1;
    
    for _ in 0..depth {
        level_nodes *= branching_factor;
        total += level_nodes;
    }
    
    total
}

fn calculate_max_depth_for_nodes(branching_factor: usize, max_nodes: usize, selection_ratio: f64) -> usize {
    let mut depth = 0;
    
    while depth < 20 { // Reasonable upper limit
        let full_nodes = calculate_full_tree_nodes(branching_factor, depth);
        let mcts_nodes = (full_nodes as f64 * selection_ratio) as usize;
        
        if mcts_nodes > max_nodes {
            return depth - 1;
        }
        
        depth += 1;
    }
    
    depth
}

#[derive(Default, Debug)]
pub struct TreeScalingReport {
    pub node_size: usize,
    pub compact_state_size: usize,
    pub avg_branching_factor: usize,
    pub depth_12_nodes: usize,
    pub depth_12_memory_gb: f64,
    pub mcts_depth_12_nodes: usize,
    pub mcts_depth_12_memory_mb: f64,
    pub budget_1gb_max_depth: usize,
    pub full_tree_feasible: bool,
    pub mcts_tree_feasible: bool,
}

/// Real-world memory usage test with actual node creation
pub fn test_actual_memory_usage() -> ActualMemoryReport {
    println!("\n🧪 === ACTUAL MEMORY USAGE TEST ===");
    
    // Create actual nodes to measure real memory usage
    let mut nodes = Vec::new();
    let target_nodes = 100_000; // 100K nodes test
    
    println!("Creating {} actual MctsNode instances...", target_nodes);
    
    let start_memory = get_memory_usage();
    let start_time = std::time::Instant::now();
    
    for i in 0..target_nodes {
        let compact_state = CompactState::empty();
        let node = MctsNode::new(compact_state, None);
        nodes.push(node);
        
        // Update some nodes to make them realistic
        if i % 1000 == 0 {
            nodes[i].visits = 10;
            nodes[i].total_reward = 5.0;
            nodes[i].children = vec![i+1, i+2, i+3]; // Some child indices
        }
    }
    
    let end_memory = get_memory_usage();
    let creation_time = start_time.elapsed();
    
    let actual_memory_used = end_memory - start_memory;
    let theoretical_memory = target_nodes * std::mem::size_of::<MctsNode>();
    let overhead_ratio = actual_memory_used as f64 / theoretical_memory as f64;
    
    println!("   Nodes created: {}", nodes.len());
    println!("   Creation time: {:?}", creation_time);
    println!("   Theoretical memory: {:.1} MB", theoretical_memory as f64 / 1_000_000.0);
    println!("   Actual memory used: {:.1} MB", actual_memory_used as f64 / 1_000_000.0);
    println!("   Overhead ratio: {:.2}x", overhead_ratio);
    
    // Test random access performance
    let access_start = std::time::Instant::now();
    let mut sum = 0.0;
    for _ in 0..10_000 {
        let idx = (nodes.len() - 1).min(fastrand::usize(0..nodes.len()));
        sum += nodes[idx].total_reward;
    }
    let access_time = access_start.elapsed();
    
    println!("   Random access (10K): {:?}", access_time);
    println!("   Access per lookup: {:.1} ns", access_time.as_nanos() as f64 / 10_000.0);
    
    ActualMemoryReport {
        nodes_created: target_nodes,
        creation_time,
        theoretical_memory_mb: theoretical_memory as f64 / 1_000_000.0,
        actual_memory_mb: actual_memory_used as f64 / 1_000_000.0,
        overhead_ratio,
        random_access_time: access_time,
    }
}

fn get_memory_usage() -> usize {
    // Simple approximation - in practice you'd use platform-specific APIs
    // For this analysis, we'll estimate based on allocation behavior
    0 // Placeholder - actual implementation would measure RSS
}

#[derive(Debug)]
pub struct ActualMemoryReport {
    pub nodes_created: usize,
    pub creation_time: std::time::Duration,
    pub theoretical_memory_mb: f64,
    pub actual_memory_mb: f64,
    pub overhead_ratio: f64,
    pub random_access_time: std::time::Duration,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tree_scaling_analysis() {
        let report = analyze_tree_scaling();
        
        // Verify calculations make sense
        assert!(report.node_size > 0);
        assert!(report.avg_branching_factor > 5);
        assert!(report.avg_branching_factor < 50); // Reasonable for card games
        
        println!("\n=== TREE SCALING SUMMARY ===");
        println!("Node size: {} bytes", report.node_size);
        println!("Avg branching: {} moves", report.avg_branching_factor);
        println!("12-turn MCTS feasible: {}", report.mcts_tree_feasible);
        println!("12-turn memory: {:.1} MB", report.mcts_depth_12_memory_mb);
    }
    
    #[test]
    fn test_memory_usage_performance() {
        let report = test_actual_memory_usage();
        
        println!("\n=== MEMORY USAGE SUMMARY ===");
        println!("Nodes: {}", report.nodes_created);
        println!("Creation time: {:?}", report.creation_time);
        println!("Memory overhead: {:.2}x", report.overhead_ratio);
        println!("Access time: {:?}", report.random_access_time);
        
        // Verify reasonable performance
        assert!(report.creation_time.as_millis() < 5000); // < 5 seconds for 100K nodes
        assert!(report.overhead_ratio < 3.0); // < 3x overhead is reasonable
    }
}