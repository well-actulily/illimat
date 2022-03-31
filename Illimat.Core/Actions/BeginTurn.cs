using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class BeginTurn : IAction
    {
        public IActor Actor { get; }

        public BeginTurn(Player player)
        {
            Actor = player;
        }

        public void Perform(GameState gameState)
        {
            gameState.ActivePlayerIndex = gameState.Players.IndexOf((Player)Actor);
            // Console.WriteLine($"Begin {Actor}'s turn.");
        }

        public void Unwind(GameState gameState)
        {
            gameState.ActivePlayerIndex = (gameState.Players.IndexOf((Player)Actor) - 1) % gameState.Players.Count;
        }
    }
}
