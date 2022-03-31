using Illimat.Core.Models;

namespace Illimat.Core.Extensions
{
    public static class SuitExtensions
    {
        public static string ToShortString(this Suit suit) => suit switch
        {
            Suit.Spring => "Sp",
            Suit.Summer => "Su",
            Suit.Autumn => "Au",
            Suit.Winter => "Wi",
            Suit.Stars => "St",
            _ => throw new ArgumentException($"Suit '{suit}' is undefined.")
        };

        public static string ToFriendlyString(this Suit suit) => suit switch
        {
            Suit.Spring => "Spring",
            Suit.Summer => "Summer",
            Suit.Autumn => "Autumn",
            Suit.Winter => "Winter",
            Suit.Stars => "Stars",
            _ => ((int)suit).ToString()
        };

        public static ConsoleColor ToConsoleColor(this Suit suit) => suit switch
        {
            Suit.Spring => ConsoleColor.Green,
            Suit.Summer => ConsoleColor.Yellow,
            Suit.Autumn => ConsoleColor.Red,
            Suit.Winter => ConsoleColor.Cyan,
            Suit.Stars => ConsoleColor.Magenta,
            _ => throw new ArgumentException($"Suit '{suit}' is undefined.")
        };

        public static Suit ToSuit(this string suitString) => suitString.ToLowerInvariant() switch
        {
            "spring" or "sp" => Suit.Spring,
            "summer" or "su" => Suit.Summer,
            "autumn" or "au" or "a" or "fall" or "fa" or "f" => Suit.Autumn,
            "winter" or "wi" or "w" => Suit.Winter,
            "stars" or "star" or "st" => Suit.Stars,
            _ => throw new ArgumentException($"Suit '{suitString}' is undefined.")
        };
    }
}
