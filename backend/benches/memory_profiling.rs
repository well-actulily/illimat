use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use pprof::criterion::{Output, PProfProfiler};
use illimat_core::game::{
    card::{Card, Rank, Suit},
    state::IllimatState,
    game_config::GameConfig,
    actions::Action,
    field_id::FieldId,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Custom allocator wrapper for tracking memory usage
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
static DEALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        DEALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

/// Reset allocation counters
fn reset_allocation_counters() {
    ALLOCATED.store(0, Ordering::SeqCst);
    DEALLOCATED.store(0, Ordering::SeqCst);
}

/// Get current allocation statistics
fn get_allocation_stats() -> (usize, usize, usize) {
    let allocated = ALLOCATED.load(Ordering::SeqCst);
    let deallocated = DEALLOCATED.load(Ordering::SeqCst);
    let net_allocated = allocated.saturating_sub(deallocated);
    (allocated, deallocated, net_allocated)
}

/// Create game configurations for different scenarios
fn create_memory_test_configs() -> Vec<(String, GameConfig)> {
    vec![
        ("2_player_minimal".to_string(), GameConfig::new(2).with_deck_size(false)),
        ("4_player_full".to_string(), GameConfig::new(4).with_deck_size(true)),
    ]
}

/// Benchmark memory usage during game state creation
fn bench_memory_game_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_game_creation");
    
    for (name, config) in create_memory_test_configs() {
        group.bench_with_input(
            BenchmarkId::new("create_game", &name),
            &config,
            |b, config| {
                b.iter_batched_ref(
                    || {
                        reset_allocation_counters();
                        config.clone()
                    },
                    |config| {
                        let game = IllimatState::new(config.clone());
                        black_box(game);
                        let (allocated, _deallocated, net) = get_allocation_stats();
                        println!("Game creation ({}): {} bytes allocated, {} net", name, allocated, net);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage during game state cloning
fn bench_memory_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_cloning");
    
    let game = IllimatState::new(GameConfig::new(2).with_deck_size(false));
    
    group.bench_function("clone_simple_state", |b| {
        b.iter_batched_ref(
            || {
                reset_allocation_counters();
                &game
            },
            |game| {
                let cloned = (*game).clone();
                black_box(cloned);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("Game clone: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    // Test cloning complex state with many cards in fields
    let mut complex_game = IllimatState::new(GameConfig::new(4).with_deck_size(true));
    
    // Add cards to all fields to create more complex state
    for field_idx in 0..4 {
        for i in 0..10 {
            let card = Card::new(
                match i % 4 {
                    0 => Rank::Five,
                    1 => Rank::Eight,
                    2 => Rank::Ten,
                    _ => Rank::King,
                },
                match field_idx {
                    0 => Suit::Spring,
                    1 => Suit::Summer,
                    2 => Suit::Autumn,
                    _ => Suit::Winter,
                }
            );
            complex_game.field_cards[field_idx].push(card);
        }
    }
    
    group.bench_function("clone_complex_state", |b| {
        b.iter_batched_ref(
            || {
                reset_allocation_counters();
                &complex_game
            },
            |game| {
                let cloned = (*game).clone();
                black_box(cloned);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("Complex game clone: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark memory usage during action application
fn bench_memory_actions(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_actions");
    
    // Test sow action memory usage
    group.bench_function("sow_action_memory", |b| {
        b.iter_batched(
            || {
                reset_allocation_counters();
                let game = IllimatState::new(GameConfig::new(2));
                let card = game.player_hands[0][0];
                (game, card)
            },
            |(mut game, card)| {
                let action = Action::Sow {
                    field: FieldId(1),
                    card,
                };
                let _ = game.apply_action(action);
                black_box(game);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("Sow action: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    // Test harvest action memory usage (more complex)
    group.bench_function("harvest_action_memory", |b| {
        b.iter_batched(
            || {
                reset_allocation_counters();
                let mut game = IllimatState::new(GameConfig::new(2));
                // Add a target card to harvest
                let target_card = Card::new(Rank::Five, Suit::Spring);
                game.field_cards[1].push(target_card);
                let harvest_card = game.player_hands[0][0];
                (game, harvest_card, target_card)
            },
            |(mut game, harvest_card, target_card)| {
                let action = Action::Harvest {
                    field: FieldId(1),
                    card: harvest_card,
                    targets: vec![target_card],
                };
                let _ = game.apply_action(action);
                black_box(game);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("Harvest action: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark memory usage patterns for vector operations
fn bench_memory_vectors(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_vectors");
    
    // Test different vector sizes and allocation patterns
    for size in [10, 50, 100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("card_vector_push", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || {
                        reset_allocation_counters();
                        size
                    },
                    |size| {
                        let mut cards = Vec::new();
                        for i in 0..size {
                            cards.push(Card::new(
                                match i % 4 {
                                    0 => Rank::Five,
                                    1 => Rank::Eight,
                                    2 => Rank::Ten,
                                    _ => Rank::King,
                                },
                                Suit::Spring
                            ));
                        }
                        black_box(cards);
                        let (allocated, _deallocated, net) = get_allocation_stats();
                        println!("Vector push ({}): {} bytes allocated, {} net", size, allocated, net);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
        
        group.bench_with_input(
            BenchmarkId::new("card_vector_with_capacity", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || {
                        reset_allocation_counters();
                        size
                    },
                    |size| {
                        let mut cards = Vec::with_capacity(size);
                        for i in 0..size {
                            cards.push(Card::new(
                                match i % 4 {
                                    0 => Rank::Five,
                                    1 => Rank::Eight,
                                    2 => Rank::Ten,
                                    _ => Rank::King,
                                },
                                Suit::Spring
                            ));
                        }
                        black_box(cards);
                        let (allocated, _deallocated, net) = get_allocation_stats();
                        println!("Vector with_capacity ({}): {} bytes allocated, {} net", size, allocated, net);
                    },
                    BatchSize::SmallInput,
                );
            },
        );
    }
    
    group.finish();
}

/// Benchmark memory usage for JSON serialization
fn bench_memory_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_serialization");
    
    let game = IllimatState::new(GameConfig::new(4).with_deck_size(true));
    
    group.bench_function("serialize_memory", |b| {
        b.iter_batched_ref(
            || {
                reset_allocation_counters();
                &game
            },
            |game| {
                let json = serde_json::to_string(game).unwrap();
                black_box(json);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("JSON serialize: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    let json_data = serde_json::to_string(&game).unwrap();
    group.bench_function("deserialize_memory", |b| {
        b.iter_batched_ref(
            || {
                reset_allocation_counters();
                &json_data
            },
            |json| {
                let game: IllimatState = serde_json::from_str(json).unwrap();
                black_box(game);
                let (allocated, _deallocated, net) = get_allocation_stats();
                println!("JSON deserialize: {} bytes allocated, {} net", allocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark memory fragmentation patterns
fn bench_memory_fragmentation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_fragmentation");
    
    // Test repeated allocation/deallocation patterns
    group.bench_function("repeated_game_creation", |b| {
        b.iter_batched(
            || {
                reset_allocation_counters();
                GameConfig::new(2)
            },
            |config| {
                // Create and drop multiple games to test fragmentation
                for _ in 0..10 {
                    let game = IllimatState::new(config.clone());
                    black_box(game);
                }
                let (allocated, deallocated, net) = get_allocation_stats();
                println!("Repeated creation: {} allocated, {} deallocated, {} net", 
                         allocated, deallocated, net);
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Generate a memory profiling report
fn memory_profile_criterion() -> Criterion {
    Criterion::default()
        .sample_size(20) // Reduce sample size for memory profiling
        .measurement_time(std::time::Duration::from_secs(10))
}

criterion_group! {
    name = memory_benches;
    config = memory_profile_criterion();
    targets = 
        bench_memory_game_creation,
        bench_memory_cloning,
        bench_memory_actions,
        bench_memory_vectors,
        bench_memory_serialization,
        bench_memory_fragmentation,
}

criterion_main!(memory_benches);