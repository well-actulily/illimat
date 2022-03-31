namespace Illimat.Core.Models
{
    public class RankSet
    {
        public static readonly IReadOnlyCollection<Rank> AllRanks = new List<Rank>
        {
            Rank.Two,
            Rank.Three,
            Rank.Four,
            Rank.Five,
            Rank.Six,
            Rank.Seven,
            Rank.Eight,
            Rank.Nine,
            Rank.Ten,
            Rank.Knight,
            Rank.Queen,
            Rank.King,
            Rank.Fool
        };
    }
}
