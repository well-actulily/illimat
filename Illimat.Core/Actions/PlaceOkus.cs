using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class PlaceOkus : IAction
    {
        public IActor Actor { get; }

        public PlaceOkus(Player player)
        {
            Actor = player;
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
