using Illimat.Core.Models;
using Illimat.Core.Extensions;

namespace Illimat.Core
{
    public record class Pile
    {
        public IList<Card> Cards { get; init; }
        public IList<int> Values { get; init; }

        public Pile(IList<Card> cards)
        {
            Cards = cards;
            Values = GetValues();
        }


        public IList<int> GetValues()
        {
            var result = new List<int>();

            var othersValues = Cards
                .Where(c => c.Rank != Rank.Fool)
                .Select(c => (int)c.Rank);
            var othersSum = othersValues.Sum();
            var foolsCount = Cards.Count - othersValues.Count();

            for (int i = 0; i < foolsCount * 13; i += 13)
            {
                result.Add(othersSum + foolsCount + i);
            }

            return result;
        }

        public static Dictionary<int, IList<IList<Pile>>> GetPilesSetsValues(IList<Pile> piles)
        {
            var values = new Dictionary<int, IList<IList<Pile>>>();
            var subsets = piles.GetSubsets();

            foreach(var subset in subsets)
            {
                var subsetValues = subset
                    .Select(x => x.Values)
                    .Cartesian()
                    .Select(x => x.Sum());

                foreach(var subsetValue in subsetValues)
                {
                    if (!values.TryGetValue(subsetValue, out IList<IList<Pile>>? pileSets)) 
                        pileSets = new List<IList<Pile>>();
                    pileSets.Add(subset);
                    values[subsetValue] = pileSets;
                }
            }

            return values;
        }
    }
}
