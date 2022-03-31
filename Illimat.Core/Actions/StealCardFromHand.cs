using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class StealCardFromHand : IAction
    {
        public IActor Actor { get; }
        public Player Victim { get; }
        private Card? Card { get; set; }

        public StealCardFromHand(Player player, Player victim, Card? card = null)
        {
            Actor = player;
            Victim = victim;
            Card = card;
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
