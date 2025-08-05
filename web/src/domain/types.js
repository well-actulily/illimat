/**
 * Core domain types for Illimat game representation
 * These types define the shape of data flowing between WASM backend and Vue frontend
 */

/**
 * Represents a single playing card in the Illimat deck
 */
export class Card {
  constructor({ id, rank, suit, value, displayValue = null }) {
    this.id = id                    // Unique identifier for this card instance
    this.rank = rank                // "F", "2"-"10", "N" (Knight), "Q", "K"  
    this.suit = suit                // "Spring", "Summer", "Autumn", "Winter", "Stars"
    this.value = value              // Base numeric value 1-14
    this.displayValue = displayValue // For Fool cards: which value (1 or 14) player chose
  }

  /**
   * Get the effective game value (handles Fool ambiguity)
   */
  getGameValue() {
    return this.displayValue ?? this.value
  }

  /**
   * Check if this is a face card (affects season rotation)
   */
  isFaceCard() {
    return ['F', 'N', 'Q', 'K'].includes(this.rank)
  }

  /**
   * Get Unicode symbol for the suit
   */
  getSuitSymbol() {
    const symbols = {
      'Spring': '🜁',
      'Summer': '☼', 
      'Autumn': '🍂',
      'Winter': '❄️',
      'Stars': '✦'
    }
    return symbols[this.suit] || '?'
  }

  /**
   * Get season associated with this card's suit
   */
  getSeason() {
    const seasons = {
      'Spring': 'Spring',
      'Summer': 'Summer', 
      'Autumn': 'Autumn',
      'Winter': 'Winter',
      'Stars': null // Stars can be any season
    }
    return seasons[this.suit]
  }

  /**
   * Get display text for the rank
   */
  getRankDisplay() {
    const displays = {
      'F': 'Fool',
      'N': 'Knight', 
      'Q': 'Queen',
      'K': 'King'
    }
    return displays[this.rank] || this.rank
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      id: this.id,
      rank: this.rank,
      suit: this.suit,
      value: this.value,
      display_value: this.displayValue
    }
  }

  /**
   * Create Card from WASM JSON
   */
  static fromJSON(json) {
    return new Card({
      id: json.id,
      rank: json.rank,
      suit: json.suit,
      value: json.value,
      displayValue: json.display_value
    })
  }
}

/**
 * Represents a stockpile of cards in a field
 * Cards grouped together that must be harvested as a unit
 */
export class Stockpile {
  constructor({ cards, totalValue, createdTurn, canHarvest = true }) {
    this.cards = cards.map(c => c instanceof Card ? c : Card.fromJSON(c))
    this.totalValue = totalValue    // Combined value of all cards
    this.createdTurn = createdTurn  // Turn when stockpile was created
    this.canHarvest = canHarvest    // False if created this turn
  }

  /**
   * Get all card IDs in this stockpile
   */
  getCardIds() {
    return this.cards.map(card => card.id)
  }

  /**
   * Check if stockpile contains a specific card
   */
  containsCard(cardId) {
    return this.cards.some(card => card.id === cardId)
  }

  /**
   * Get display summary of stockpile contents
   */
  getDisplaySummary() {
    const cardCount = this.cards.length
    return `${cardCount} cards (${this.totalValue})`
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      cards: this.cards.map(card => card.toJSON()),
      total_value: this.totalValue,
      created_turn: this.createdTurn,
      can_harvest: this.canHarvest
    }
  }

  /**
   * Create Stockpile from WASM JSON
   */
  static fromJSON(json) {
    return new Stockpile({
      cards: json.cards.map(Card.fromJSON),
      totalValue: json.total_value,
      createdTurn: json.created_turn,
      canHarvest: json.can_harvest
    })
  }
}

/**
 * Represents a Luminary card with its game effects
 */
export class LuminaryCard {
  constructor({ name, expansion, description, revealEffect = null, claimEffect = null, ongoingEffect = null }) {
    this.name = name                // "The Maiden", "The Changeling", etc.
    this.expansion = expansion      // "core", "false_baron", "crane_wife", "other"
    this.description = description  // Full rules text
    this.revealEffect = revealEffect // Effect when first revealed
    this.claimEffect = claimEffect   // Effect when claimed by player
    this.ongoingEffect = ongoingEffect // Ongoing effect while active
  }

  /**
   * Get short name for display
   */
  getShortName() {
    return this.name.replace('The ', '')
  }

  /**
   * Check if this Luminary has ongoing effects
   */
  hasOngoingEffect() {
    return this.ongoingEffect !== null
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      name: this.name,
      expansion: this.expansion,
      description: this.description,
      reveal_effect: this.revealEffect,
      claim_effect: this.claimEffect,
      ongoing_effect: this.ongoingEffect
    }
  }

  /**
   * Create LuminaryCard from WASM JSON
   */
  static fromJSON(json) {
    return new LuminaryCard({
      name: json.name,
      expansion: json.expansion,
      description: json.description,
      revealEffect: json.reveal_effect,
      claimEffect: json.claim_effect,
      ongoingEffect: json.ongoing_effect
    })
  }
}

/**
 * Represents the state of a Luminary in the game
 */
export class LuminaryState {
  constructor({ card, isFaceUp, owner = null, effectsActive = false }) {
    this.card = card instanceof LuminaryCard ? card : LuminaryCard.fromJSON(card)
    this.isFaceUp = isFaceUp        // Whether Luminary is revealed
    this.owner = owner              // Player ID who claimed it, null if unclaimed
    this.effectsActive = effectsActive // Whether effects are currently active
  }

  /**
   * Check if Luminary is claimed by a player
   */
  isClaimed() {
    return this.owner !== null
  }

  /**
   * Check if Luminary is available to be claimed
   */
  isAvailableForClaim() {
    return this.isFaceUp && !this.isClaimed()
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      card: this.card.toJSON(),
      is_face_up: this.isFaceUp,
      owner: this.owner,
      effects_active: this.effectsActive
    }
  }

  /**
   * Create LuminaryState from WASM JSON
   */
  static fromJSON(json) {
    return new LuminaryState({
      card: LuminaryCard.fromJSON(json.card),
      isFaceUp: json.is_face_up,
      owner: json.owner,
      effectsActive: json.effects_active
    })
  }
}

/**
 * Represents a game field with its contents and state
 */
export class GameField {
  constructor({ 
    looseCards = [], 
    stockpiles = [], 
    season = 'Winter', 
    seasonRestrictions = [], 
    hasOkus = false, 
    luminary = null 
  }) {
    this.looseCards = looseCards.map(c => c instanceof Card ? c : Card.fromJSON(c))
    this.stockpiles = stockpiles.map(s => s instanceof Stockpile ? s : Stockpile.fromJSON(s))
    this.season = season              // "Winter", "Spring", "Summer", "Autumn"
    this.seasonRestrictions = seasonRestrictions // ["no_harvesting", "no_stockpiling", "no_sowing"]
    this.hasOkus = hasOkus           // Whether field has okus token
    this.luminary = luminary ? (luminary instanceof LuminaryState ? luminary : LuminaryState.fromJSON(luminary)) : null
  }

  /**
   * Get all cards in field (loose + stockpiled)
   */
  getAllCards() {
    const stockpiledCards = this.stockpiles.flatMap(s => s.cards)
    return [...this.looseCards, ...stockpiledCards]
  }

  /**
   * Get total card count in field
   */
  getCardCount() {
    return this.looseCards.length + this.stockpiles.reduce((sum, s) => sum + s.cards.length, 0)
  }

  /**
   * Check if field allows specific action
   */
  allowsAction(action) {
    return !this.seasonRestrictions.includes(`no_${action}`)
  }

  /**
   * Check if field is empty
   */
  isEmpty() {
    return this.looseCards.length === 0 && this.stockpiles.length === 0
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      loose_cards: this.looseCards.map(card => card.toJSON()),
      stockpiles: this.stockpiles.map(stockpile => stockpile.toJSON()),
      season: this.season,
      season_restrictions: this.seasonRestrictions,
      has_okus: this.hasOkus,
      luminary: this.luminary?.toJSON() || null
    }
  }

  /**
   * Create GameField from WASM JSON
   */
  static fromJSON(json) {
    return new GameField({
      looseCards: json.loose_cards?.map(Card.fromJSON) || [],
      stockpiles: json.stockpiles?.map(Stockpile.fromJSON) || [],
      season: json.season,
      seasonRestrictions: json.season_restrictions || [],
      hasOkus: json.has_okus,
      luminary: json.luminary ? LuminaryState.fromJSON(json.luminary) : null
    })
  }
}

/**
 * Represents a player and their game state
 */
export class Player {
  constructor({ 
    id, 
    name = `Player ${id + 1}`, 
    hand = [], 
    harvest = [], 
    score = 0, 
    isAI = false,
    totalScore = 0
  }) {
    this.id = id
    this.name = name
    this.hand = hand.map(c => c instanceof Card ? c : Card.fromJSON(c))
    this.harvest = harvest.map(c => c instanceof Card ? c : Card.fromJSON(c))
    this.score = score                // Current round score
    this.isAI = isAI
    this.totalScore = totalScore      // Cumulative score across rounds
  }

  /**
   * Get hand size
   */
  getHandSize() {
    return this.hand.length
  }

  /**
   * Get harvest statistics for scoring
   */
  getHarvestStats() {
    const totalCards = this.harvest.length
    const summerCards = this.harvest.filter(c => c.suit === 'Summer').length
    const winterCards = this.harvest.filter(c => c.suit === 'Winter').length
    const fools = this.harvest.filter(c => c.rank === 'F').length
    const luminaries = 0 // TODO: Count claimed luminaries
    const okuses = 0     // TODO: Count collected okuses

    return {
      totalCards,
      summerCards, 
      winterCards,
      fools,
      luminaries,
      okuses
    }
  }

  /**
   * Check if player has cards in hand
   */
  hasCards() {
    return this.hand.length > 0
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      id: this.id,
      name: this.name,
      hand: this.hand.map(card => card.toJSON()),
      harvest: this.harvest.map(card => card.toJSON()),
      score: this.score,
      is_ai: this.isAI,
      total_score: this.totalScore
    }
  }

  /**
   * Create Player from WASM JSON
   */
  static fromJSON(json) {
    return new Player({
      id: json.id,
      name: json.name,
      hand: json.hand?.map(Card.fromJSON) || [],
      harvest: json.harvest?.map(Card.fromJSON) || [],
      score: json.score,
      isAI: json.is_ai,
      totalScore: json.total_score || 0
    })
  }
}

/**
 * Represents different types of moves players can make
 */
export class Move {
  constructor({ type, playerId, fieldId, playedCard, targetCards = [], foolValue = null }) {
    this.type = type                // "sow", "harvest", "stockpile"
    this.playerId = playerId
    this.fieldId = fieldId          // Which field the move targets
    this.playedCard = playedCard instanceof Card ? playedCard : Card.fromJSON(playedCard)
    this.targetCards = targetCards.map(c => c instanceof Card ? c : Card.fromJSON(c))
    this.foolValue = foolValue      // If playing Fool, which value (1 or 14)
  }

  /**
   * Get human-readable description of move
   */
  getDescription() {
    const card = this.playedCard
    const cardDesc = `${card.getRankDisplay()} of ${card.suit}`
    
    switch (this.type) {
      case 'sow':
        return `Sow ${cardDesc} to Field ${this.fieldId}`
      case 'harvest':
        const targetDesc = this.targetCards.length > 1 
          ? `${this.targetCards.length} cards` 
          : `${this.targetCards[0]?.getRankDisplay()} of ${this.targetCards[0]?.suit}`
        return `Harvest ${targetDesc} with ${cardDesc}`
      case 'stockpile':
        return `Stockpile ${cardDesc} with ${this.targetCards.length} cards`
      default:
        return `Unknown move: ${this.type}`
    }
  }

  /**
   * Check if move changes season (face card played)
   */
  changesSeason() {
    return this.playedCard.isFaceCard()
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      type: this.type,
      player_id: this.playerId,
      field_id: this.fieldId,
      played_card: this.playedCard.toJSON(),
      target_cards: this.targetCards.map(card => card.toJSON()),
      fool_value: this.foolValue
    }
  }

  /**
   * Create Move from WASM JSON
   */
  static fromJSON(json) {
    return new Move({
      type: json.type,
      playerId: json.player_id,
      fieldId: json.field_id,
      playedCard: Card.fromJSON(json.played_card),
      targetCards: json.target_cards?.map(Card.fromJSON) || [],
      foolValue: json.fool_value
    })
  }
}

/**
 * Represents the complete state of an Illimat game
 */
export class GameState {
  constructor({
    config = {},
    phase = 'setup',
    currentPlayer = 0,
    dealer = 0,
    roundNumber = 1,
    turnNumber = 1,
    totalScores = [],
    fields = [],
    players = [],
    deckRemaining = 65,
    illimatOrientation = 0,
    okusPositions = [false, false, false, false],
    okusOnIllimat = 4
  }) {
    this.config = config
    this.phase = phase              // "setup", "playing", "scoring", "finished"
    this.currentPlayer = currentPlayer
    this.dealer = dealer
    this.roundNumber = roundNumber
    this.turnNumber = turnNumber
    this.totalScores = [...totalScores]
    this.fields = fields.map(f => f instanceof GameField ? f : GameField.fromJSON(f))
    this.players = players.map(p => p instanceof Player ? p : Player.fromJSON(p))
    this.deckRemaining = deckRemaining
    this.illimatOrientation = illimatOrientation // 0-3, maps to field seasons
    this.okusPositions = [...okusPositions]
    this.okusOnIllimat = okusOnIllimat
  }

  /**
   * Get current player object
   */
  getCurrentPlayer() {
    return this.players[this.currentPlayer]
  }

  /**
   * Get season for specific field based on Illimat orientation
   */
  getFieldSeason(fieldIndex) {
    const seasons = ['Winter', 'Spring', 'Summer', 'Autumn']
    return seasons[(fieldIndex + this.illimatOrientation) % 4]
  }

  /**
   * Check if game is over
   */
  isGameOver() {
    return this.phase === 'finished' || this.totalScores.some(score => score >= 17)
  }

  /**
   * Check if round is over (deck exhausted)
   */
  isRoundOver() {
    return this.deckRemaining === 0 && this.players.every(p => p.hand.length === 0)
  }

  /**
   * Get player who should go next
   */
  getNextPlayer() {
    return (this.currentPlayer + 1) % this.players.length
  }

  /**
   * Serialize for WASM interface
   */
  toJSON() {
    return {
      config: this.config,
      phase: this.phase,
      current_player: this.currentPlayer,
      dealer: this.dealer,
      round_number: this.roundNumber,
      turn_number: this.turnNumber,
      total_scores: this.totalScores,
      fields: this.fields.map(field => field.toJSON()),
      players: this.players.map(player => player.toJSON()),
      deck_remaining: this.deckRemaining,
      illimat_orientation: this.illimatOrientation,
      okus_positions: this.okusPositions,
      okus_on_illimat: this.okusOnIllimat
    }
  }

  /**
   * Create GameState from WASM JSON
   */
  static fromJSON(json) {
    return new GameState({
      config: json.config || {},
      phase: json.phase,
      currentPlayer: json.current_player,
      dealer: json.dealer,
      roundNumber: json.round_number,
      turnNumber: json.turn_number,
      totalScores: json.total_scores || [],
      fields: json.fields?.map(GameField.fromJSON) || [],
      players: json.players?.map(Player.fromJSON) || [],
      deckRemaining: json.deck_remaining,
      illimatOrientation: json.illimat_orientation,
      okusPositions: json.okus_positions || [false, false, false, false],
      okusOnIllimat: json.okus_on_illimat || 4
    })
  }
}

/**
 * Utility functions for working with domain types
 */
export const DomainUtils = {
  /**
   * Create a standard 65-card Illimat deck
   */
  createStandardDeck() {
    const suits = ['Spring', 'Summer', 'Autumn', 'Winter', 'Stars']
    const ranks = ['F', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'N', 'Q', 'K']
    const cards = []
    
    let id = 0
    for (const suit of suits) {
      for (const rank of ranks) {
        let value
        if (rank === 'F') value = 1      // Fool can be 1 or 14
        else if (rank === 'N') value = 11 // Knight
        else if (rank === 'Q') value = 12 // Queen  
        else if (rank === 'K') value = 13 // King
        else value = parseInt(rank)       // Number cards
        
        cards.push(new Card({ id: id++, rank, suit, value }))
      }
    }
    
    return cards
  },

  /**
   * Get all core Luminary cards
   */
  getCoreLuminaries() {
    return [
      new LuminaryCard({
        name: 'The Maiden',
        expansion: 'core',
        description: 'While on board, Winter has no effect: cards may be harvested from Winter field.',
        ongoingEffect: 'Disables Winter restrictions'
      }),
      new LuminaryCard({
        name: 'The Changeling', 
        expansion: 'core',
        description: 'Once per turn, exchange a card from hand with card in same field. When claimed, exchange two cards from hand for any two on board.',
        ongoingEffect: 'Card exchange ability',
        claimEffect: 'Exchange two cards'
      }),
      // ... other core luminaries would be defined here
    ]
  }
}