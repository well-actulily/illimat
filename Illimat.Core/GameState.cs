using Illimat.Core.Actions;
using Illimat.Core.Extensions;
using Illimat.Core.Models;

namespace Illimat.Core
{
    public record class GameState
    {
        public Game Game { get; }
        public List<Player> Players { get; init; }
        public Deck<Card> CardDeck { get; init; }
        public Deck<Luminary> LuminaryDeck { get; } = new Deck<Luminary>(Luminary.AllLuminaries());
        public Field[] Fields = new Field[4] 
        { 
            new Field(Season.Spring), new Field(Season.Summer), new Field(Season.Autumn), new Field(Season.Winter) 
        };
        public List<Okus> IllimatOkuses = new();
        public List<IActor> IllimatLockers { get; } = new List<IActor>();
        public int Dealer = 0;
        public int ActivePlayerIndex = 0;

        public GameState(Game game)
        {
            Game = game;

            Players = GeneratePlayers(game.PlayerCounts, Game.Random);
            CardDeck = GenerateShuffledCardDeck(game.PlayerCounts, Game.Random);

            var initialDeal = new InitialDeal(Players[0]);
            initialDeal.Perform(this);
            Game.PendingActions.Enqueue(new BeginTurn(Players[1 % Players.Count]));
        }

        private static List<Player> GeneratePlayers(Dictionary<PlayerType, int> playerCounts, Random random)
        {
            var players = new List<Player>();

            for (int i = 0; i < playerCounts[PlayerType.Human]; i++)
            {
                players.Add(new Player($"Player {i}", PlayerType.Human));
            }
            for (int i = 0; i < playerCounts[PlayerType.Computer]; i++)
            {
                players.Add(new Player($"Computer {i}", PlayerType.Computer));
            }

            players.Shuffle(random);

            return players;
        }

        private static Deck<Card> GenerateShuffledCardDeck(Dictionary<PlayerType, int> playerCounts, Random random)
        {
            var cardDeck = playerCounts[PlayerType.Human] + playerCounts[PlayerType.Computer] < 4 ?
                new Deck<Card>(Card.GetCards(SuitSet.NoStars)) :
                new Deck<Card>(Card.GetCards(SuitSet.AllSuits));

            cardDeck.Shuffle(random);

            return cardDeck;
        }
    }
}
