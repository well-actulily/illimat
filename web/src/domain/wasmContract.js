/**
 * WASM Interface Contract
 * Defines the expected API between Rust backend and JavaScript frontend
 */

import { GameState, Move, Card, Player, GameField } from './types.js'

/**
 * Contract for the WASM Game Engine interface
 * This defines exactly what methods the Rust backend must expose
 */
export class WasmGameEngineContract {
  /**
   * Initialize a new game with specified configuration
   * @param {Object} config - Game configuration
   * @param {number} config.player_count - Number of players (2-4)
   * @param {boolean} config.use_stars_suit - Include Stars suit (false for 2-3 players)
   * @param {boolean} config.enable_luminaries - Include Luminary cards
   * @param {string[]} config.luminary_expansions - Which Luminary sets to include
   * @param {boolean} config.beginner_mode - Simplified rules for beginners
   * @returns {string} JSON-serialized GameState
   */
  initialize_game(config) {
    throw new Error('WASM method not implemented: initialize_game')
  }

  /**
   * Get the current complete game state
   * @returns {string} JSON-serialized GameState
   */
  get_game_state() {
    throw new Error('WASM method not implemented: get_game_state')
  }

  /**
   * Get all legal moves for a specific player in current game state
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - Player requesting moves
   * @returns {string} JSON array of Move objects
   */
  get_legal_moves(gameStateJson, playerId) {
    throw new Error('WASM method not implemented: get_legal_moves')
  }

  /**
   * Validate a proposed move without applying it
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - Player making the move
   * @param {string} moveJson - Proposed move as JSON
   * @returns {string} JSON validation result with success/error info
   */
  validate_move(gameStateJson, playerId, moveJson) {
    throw new Error('WASM method not implemented: validate_move')
  }

  /**
   * Apply a move to the game state and return updated state
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - Player making the move
   * @param {string} moveJson - Move to apply as JSON
   * @returns {string} JSON-serialized updated GameState
   */
  apply_move(gameStateJson, playerId, moveJson) {
    throw new Error('WASM method not implemented: apply_move')
  }

  /**
   * Get best move for AI player using MCTS or other algorithm
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - AI player ID
   * @param {number} timeLimitMs - Time limit for computation in milliseconds
   * @returns {string} JSON-serialized Move representing best choice
   */
  get_best_move(gameStateJson, playerId, timeLimitMs) {
    throw new Error('WASM method not implemented: get_best_move')
  }

  /**
   * Evaluate win probability for a specific move
   * @param {string} gameStateJson - Current game state as JSON
   * @param {string} moveJson - Move to evaluate as JSON
   * @returns {number} Win probability between 0.0 and 1.0
   */
  get_move_evaluation(gameStateJson, moveJson) {
    throw new Error('WASM method not implemented: get_move_evaluation')
  }

  /**
   * Check if current round can end (all cards played)
   * @param {string} gameStateJson - Current game state as JSON
   * @returns {boolean} True if round can end
   */
  can_end_round(gameStateJson) {
    throw new Error('WASM method not implemented: can_end_round')
  }

  /**
   * Calculate round scoring and update player scores
   * @param {string} gameStateJson - Current game state as JSON
   * @returns {string} JSON-serialized updated GameState with scores
   */
  calculate_round_scoring(gameStateJson) {
    throw new Error('WASM method not implemented: calculate_round_scoring')
  }

  /**
   * Get detailed scoring breakdown for current round
   * @param {string} gameStateJson - Current game state as JSON
   * @returns {string} JSON object with scoring details per player
   */
  get_scoring_breakdown(gameStateJson) {
    throw new Error('WASM method not implemented: get_scoring_breakdown')
  }

  /**
   * Get season and restrictions for a specific field
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} fieldId - Field index (0-3)
   * @returns {string} JSON object with season and allowed actions
   */
  get_field_info(gameStateJson, fieldId) {
    throw new Error('WASM method not implemented: get_field_info')
  }

  /**
   * Get all active Luminary effects in current game
   * @param {string} gameStateJson - Current game state as JSON
   * @returns {string} JSON array of active Luminary effects
   */
  get_active_luminary_effects(gameStateJson) {
    throw new Error('WASM method not implemented: get_active_luminary_effects')
  }

  /**
   * Check if player can use a specific Luminary ability
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - Player ID
   * @param {string} luminaryName - Name of Luminary ability
   * @returns {boolean} True if ability can be used
   */
  can_use_luminary_ability(gameStateJson, playerId, luminaryName) {
    throw new Error('WASM method not implemented: can_use_luminary_ability')
  }

  /**
   * Use a Luminary ability and return updated game state
   * @param {string} gameStateJson - Current game state as JSON
   * @param {number} playerId - Player using ability
   * @param {string} luminaryName - Name of Luminary ability
   * @param {string} parametersJson - Ability parameters as JSON
   * @returns {string} JSON-serialized updated GameState
   */
  use_luminary_ability(gameStateJson, playerId, luminaryName, parametersJson) {
    throw new Error('WASM method not implemented: use_luminary_ability')
  }
}

/**
 * Expected JSON structure for GameState from WASM
 * This serves as documentation and validation for the WASM interface
 */
export const WASM_GAME_STATE_SCHEMA = {
  // Game metadata
  config: {
    player_count: "number",
    use_stars_suit: "boolean", 
    enable_luminaries: "boolean",
    luminary_expansions: ["string"], // ["core", "false_baron", "crane_wife", "other"]
    beginner_mode: "boolean"
  },

  // Game flow
  phase: "string", // "setup" | "playing" | "scoring" | "finished"
  current_player: "number",
  dealer: "number", 
  round_number: "number",
  turn_number: "number", 
  total_scores: ["number"], // Cumulative scores across rounds

  // Game board state
  fields: [{
    loose_cards: [{
      id: "number",
      rank: "string", // "F", "2"-"10", "N", "Q", "K"
      suit: "string", // "Spring", "Summer", "Autumn", "Winter", "Stars" 
      value: "number", // 1-14
      display_value: "number|null" // For Fool cards
    }],
    stockpiles: [{
      cards: ["Card"], // Array of Card objects
      total_value: "number",
      created_turn: "number",
      can_harvest: "boolean"
    }],
    season: "string", // "Winter", "Spring", "Summer", "Autumn"
    season_restrictions: ["string"], // ["no_harvesting", "no_stockpiling", "no_sowing"]
    has_okus: "boolean",
    luminary: {
      card: {
        name: "string",
        expansion: "string",
        description: "string",
        reveal_effect: "string|null",
        claim_effect: "string|null", 
        ongoing_effect: "string|null"
      },
      is_face_up: "boolean",
      owner: "number|null",
      effects_active: "boolean"
    } // "LuminaryState|null"
  }],

  // Player data
  players: [{
    id: "number",
    name: "string",
    hand: ["Card"],
    harvest: ["Card"], 
    score: "number", // Current round score
    is_ai: "boolean",
    total_score: "number" // Cumulative score
  }],

  // Deck state
  deck_remaining: "number",
  deck_cards: ["Card"], // Only in debug mode

  // Physical game state
  illimat_orientation: "number", // 0-3, determines field seasons
  okus_positions: ["boolean"], // [field0_has_okus, field1_has_okus, field2_has_okus, field3_has_okus]
  okus_on_illimat: "number" // Count of okus still on central Illimat
}

/**
 * Expected JSON structure for Move from WASM
 */
export const WASM_MOVE_SCHEMA = {
  type: "string", // "sow" | "harvest" | "stockpile"
  player_id: "number",
  field_id: "number", // 0-3
  played_card: "Card",
  target_cards: ["Card"], // Cards being harvested/stockpiled
  fool_value: "number|null" // If playing Fool, chosen value (1 or 14)
}

/**
 * Expected JSON structure for move validation result
 */
export const WASM_VALIDATION_RESULT_SCHEMA = {
  is_valid: "boolean",
  error_message: "string|null",
  warnings: ["string"], // Non-fatal issues
  move_effects: {
    cards_gained: ["Card"],
    cards_lost: ["Card"], 
    season_change: "string|null", // New season if face card played
    field_cleared: "boolean",
    okus_gained: "boolean",
    luminary_revealed: "string|null",
    luminary_claimed: "string|null"
  }
}

/**
 * Expected JSON structure for scoring breakdown
 */
export const WASM_SCORING_BREAKDOWN_SCHEMA = {
  players: [{
    player_id: "number",
    harvest_stats: {
      total_cards: "number",
      summer_cards: "number", 
      winter_cards: "number",
      fool_cards: "number",
      luminary_cards: "number",
      okus_tokens: "number"
    },
    round_scores: {
      bumper_crop: "number", // +4, 0, or tied
      sunkissed: "number",   // +2, 0, or tied
      frostbit: "number",    // -2, 0, or tied
      fools: "number",       // +1 each
      luminaries: "number",  // +1 each
      okuses: "number",      // +1 each
      total: "number"
    },
    awards: ["string"] // ["Bumper Crop", "Sunkissed", etc.]
  }],
  tie_breakers: {
    bumper_crop_tied: "boolean",
    sunkissed_tied: "boolean", 
    frostbit_tied: "boolean"
  }
}

/**
 * JavaScript wrapper for WASM engine with proper type conversion
 */
export class WasmGameEngineWrapper {
  constructor(wasmEngine) {
    this.wasmEngine = wasmEngine
  }

  /**
   * Initialize game and return typed GameState
   */
  async initializeGame(config) {
    const configJson = JSON.stringify(config)
    const resultJson = this.wasmEngine.initialize_game(configJson)
    return GameState.fromJSON(JSON.parse(resultJson))
  }

  /**
   * Get current game state as typed object
   */
  async getGameState() {
    const resultJson = this.wasmEngine.get_game_state()
    return GameState.fromJSON(JSON.parse(resultJson))
  }

  /**
   * Get legal moves as typed Move objects
   */
  async getLegalMoves(gameState, playerId) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const resultJson = this.wasmEngine.get_legal_moves(gameStateJson, playerId)
    return JSON.parse(resultJson).map(Move.fromJSON)
  }

  /**
   * Apply move and return updated GameState
   */
  async applyMove(gameState, playerId, move) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const moveJson = JSON.stringify(move.toJSON())
    const resultJson = this.wasmEngine.apply_move(gameStateJson, playerId, moveJson)
    return GameState.fromJSON(JSON.parse(resultJson))
  }

  /**
   * Get AI move as typed Move object
   */
  async getBestMove(gameState, playerId, timeLimit = 5000) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const resultJson = this.wasmEngine.get_best_move(gameStateJson, playerId, timeLimit)
    return Move.fromJSON(JSON.parse(resultJson))
  }

  /**
   * Validate move with detailed result
   */
  async validateMove(gameState, playerId, move) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const moveJson = JSON.stringify(move.toJSON())
    const resultJson = this.wasmEngine.validate_move(gameStateJson, playerId, moveJson)
    return JSON.parse(resultJson) // Returns validation result object
  }

  /**
   * Calculate round scoring and return updated state
   */
  async calculateRoundScoring(gameState) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const resultJson = this.wasmEngine.calculate_round_scoring(gameStateJson)
    return GameState.fromJSON(JSON.parse(resultJson))
  }

  /**
   * Get detailed scoring breakdown
   */
  async getScoringBreakdown(gameState) {
    const gameStateJson = JSON.stringify(gameState.toJSON())
    const resultJson = this.wasmEngine.get_scoring_breakdown(gameStateJson)
    return JSON.parse(resultJson) // Returns scoring breakdown object
  }
}

/**
 * Mock WASM engine for development and testing
 * Implements the contract with fake data
 */
export class MockWasmGameEngine extends WasmGameEngineContract {
  constructor() {
    super()
    this.gameState = null
  }

  initialize_game(configJson) {
    const config = JSON.parse(configJson)
    
    // Create mock game state
    this.gameState = new GameState({
      config,
      phase: 'playing',
      currentPlayer: 0,
      players: Array.from({ length: config.player_count }, (_, i) => 
        new Player({ 
          id: i, 
          hand: this._createMockHand(),
          isAI: i > 0 // Player 0 is human, others are AI
        })
      ),
      fields: Array.from({ length: 4 }, (_, i) => 
        new GameField({
          season: ['Winter', 'Spring', 'Summer', 'Autumn'][i],
          looseCards: i === 0 ? [this._createMockCard()] : [] // Field 0 has one card
        })
      )
    })

    return JSON.stringify(this.gameState.toJSON())
  }

  get_game_state() {
    if (!this.gameState) {
      throw new Error('Game not initialized')
    }
    return JSON.stringify(this.gameState.toJSON())
  }

  get_legal_moves(gameStateJson, playerId) {
    // Mock implementation: return basic sow moves for all hand cards
    const gameState = GameState.fromJSON(JSON.parse(gameStateJson))
    const player = gameState.players[playerId]
    
    const moves = []
    for (const card of player.hand) {
      for (let fieldId = 0; fieldId < 4; fieldId++) {
        moves.push(new Move({
          type: 'sow',
          playerId,
          fieldId,
          playedCard: card
        }))
      }
    }

    return JSON.stringify(moves.map(move => move.toJSON()))
  }

  apply_move(gameStateJson, playerId, moveJson) {
    const gameState = GameState.fromJSON(JSON.parse(gameStateJson))
    const move = Move.fromJSON(JSON.parse(moveJson))

    // Simple mock implementation: remove card from hand, add to field
    const player = gameState.players[playerId]
    const cardIndex = player.hand.findIndex(c => c.id === move.playedCard.id)
    if (cardIndex >= 0) {
      player.hand.splice(cardIndex, 1)
      gameState.fields[move.fieldId].looseCards.push(move.playedCard)
      gameState.currentPlayer = (gameState.currentPlayer + 1) % gameState.players.length
      gameState.turnNumber++
    }

    return JSON.stringify(gameState.toJSON())
  }

  get_best_move(gameStateJson, playerId, timeLimitMs) {
    // Mock AI: just return first legal move
    const legalMovesJson = this.get_legal_moves(gameStateJson, playerId)
    const legalMoves = JSON.parse(legalMovesJson)
    
    if (legalMoves.length === 0) {
      throw new Error('No legal moves available')
    }

    return JSON.stringify(legalMoves[0])
  }

  // Helper methods for mock data
  _createMockCard() {
    const suits = ['Spring', 'Summer', 'Autumn', 'Winter']
    const ranks = ['2', '3', '4', '5', '6', '7', '8', '9', '10']
    
    return new Card({
      id: Math.floor(Math.random() * 1000),
      rank: ranks[Math.floor(Math.random() * ranks.length)],
      suit: suits[Math.floor(Math.random() * suits.length)],
      value: 2 + Math.floor(Math.random() * 9)
    })
  }

  _createMockHand() {
    return Array.from({ length: 4 }, () => this._createMockCard())
  }
}