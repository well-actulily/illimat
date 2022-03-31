using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class CollectOkus : IAction
    {
        public IActor Actor { get; }
        public Okus Okus { get; init; }

        public CollectOkus(Player player, Okus okus)
        {
            Actor = player;
            Okus = okus;
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
