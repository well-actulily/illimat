using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class ScoreRound : IAction
    {
        public IActor Actor { get; }

        public ScoreRound(Game game)
        {
            Actor = game;
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
