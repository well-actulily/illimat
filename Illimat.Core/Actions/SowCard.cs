using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class SowCards : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; }
        public List<Card> Cards { get; }

        public SowCards(Player player, List<Card> cards, Field field)
        {
            Actor = player;
            Cards = cards;
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
