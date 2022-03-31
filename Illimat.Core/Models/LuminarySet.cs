namespace Illimat.Core.Models
{
    public static class LuminarySet
    {
        public static readonly IReadOnlyCollection<LuminaryName> BaseLuminaries = new List<LuminaryName>
        {
            LuminaryName.TheMaiden,
            LuminaryName.TheChangeling,
            LuminaryName.TheRiver,
            LuminaryName.TheChildren,
            LuminaryName.TheForestQueen,
            LuminaryName.TheRake,
            LuminaryName.TheUnion,
            LuminaryName.TheNewborn
        };

        public static readonly IReadOnlyCollection<LuminaryName> TheCraneWifeLuminaries = new List<LuminaryName>
        {
            LuminaryName.TheLoom,
            LuminaryName.TheIsland,
            LuminaryName.ThePerfectCrime,
            LuminaryName.TheButcher,
            LuminaryName.TheSoldiers,
            LuminaryName.TheBoat
        };

        public static readonly IReadOnlyCollection<LuminaryName> PromoLuminaries = new List<LuminaryName>
        {
            LuminaryName.TheAudience,
            LuminaryName.TheRusalka
        };

        public static readonly IReadOnlyCollection<IEnumerable<LuminaryName>> AllLuminarySets =
            new List<IReadOnlyCollection<LuminaryName>> { BaseLuminaries, TheCraneWifeLuminaries, PromoLuminaries };

        public static readonly IReadOnlyCollection<LuminaryName> AllLuminaries =
            AllLuminarySets.Aggregate((acc, list) => { return acc.Concat(list); }).ToList();
    }
}
