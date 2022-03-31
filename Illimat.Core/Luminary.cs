using Illimat.Core.Models;
using Illimat.Core.Extensions;

namespace Illimat.Core
{
    public record class Luminary : ICard, IActor
    {
        public LuminaryName LuminaryName { get; init; }
        private bool isRevealed = false;

        public bool IsRevealed { get => isRevealed; set => isRevealed = value; }
        public string Name { get => IsRevealed ? "Unknown" : LuminaryName.ToFriendlyString(); }

        public Luminary(LuminaryName luminaryName)
        {
            LuminaryName = luminaryName;
        }

        public static IList<Luminary> AllLuminaries()
        {
            return LuminarySet.AllLuminaries.Select(luminaryName => new Luminary(luminaryName)).ToList();
        }

        public void Reveal(GameState gameState)
        {
            isRevealed = true;
        }

        public void AddActiveAction(GameState gameState)
        {

        }

        public void Deactivate(GameState gameState)
        {

        }

        public void Unreveal(GameState gameState)
        {

        }

        public void Restore(GameState gameState)
        {

        }
    }
}
