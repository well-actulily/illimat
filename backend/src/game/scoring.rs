use crate::game::card::{Card, Rank, Suit};
use crate::game::player::PlayerId;
use crate::game::okus::{OkusPosition, OkusManager};

/// Competitive scoring results for end of round
#[derive(Debug, Clone)]
pub struct RoundScoring {
    pub bumper_crop_winner: Option<PlayerId>,
    pub sunkissed_winner: Option<PlayerId>, 
    pub frostbit_players: Vec<PlayerId>,
    pub individual_scores: [u8; 4], // Fools + okus for each player
}

/// Scoring utilities and calculations
pub struct ScoringManager;

impl ScoringManager {
    /// Calculate end-of-round scoring for all players
    pub fn calculate_round_scoring(
        player_harvests: &[Vec<Card>; 4],
        okus_positions: &[OkusPosition; 4]
    ) -> RoundScoring {
        let mut scoring = RoundScoring {
            bumper_crop_winner: None,
            sunkissed_winner: None,
            frostbit_players: Vec::new(),
            individual_scores: [0; 4],
        };
        
        // Calculate individual scores (Fools + okus)
        for player_id in 0..4 {
            let player = PlayerId(player_id as u8);
            let fools_count = Self::count_fools(&player_harvests[player_id]);
            let okus_count = OkusManager::count_player_okus(okus_positions, player);
            scoring.individual_scores[player_id] = (fools_count + okus_count) as u8;
        }
        
        // Competitive scoring
        scoring.bumper_crop_winner = Self::find_bumper_crop_winner(player_harvests);
        scoring.sunkissed_winner = Self::find_sunkissed_winner(player_harvests);
        scoring.frostbit_players = Self::find_frostbit_players(player_harvests);
        
        scoring
    }
    
    /// Apply round scoring to total scores
    pub fn apply_round_scoring(
        total_scores: &mut [u8; 4],
        scoring: &RoundScoring
    ) {
        // Apply competitive bonuses/penalties
        if let Some(winner) = scoring.bumper_crop_winner {
            total_scores[winner.0 as usize] += 4; // Bumper Crop +4
        }
        
        if let Some(winner) = scoring.sunkissed_winner {
            total_scores[winner.0 as usize] += 2; // Sunkissed +2
        }
        
        for &player in &scoring.frostbit_players {
            // Prevent underflow - scores can't go below 0
            if total_scores[player.0 as usize] >= 2 {
                total_scores[player.0 as usize] -= 2; // Frostbit -2
            } else {
                total_scores[player.0 as usize] = 0;
            }
        }
        
        // Apply individual scores (Fools + okus)
        for player_id in 0..4 {
            let individual_score = scoring.individual_scores[player_id];
            if individual_score > 0 {
                total_scores[player_id] += individual_score as u8;
            }
            // Individual scores shouldn't be negative, but just in case
        }
    }
    
    /// Check if any player has won (17+ points)
    pub fn check_victory(total_scores: &[u8; 4]) -> Option<PlayerId> {
        for (player_id, &score) in total_scores.iter().enumerate() {
            if score >= 17 {
                return Some(PlayerId(player_id as u8));
            }
        }
        None
    }
    
    /// Format scoring results for display
    pub fn format_round_scoring(scoring: &RoundScoring) -> String {
        let mut parts = Vec::new();
        
        if let Some(winner) = scoring.bumper_crop_winner {
            parts.push(format!("🌾 Bumper Crop (+4): Player {}", winner.0));
        }
        
        if let Some(winner) = scoring.sunkissed_winner {
            parts.push(format!("☀️ Sunkissed (+2): Player {}", winner.0));
        }
        
        if !scoring.frostbit_players.is_empty() {
            let players_str = scoring.frostbit_players.iter()
                .map(|p| p.0.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(format!("❄️ Frostbit (-2): Player(s) {}", players_str));
        }
        
        // Individual scores
        for (player_id, &score) in scoring.individual_scores.iter().enumerate() {
            if score > 0 {
                parts.push(format!("🃏 Individual (+{}): Player {}", score, player_id));
            }
        }
        
        if parts.is_empty() {
            "No special scoring this round".to_string()
        } else {
            parts.join("\n")
        }
    }
    
    // Private helper functions
    
    /// Find Bumper Crop winner (most Spring cards)
    fn find_bumper_crop_winner(player_harvests: &[Vec<Card>; 4]) -> Option<PlayerId> {
        let spring_counts: Vec<_> = player_harvests.iter()
            .map(|harvest| Self::count_suit(harvest, Suit::Spring))
            .collect();
        
        let max_spring = *spring_counts.iter().max().unwrap();
        
        if max_spring == 0 {
            return None; // No one harvested Spring cards
        }
        
        // Check for ties
        let winners: Vec<_> = spring_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count == max_spring)
            .map(|(player_id, _)| PlayerId(player_id as u8))
            .collect();
        
        if winners.len() == 1 {
            Some(winners[0])
        } else {
            None // Tie, no winner
        }
    }
    
    /// Find Sunkissed winner (most Summer cards)
    fn find_sunkissed_winner(player_harvests: &[Vec<Card>; 4]) -> Option<PlayerId> {
        let summer_counts: Vec<_> = player_harvests.iter()
            .map(|harvest| Self::count_suit(harvest, Suit::Summer))
            .collect();
        
        let max_summer = *summer_counts.iter().max().unwrap();
        
        if max_summer == 0 {
            return None;
        }
        
        let winners: Vec<_> = summer_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count == max_summer)
            .map(|(player_id, _)| PlayerId(player_id as u8))
            .collect();
        
        if winners.len() == 1 {
            Some(winners[0])
        } else {
            None
        }
    }
    
    /// Find Frostbit players (most Winter cards, if any)
    fn find_frostbit_players(player_harvests: &[Vec<Card>; 4]) -> Vec<PlayerId> {
        let winter_counts: Vec<_> = player_harvests.iter()
            .map(|harvest| Self::count_suit(harvest, Suit::Winter))
            .collect();
        
        let max_winter = *winter_counts.iter().max().unwrap();
        
        if max_winter == 0 {
            return Vec::new();
        }
        
        winter_counts.iter()
            .enumerate()
            .filter(|(_, &count)| count == max_winter)
            .map(|(player_id, _)| PlayerId(player_id as u8))
            .collect()
    }
    
    /// Count cards of a specific suit in a harvest
    fn count_suit(harvest: &[Card], suit: Suit) -> u8 {
        harvest.iter()
            .filter(|card| card.suit() == suit)
            .count() as u8
    }
    
    /// Count Fool cards in a harvest
    fn count_fools(harvest: &[Card]) -> u8 {
        harvest.iter()
            .filter(|card| card.rank() == Rank::Fool)
            .count() as u8
    }
}