namespace Illimat.Core
{
    public record class Okus
    {
        public Player Owner { get; init; }
        public string Description { get; init; }

        public Okus(Player owner, string description)
        {
            Owner = owner;
            Description = description;
        }
    }
}
