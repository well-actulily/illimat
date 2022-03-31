using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class HarvestCards : IAction
    {
        public IActor Actor { get; init; }
        public Field Field { get; init; }
        public List<Card> Cards { get; init; }
        public List<Pile> Piles { get; init; }

        public HarvestCards(Player player, Card card, Field field, List<Pile> piles)
        {
            Actor = player;
            Cards = new List<Card> { card };
            Field = field;
            Piles = piles;
        }

        public HarvestCards(Player player, List<Card> cards, Field field, List<Pile> piles)
        {
            Actor = player;
            Cards = cards;
            Field = field;
            Piles = piles;
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
