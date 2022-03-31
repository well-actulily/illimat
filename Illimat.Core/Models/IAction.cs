namespace Illimat.Core.Models
{
    public interface IAction
    {
        public IActor Actor { get; }

        public void Perform(GameState gameState);

        public void Unwind(GameState gameState);
    }
}
