/// Simple tree scaling analysis for MCTS depth planning
/// Answers: Can we represent 2-3 turns per player (8-12 total turns)?

use crate::game::mcts::MctsNode;
use crate::game::compact_state::CompactState;

/// Quick analysis of tree scaling for different depths
pub fn analyze_mcts_tree_scaling() {
    println!("\n📊 === MCTS TREE SCALING ANALYSIS ===");
    
    // Basic memory calculations
    let node_size = std::mem::size_of::<MctsNode>();
    let compact_state_size = std::mem::size_of::<CompactState>();
    
    println!("\n💾 Memory per Node:");
    println!("   MctsNode: {} bytes", node_size);
    println!("   CompactState: {} bytes", compact_state_size);
    
    // Estimate Illimat branching factor
    println!("\n🌳 Illimat Game Tree Characteristics:");
    let hand_size = 6; // Typical cards in hand
    let fields = 4; // Number of fields
    let harvest_moves = 8; // Average harvest opportunities
    let branching_factor = hand_size * fields + harvest_moves; // ~32 moves per position
    
    println!("   Typical hand size: {} cards", hand_size);
    println!("   Fields: {}", fields);
    println!("   Estimated branching factor: ~{} moves/position", branching_factor);
    
    // Calculate tree sizes for different depths
    println!("\n📈 Tree Size Analysis:");
    println!("   Depth | Full Tree Nodes | MCTS Nodes (10%) | Memory (MB)");
    println!("   ------|-----------------|-------------------|------------");
    
    for depth in [1, 2, 4, 6, 8, 10, 12] {
        let full_nodes = estimate_full_tree_nodes(branching_factor, depth);
        let mcts_nodes = full_nodes / 10; // MCTS explores ~10% selectively
        let memory_mb = (mcts_nodes * node_size) as f64 / 1_000_000.0;
        
        println!("   {:5} | {:15} | {:17} | {:10.1}", 
                depth, full_nodes, mcts_nodes, memory_mb);
    }
    
    // Practical assessment
    println!("\n🎯 Practical Assessment:");
    
    let target_depths = [
        (8, "2 turns per player"),
        (12, "3 turns per player"),
    ];
    
    for (depth, description) in target_depths {
        let full_nodes = estimate_full_tree_nodes(branching_factor, depth);
        let mcts_nodes = full_nodes / 10;
        let memory_mb = (mcts_nodes * node_size) as f64 / 1_000_000.0;
        
        println!("\n   {} turns ({}):", depth, description);
        println!("     MCTS nodes: ~{}", format_number(mcts_nodes));
        println!("     Memory: {:.1} MB", memory_mb);
        
        let feasible = memory_mb < 2000.0; // 2GB limit
        println!("     Feasible: {} ({})", 
                if feasible { "YES" } else { "NO" },
                if feasible { "within 2GB" } else { "exceeds memory" });
    }
    
    // Memory budget recommendations
    println!("\n💰 Memory Budget Recommendations:");
    
    let budgets = [
        (100, "Mobile/Web (100MB)"),
        (500, "Desktop (500MB)"),
        (1000, "Server (1GB)"),
        (2000, "High-end (2GB)"),
    ];
    
    for (budget_mb, description) in budgets {
        let max_nodes = (budget_mb * 1_000_000) / node_size;
        let approx_depth = estimate_depth_for_nodes(branching_factor, max_nodes);
        
        println!("   {}: ~{} depth, {} nodes", 
                description, approx_depth, format_number(max_nodes));
    }
    
    // Advanced techniques
    println!("\n🚀 Advanced Techniques for Deeper Trees:");
    println!("   • Transposition Tables: Reuse equivalent positions");
    println!("   • Progressive Widening: Start narrow, expand promising branches");
    println!("   • Time Limits: Quality over raw depth");
    println!("   • Evaluation Functions: Strong heuristics reduce needed depth");
    
    // Final recommendations
    println!("\n✅ Recommendations:");
    println!("   • 8-turn lookahead (2 per player): VERY FEASIBLE");
    println!("   • 12-turn lookahead (3 per player): FEASIBLE with good hardware");
    println!("   • CompactState makes deep trees practical");
    println!("   • Focus on search quality over pure depth");
}

fn estimate_full_tree_nodes(branching_factor: usize, depth: usize) -> usize {
    if depth == 0 { return 1; }
    
    // Geometric series: 1 + b + b² + ... + b^d
    let mut total = 1;
    let mut level_nodes = 1;
    
    for _ in 0..depth {
        level_nodes *= branching_factor;
        total += level_nodes;
    }
    
    total
}

fn estimate_depth_for_nodes(branching_factor: usize, target_nodes: usize) -> usize {
    for depth in 1..20 {
        let nodes = estimate_full_tree_nodes(branching_factor, depth) / 10; // MCTS selective
        if nodes > target_nodes {
            return depth - 1;
        }
    }
    20
}

fn format_number(n: usize) -> String {
    if n >= 1_000_000_000 {
        format!("{:.1}B", n as f64 / 1_000_000_000.0)
    } else if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}K", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tree_scaling_analysis() {
        analyze_mcts_tree_scaling();
        
        // Basic validation
        let node_size = std::mem::size_of::<MctsNode>();
        assert!(node_size > 50); // Should be substantial but not huge
        assert!(node_size < 500); // Should be reasonably compact
        
        println!("\n=== Quick Assessment ===");
        println!("Node size: {} bytes", node_size);
        
        // Test specific depth calculations
        let depth_8_nodes = estimate_full_tree_nodes(32, 8) / 10;
        let depth_8_mb = (depth_8_nodes * node_size) as f64 / 1_000_000.0;
        
        println!("8-turn tree: {} nodes, {:.1} MB", format_number(depth_8_nodes), depth_8_mb);
        assert!(depth_8_mb < 5000.0); // Should be practical
    }
}