use criterion::{black_box, criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use illimat_core::game::{
    card::{Card, Rank, Suit},
    state::IllimatState,
    game_config::GameConfig,
    actions::Action,
    field_id::FieldId,
};

/// Create a standard test game configuration
fn create_test_config(player_count: u8) -> GameConfig {
    GameConfig::new(player_count).with_deck_size(false) // Use 4-suit deck for predictable performance
}

/// Create a game state with known setup for benchmarking
fn create_benchmark_game() -> IllimatState {
    let config = create_test_config(2);
    IllimatState::new(config)
}

/// Create a game state with cards in various field positions for complex benchmarks
fn create_complex_game_state() -> IllimatState {
    let mut game = create_benchmark_game();
    
    // Add some cards to fields to make benchmarks more realistic
    // This simulates a mid-game state with various card distributions
    let test_cards = vec![
        Card::new(Rank::Five, Suit::Spring),
        Card::new(Rank::Eight, Suit::Summer),
        Card::new(Rank::Three, Suit::Autumn),
        Card::new(Rank::Ten, Suit::Winter),
    ];
    
    // Distribute cards across fields
    for (i, card) in test_cards.iter().enumerate() {
        if i < game.field_cards.len() {
            game.field_cards[i].push(*card);
        }
    }
    
    game
}

/// Benchmark game state creation and initialization
fn bench_game_state_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_state_creation");
    
    // Benchmark different player counts
    for player_count in [2, 3, 4] {
        group.bench_with_input(
            BenchmarkId::new("new_game", player_count),
            &player_count,
            |b, &player_count| {
                b.iter(|| {
                    let config = create_test_config(player_count);
                    black_box(IllimatState::new(config))
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark game state cloning (important for MCTS tree search)
fn bench_game_state_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_state_cloning");
    
    // Test with simple state
    let simple_game = create_benchmark_game();
    group.bench_function("clone_simple_state", |b| {
        b.iter(|| black_box(simple_game.clone()));
    });
    
    // Test with complex state (more cards in fields)
    let complex_game = create_complex_game_state();
    group.bench_function("clone_complex_state", |b| {
        b.iter(|| black_box(complex_game.clone()));
    });
    
    group.finish();
}

/// Benchmark action application performance
fn bench_action_application(c: &mut Criterion) {
    let mut group = c.benchmark_group("action_application");
    
    // Benchmark sow actions (simplest)
    group.bench_function("apply_sow_action", |b| {
        b.iter_batched(
            || {
                let game = create_benchmark_game();
                let player_hand = &game.player_hands[0];
                let card = player_hand[0]; // Get first card from hand
                (game, card)
            },
            |(mut game, card)| {
                let action = Action::Sow {
                    field: FieldId(1), // Summer field (no restrictions)
                    card,
                };
                black_box(game.apply_action(action).unwrap_or(false))
            },
            BatchSize::SmallInput,
        );
    });
    
    // Benchmark harvest actions (more complex)
    group.bench_function("apply_harvest_action", |b| {
        b.iter_batched(
            || {
                let game = create_complex_game_state();
                // Set up a harvest scenario
                let player_hand = &game.player_hands[0];
                let harvest_card = player_hand[0];
                let field_card = game.field_cards[1][0]; // Get a card from Summer field
                (game, harvest_card, field_card)
            },
            |(mut game, harvest_card, target_card)| {
                let action = Action::Harvest {
                    field: FieldId(1),
                    card: harvest_card,
                    targets: vec![target_card],
                };
                // Action might fail due to game rules, that's ok for benchmark
                black_box(game.apply_action(action).unwrap_or(false))
            },
            BatchSize::SmallInput,
        );
    });
    
    // Benchmark stockpile actions (most complex)
    group.bench_function("apply_stockpile_action", |b| {
        b.iter_batched(
            || {
                let game = create_complex_game_state();
                let player_hand = &game.player_hands[0];
                let active_card = player_hand[0];
                let field_card = game.field_cards[1][0];
                (game, active_card, field_card)
            },
            |(mut game, active_card, passive_card)| {
                let action = Action::Stockpile {
                    field: FieldId(1),
                    card: active_card,
                    targets: vec![passive_card],
                };
                black_box(game.apply_action(action).unwrap_or(false))
            },
            BatchSize::SmallInput,
        );
    });
    
    group.finish();
}

/// Benchmark move generation and validation (critical for MCTS)
fn bench_move_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("move_generation");
    
    // Benchmark finding harvest combinations (computationally intensive)
    group.bench_function("find_harvest_combinations", |b| {
        let game = create_complex_game_state();
        let field_cards = &game.field_cards[1]; // Summer field
        let field_stockpiles = &game.field_stockpiles[1];
        let hand_card = game.player_hands[0][0];
        
        b.iter(|| {
            // Use the same algorithm as the console application
            black_box(find_harvest_combinations_benchmark(
                field_cards,
                field_stockpiles,
                hand_card,
            ))
        });
    });
    
    // Benchmark round completion checking
    group.bench_function("should_end_round", |b| {
        let game = create_complex_game_state();
        b.iter(|| black_box(game.should_end_round()));
    });
    
    group.finish();
}

/// Benchmark card operations and data structure performance
fn bench_card_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("card_operations");
    
    // Benchmark card creation and manipulation
    group.bench_function("card_creation", |b| {
        b.iter(|| {
            let cards: Vec<Card> = (0..52)
                .map(|i| {
                    let suit = match i % 4 {
                        0 => Suit::Spring,
                        1 => Suit::Summer,
                        2 => Suit::Autumn,
                        _ => Suit::Winter,
                    };
                    let rank = match i % 13 {
                        0 => Rank::Fool,
                        1 => Rank::Two,
                        2 => Rank::Three,
                        3 => Rank::Four,
                        4 => Rank::Five,
                        5 => Rank::Six,
                        6 => Rank::Seven,
                        7 => Rank::Eight,
                        8 => Rank::Nine,
                        9 => Rank::Ten,
                        10 => Rank::Knight,
                        11 => Rank::Queen,
                        _ => Rank::King,
                    };
                    Card::new(rank, suit)
                })
                .collect();
            black_box(cards)
        });
    });
    
    // Benchmark vector operations on card collections
    group.bench_function("card_vector_operations", |b| {
        let cards: Vec<Card> = (0..20)
            .map(|i| Card::new(Rank::Five, if i % 2 == 0 { Suit::Spring } else { Suit::Summer }))
            .collect();
        
        b.iter(|| {
            let mut result = Vec::new();
            for &card in &cards {
                if card.suit() == Suit::Spring {
                    result.push(card);
                }
            }
            black_box(result)
        });
    });
    
    group.finish();
}

/// Benchmark serialization performance (important for save/load)
fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");
    
    let game = create_complex_game_state();
    
    // Benchmark JSON serialization
    group.bench_function("serialize_to_json", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(&game).unwrap())
        });
    });
    
    // Benchmark JSON deserialization
    let json_data = serde_json::to_string(&game).unwrap();
    group.bench_function("deserialize_from_json", |b| {
        b.iter(|| {
            black_box(serde_json::from_str::<IllimatState>(&json_data).unwrap())
        });
    });
    
    group.finish();
}

/// Benchmark memory allocation patterns
fn bench_memory_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_allocation");
    
    // Test different collection sizes to understand allocation behavior
    for size in [10, 50, 100, 500] {
        group.bench_with_input(
            BenchmarkId::new("vec_allocation", size),
            &size,
            |b, &size| {
                b.iter(|| {
                    let mut cards = Vec::with_capacity(size);
                    for _i in 0..size {
                        cards.push(Card::new(Rank::Five, Suit::Spring));
                    }
                    black_box(cards)
                });
            },
        );
    }
    
    group.finish();
}

/// Helper function for benchmarking harvest combinations (simplified version)
fn find_harvest_combinations_benchmark(
    field_cards: &[Card],
    _field_stockpiles: &[illimat_core::game::stockpile::Stockpile],
    played_card: Card,
) -> Vec<Vec<Card>> {
    // Simplified version of harvest combination finding for benchmarking
    let mut combinations = Vec::new();
    let target_value = played_card.value();
    
    // Single card matches
    for &card in field_cards {
        if card.value() == target_value {
            combinations.push(vec![card]);
        }
    }
    
    // Simple two-card combinations
    for (i, &card1) in field_cards.iter().enumerate() {
        for &card2 in field_cards.iter().skip(i + 1) {
            if card1.value() + card2.value() == target_value {
                combinations.push(vec![card1, card2]);
            }
        }
    }
    
    combinations
}

/// Performance regression detection helper
fn bench_performance_regression(c: &mut Criterion) {
    let mut group = c.benchmark_group("regression_detection");
    
    // This benchmark serves as a canary for performance regressions
    // It runs a complete mini-game simulation
    group.bench_function("complete_mini_game", |b| {
        b.iter(|| {
            let mut game = create_benchmark_game();
            let mut actions_applied = 0;
            let max_actions = 10; // Limit to keep benchmark fast
            
            // Play some actions to simulate game progression
            while actions_applied < max_actions && !game.should_end_round() {
                let current_player = game.current_player;
                if let Some(&card) = game.player_hands[current_player.0 as usize].first() {
                    let action = Action::Sow {
                        field: FieldId(1), // Summer field
                        card,
                    };
                    
                    if game.apply_action(action).is_ok() {
                        actions_applied += 1;
                    }
                } else {
                    break;
                }
            }
            
            black_box(actions_applied)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_game_state_creation,
    bench_game_state_cloning,
    bench_action_application,
    bench_move_generation,
    bench_card_operations,
    bench_serialization,
    bench_memory_allocation,
    bench_performance_regression,
);

criterion_main!(benches);