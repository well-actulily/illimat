using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class SeedField : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; init; }
        public int RevealedCount { get; init; }
        public int HiddenCount { get; init; }
        private List<Card>? Cards { get; set; }

        public SeedField(IActor actor, Field field, int revealedCount = 3, int hiddenCount = 0)
        {
            Actor = actor;
            Field = field;
            RevealedCount = revealedCount;
            HiddenCount = hiddenCount;
        }

        public void Perform(GameState gameState)
        {
            Cards = gameState.CardDeck.DrawUpTo(RevealedCount).ToList();

            for (int i = 0; i < Cards.Count; i++)
            {
                Cards[i].IsRevealed = i < RevealedCount;
                Field.Piles.Add(new Pile(new List<Card> { Cards[i] }));
            }

            Console.WriteLine($"Seeded field {Array.IndexOf(gameState.Fields, Field)} with {Cards.Count} cards.");
        }

        public void Unwind(GameState gameState)
        {
            if (Cards != null)
            {
                var reversedCards = Cards.Select(x => x).Reverse();

                foreach (Card card in reversedCards)
                {
                    Field.Piles.Remove(Field.Piles
                        .Where(x => x.Cards.Contains(card))
                        .Single());
                    card.IsRevealed = false;
                    gameState.CardDeck.Cards.Insert(0, card);
                }
            }
        }
    }
}
