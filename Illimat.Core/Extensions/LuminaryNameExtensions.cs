using Illimat.Core.Models;

namespace Illimat.Core.Extensions
{
    public static class LuminaryNameExtensions
    {
        public static string ToFriendlyString(this LuminaryName luminaryName) => luminaryName switch
        {
            LuminaryName.TheMaiden => "The Maiden",
            LuminaryName.TheChangeling => "The Changeling",
            LuminaryName.TheRiver => "The River",
            LuminaryName.TheChildren => "The Children",
            LuminaryName.TheForestQueen => "The Forest Queen",
            LuminaryName.TheRake => "The Rake",
            LuminaryName.TheUnion => "The Union",
            LuminaryName.TheNewborn => "The Newborn",
            LuminaryName.TheLoom => "The Loom",
            LuminaryName.TheIsland => "The Island",
            LuminaryName.ThePerfectCrime => "The Perfect Crime",
            LuminaryName.TheButcher => "The Butcher",
            LuminaryName.TheSoldiers => "The Soldiers",
            LuminaryName.TheBoat => "The Boat",
            LuminaryName.TheAudience => "The Audience",
            LuminaryName.TheRusalka => "The Rusalka",
            _ => throw new ArgumentException($"LuminaryName '{(int)luminaryName}' is undefined.")
        };
    }
}
