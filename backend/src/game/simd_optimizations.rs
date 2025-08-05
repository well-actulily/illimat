//! SIMD Optimization Prototypes for Illimat Card Operations
//! 
//! This module explores SIMD vectorization opportunities for high-frequency
//! card operations, particularly useful for Monte Carlo Tree Search where
//! batch processing of game states and move generation is critical.

use crate::game::card::{Card, Rank, Suit};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
#[allow(unused_imports)]
use std::arch::aarch64::*;

/// SIMD-optimized operations for card collections
pub struct SimdCardOps;

impl SimdCardOps {
    /// Vectorized card value extraction using AVX2
    /// 
    /// Processes 32 cards at once, extracting their values for batch operations.
    /// This is particularly useful for finding card combinations that sum to target values.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn extract_values_avx2(cards: &[Card], values: &mut [u8]) {
        assert!(cards.len() <= 32);
        assert!(values.len() >= cards.len());
        
        if cards.len() < 16 {
            // Fall back to scalar for small inputs
            Self::extract_values_scalar(cards, values);
            return;
        }
        
        // Load 32 bytes (32 cards) at once using AVX2
        let card_bytes = std::slice::from_raw_parts(cards.as_ptr() as *const u8, cards.len());
        
        if card_bytes.len() >= 32 {
            let cards_vec = _mm256_loadu_si256(card_bytes.as_ptr() as *const __m256i);
            
            // Extract rank bits (lower 4 bits) from each card
            let rank_mask = _mm256_set1_epi8(0x0F);
            let ranks = _mm256_and_si256(cards_vec, rank_mask);
            
            // Convert rank values to game values using lookup table
            let value_lookup = _mm256_setr_epi8(
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, // Ranks 0-12
                0, 0, 0, // Unused (padding)
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, // Duplicate for second half
                0, 0, 0  // Unused (padding)
            );
            
            let values_vec = _mm256_shuffle_epi8(value_lookup, ranks);
            
            // Store results
            _mm256_storeu_si256(values.as_mut_ptr() as *mut __m256i, values_vec);
        }
        
        // Handle remaining cards with scalar code
        let processed = (card_bytes.len() / 32) * 32;
        if processed < cards.len() {
            Self::extract_values_scalar(&cards[processed..], &mut values[processed..]);
        }
    }
    
    /// Scalar fallback for card value extraction
    pub fn extract_values_scalar(cards: &[Card], values: &mut [u8]) {
        for (i, &card) in cards.iter().enumerate() {
            if i >= values.len() { break; }
            values[i] = card.value();
        }
    }
    
    /// Vectorized card matching for harvest operations
    /// 
    /// Finds all cards in a collection that can be played as a specific value.
    /// This is critical for MCTS move generation where we need to quickly identify
    /// valid harvest targets.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn find_matching_cards_avx2(
        cards: &[Card], 
        target_value: u8, 
        matches: &mut [bool]
    ) -> usize {
        assert!(matches.len() >= cards.len());
        
        if cards.len() < 16 {
            return Self::find_matching_cards_scalar(cards, target_value, matches);
        }
        
        let mut match_count = 0;
        let card_bytes = std::slice::from_raw_parts(cards.as_ptr() as *const u8, cards.len());
        
        // Process 32 cards at once
        let mut i = 0;
        while i + 32 <= card_bytes.len() {
            let cards_vec = _mm256_loadu_si256(card_bytes[i..].as_ptr() as *const __m256i);
            
            // Extract ranks (lower 4 bits)
            let rank_mask = _mm256_set1_epi8(0x0F);
            let ranks = _mm256_and_si256(cards_vec, rank_mask);
            
            // Check for exact value matches
            let target_vec = _mm256_set1_epi8(match target_value {
                1 => 0,   // Fool can be 1
                2 => 1,   // Two
                3 => 2,   // Three
                4 => 3,   // Four
                5 => 4,   // Five
                6 => 5,   // Six
                7 => 6,   // Seven
                8 => 7,   // Eight
                9 => 8,   // Nine
                10 => 9,  // Ten
                11 => 10, // Knight
                12 => 11, // Queen
                13 => 12, // King
                14 => 0,  // Fool can be 14
                _ => 255, // No match
            });
            
            let exact_matches = _mm256_cmpeq_epi8(ranks, target_vec);
            
            // For Fool cards (rank 0), also check if target is 14
            let fool_mask = _mm256_cmpeq_epi8(ranks, _mm256_setzero_si256());
            let fool_matches = if target_value == 14 {
                fool_mask
            } else {
                _mm256_setzero_si256()
            };
            
            // Combine exact matches and fool matches
            let all_matches = _mm256_or_si256(exact_matches, fool_matches);
            
            // Extract match bits and store
            let match_bits = _mm256_movemask_epi8(all_matches);
            for bit_pos in 0..32 {
                if i + bit_pos >= matches.len() { break; }
                let is_match = (match_bits & (1 << bit_pos)) != 0;
                matches[i + bit_pos] = is_match;
                if is_match {
                    match_count += 1;
                }
            }
            
            i += 32;
        }
        
        // Handle remaining cards
        if i < cards.len() {
            match_count += Self::find_matching_cards_scalar(
                &cards[i..], 
                target_value, 
                &mut matches[i..]
            );
        }
        
        match_count
    }
    
    /// Scalar fallback for card matching  
    pub fn find_matching_cards_scalar(
        cards: &[Card], 
        target_value: u8, 
        matches: &mut [bool]
    ) -> usize {
        let mut match_count = 0;
        for (i, &card) in cards.iter().enumerate() {
            if i >= matches.len() { break; }
            let is_match = card.can_be_value(target_value);
            matches[i] = is_match;
            if is_match {
                match_count += 1;
            }
        }
        match_count
    }
    
    /// Vectorized sum calculation for card combinations
    /// 
    /// Rapidly computes sums of card values for harvest combination validation.
    /// Essential for MCTS where we need to evaluate thousands of potential moves.
    #[cfg(target_arch = "x86_64")]
    pub unsafe fn sum_card_values_avx2(cards: &[Card]) -> u32 {
        if cards.len() < 16 {
            return Self::sum_card_values_scalar(cards);
        }
        
        let card_bytes = std::slice::from_raw_parts(cards.as_ptr() as *const u8, cards.len());
        let mut total_sum = 0u32;
        
        // Process 32 cards at once
        let mut i = 0;
        while i + 32 <= card_bytes.len() {
            let cards_vec = _mm256_loadu_si256(card_bytes[i..].as_ptr() as *const __m256i);
            
            // Extract ranks and convert to values
            let rank_mask = _mm256_set1_epi8(0x0F);
            let ranks = _mm256_and_si256(cards_vec, rank_mask);
            
            let value_lookup = _mm256_setr_epi8(
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0, 0, 0,
                1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 0, 0, 0
            );
            
            let values_vec = _mm256_shuffle_epi8(value_lookup, ranks);
            
            // Sum the values using horizontal add
            let sum_vec = _mm256_sad_epu8(values_vec, _mm256_setzero_si256());
            let sum_parts = _mm256_extract_epi64(sum_vec, 0) as u32 +
                           _mm256_extract_epi64(sum_vec, 1) as u32 +
                           _mm256_extract_epi64(sum_vec, 2) as u32 +
                           _mm256_extract_epi64(sum_vec, 3) as u32;
            
            total_sum += sum_parts;
            i += 32;
        }
        
        // Handle remaining cards
        if i < cards.len() {
            total_sum += Self::sum_card_values_scalar(&cards[i..]);
        }
        
        total_sum
    }
    
    /// Scalar fallback for card sum calculation
    pub fn sum_card_values_scalar(cards: &[Card]) -> u32 {
        cards.iter().map(|card| card.value() as u32).sum()
    }
}

/// Vectorized field operations for batch processing
pub struct SimdFieldOps;

impl SimdFieldOps {
    /// Batch process multiple field states for MCTS tree search
    /// 
    /// This function can process multiple game states simultaneously,
    /// essential for parallel MCTS evaluation.
    pub fn batch_evaluate_field_states(
        field_states: &[&[Card]], 
        target_sums: &[u8],
        results: &mut [bool]
    ) {
        assert_eq!(field_states.len(), target_sums.len());
        assert_eq!(field_states.len(), results.len());
        
        for (i, (&field_cards, &target_sum)) in field_states.iter().zip(target_sums.iter()).enumerate() {
            if i >= results.len() { break; }
            
            let field_sum = if field_cards.len() >= 16 {
                #[cfg(target_arch = "x86_64")]
                unsafe { SimdCardOps::sum_card_values_avx2(field_cards) }
                #[cfg(not(target_arch = "x86_64"))]
                SimdCardOps::sum_card_values_scalar(field_cards)
            } else {
                SimdCardOps::sum_card_values_scalar(field_cards)
            };
            
            results[i] = field_sum == target_sum as u32;
        }
    }
}

/// Performance analysis and benchmarking for SIMD operations
pub mod benchmarks {
    use super::*;
    use std::time::Instant;
    
    /// Benchmark SIMD vs scalar card value extraction
    pub fn benchmark_value_extraction(card_count: usize, iterations: usize) -> (f64, f64) {
        let cards: Vec<Card> = (0..card_count)
            .map(|i| Card::new(
                match i % 13 {
                    0 => Rank::Fool, 1 => Rank::Two, 2 => Rank::Three, 3 => Rank::Four,
                    4 => Rank::Five, 5 => Rank::Six, 6 => Rank::Seven, 7 => Rank::Eight,
                    8 => Rank::Nine, 9 => Rank::Ten, 10 => Rank::Knight, 11 => Rank::Queen,
                    _ => Rank::King,
                },
                match i % 4 {
                    0 => Suit::Spring, 1 => Suit::Summer, 2 => Suit::Autumn, _ => Suit::Winter,
                }
            ))
            .collect();
        
        let mut values_scalar = vec![0u8; card_count];
        let mut values_simd = vec![0u8; card_count];
        
        // Benchmark scalar version
        let start = Instant::now();
        for _ in 0..iterations {
            SimdCardOps::extract_values_scalar(&cards, &mut values_scalar);
        }
        let scalar_time = start.elapsed().as_nanos() as f64 / iterations as f64;
        
        // Benchmark SIMD version
        let start = Instant::now();
        for _ in 0..iterations {
            #[cfg(target_arch = "x86_64")]
            unsafe { SimdCardOps::extract_values_avx2(&cards, &mut values_simd); }
            #[cfg(not(target_arch = "x86_64"))]
            SimdCardOps::extract_values_scalar(&cards, &mut values_simd);
        }
        let simd_time = start.elapsed().as_nanos() as f64 / iterations as f64;
        
        // Verify results are identical
        assert_eq!(values_scalar, values_simd);
        
        (scalar_time, simd_time)
    }
    
    /// Performance analysis report
    pub fn generate_simd_analysis_report() -> String {
        let mut report = String::new();
        report.push_str("=== SIMD Optimization Analysis Report ===\n\n");
        
        // Test different input sizes
        let test_sizes = [16, 32, 64, 128, 256, 512];
        let iterations = 10000;
        
        report.push_str("Card Value Extraction Performance:\n");
        report.push_str("Size\tScalar (ns)\tSIMD (ns)\tSpeedup\n");
        
        for &size in &test_sizes {
            let (scalar_time, simd_time) = benchmark_value_extraction(size, iterations);
            let speedup = scalar_time / simd_time;
            report.push_str(&format!("{}\t{:.1}\t{:.1}\t{:.2}x\n", 
                size, scalar_time, simd_time, speedup));
        }
        
        report.push_str("\n");
        report.push_str("Optimization Recommendations:\n");
        report.push_str("- SIMD provides significant benefits for card collections > 32 cards\n");
        report.push_str("- Use vectorized operations for MCTS batch processing\n");
        report.push_str("- Consider SIMD for hot paths in move generation\n");
        report.push_str("- WebAssembly SIMD support varies by browser - provide fallbacks\n");
        
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scalar_value_extraction() {
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
            Card::new(Rank::Fool, Suit::Autumn),
        ];
        
        let mut values = vec![0u8; 3];
        SimdCardOps::extract_values_scalar(&cards, &mut values);
        
        assert_eq!(values, vec![5, 10, 1]);
    }
    
    #[test]
    fn test_scalar_card_matching() {
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
            Card::new(Rank::Fool, Suit::Autumn),
        ];
        
        let mut matches = vec![false; 3];
        let count = SimdCardOps::find_matching_cards_scalar(&cards, 5, &mut matches);
        
        assert_eq!(count, 1);
        assert_eq!(matches, vec![true, false, false]);
        
        // Test Fool matching
        let mut fool_matches = vec![false; 3];
        let fool_count = SimdCardOps::find_matching_cards_scalar(&cards, 14, &mut fool_matches);
        
        assert_eq!(fool_count, 1);
        assert_eq!(fool_matches, vec![false, false, true]);
    }
    
    #[test]
    fn test_scalar_sum_calculation() {
        let cards = vec![
            Card::new(Rank::Five, Suit::Spring),
            Card::new(Rank::Ten, Suit::Summer),
            Card::new(Rank::Three, Suit::Autumn),
        ];
        
        let sum = SimdCardOps::sum_card_values_scalar(&cards);
        assert_eq!(sum, 18); // 5 + 10 + 3
    }
}