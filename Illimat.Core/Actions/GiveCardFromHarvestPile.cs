using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class GiveCardFromHarvestPile : IAction
    {
        public IActor Actor { get; }
        public Player Recipient { get; }
        public Season? Season { get; init; }

        public GiveCardFromHarvestPile(Player player, Player recipient, Season? season = null)
        {
            Actor = player;
            Recipient = recipient;
            Season = season;
        }

        public void Perform(GameState gameState)
        {
            throw new NotImplementedException();
        }

        public void Unwind(GameState gameState)
        {
            throw new NotImplementedException();
        }
    }
}
