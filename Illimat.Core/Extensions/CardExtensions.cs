namespace Illimat.Core.Extensions
{
    public static class CardExtensions
    {
        public static string ToShortString(this Card card)
        {
            return card.IsRevealed ?
                $"{card.Rank.ToShortString()}{card.Suit.ToShortString()}" :
                "Unk";
        }

        public static string ToString(this Card card)
        {
            return card.IsRevealed ?
                $"{card.Rank.ToFriendlyString()} of {card.Suit.ToFriendlyString()}" :
                "Unknown of Unknown";
        }
    }
}
