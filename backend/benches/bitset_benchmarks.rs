use criterion::{black_box, criterion_group, criterion_main, Criterion};
use illimat_core::game::bitset::CardBitset;
use illimat_core::game::card::{Card, Rank, Suit};

// Generate test cards (avoiding Stars King which is card 64)
fn generate_test_cards() -> Vec<Card> {
    let mut cards = Vec::new();
    for suit in [Suit::Spring, Suit::Summer, Suit::Autumn, Suit::Winter] {
        for rank in [Rank::Fool, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
                    Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                    Rank::Knight, Rank::Queen, Rank::King] {
            cards.push(Card::new(rank, suit));
        }
    }
    // Add Stars cards except King
    for rank in [Rank::Fool, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
                Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                Rank::Knight, Rank::Queen] {
        cards.push(Card::new(rank, Suit::Stars));
    }
    cards
}

fn benchmark_card_operations(c: &mut Criterion) {
    let test_cards = generate_test_cards();
    let sample_cards = &test_cards[0..10]; // Use first 10 cards for benchmarking
    
    // Benchmark Vec<Card> operations
    c.bench_function("vec_contains_operation", |b| {
        b.iter(|| {
            let mut vec_set: Vec<Card> = Vec::new();
            for &card in sample_cards {
                vec_set.push(card);
            }
            
            // Test contains operation
            for &card in sample_cards {
                black_box(vec_set.contains(&card));
            }
        })
    });
    
    // Benchmark CardBitset operations  
    c.bench_function("bitset_has_card_operation", |b| {
        b.iter(|| {
            let mut bitset = CardBitset::empty();
            for &card in sample_cards {
                bitset.add_card(card);
            }
            
            // Test has_card operation
            for &card in sample_cards {
                black_box(bitset.has_card(card));
            }
        })
    });
    
    // Benchmark Vec<Card> add operations
    c.bench_function("vec_add_operation", |b| {
        b.iter(|| {
            let mut vec_set: Vec<Card> = Vec::new();
            for &card in sample_cards {
                if !vec_set.contains(&card) {
                    vec_set.push(card);
                }
            }
        })
    });
    
    // Benchmark CardBitset add operations
    c.bench_function("bitset_add_operation", |b| {
        b.iter(|| {
            let mut bitset = CardBitset::empty();
            for &card in sample_cards {
                black_box(bitset.add_card(card));
            }
        })
    });
    
    // Benchmark Vec<Card> remove operations
    c.bench_function("vec_remove_operation", |b| {
        b.iter(|| {
            let mut vec_set: Vec<Card> = sample_cards.to_vec();
            for &card in sample_cards {
                if let Some(pos) = vec_set.iter().position(|&x| x == card) {
                    vec_set.remove(pos);
                }
            }
        })
    });
    
    // Benchmark CardBitset remove operations
    c.bench_function("bitset_remove_operation", |b| {
        b.iter(|| {
            let mut bitset = CardBitset::from(sample_cards);
            for &card in sample_cards {
                black_box(bitset.remove_card(card));
            }
        })
    });
    
    // Benchmark Vec<Card> count operation
    c.bench_function("vec_count_operation", |b| {
        let vec_set: Vec<Card> = sample_cards.to_vec();
        b.iter(|| {
            black_box(vec_set.len());
        })
    });
    
    // Benchmark CardBitset count operation
    c.bench_function("bitset_count_operation", |b| {
        let bitset = CardBitset::from(sample_cards);
        b.iter(|| {
            black_box(bitset.count());
        })
    });
}

fn benchmark_memory_operations(c: &mut Criterion) {
    let test_cards = generate_test_cards();
    let sample_cards = &test_cards[0..20]; // Use first 20 cards
    
    // Benchmark Vec<Card> creation from slice
    c.bench_function("vec_from_slice", |b| {
        b.iter(|| {
            black_box(Vec::from(sample_cards));
        })
    });
    
    // Benchmark CardBitset creation from slice
    c.bench_function("bitset_from_slice", |b| {
        b.iter(|| {
            black_box(CardBitset::from(sample_cards));
        })
    });
    
    // Benchmark Vec<Card> copying
    c.bench_function("vec_clone", |b| {
        let vec_set: Vec<Card> = sample_cards.to_vec();
        b.iter(|| {
            black_box(vec_set.clone());
        })
    });
    
    // Benchmark CardBitset copying
    c.bench_function("bitset_copy", |b| {
        let bitset = CardBitset::from(sample_cards);
        b.iter(|| {
            black_box(bitset); // Copy operation
        })
    });
}

fn benchmark_set_operations(c: &mut Criterion) {
    let test_cards = generate_test_cards();
    let cards1 = &test_cards[0..10];
    let cards2 = &test_cards[5..15]; // 50% overlap
    
    // Vec<Card> intersection (simulated)
    c.bench_function("vec_intersection", |b| {
        let vec1: Vec<Card> = cards1.to_vec();
        let vec2: Vec<Card> = cards2.to_vec();
        b.iter(|| {
            let mut intersection = Vec::new();
            for &card in &vec1 {
                if vec2.contains(&card) {
                    intersection.push(card);
                }
            }
            black_box(intersection);
        })
    });
    
    // CardBitset intersection
    c.bench_function("bitset_intersection", |b| {
        let bitset1 = CardBitset::from(cards1);
        let bitset2 = CardBitset::from(cards2);
        b.iter(|| {
            black_box(bitset1.intersection(bitset2));
        })
    });
    
    // Vec<Card> union (simulated)
    c.bench_function("vec_union", |b| {
        let vec1: Vec<Card> = cards1.to_vec();
        let vec2: Vec<Card> = cards2.to_vec();
        b.iter(|| {
            let mut union = vec1.clone();
            for &card in &vec2 {
                if !union.contains(&card) {
                    union.push(card);
                }
            }
            black_box(union);
        })
    });
    
    // CardBitset union
    c.bench_function("bitset_union", |b| {
        let bitset1 = CardBitset::from(cards1);
        let bitset2 = CardBitset::from(cards2);
        b.iter(|| {
            black_box(bitset1.union(bitset2));
        })
    });
}

criterion_group!(benches, benchmark_card_operations, benchmark_memory_operations, benchmark_set_operations);
criterion_main!(benches);