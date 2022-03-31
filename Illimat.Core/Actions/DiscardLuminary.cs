using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class DiscardLuminary : IAction
    {
        public IActor Actor { get; }
        public Field Field { get; init; }
        public Luminary? DiscardedLuminary { get; set; }

        public DiscardLuminary(IActor actor, Field field)
        {
            Actor = actor;
            Field = field;
        }

        public void Perform(GameState gameState)
        {
            var luminary = Field.Luminary;

            if (luminary != null)
            {
                luminary.Deactivate(gameState);
                Field.Luminary = null;
                Console.WriteLine($"{Actor} discarded Luminary {luminary} from field {Field}. Any ongoing effects from {luminary} are no longer active.");
                return;
            }

            DiscardedLuminary = luminary;
            Console.WriteLine($"There was no Luminary in field {Field} to discard.");
        }

        public void Unwind(GameState gameState)
        {
            if (DiscardedLuminary != null)
            {
                Field.Luminary = DiscardedLuminary;
                Field.Luminary.Restore(gameState);
            }
        }
    }
}
