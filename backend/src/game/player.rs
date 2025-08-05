/// Player identifier (0-3)
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PlayerId(pub u8);

/// Player type for future AI support
#[derive(Copy, Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PlayerType {
    Human,
    Computer,
}