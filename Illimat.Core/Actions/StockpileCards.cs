using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class StockpileCards : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; }
        public List<Card> Cards { get; }

        public StockpileCards(Player player, Field field, List<Card> cards)
        {
            Actor = player;
            Field = field;
            Cards = cards;
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
