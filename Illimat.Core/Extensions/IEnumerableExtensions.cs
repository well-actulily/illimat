using System.Collections;

namespace Illimat.Core.Extensions
{
    public static class IEnumerableExtensions
    {
        public static IEnumerable<IEnumerable<T>> Cartesian<T>(this IEnumerable<IEnumerable<T>> items)
        {
            var slots = items
               .Select(x => x.GetEnumerator())
               .Where(x => x.MoveNext())
               .ToArray();

            while (true)
            {
                yield return slots.Select(x => x.Current);

                foreach (var slot in slots)
                {
                    if (!slot.MoveNext())
                    {
                        if (slot == slots.Last()) { yield break; }
                        slot.Reset();
                        slot.MoveNext();
                        continue;
                    }
                    break;
                }
            }
        }
    }
}
