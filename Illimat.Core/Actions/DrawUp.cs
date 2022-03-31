using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class DrawUp : IAction
    {
        public IActor Actor { get; }
        private List<Card>? Cards { get; set; }

        const int MAX_CARDS = 4;

        public DrawUp(Player player)
        {
            Actor = player;
        }

        public void Perform(GameState gameState)
        {
            var player = (Player)Actor;
            var count = MAX_CARDS - player.Hand.Count;
            count = count < 0 ? 0 : count;

            Cards = gameState.CardDeck.DrawUpTo(count).ToList();
            player.Hand.AddRange(Cards);
            Console.WriteLine($"{Actor} drew up {Cards.Count} cards. They now have {player.Hand.Count} cards in their hand.");
        }

        public void Unwind(GameState gameState)
        {
            if (Cards != null)
            {
                var player = (Player)Actor;

                for (int i = Cards.Count - 1; i >= 0; i--)
                {
                    var card = Cards[i];
                    player.Hand.Remove(card);
                    gameState.CardDeck.Cards.Insert(0, card);
                }
            }
        }
    }
}
