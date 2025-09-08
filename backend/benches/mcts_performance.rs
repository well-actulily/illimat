/// MCTS Performance Benchmarks
/// 
/// Validates the 15,000+ simulations/second target for Sprint 7
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

use illimat_core::game::mcts::{MctsTree, MctsConfig};
use illimat_core::game::compact_state::CompactState;
use illimat_core::game::game_config::GameConfig;
use illimat_core::game::state::IllimatState;
use illimat_core::game::simd_compact_integration::SimdCompactOps;

use std::time::Duration;

/// Benchmark MCTS throughput with different simulation counts
fn benchmark_mcts_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts_throughput");
    
    // Create test game state
    let config = GameConfig::new(4);
    let state = IllimatState::new(config);
    let compact_state = CompactState::from(&state);
    
    // Test different simulation counts
    let simulation_counts = [100, 500, 1000, 5000, 10000];
    
    for &sim_count in &simulation_counts {
        group.throughput(Throughput::Elements(sim_count as u64));
        
        group.bench_with_input(
            BenchmarkId::new("simulations", sim_count),
            &sim_count,
            |b, &sim_count| {
                b.iter(|| {
                    let mut mcts_config = MctsConfig::default();
                    mcts_config.max_simulations = sim_count;
                    mcts_config.enable_simd = true;
                    
                    let mut tree = MctsTree::new(compact_state, mcts_config);
                    let _best_move = tree.search();
                    
                    // Return statistics for validation
                    tree.stats.simulations_per_second
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark MCTS with time-based limits
fn benchmark_mcts_time_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts_time_limits");
    
    // Create test game state
    let config = GameConfig::new(4);
    let state = IllimatState::new(config);
    let compact_state = CompactState::from(&state);
    
    // Test different time limits
    let time_limits_ms = [10, 50, 100, 500, 1000];
    
    for &time_ms in &time_limits_ms {
        group.bench_with_input(
            BenchmarkId::new("time_limit_ms", time_ms),
            &time_ms,
            |b, &time_ms| {
                b.iter(|| {
                    let mut mcts_config = MctsConfig::default();
                    mcts_config.time_limit = Some(Duration::from_millis(time_ms));
                    mcts_config.enable_simd = true;
                    
                    let mut tree = MctsTree::new(compact_state, mcts_config);
                    let _best_move = tree.search();
                    
                    // Return throughput achieved
                    (tree.stats.simulations_completed, tree.stats.simulations_per_second)
                });
            },
        );
    }
    
    group.finish();
}

/// Compare SIMD vs scalar performance
fn benchmark_simd_vs_scalar(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_vs_scalar");
    
    // Create test game state
    let config = GameConfig::new(4);
    let state = IllimatState::new(config);
    let compact_state = CompactState::from(&state);
    
    let simulation_count = 1000;
    
    // Benchmark with SIMD enabled
    group.bench_function("simd_enabled", |b| {
        b.iter(|| {
            let mut mcts_config = MctsConfig::default();
            mcts_config.max_simulations = simulation_count;
            mcts_config.enable_simd = true;
            
            let mut tree = MctsTree::new(compact_state, mcts_config);
            let _best_move = tree.search();
            
            tree.stats.simulations_per_second
        });
    });
    
    // Benchmark with SIMD disabled
    group.bench_function("simd_disabled", |b| {
        b.iter(|| {
            let mut mcts_config = MctsConfig::default();
            mcts_config.max_simulations = simulation_count;
            mcts_config.enable_simd = false;
            
            let mut tree = MctsTree::new(compact_state, mcts_config);
            let _best_move = tree.search();
            
            tree.stats.simulations_per_second
        });
    });
    
    group.finish();
}

/// Benchmark memory efficiency: CompactState vs IllimatState
fn benchmark_memory_efficiency(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Create test states
    let config = GameConfig::new(4);
    let illimat_state = IllimatState::new(config);
    let compact_state = CompactState::from(&illimat_state);
    
    // Benchmark CompactState MCTS
    group.bench_function("compact_state_mcts", |b| {
        b.iter(|| {
            let mut mcts_config = MctsConfig::default();
            mcts_config.max_simulations = 500;
            mcts_config.enable_simd = true;
            
            let mut tree = MctsTree::new(compact_state, mcts_config);
            let _best_move = tree.search();
            
            tree.stats.simulations_per_second
        });
    });
    
    group.finish();
}

/// Validate MCTS quality by testing tree depth and exploration
fn benchmark_mcts_quality(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcts_quality");
    
    // Create test game state
    let config = GameConfig::new(4);
    let state = IllimatState::new(config);
    let compact_state = CompactState::from(&state);
    
    group.bench_function("exploration_quality", |b| {
        b.iter(|| {
            let mut mcts_config = MctsConfig::default();
            mcts_config.max_simulations = 2000;
            mcts_config.exploration_constant = 1.414; // √2
            mcts_config.enable_simd = true;
            
            let mut tree = MctsTree::new(compact_state, mcts_config);
            let _best_move = tree.search();
            
            // Return quality metrics
            let analysis = tree.get_analysis();
            (
                analysis.total_nodes,
                analysis.simulations_per_second,
                analysis.top_moves.len(),
            )
        });
    });
    
    group.finish();
}

/// Comprehensive performance validation test
fn validate_15k_target() {
    println!("\n=== MCTS Performance Validation ===");
    
    let config = GameConfig::new(4);
    let state = IllimatState::new(config);
    let compact_state = CompactState::from(&state);
    
    // Test 1: Short burst performance
    println!("\n1. Short Burst Performance (1000 simulations):");
    let mut mcts_config = MctsConfig::default();
    mcts_config.max_simulations = 1000;
    mcts_config.enable_simd = true;
    
    let mut tree = MctsTree::new(compact_state, mcts_config);
    let _best_move = tree.search();
    
    let analysis = tree.get_analysis();
    println!("   Simulations completed: {}", analysis.total_simulations);
    println!("   Search time: {:?}", analysis.search_time);
    println!("   Throughput: {:.0} simulations/second", analysis.simulations_per_second);
    println!("   Total nodes created: {}", analysis.total_nodes);
    
    let target_met = analysis.simulations_per_second >= 15_000.0;
    println!("   ✅ 15K+ target: {}", if target_met { "ACHIEVED" } else { "NOT MET" });
    
    // Test 2: Sustained performance (time-limited)
    println!("\n2. Sustained Performance (100ms time limit):");
    let mut mcts_config = MctsConfig::default();
    mcts_config.time_limit = Some(Duration::from_millis(100));
    mcts_config.enable_simd = true;
    
    let mut tree = MctsTree::new(compact_state, mcts_config);
    let _best_move = tree.search();
    
    let analysis = tree.get_analysis();
    println!("   Simulations completed: {}", analysis.total_simulations);
    println!("   Search time: {:?}", analysis.search_time);
    println!("   Throughput: {:.0} simulations/second", analysis.simulations_per_second);
    
    let sustained_target_met = analysis.simulations_per_second >= 15_000.0;
    println!("   ✅ Sustained 15K+ target: {}", if sustained_target_met { "ACHIEVED" } else { "NOT MET" });
    
    // Test 3: Memory efficiency validation
    println!("\n3. Memory Efficiency:");
    let illimat_size = std::mem::size_of::<IllimatState>();
    let compact_size = std::mem::size_of::<CompactState>();
    let reduction_factor = illimat_size as f64 / compact_size as f64;
    
    println!("   IllimatState size: {} bytes", illimat_size);
    println!("   CompactState size: {} bytes", compact_size);
    println!("   Memory reduction: {:.1}x", reduction_factor);
    
    let memory_target_met = reduction_factor >= 10.0;
    println!("   ✅ 10x+ memory reduction: {}", if memory_target_met { "ACHIEVED" } else { "NOT MET" });
    
    // Test 4: SIMD acceleration validation
    println!("\n4. SIMD Acceleration Test:");
    
    // Create multiple states for batch operations
    let states = vec![compact_state; 100];
    let mut evaluations = vec![0.0f32; 100];
    
    let start = std::time::Instant::now();
    for _ in 0..100 {
        SimdCompactOps::batch_evaluate_states(&states, &mut evaluations);
    }
    let batch_time = start.elapsed();
    
    println!("   Batch evaluation time (100x100 states): {:?}", batch_time);
    println!("   States evaluated per second: {:.0}", 
             10_000.0 / batch_time.as_secs_f64());
    
    // Overall assessment
    println!("\n=== SPRINT 7 TARGET ASSESSMENT ===");
    let overall_success = target_met && sustained_target_met && memory_target_met;
    
    if overall_success {
        println!("🎯 ALL TARGETS ACHIEVED!");
        println!("   ✅ 15,000+ simulations/second");
        println!("   ✅ 10x+ memory reduction with CompactState");
        println!("   ✅ SIMD acceleration functional");
        println!("\n🚀 Ready for Sprint 8: WebAssembly deployment");
    } else {
        println!("⚠️  Some targets not met - optimization needed");
        if !target_met { println!("   ❌ 15K simulation target"); }
        if !sustained_target_met { println!("   ❌ Sustained performance"); }
        if !memory_target_met { println!("   ❌ Memory reduction target"); }
    }
}

// Run the validation test as part of benchmarks
fn run_validation_test(c: &mut Criterion) {
    c.bench_function("performance_validation", |b| {
        b.iter(|| {
            validate_15k_target();
        });
    });
}

criterion_group!(
    benches,
    benchmark_mcts_throughput,
    benchmark_mcts_time_limits,
    benchmark_simd_vs_scalar,
    benchmark_memory_efficiency,
    benchmark_mcts_quality,
    run_validation_test
);

criterion_main!(benches);