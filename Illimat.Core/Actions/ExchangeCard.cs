using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class ExchangeCard : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; }
        public Card Lose { get; }
        public Card Gain { get; }

        public ExchangeCard(IActor actor, Field field, Card lose, Card gain)
        {
            Actor = actor;
            Field = field;
            Lose = lose;
            Gain = gain;
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
