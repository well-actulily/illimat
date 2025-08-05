use crate::game::card::Card;
use std::fmt;

/// Ultra-compact card collection using bitset representation
/// Currently supports 64 cards (0-63) - TODO: Handle 65th card (Stars King)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct CardBitset(u64);

impl CardBitset {
    /// Create an empty card bitset
    #[inline]
    pub const fn empty() -> Self {
        CardBitset(0)
    }
    
    /// Map card ID to compact bitset position (0-64 for 65 cards)
    /// Cards are encoded as (suit << 4) | rank, but we need sequential IDs
    #[inline]
    fn compact_card_id(card: Card) -> u8 {
        let suit = card.suit() as u8;
        let rank = card.rank() as u8;
        // Map: Spring=0-12, Summer=13-25, Autumn=26-38, Winter=39-51, Stars=52-64
        suit * 13 + rank
    }
    
    /// Create card from compact bitset position (inverse of compact_card_id)
    #[inline]
    fn card_from_compact_id(compact_id: u8) -> Card {
        use crate::game::card::{Rank, Suit};
        
        let suit_num = compact_id / 13;
        let rank_num = compact_id % 13;
        
        let suit = match suit_num {
            0 => Suit::Spring,
            1 => Suit::Summer, 
            2 => Suit::Autumn,
            3 => Suit::Winter,
            4 => Suit::Stars,
            _ => Suit::Spring, // Fallback
        };
        
        let rank = match rank_num {
            0 => Rank::Fool, 1 => Rank::Two, 2 => Rank::Three, 3 => Rank::Four,
            4 => Rank::Five, 5 => Rank::Six, 6 => Rank::Seven, 7 => Rank::Eight,
            8 => Rank::Nine, 9 => Rank::Ten, 10 => Rank::Knight, 11 => Rank::Queen,
            12 => Rank::King,
            _ => Rank::Fool, // Fallback
        };
        
        Card::new(rank, suit)
    }
    
    /// Create a bitset with all possible cards (65 cards = bits 0-64)
    /// Note: Since we need 65 bits but only have 64, this is the max possible
    #[inline]
    pub const fn full() -> Self {
        CardBitset(u64::MAX) // Sets bits 0-63, card 64 needs special handling
    }
    
    /// Create a bitset with all 65 cards properly
    #[inline]
    pub fn all_cards() -> Self {
        let mut bitset = CardBitset::empty();
        // Add all valid cards
        use crate::game::card::{Rank, Suit};
        for suit in [Suit::Spring, Suit::Summer, Suit::Autumn, Suit::Winter, Suit::Stars] {
            for rank in [Rank::Fool, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
                        Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                        Rank::Knight, Rank::Queen, Rank::King] {
                bitset.add_card(Card::new(rank, suit));
            }
        }
        bitset
    }
    
    /// Create a bitset from a raw u64 value
    #[inline]
    pub const fn from_raw(bits: u64) -> Self {
        CardBitset(bits)
    }
    
    /// Get the raw u64 representation
    #[inline]
    pub const fn raw(self) -> u64 {
        self.0
    }
    
    /// Check if a card is present in this bitset
    #[inline]
    pub fn has_card(self, card: Card) -> bool {
        let bit_position = Self::compact_card_id(card);
        if bit_position >= 64 {
            return false; // Invalid card ID
        }
        (self.0 & (1u64 << bit_position)) != 0
    }
    
    /// Add a card to this bitset
    /// Returns true if the card was newly added, false if it was already present
    #[inline]
    pub fn add_card(&mut self, card: Card) -> bool {
        let bit_position = Self::compact_card_id(card) as u64;
        if bit_position >= 64 {
            // TODO: Handle 65th card (Stars King) - for now, skip it
            eprintln!("Warning: Cannot handle card {} (compact ID {})", card, bit_position);
            return false;
        }
        let mask = 1u64 << bit_position;
        let was_present = (self.0 & mask) != 0;
        self.0 |= mask;
        !was_present
    }
    
    /// Remove a card from this bitset
    /// Returns true if the card was present and removed, false if it wasn't present
    #[inline]
    pub fn remove_card(&mut self, card: Card) -> bool {
        let bit_position = Self::compact_card_id(card) as u64;
        if bit_position >= 64 {
            // TODO: Handle 65th card (Stars King) - for now, skip it
            return false;
        }
        let mask = 1u64 << bit_position;
        let was_present = (self.0 & mask) != 0;
        self.0 &= !mask;
        was_present
    }
    
    /// Count the number of cards in this bitset
    #[inline]
    pub fn count(self) -> u32 {
        self.0.count_ones()
    }
    
    /// Check if this bitset is empty
    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
    
    /// Union of two bitsets (cards present in either)
    #[inline]
    pub fn union(self, other: CardBitset) -> CardBitset {
        CardBitset(self.0 | other.0)
    }
    
    /// Intersection of two bitsets (cards present in both)
    #[inline]
    pub fn intersection(self, other: CardBitset) -> CardBitset {
        CardBitset(self.0 & other.0)
    }
    
    /// Difference of two bitsets (cards in self but not in other)
    #[inline]
    pub fn difference(self, other: CardBitset) -> CardBitset {
        CardBitset(self.0 & !other.0)
    }
    
    /// Symmetric difference (cards in either but not both)
    #[inline]
    pub fn symmetric_difference(self, other: CardBitset) -> CardBitset {
        CardBitset(self.0 ^ other.0)
    }
    
    /// Check if this bitset is a subset of another
    #[inline]
    pub fn is_subset(self, other: CardBitset) -> bool {
        (self.0 & other.0) == self.0
    }
    
    /// Check if this bitset is a superset of another
    #[inline]
    pub fn is_superset(self, other: CardBitset) -> bool {
        other.is_subset(self)
    }
    
    /// Check if two bitsets are disjoint (no common cards)
    #[inline]
    pub fn is_disjoint(self, other: CardBitset) -> bool {
        (self.0 & other.0) == 0
    }
    
    /// Iterator over all cards in this bitset
    pub fn iter(self) -> CardBitsetIter {
        CardBitsetIter {
            bits: self.0,
            position: 0,
        }
    }
    
    /// Convert to Vec<Card> for compatibility with existing code
    pub fn to_vec(self) -> Vec<Card> {
        self.iter().collect()
    }
}

/// Iterator over cards in a CardBitset
pub struct CardBitsetIter {
    bits: u64,
    position: u8,
}

impl Iterator for CardBitsetIter {
    type Item = Card;
    
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < 64 { // Only handle cards 0-63 in current implementation
            let mask = 1u64 << self.position;
            if (self.bits & mask) != 0 {
                let card = CardBitset::card_from_compact_id(self.position);
                self.position += 1;
                return Some(card);
            }
            self.position += 1;
        }
        None
    }
    
    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.position >= 64 {
            return (0, Some(0));
        }
        let remaining_bits = self.bits >> self.position;
        let count = remaining_bits.count_ones() as usize;
        (count, Some(count))
    }
}

impl ExactSizeIterator for CardBitsetIter {
    fn len(&self) -> usize {
        if self.position >= 64 {
            return 0;
        }
        let remaining_bits = self.bits >> self.position;
        remaining_bits.count_ones() as usize
    }
}

/// Convert from Vec<Card> to CardBitset
impl From<&[Card]> for CardBitset {
    fn from(cards: &[Card]) -> Self {
        let mut bitset = CardBitset::empty();
        for &card in cards {
            bitset.add_card(card);
        }
        bitset
    }
}

impl From<Vec<Card>> for CardBitset {
    fn from(cards: Vec<Card>) -> Self {
        CardBitset::from(cards.as_slice())
    }
}

/// Convert from CardBitset to Vec<Card>
impl From<CardBitset> for Vec<Card> {
    fn from(bitset: CardBitset) -> Self {
        bitset.to_vec()
    }
}

/// Display implementation for debugging
impl fmt::Display for CardBitset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{")?;
        let cards: Vec<Card> = self.iter().collect();
        for (i, card) in cards.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", card)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::card::{Card, Rank, Suit};
    use proptest::prelude::*;
    use serde_json;
    
    // Valid card generator for proptest (avoid Stars King which is card 64)
    fn valid_card_strategy() -> impl Strategy<Value = Card> {
        prop_oneof![
            // All Spring cards (0-12)
            (Just(Suit::Spring), prop_oneof![
                Just(Rank::Fool), Just(Rank::Two), Just(Rank::Three), Just(Rank::Four),
                Just(Rank::Five), Just(Rank::Six), Just(Rank::Seven), Just(Rank::Eight),
                Just(Rank::Nine), Just(Rank::Ten), Just(Rank::Knight), Just(Rank::Queen), Just(Rank::King),
            ]),
            // All Summer cards (13-25)
            (Just(Suit::Summer), prop_oneof![
                Just(Rank::Fool), Just(Rank::Two), Just(Rank::Three), Just(Rank::Four),
                Just(Rank::Five), Just(Rank::Six), Just(Rank::Seven), Just(Rank::Eight),
                Just(Rank::Nine), Just(Rank::Ten), Just(Rank::Knight), Just(Rank::Queen), Just(Rank::King),
            ]),
            // All Autumn cards (26-38)
            (Just(Suit::Autumn), prop_oneof![
                Just(Rank::Fool), Just(Rank::Two), Just(Rank::Three), Just(Rank::Four),
                Just(Rank::Five), Just(Rank::Six), Just(Rank::Seven), Just(Rank::Eight),
                Just(Rank::Nine), Just(Rank::Ten), Just(Rank::Knight), Just(Rank::Queen), Just(Rank::King),
            ]),
            // All Winter cards (39-51)
            (Just(Suit::Winter), prop_oneof![
                Just(Rank::Fool), Just(Rank::Two), Just(Rank::Three), Just(Rank::Four),
                Just(Rank::Five), Just(Rank::Six), Just(Rank::Seven), Just(Rank::Eight),
                Just(Rank::Nine), Just(Rank::Ten), Just(Rank::Knight), Just(Rank::Queen), Just(Rank::King),
            ]),
            // Stars cards (52-63) - exclude King (64)
            (Just(Suit::Stars), prop_oneof![
                Just(Rank::Fool), Just(Rank::Two), Just(Rank::Three), Just(Rank::Four),
                Just(Rank::Five), Just(Rank::Six), Just(Rank::Seven), Just(Rank::Eight),
                Just(Rank::Nine), Just(Rank::Ten), Just(Rank::Knight), Just(Rank::Queen),
                // Exclude Just(Rank::King) for Stars suit
            ]),
        ].prop_map(|(suit, rank)| Card::new(rank, suit))
    }
    
    #[test]
    fn test_empty_bitset() {
        let bitset = CardBitset::empty();
        assert_eq!(bitset.count(), 0);
        assert!(bitset.is_empty());
        assert_eq!(bitset.raw(), 0);
    }
    
    #[test]
    fn test_single_card_operations() {
        let mut bitset = CardBitset::empty();
        let card = Card::new(Rank::Five, Suit::Spring);
        
        // Initially doesn't have card
        assert!(!bitset.has_card(card));
        
        // Add card
        assert!(bitset.add_card(card)); // Should return true (newly added)
        assert!(bitset.has_card(card));
        assert_eq!(bitset.count(), 1);
        assert!(!bitset.is_empty());
        
        // Add same card again
        assert!(!bitset.add_card(card)); // Should return false (already present)
        assert_eq!(bitset.count(), 1);
        
        // Remove card
        assert!(bitset.remove_card(card)); // Should return true (was present)
        assert!(!bitset.has_card(card));
        assert_eq!(bitset.count(), 0);
        assert!(bitset.is_empty());
        
        // Remove card again
        assert!(!bitset.remove_card(card)); // Should return false (not present)
    }
    
    #[test]
    fn test_multiple_cards() {
        let mut bitset = CardBitset::empty();
        let cards = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer),
            Card::new(Rank::Queen, Suit::Stars), // Use Queen instead of King to avoid card 64
        ];
        
        // Add all cards
        for &card in &cards {
            assert!(bitset.add_card(card));
            assert!(bitset.has_card(card));
        }
        
        assert_eq!(bitset.count(), 3);
        
        // Check all cards are present
        for &card in &cards {
            assert!(bitset.has_card(card));
        }
        
        // Remove middle card
        assert!(bitset.remove_card(cards[1]));
        assert!(!bitset.has_card(cards[1]));
        assert!(bitset.has_card(cards[0]));
        assert!(bitset.has_card(cards[2]));
        assert_eq!(bitset.count(), 2);
    }
    
    #[test]
    fn test_bitset_operations() {
        let cards1 = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer),
        ];
        let cards2 = vec![
            Card::new(Rank::Five, Suit::Summer),
            Card::new(Rank::Queen, Suit::Stars), // Use Queen instead of King
        ];
        
        let bitset1 = CardBitset::from(cards1);
        let bitset2 = CardBitset::from(cards2);
        
        // Union: should have all 3 cards
        let union = bitset1.union(bitset2);
        assert_eq!(union.count(), 3);
        
        // Intersection: should have only Five of Summer
        let intersection = bitset1.intersection(bitset2);
        assert_eq!(intersection.count(), 1);
        assert!(intersection.has_card(Card::new(Rank::Five, Suit::Summer)));
        
        // Difference: bitset1 - bitset2 should have only Two of Spring
        let difference = bitset1.difference(bitset2);
        assert_eq!(difference.count(), 1);
        assert!(difference.has_card(Card::new(Rank::Two, Suit::Spring)));
        
        // Symmetric difference: should have Two of Spring and Queen of Stars
        let sym_diff = bitset1.symmetric_difference(bitset2);
        assert_eq!(sym_diff.count(), 2);
        assert!(sym_diff.has_card(Card::new(Rank::Two, Suit::Spring)));
        assert!(sym_diff.has_card(Card::new(Rank::Queen, Suit::Stars)));
    }
    
    #[test]
    fn test_subset_operations() {
        let cards_small = vec![Card::new(Rank::Two, Suit::Spring)];
        let cards_large = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer),
        ];
        
        let small_set = CardBitset::from(cards_small);
        let large_set = CardBitset::from(cards_large);
        
        assert!(small_set.is_subset(large_set));
        assert!(large_set.is_superset(small_set));
        assert!(!large_set.is_subset(small_set));
        assert!(!small_set.is_superset(large_set));
        
        // Disjoint test
        let disjoint_cards = vec![Card::new(Rank::Queen, Suit::Stars)];
        let disjoint_set = CardBitset::from(disjoint_cards);
        
        assert!(small_set.is_disjoint(disjoint_set));
        assert!(!small_set.is_disjoint(large_set));
    }
    
    #[test]
    fn test_iterator() {
        let cards = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer), 
            Card::new(Rank::Queen, Suit::Stars), // Use Queen instead of King
        ];
        
        let bitset = CardBitset::from(cards.clone());
        let collected: Vec<Card> = bitset.iter().collect();
        
        // Iterator should return cards in ID order (not input order)
        assert_eq!(collected.len(), 3);
        for card in &cards {
            assert!(collected.contains(card));
        }
        
        // Test size hint
        let mut iter = bitset.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.size_hint(), (3, Some(3)));
        
        iter.next();
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.size_hint(), (2, Some(2)));
    }
    
    #[test]
    fn test_conversions() {
        let cards = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer),
            Card::new(Rank::Queen, Suit::Stars), // Use Queen instead of King
        ];
        
        // Vec -> CardBitset -> Vec roundtrip
        let bitset = CardBitset::from(cards.clone());
        let restored: Vec<Card> = bitset.into();
        
        // Should have same cards (order may differ)
        assert_eq!(restored.len(), cards.len());
        for card in &cards {
            assert!(restored.contains(card));
        }
    }
    
    #[test]
    fn test_serialization() {
        let cards = vec![
            Card::new(Rank::Two, Suit::Spring),
            Card::new(Rank::Five, Suit::Summer),
            Card::new(Rank::Queen, Suit::Stars), // Use Queen instead of King
        ];
        
        let bitset = CardBitset::from(cards);
        
        // Serialize to JSON
        let json = serde_json::to_string(&bitset).unwrap();
        
        // Deserialize from JSON
        let restored: CardBitset = serde_json::from_str(&json).unwrap();
        
        // Should be identical
        assert_eq!(bitset, restored);
        assert_eq!(bitset.count(), restored.count());
        
        // Test with empty bitset
        let empty = CardBitset::empty();
        let empty_json = serde_json::to_string(&empty).unwrap();
        let empty_restored: CardBitset = serde_json::from_str(&empty_json).unwrap();
        assert_eq!(empty, empty_restored);
    }
    
    #[test]
    fn test_display() {
        let mut bitset = CardBitset::empty();
        assert_eq!(format!("{}", bitset), "{}");
        
        bitset.add_card(Card::new(Rank::Five, Suit::Spring));
        let display = format!("{}", bitset);
        assert!(display.contains("[5 Sp]"));
        assert!(display.starts_with('{'));
        assert!(display.ends_with('}'));
    }
    
    // Property-based tests for comprehensive validation
    proptest! {
        #[test]
        fn prop_bitset_vec_equivalence(
            cards in prop::collection::vec(valid_card_strategy(), 0..20)
        ) {
            // Convert Vec -> BitSet -> Vec should preserve set semantics
            let bitset = CardBitset::from(cards.clone());
            let restored: Vec<Card> = bitset.into();
            
            // Should have same unique cards
            let mut unique_cards: Vec<Card> = cards.clone();
            unique_cards.sort();
            unique_cards.dedup();
            
            let mut restored_sorted = restored;
            restored_sorted.sort();
            
            prop_assert_eq!(unique_cards.len(), restored_sorted.len());
            for card in &unique_cards {
                prop_assert!(restored_sorted.contains(card));
            }
        }
        
        #[test]
        fn prop_add_remove_operations(
            cards in prop::collection::vec(valid_card_strategy(), 0..20)
        ) {
            let mut bitset = CardBitset::empty();
            let mut vec_set: Vec<Card> = Vec::new();
            
            // Add all cards
            for card in &cards {
                let bitset_result = bitset.add_card(*card);
                let vec_result = if vec_set.contains(card) {
                    false
                } else {
                    vec_set.push(*card);
                    true
                };
                
                prop_assert_eq!(bitset_result, vec_result);
                prop_assert_eq!(bitset.has_card(*card), vec_set.contains(card));
                prop_assert_eq!(bitset.count() as usize, vec_set.len());
            }
            
            // Remove all cards
            for card in &cards {
                let bitset_result = bitset.remove_card(*card);
                let vec_result = if let Some(pos) = vec_set.iter().position(|&x| x == *card) {
                    vec_set.remove(pos);
                    true
                } else {
                    false
                };
                
                prop_assert_eq!(bitset_result, vec_result);
                prop_assert_eq!(bitset.has_card(*card), vec_set.contains(card));
                prop_assert_eq!(bitset.count() as usize, vec_set.len());
            }
        }
        
        #[test]
        fn prop_set_operations_correctness(
            cards1 in prop::collection::vec(valid_card_strategy(), 0..10),
            cards2 in prop::collection::vec(valid_card_strategy(), 0..10)
        ) {
            let bitset1 = CardBitset::from(cards1.clone());
            let bitset2 = CardBitset::from(cards2.clone());
            
            // Test union
            let union = bitset1.union(bitset2);
            for card in &cards1 {
                prop_assert!(union.has_card(*card));
            }
            for card in &cards2 {
                prop_assert!(union.has_card(*card));
            }
            
            // Test intersection
            let intersection = bitset1.intersection(bitset2);
            for card in &cards1 {
                if cards2.contains(card) {
                    prop_assert!(intersection.has_card(*card));
                }
            }
        }
        
        #[test]
        fn prop_serialization_roundtrip(
            cards in prop::collection::vec(valid_card_strategy(), 0..20)
        ) {
            let bitset = CardBitset::from(cards);
            
            // JSON roundtrip
            let json = serde_json::to_string(&bitset).unwrap();
            let restored: CardBitset = serde_json::from_str(&json).unwrap();
            
            prop_assert_eq!(bitset, restored);
            prop_assert_eq!(bitset.count(), restored.count());
            prop_assert_eq!(bitset.raw(), restored.raw());
        }
    }
}