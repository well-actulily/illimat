using Illimat.Core.Models;

namespace Illimat.Core
{
    public record Player : IActor
    {
        public string Name { get; init; }
        public PlayerType Type { get; init; }
        public int Score { get; set; }
        public List<Card> Hand { get; set; } = new();
        public List<Card> HarvestPile { get; } = new();
        public List<Okus> ScorePileOkuses { get; } = new();
        public List<Luminary> ScorePileLuminaries { get; } = new();
        public Dictionary<Superlative, int> SuperlativeScoring { get; } = new Dictionary<Superlative, int> {
            {Superlative.BumperCrop, 4 },
            {Superlative.Frostbit, -2 },
            {Superlative.Sunkissed, 2 }
        };

        public Player(string name, PlayerType type) 
        {
            Name = name;
            Type = type;
        }
    }
}
