using Illimat.Core.Extensions;
using Illimat.Core.Models;

namespace Illimat.Core
{
    public record Card : ICard
    {
        public Rank Rank { get; init; }
        public Suit Suit { get; init; }
        private bool isRevealed = false;
        public bool IsRevealed { get => isRevealed; set => isRevealed = value; }

        public Card (Rank rank, Suit suit, bool isRevealed = false)
        {
            Rank = rank;
            Suit = suit;
            IsRevealed = isRevealed;
        }

        public Card(string rankString, string suitString, bool isRevealed = false)
        {
            Rank = rankString.ToRank();
            Suit = suitString.ToSuit();
            IsRevealed = isRevealed;
        }

        public static IList<Card> GetCards(IEnumerable<Suit> suitSet)
        {
            return RankSet.AllRanks
                .SelectMany((r) => suitSet, (r, s) => new Card(r, s)).ToList();
        }
    }
}
