using Illimat.Core.Models;

namespace Illimat.Core
{
    public record class Deck<T> where T : ICard
    {
        public readonly IList<T> Cards;

        public Deck(IList<T> cards) {
            Cards = cards;
        }


        public void Shuffle(Random random)
        {
            int n = Cards.Count;
            while (n > 1)
            {
                n--;
                var k = random.Next(n + 1);
                (Cards[n], Cards[k]) = (Cards[k], Cards[n]);
            }
        }

        public IEnumerable<T> DrawUpTo(int count)
        {
            var draw = new List<T>();
            var drawAmount = Cards.Count < count ? Cards.Count : count;

            lock(Cards)
            {
                for (int i = 0; i < drawAmount; i++)
                {
                    T value = Cards[0];
                    Cards.RemoveAt(0);
                    draw.Add(value);
                }
            }
            return draw;
        }

        public KeyValuePair<int, T> DrawSpecific(T card)
        {
            var cardFromDeck = Cards
                        .Select((x, i) => new { Value = x, Index = i })
                        .Where(x => x.Value.Equals(card))
                        .SingleOrDefault();

            if (cardFromDeck != null)
            {
                Cards.RemoveAt(cardFromDeck.Index);
                return new KeyValuePair<int, T>(cardFromDeck.Index, cardFromDeck.Value);
            }

            throw new ArgumentException($"Card {card} not found in deck.");
        }
    }
}
