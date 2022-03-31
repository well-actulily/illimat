using Illimat.Core.Models;

namespace Illimat.Core.Extensions
{
    public static class RankExtensions
    {
        public static string ToShortString(this Rank rank) => rank switch
        {
            Rank.Two => "2",
            Rank.Three => "3",
            Rank.Four => "4",
            Rank.Five => "5",
            Rank.Six => "6",
            Rank.Seven => "7",
            Rank.Eight => "8",
            Rank.Nine => "9",
            Rank.Ten => "T",
            Rank.Knight => "N",
            Rank.Queen => "Q",
            Rank.King => "K",
            Rank.Fool => "F",
            _ => throw new ArgumentException($"Rank '{rank}' is undefined.")
        };

        public static string ToFriendlyString(this Rank rank) => rank switch
        {
            Rank.Two => "2",
            Rank.Three => "3",
            Rank.Four => "4",
            Rank.Five => "5",
            Rank.Six => "6",
            Rank.Seven => "7",
            Rank.Eight => "8",
            Rank.Nine => "9",
            Rank.Ten => "10",
            Rank.Knight => "Knight",
            Rank.Queen => "Queen",
            Rank.King => "King",
            Rank.Fool => "Fool",
            _ => ((int)rank).ToString()
        };

        public static List<int> Values(this Rank rank) => rank switch
        {
            Rank.Two => new List<int> { 2 },
            Rank.Three => new List<int> { 3 },
            Rank.Four => new List<int> { 4 },
            Rank.Five => new List<int> { 5 },
            Rank.Six => new List<int> { 6 },
            Rank.Seven => new List<int> { 7 },
            Rank.Eight => new List<int> { 8 },
            Rank.Nine => new List<int> { 9 },
            Rank.Ten => new List<int> { 10 },
            Rank.Knight => new List<int> { 11 },
            Rank.Queen => new List<int> { 12 },
            Rank.King => new List<int> { 13 },
            Rank.Fool => new List<int> { 14, 1 },
            _ => throw new ArgumentException($"Rank '{(int)rank}' is undefined.")
        };

        public static Rank ToRank(this string rankString)
        {
            return rankString.ToLowerInvariant() switch
            {
                "2" or "two" => Rank.Two,
                "3" or "three" => Rank.Three,
                "4" or "four" => Rank.Four,
                "5" or "five" => Rank.Five,
                "6" or "six" => Rank.Six,
                "7" or "seven" => Rank.Seven,
                "8" or "eight" => Rank.Eight,
                "9" or "nine" => Rank.Nine,
                "10" or "ten" or "t" => Rank.Ten,
                "11" or "knight" or "n" or "kn" => Rank.Knight,
                "12" or "queen" or "q" or "qu" => Rank.Queen,
                "13" or "king" or "k" or "ki" => Rank.King,
                "14" or "1" or "fool" or "f" => Rank.Fool,
                _ => throw new ArgumentException($"Failed to parse rank '{rankString}'.")
            };
        }
    }
}
