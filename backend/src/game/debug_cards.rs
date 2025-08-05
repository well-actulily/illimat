use crate::game::card::{Card, Rank, Suit};

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn debug_card_ids() {
        println!("Card ID ranges:");
        
        for suit in [Suit::Spring, Suit::Summer, Suit::Autumn, Suit::Winter, Suit::Stars] {
            for rank in [Rank::Fool, Rank::Two, Rank::Three, Rank::Four, Rank::Five, 
                        Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten, 
                        Rank::Knight, Rank::Queen, Rank::King] {
                let card = Card::new(rank, suit);
                println!("Card::new({:?}, {:?}) = Card({}) = {}", rank, suit, card.id(), card);
            }
        }
        
        // Find the maximum card ID
        let max_card = Card::new(Rank::King, Suit::Stars);
        println!("Maximum card ID: {}", max_card.id());
        
        // Check if 64 is a valid card
        let card_64 = Card::from_id(64);
        println!("Card::from_id(64) = {} (rank: {:?}, suit: {:?})", 
                 card_64, card_64.rank(), card_64.suit());
    }
}