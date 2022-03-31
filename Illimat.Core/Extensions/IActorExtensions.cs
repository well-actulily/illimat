using Illimat.Core.Models;

namespace Illimat.Core.Extensions
{
    public static class IActorExtensions
    {
        public static string ToString(this IActor actor) => actor.Name;
    }
}
