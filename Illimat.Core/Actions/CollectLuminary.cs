using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class CollectLuminary : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; init; }

        public CollectLuminary(Player player, Field field)
        {
            Actor = player;
            Field = field;
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
