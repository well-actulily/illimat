/// Player identifier (0-3)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlayerId(pub u8);

/// Player type for AI support
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerType {
    Human,
    /// Simple CPU that follows harvest > stockpile > sow strategy
    SimpleCpu,
    /// Advanced CPU using Monte Carlo Tree Search (not yet implemented)
    MctsCpu,
}