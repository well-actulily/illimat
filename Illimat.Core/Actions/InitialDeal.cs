using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class InitialDeal : IAction
    {
        public IActor Actor { get; }

        public InitialDeal(Player dealer)
        {
            Actor = dealer;
        }

        public void Perform(GameState gameState)
        {
            AddSeedFieldActions(gameState);
            AddDealHandActions(gameState);
            AddPlaceOkusActions(gameState);
            AddDealLuminaryActions(gameState);
        }

        public void Unwind(GameState gameState)
        {
            throw new NotImplementedException();
        }

        private void AddSeedFieldActions(GameState gameState)
        {
            foreach (Field field in gameState.Fields)
            {
                gameState.Game.PendingActions.Enqueue(new SeedField(Actor, field));
            }
        }

        private void AddDealHandActions(GameState gameState)
        {
            var dealer = (Player)Actor;
            if (gameState.Players.Count >= 2) gameState.Game.PendingActions.Enqueue(new DealHand(dealer, gameState.Players[1], 3));
            if (gameState.Players.Count >= 3) gameState.Game.PendingActions.Enqueue(new DealHand(dealer, gameState.Players[2], 4));
            if (gameState.Players.Count == 4) gameState.Game.PendingActions.Enqueue(new DealHand(dealer, gameState.Players[3], 4));
            gameState.Game.PendingActions.Enqueue(new DealHand(dealer, gameState.Players[0], 4));
        }

        private static void AddPlaceOkusActions(GameState gameState)
        {
            foreach (Player player in gameState.Players)
            {
                gameState.Game.PendingActions.Enqueue(new PlaceOkus(player));
            }
        }

        private void AddDealLuminaryActions(GameState gameState)
        {
            foreach(Field field in gameState.Fields)
            {
                gameState.Game.PendingActions.Enqueue(new DealLuminary(Actor, field));
            }
        }
    }
}
