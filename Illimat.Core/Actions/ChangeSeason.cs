using Illimat.Core.Models;

namespace Illimat.Core.Actions
{
    public class ChangeSeason : IAction
    {
        public IActor Actor { get; }
        public Season Season { get; init; }
        private Season PreviousSeason { get; set; }
        public int FieldIndex { get; init; }
        public bool LockIllimat { get; init; }

        public ChangeSeason(IActor actor, Season season, int fieldIndex, bool lockIllimat = false)
        {
            Actor = actor;
            Season = season;
            FieldIndex = fieldIndex;
            LockIllimat = lockIllimat;
        }

        public void Perform(GameState gameState)
        {
            if (gameState.IllimatLockers.Count == 0)
            {
                PreviousSeason = gameState.Fields[FieldIndex].Season;
                var seasonsAreAlreadyAligned = gameState.Fields[FieldIndex].Season == Season;

                if (!seasonsAreAlreadyAligned)
                {
                    AlignSeasons(gameState, Season);
                }

                // Console.WriteLine($"The Illimat {(seasonsAreAlreadyAligned ? "is already" : "has been")} aligned such that field {FieldIndex} is in {Season}.");
            }

            if (LockIllimat)
            {
                gameState.IllimatLockers.Add(Actor);
                // Console.WriteLine($"Seasons are locked while {Actor} is active.");
            }
            return;

            // Console.WriteLine($"Seasons may not be changed while {gameState.IllimatLocker} is active.");
        }

        public void Unwind(GameState gameState)
        {
            if (LockIllimat)
            {
                gameState.IllimatLockers.Remove(Actor);
            }

            if (gameState.IllimatLockers.Count == 0 && Season != PreviousSeason)
            {
                AlignSeasons(gameState, PreviousSeason);
            }
        }

        public void AlignSeasons(GameState gameState, Season season)
        {
            for (int i = 0; i < 4; i++)
            {
                gameState.Fields[(FieldIndex + i) % 4].Season = (Season)(((int)season + i) % 4);
            }
        }
    }
}
