namespace Illimat.Core.Extensions
{
    public static class LuminaryExtensions
    {
        public static bool Equals(this Luminary luminary, Luminary other)
        {
            return luminary.LuminaryName == other.LuminaryName;
        }
    }
}
