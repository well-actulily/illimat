//! Zero-Copy Optimization Patterns for WebAssembly Deployment
//! 
//! This module provides memory-efficient patterns designed specifically for
//! WebAssembly deployment where memory allocation is expensive and heap 
//! space is limited. These patterns minimize allocations and copies during
//! high-frequency game operations.

use crate::game::card::Card;
use crate::game::field_id::FieldId;
use arrayvec::ArrayVec;
use std::marker::PhantomData;

/// Maximum cards in a single field (reasonable upper bound)
const MAX_FIELD_CARDS: usize = 32;

/// Maximum cards in a player's hand
const MAX_HAND_CARDS: usize = 13;

/// Maximum harvest targets in a single move
const MAX_HARVEST_TARGETS: usize = 8;

/// Stack-allocated card collection for zero-allocation operations
pub type StackCardVec<const N: usize> = ArrayVec<Card, N>;

/// Field-specific stack collection
pub type FieldCards = StackCardVec<MAX_FIELD_CARDS>;

/// Hand-specific stack collection  
pub type HandCards = StackCardVec<MAX_HAND_CARDS>;

/// Harvest target collection
pub type HarvestTargets = StackCardVec<MAX_HARVEST_TARGETS>;

/// Zero-copy view into a card collection
/// 
/// This provides read-only access to cards without allocation,
/// essential for MCTS tree search where we need to examine
/// game states without modifying them.
#[derive(Debug)]
pub struct CardView<'a> {
    cards: &'a [Card],
    field_id: Option<FieldId>,
}

impl<'a> CardView<'a> {
    /// Create a new card view
    pub fn new(cards: &'a [Card]) -> Self {
        Self {
            cards,
            field_id: None,
        }
    }
    
    /// Create a field-specific card view
    pub fn from_field(cards: &'a [Card], field_id: FieldId) -> Self {
        Self {
            cards,
            field_id: Some(field_id),
        }
    }
    
    /// Get the number of cards
    pub fn len(&self) -> usize {
        self.cards.len()
    }
    
    /// Check if the view is empty
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }
    
    /// Iterate over cards
    pub fn iter(&self) -> std::slice::Iter<'_, Card> {
        self.cards.iter()
    }
    
    /// Get card by index
    pub fn get(&self, index: usize) -> Option<&Card> {
        self.cards.get(index)
    }
    
    /// Find cards matching a predicate without allocation
    pub fn find_matching<F>(&self, predicate: F, results: &mut HarvestTargets) -> usize 
    where
        F: Fn(&Card) -> bool,
    {
        results.clear();
        let mut count = 0;
        
        for &card in self.cards {
            if predicate(&card) && results.try_push(card).is_ok() {
                count += 1;
            }
        }
        
        count
    }
    
    /// Calculate sum of card values without allocation
    pub fn sum(&self) -> u32 {
        self.cards.iter().map(|card| card.value() as u32).sum()
    }
    
    /// Check if collection contains a specific card
    pub fn contains(&self, target: &Card) -> bool {
        self.cards.contains(target)
    }
    
    /// Get field ID if this is a field view
    pub fn field_id(&self) -> Option<FieldId> {
        self.field_id
    }
}

/// Zero-copy mutable view for in-place modifications
/// 
/// Allows modification of card collections without heap allocation,
/// crucial for action application in WebAssembly environments.
#[derive(Debug)]
pub struct CardViewMut<'a> {
    cards: &'a mut [Card],
    field_id: Option<FieldId>,
}

impl<'a> CardViewMut<'a> {
    /// Create a new mutable card view
    pub fn new(cards: &'a mut [Card]) -> Self {
        Self {
            cards,
            field_id: None,
        }
    }
    
    /// Create a field-specific mutable card view
    pub fn from_field(cards: &'a mut [Card], field_id: FieldId) -> Self {
        Self {
            cards,
            field_id: Some(field_id),
        }
    }
    
    /// Get immutable view
    pub fn as_view(&self) -> CardView<'_> {
        CardView {
            cards: self.cards,
            field_id: self.field_id,
        }
    }
    
    /// Remove a card by value (first occurrence)
    pub fn remove_card(&mut self, target: Card) -> bool {
        if let Some(pos) = self.cards.iter().position(|&card| card == target) {
            // Shift remaining cards to fill the gap
            self.cards.copy_within(pos + 1.., pos);
            true
        } else {
            false
        }
    }
    
    /// Remove multiple cards efficiently
    pub fn remove_cards(&mut self, targets: &[Card]) -> usize {
        let mut removed = 0;
        
        for &target in targets {
            if self.remove_card(target) {
                removed += 1;
            }
        }
        
        removed
    }
}

/// Zero-allocation move builder for WebAssembly efficiency
/// 
/// Constructs game moves without heap allocation, essential for
/// MCTS where thousands of moves may be generated per second.
pub struct ZeroCopyMoveBuilder<'a> {
    field_views: [CardView<'a>; 4],
    temp_targets: HarvestTargets,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> ZeroCopyMoveBuilder<'a> {
    /// Create a new move builder from field references
    pub fn new(field_cards: [&'a [Card]; 4]) -> Self {
        Self {
            field_views: [
                CardView::from_field(field_cards[0], FieldId(0)),
                CardView::from_field(field_cards[1], FieldId(1)),
                CardView::from_field(field_cards[2], FieldId(2)),
                CardView::from_field(field_cards[3], FieldId(3)),
            ],
            temp_targets: HarvestTargets::new(),
            _phantom: PhantomData,
        }
    }
    
    /// Find harvest targets for a card without allocation
    pub fn find_harvest_targets(&mut self, field_id: FieldId, played_card: Card) -> &[Card] {
        let field_view = &self.field_views[field_id.0 as usize];
        let target_value = played_card.value();
        
        // Clear previous results
        self.temp_targets.clear();
        
        // Find exact matches
        field_view.find_matching(
            |card| card.can_be_value(target_value),
            &mut self.temp_targets
        );
        
        // TODO: Add combination finding for multi-card targets
        // This would require more sophisticated zero-allocation algorithms
        
        &self.temp_targets
    }
    
    /// Check if a sow move is valid
    pub fn can_sow(&self, field_id: FieldId, _card: Card) -> bool {
        // For now, simple check - could be expanded with season restrictions
        field_id.0 < 4
    }
    
    /// Get field view by ID
    pub fn get_field_view(&self, field_id: FieldId) -> &CardView<'a> {
        &self.field_views[field_id.0 as usize]
    }
}

/// WebAssembly-optimized memory pool for card operations
/// 
/// Pre-allocates common data structures to avoid allocation
/// overhead during gameplay, especially important in WASM
/// where garbage collection can cause frame drops.
pub struct WasmMemoryPool {
    card_buffers: [FieldCards; 4],
    harvest_buffer: HarvestTargets,
    temp_buffer: HarvestTargets,
}

impl WasmMemoryPool {
    /// Create a new memory pool
    pub fn new() -> Self {
        Self {
            card_buffers: [
                FieldCards::new(),
                FieldCards::new(), 
                FieldCards::new(),
                FieldCards::new(),
            ],
            harvest_buffer: HarvestTargets::new(),
            temp_buffer: HarvestTargets::new(),
        }
    }
    
    /// Get a cleared card buffer for a field
    pub fn get_field_buffer(&mut self, field_id: FieldId) -> &mut FieldCards {
        let buffer = &mut self.card_buffers[field_id.0 as usize];
        buffer.clear();
        buffer
    }
    
    /// Get the harvest buffer
    pub fn get_harvest_buffer(&mut self) -> &mut HarvestTargets {
        self.harvest_buffer.clear();
        &mut self.harvest_buffer
    }
    
    /// Get temporary buffer for calculations
    pub fn get_temp_buffer(&mut self) -> &mut HarvestTargets {
        self.temp_buffer.clear();
        &mut self.temp_buffer
    }
    
    /// Copy cards from slice to field buffer
    pub fn copy_to_field_buffer(&mut self, field_id: FieldId, cards: &[Card]) -> Result<(), ()> {
        let buffer = self.get_field_buffer(field_id);
        
        for &card in cards {
            buffer.try_push(card).map_err(|_| ())?;
        }
        
        Ok(())
    }
}

impl Default for WasmMemoryPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance monitoring for zero-copy operations
pub mod performance {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark zero-copy vs allocation-based operations
    pub fn benchmark_zero_copy_vs_allocation(iterations: usize) -> (f64, f64) {
        let test_cards: Vec<Card> = (0..20)
            .map(|i| Card::new(
                crate::game::card::Rank::Five,
                match i % 4 {
                    0 => crate::game::card::Suit::Spring,
                    1 => crate::game::card::Suit::Summer,
                    2 => crate::game::card::Suit::Autumn,
                    _ => crate::game::card::Suit::Winter,
                }
            ))
            .collect();
        
        // Benchmark allocation-based approach
        let start = Instant::now();
        for _ in 0..iterations {
            let _filtered: Vec<Card> = test_cards
                .iter()
                .copied()
                .filter(|card| card.value() > 3)
                .collect();
        }
        let allocation_time = start.elapsed().as_nanos() as f64 / iterations as f64;
        
        // Benchmark zero-copy approach
        let mut results = HarvestTargets::new();
        let start = Instant::now();
        for _ in 0..iterations {
            let view = CardView::new(&test_cards);
            view.find_matching(|card| card.value() > 3, &mut results);
        }
        let zero_copy_time = start.elapsed().as_nanos() as f64 / iterations as f64;
        
        (allocation_time, zero_copy_time)
    }
    
    /// Generate WebAssembly optimization report
    pub fn generate_wasm_optimization_report() -> String {
        let mut report = String::new();
        report.push_str("=== WebAssembly Zero-Copy Optimization Report ===\n\n");
        
        let iterations = 10000;
        let (alloc_time, zero_copy_time) = benchmark_zero_copy_vs_allocation(iterations);
        let speedup = alloc_time / zero_copy_time;
        
        report.push_str(&format!("Performance Comparison ({} iterations):\n", iterations));
        report.push_str(&format!("Allocation-based: {:.1} ns/iteration\n", alloc_time));
        report.push_str(&format!("Zero-copy:        {:.1} ns/iteration\n", zero_copy_time));
        report.push_str(&format!("Speedup:          {:.2}x\n\n", speedup));
        
        report.push_str("Memory Usage Analysis:\n");
        report.push_str(&format!("Max field cards:    {} (stack allocated)\n", MAX_FIELD_CARDS));
        report.push_str(&format!("Max hand cards:     {} (stack allocated)\n", MAX_HAND_CARDS));
        report.push_str(&format!("Max harvest targets: {} (stack allocated)\n", MAX_HARVEST_TARGETS));
        report.push_str(&format!("Total stack usage:  ~{} bytes per pool\n", 
            (MAX_FIELD_CARDS * 4 + MAX_HAND_CARDS + MAX_HARVEST_TARGETS * 2) * std::mem::size_of::<Card>()));
        
        report.push_str("\nWebAssembly Recommendations:\n");
        report.push_str("- Use WasmMemoryPool for pre-allocated buffers\n");
        report.push_str("- Prefer CardView for read-only operations\n");
        report.push_str("- Use ZeroCopyMoveBuilder for move generation\n");
        report.push_str("- Avoid Vec allocations in hot paths\n");
        report.push_str("- Stack allocation is much faster than heap in WASM\n");
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::{Rank, Suit};
    
    #[test]
    fn test_card_view_operations() {
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
            Card::new(Rank::Three, Suit::Autumn),
        ];
        
        let view = CardView::new(&cards);
        
        assert_eq!(view.len(), 3);
        assert!(!view.is_empty());
        assert_eq!(view.sum(), 18); // 5 + 10 + 3
        assert!(view.contains(&cards[0]));
    }
    
    #[test]
    fn test_card_view_find_matching() {
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer), 
            Card::new(Rank::Five, Suit::Autumn),
        ];
        
        let view = CardView::new(&cards);
        let mut results = HarvestTargets::new();
        
        let count = view.find_matching(|card| card.value() == 5, &mut results);
        
        assert_eq!(count, 2);
        assert_eq!(results.len(), 2);
    }
    
    #[test]
    fn test_zero_copy_move_builder() {
        let field_cards = [
            vec![Card::new(Rank::Five, Suit::Spring)],
            vec![Card::new(Rank::Ten, Suit::Summer)],
            vec![Card::new(Rank::Three, Suit::Autumn)],
            vec![Card::new(Rank::King, Suit::Winter)],
        ];
        
        let field_refs = [
            field_cards[0].as_slice(),
            field_cards[1].as_slice(), 
            field_cards[2].as_slice(),
            field_cards[3].as_slice(),
        ];
        
        let mut builder = ZeroCopyMoveBuilder::new(field_refs);
        
        let played_card = Card::new(Rank::Five, Suit::Stars);
        let targets = builder.find_harvest_targets(FieldId(0), played_card);
        
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0], Card::new(Rank::Five, Suit::Spring));
    }
    
    #[test]
    fn test_wasm_memory_pool() {
        let mut pool = WasmMemoryPool::new();
        
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
        ];
        
        let result = pool.copy_to_field_buffer(FieldId(0), &cards);
        assert!(result.is_ok());
        
        let buffer = pool.get_field_buffer(FieldId(0));
        assert_eq!(buffer.len(), 0); // Should be cleared by get_field_buffer
    }
}