namespace Illimat.Core.Models
{
    public class SuitSet
    {
        public static readonly IReadOnlyCollection<Suit> AllSuits = new List<Suit>
        {
            Suit.Spring,
            Suit.Summer,
            Suit.Autumn,
            Suit.Winter,
            Suit.Stars
        };

        public static readonly IReadOnlyCollection<Suit> NoStars = new List<Suit>
        {
            Suit.Spring,
            Suit.Summer,
            Suit.Autumn,
            Suit.Winter
        };
    }
}
