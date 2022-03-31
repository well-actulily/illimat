using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class StealOkus : IAction
    {
        public IActor Actor { get; }
        public Player Victim { get; }
        private Okus? Okus { get; set; }

        public StealOkus(Player player, Player victim)
        {
            Actor = player;
            Victim = victim;
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
