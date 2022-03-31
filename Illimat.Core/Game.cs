using Illimat.Core.Models;

namespace Illimat.Core
{
    public class Game : IActor
    {
        public string Name { get; } = "Game";
        public Dictionary<PlayerType, int> PlayerCounts { get; } = new() { { PlayerType.Human, 0 }, { PlayerType.Computer, 0 } };
        public Random Random { get; }
        public GameState? State { get; set; }
        public Queue<IAction> PendingActions = new();
        public Stack<IAction> CompletedActions = new();

        const int DEFAULT_HUMAN_PLAYER_COUNT = 1;
        const int DEFAULT_COMPUTER_PLAYER_COUNT = 3;

        public Game(int humanCount = DEFAULT_HUMAN_PLAYER_COUNT, int computerCount = DEFAULT_COMPUTER_PLAYER_COUNT, int? seed = null)
        {
            PlayerCounts[PlayerType.Human] = humanCount;
            PlayerCounts[PlayerType.Computer] = computerCount;
            seed ??= (int)DateTime.Now.Ticks;
            Random = new Random(seed.Value);
            State = new GameState(this);

            Console.WriteLine($"Created a new game of Illimat with {PlayerCounts[PlayerType.Human]} human and " +
                $"{PlayerCounts[PlayerType.Computer]} computer players. Game seed: {seed}.");
        }

        public static void Main() {}
    }
}
