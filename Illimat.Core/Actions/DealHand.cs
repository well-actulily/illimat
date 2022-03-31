using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class DealHand : IAction
    {
        public IActor Actor { get; }
        public Player Player { get; init; }
        public int Count { get; init; }
        private List<Card>? Cards { get; set; }

        public DealHand(Player dealer, Player player, int count)
        {
            Actor = dealer;
            Player = player;
            Count = count;
        }

        public void Perform(GameState gameState)
        {
            Cards = gameState.CardDeck.DrawUpTo(Count).ToList();
            Player.Hand.AddRange(Cards);
            Console.WriteLine($"{Actor} (as dealer) dealt {Cards.Count} cards to {Player}.");
        }

        public void Unwind(GameState gameState)
        {
            if (Cards != null)
            {
                var reversedCards = Cards.Select(x => x).Reverse();

                foreach (Card card in reversedCards)
                {
                    Player.Hand.Remove(card);
                    gameState.CardDeck.Cards.Insert(0, card);
                }
            }
        }
    }
}
