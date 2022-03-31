using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class DealLuminary : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; init; }
        public Luminary? Luminary { get; set; }

        public DealLuminary(IActor actor, Field field)
        {
            Actor = actor;
            Field = field;
        }

        public void Perform(GameState gameState)
        {
            Luminary = gameState.LuminaryDeck.DrawUpTo(1).FirstOrDefault();
            Field.Luminary = Luminary;
            Console.WriteLine($"{Actor} (as dealer) {(Field.Luminary != null ? "dealt" : "was unable to deal")} a Luminary to field {Field}.");
        }

        public void Unwind(GameState gameState)
        {
            if (Luminary != null)
            {
                Field.Luminary = null;
                gameState.LuminaryDeck.Cards.Insert(0, Luminary);
            }
        }
    }
}
