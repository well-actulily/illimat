use std::fmt;

/// Illimat suits: Spring, Summer, Autumn, Winter, Stars
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Suit {
    Spring = 0,
    Summer = 1,
    Autumn = 2,
    Winter = 3,
    Stars = 4,
}

/// Illimat ranks: Fool (1 or 14), 2-10, Knight (11), Queen (12), King (13)
#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Rank {
    Fool = 0,   // Value 1 or 14
    Two = 1,    // Value 2
    Three = 2,  // Value 3
    Four = 3,   // Value 4
    Five = 4,   // Value 5
    Six = 5,    // Value 6
    Seven = 6,  // Value 7
    Eight = 7,  // Value 8
    Nine = 8,   // Value 9
    Ten = 9,    // Value 10
    Knight = 10, // Value 11
    Queen = 11,  // Value 12
    King = 12,   // Value 13
}

/// Compact card representation: 8 bits total (4 bits suit + 4 bits rank)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
pub struct Card(u8);

impl Card {
    /// Create a new card
    pub const fn new(rank: Rank, suit: Suit) -> Self {
        Card(((suit as u8) << 4) | (rank as u8))
    }
    
    /// Get the rank of this card
    pub const fn rank(self) -> Rank {
        // SAFETY: We only create cards with valid rank values
        unsafe { std::mem::transmute(self.0 & 0x0F) }
    }
    
    /// Get the suit of this card
    pub const fn suit(self) -> Suit {
        // SAFETY: We only create cards with valid suit values
        unsafe { std::mem::transmute(self.0 >> 4) }
    }
    
    /// Get unique ID for bitset operations (0-64)
    pub const fn id(self) -> u8 {
        self.0
    }
    
    /// Create card from ID (inverse of id())
    pub const fn from_id(id: u8) -> Self {
        Card(id)
    }
    
    /// Get the value of this card for game purposes
    pub const fn value(self) -> u8 {
        match self.rank() {
            Rank::Fool => 1, // Fool defaults to 1, but can be played as 14
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Knight => 11,
            Rank::Queen => 12,
            Rank::King => 13,
        }
    }
    
    /// Check if this card can be played as a specific value (for Fool handling)
    pub const fn can_be_value(self, target_value: u8) -> bool {
        match self.rank() {
            Rank::Fool => target_value == 1 || target_value == 14,
            _ => self.value() == target_value,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let abbrev = match self {
            Suit::Spring => "Sp",
            Suit::Summer => "Su",   
            Suit::Autumn => "Au",
            Suit::Winter => "Wi",
            Suit::Stars => "St",
        };
        write!(f, "{}", abbrev)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Rank::Fool => "F",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "T",  // Single character
            Rank::Knight => "N",
            Rank::Queen => "Q",
            Rank::King => "K",
        };
        write!(f, "{}", name)
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{} {}]", self.rank(), self.suit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_card_encoding_roundtrip() {
        let card = Card::new(Rank::Eight, Suit::Winter);
        assert_eq!(card.rank(), Rank::Eight);
        assert_eq!(card.suit(), Suit::Winter);
    }

    #[test] 
    fn test_card_id_roundtrip() {
        let original = Card::new(Rank::Queen, Suit::Stars);
        let id = original.id();
        let recovered = Card::from_id(id);
        assert_eq!(original, recovered);
    }

    #[test]
    fn test_card_values() {
        assert_eq!(Card::new(Rank::Fool, Suit::Spring).value(), 1);
        assert_eq!(Card::new(Rank::Five, Suit::Summer).value(), 5);
        assert_eq!(Card::new(Rank::King, Suit::Autumn).value(), 13);
    }

    #[test]
    fn test_fool_can_be_values() {
        let fool = Card::new(Rank::Fool, Suit::Winter);
        assert!(fool.can_be_value(1));
        assert!(fool.can_be_value(14));
        assert!(!fool.can_be_value(7));
        
        let seven = Card::new(Rank::Seven, Suit::Spring);
        assert!(seven.can_be_value(7));
        assert!(!seven.can_be_value(1));
        assert!(!seven.can_be_value(14));
    }

    #[test]
    fn test_card_display() {
        let card = Card::new(Rank::Eight, Suit::Winter);
        let display = format!("{}", card);
        assert_eq!(display, "[8 Wi]");
        
        let knight = Card::new(Rank::Knight, Suit::Stars);
        let knight_display = format!("{}", knight);
        assert_eq!(knight_display, "[N St]");
        
        let ten = Card::new(Rank::Ten, Suit::Summer);
        let ten_display = format!("{}", ten);
        assert_eq!(ten_display, "[T Su]");
        
        let fool = Card::new(Rank::Fool, Suit::Stars);
        let fool_display = format!("{}", fool);
        assert_eq!(fool_display, "[F St]");
    }
}

#[cfg(test)]
mod proptest_impls {
    use super::*;
    use proptest::prelude::*;
    
    impl Arbitrary for Suit {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Suit::Spring),
                Just(Suit::Summer),
                Just(Suit::Autumn),
                Just(Suit::Winter),
                Just(Suit::Stars),
            ].boxed()
        }
    }
    
    impl Arbitrary for Rank {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            prop_oneof![
                Just(Rank::Fool),
                Just(Rank::Two),
                Just(Rank::Three),
                Just(Rank::Four),
                Just(Rank::Five),
                Just(Rank::Six),
                Just(Rank::Seven),
                Just(Rank::Eight),
                Just(Rank::Nine),
                Just(Rank::Ten),
                Just(Rank::Knight),
                Just(Rank::Queen),
                Just(Rank::King),
            ].boxed()
        }
    }
    
    impl Arbitrary for Card {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;
        
        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<Rank>(), any::<Suit>())
                .prop_map(|(rank, suit)| Card::new(rank, suit))
                .boxed()
        }
    }
}