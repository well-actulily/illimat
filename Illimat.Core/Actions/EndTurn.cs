using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class EndTurn : IAction
    {
        public IActor Actor { get; }

        public EndTurn(Player player)
        {
            Actor = player;
        }

        public void Perform(GameState gameState)
        {
            Console.WriteLine($"{Actor} has ended their turn.");
            Player? nextPlayer;

            for (int i = 1; i < gameState.Players.Count - 1; i++)
            {
                var nextPlayerIndex = (gameState.ActivePlayerIndex + i) % gameState.Players.Count;
                nextPlayer = gameState.Players[nextPlayerIndex];

                if (nextPlayer.Hand.Count > 0) {
                    gameState.Game.PendingActions.Enqueue(new BeginTurn(gameState.Players[nextPlayerIndex]));
                    break;
                }

                Console.WriteLine($"{Actor} has no cards remaining in their hand, and cannot take a turn.");
            }

            Console.WriteLine($"There are no players with cards remaining in their hand. Entering round scoring phase.");
            gameState.Game.PendingActions.Enqueue(new ScoreRound(gameState.Game));
        }

        public void Unwind(GameState gameState)
        {
            throw new NotImplementedException();
        }
    }
}
