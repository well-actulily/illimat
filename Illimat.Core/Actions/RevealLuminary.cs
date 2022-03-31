using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class RevealLuminary : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; }

        public RevealLuminary(IActor actor, Field field)
        {
            Actor = actor;
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
