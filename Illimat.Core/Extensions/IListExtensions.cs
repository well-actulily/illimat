namespace Illimat.Core.Extensions
{
    public static class IListExtensions
    {
        // Implements Fisher-Yates shuffle
        public static void Shuffle<T>(this IList<T> list, Random random)
        {
            int n = list.Count;
            while (n > 1)
            {
                n--;
                var k = random.Next(n + 1);
                (list[n], list[k]) = (list[k], list[n]);
            }
        }

        public static IEnumerable<IList<T>> GetSubsets<T>(this IList<T> list)
        {
            int max = (int)Math.Pow(2, list.Count);

            for (int count = 0; count < max; count++)
            {
                List<T> subset = new();
                uint rs = 0;
                while (rs < list.Count)
                {
                    if ((count & (1u << (int)rs)) > 0)
                    {
                        subset.Add(list[(int)rs]);
                    }
                    rs++;
                }
                yield return subset;
            }
        }
    }
}
