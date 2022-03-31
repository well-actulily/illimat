using Illimat.Core.Extensions;
using Illimat.Core.Models;

namespace Illimat.Core
{
    public record class Field
    {
        public List<Pile> Piles { get; init; } = new List<Pile>();
        public Luminary? Luminary { get; set; }
        public Season Season { get; set; }
        public bool IgnoreSeason { get; set; } = false;
        public bool IgnoreField { get; set; } = false;
        public int HarvestCardMax { get; set; } = 1;
        public Dictionary<int, IList<IList<Pile>>> PileSetsByValue { get; } = new();

        public Field(Season season)
        {
            Season = season;
        }

        public void AddPile(Pile pile)
        {
            Piles.Add(pile);
            
            foreach(var pileSets in PileSetsByValue)
            {
                foreach(var pileSet in pileSets.Value)
                {
                    var newPileSet = pileSet.Select(x => x).ToList();
                    newPileSet.Add(pile);
                    foreach(var value in pile.Values)
                    {
                        var pileSetValue = pileSets.Key + value;
                        PileSetsByValue[pileSetValue] ??= new List<IList<Pile>>();
                        PileSetsByValue[pileSetValue].Add(newPileSet);
                    }
                }
            }
        }

        public void RemovePile(Pile pile)
        {
            Piles.Remove(pile);
            foreach (var pileSets in PileSetsByValue.Values)
            {
                foreach (var pileSet in pileSets)
                {
                    if (pileSet.Contains(pile)) pileSets.Remove(pileSet);
                }
            }
        }
    }
}
