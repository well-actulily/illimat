/**
 * Vue composable for game state management with domain types
 */

import { ref, computed, reactive } from 'vue'
import { GameState, Player, GameField, Card, Move, DomainUtils } from '@/domain/types.js'
import { WasmGameEngineWrapper, MockWasmGameEngine } from '@/domain/wasmContract.js'

export function useGameState() {
  // Game state using domain types
  const gameState = ref(null)
  
  // WASM engine (use mock for development)
  const wasmEngine = ref(null)
  const isWasmReady = ref(false)
  
  // Game configuration
  const gameConfig = reactive({
    playerCount: 2,
    enableAI: false,
    difficulty: 'medium'
  })
  
  /**
   * Initialize WASM engine (or mock for development)
   */
  const initializeEngine = async () => {
    try {
      // TODO: Load real WASM module
      // const wasmModule = await import('../../backend/pkg/illimat.js')
      // await wasmModule.default()
      // wasmEngine.value = new WasmGameEngineWrapper(new wasmModule.WasmGameEngine())
      
      // For now, use mock engine
      wasmEngine.value = new WasmGameEngineWrapper(new MockWasmGameEngine())
      isWasmReady.value = true
    } catch (error) {
      console.error('Failed to initialize WASM engine:', error)
      throw error
    }
  }
  
  /**
   * Create a new game with proper domain types
   */
  const createNewGame = async (config = {}) => {
    if (!wasmEngine.value) {
      await initializeEngine()
    }
    
    const gameConfig = {
      player_count: config.playerCount || 2,
      use_stars_suit: config.useStarsSuit ?? (config.playerCount > 3),
      enable_luminaries: config.enableLuminaries ?? true,
      luminary_expansions: config.luminaryExpansions || ['core'],
      beginner_mode: config.beginnerMode ?? false
    }
    
    try {
      const newGameState = await wasmEngine.value.initializeGame(gameConfig)
      gameState.value = newGameState
      return newGameState
    } catch (error) {
      console.error('Failed to create new game:', error)
      throw error
    }
  }
  
  /**
   * Create a test game with sample data for UI development
   */
  const createTestGame = async () => {
    // Create sample cards using domain types
    const sampleCards = [
      new Card({ id: 1, rank: '5', suit: 'Spring', value: 5 }),
      new Card({ id: 2, rank: 'Q', suit: 'Stars', value: 12 }),
      new Card({ id: 3, rank: '8', suit: 'Autumn', value: 8 }),
      new Card({ id: 4, rank: 'F', suit: 'Winter', value: 1 }),
      new Card({ id: 5, rank: '3', suit: 'Summer', value: 3 }),
      new Card({ id: 6, rank: 'K', suit: 'Spring', value: 13 }),
      new Card({ id: 7, rank: '7', suit: 'Winter', value: 7 }),
      new Card({ id: 8, rank: 'N', suit: 'Stars', value: 11 })
    ]
    
    // Create test game state with domain types
    gameState.value = new GameState({
      phase: 'playing',
      currentPlayer: 0,
      roundNumber: 1,
      fields: [
        new GameField({ 
          looseCards: [sampleCards[0], sampleCards[1]],
          season: 'Winter',
          hasOkus: true
        }),
        new GameField({ 
          looseCards: [sampleCards[4]],
          season: 'Spring' 
        }),
        new GameField({ 
          season: 'Summer',
          hasOkus: true
        }),
        new GameField({ 
          looseCards: [sampleCards[6], sampleCards[7]],
          season: 'Autumn'
        })
      ],
      players: [
        new Player({ 
          id: 0,
          name: 'Player 1',
          hand: [sampleCards[0], sampleCards[1], sampleCards[2], sampleCards[3]], 
          harvest: [sampleCards[4]], 
          score: 2 
        }),
        new Player({ 
          id: 1,
          name: 'Player 2',
          hand: [sampleCards[4], sampleCards[5], sampleCards[6]], 
          harvest: [sampleCards[7]], 
          score: 1,
          isAI: true
        })
      ],
      okusPositions: [true, false, true, false],
      okusOnIllimat: 2
    })
    
    return gameState.value
  }
  
  /**
   * Get current player data
   */
  const currentPlayer = computed(() => {
    if (!gameState.value) return null
    return gameState.value.getCurrentPlayer()
  })
  
  /**
   * Get current player's hand
   */
  const currentPlayerHand = computed(() => {
    return currentPlayer.value?.hand || []
  })
  
  /**
   * Get all field cards for 3D rendering
   */
  const allFieldCards = computed(() => {
    if (!gameState.value) return []
    
    const cards = []
    
    gameState.value.fields.forEach((field, fieldIndex) => {
      // Loose cards
      field.looseCards.forEach((card, cardIndex) => {
        cards.push({
          card,
          location: 'field',
          fieldIndex,
          cardIndex,
          isLoose: true,
          position: { x: 0, y: 0, z: 0 } // TODO: Calculate 3D position
        })
      })
      
      // Stockpiled cards
      field.stockpiles.forEach((stockpile, stockpileIndex) => {
        stockpile.cards.forEach((card, cardIndex) => {
          cards.push({
            card,
            location: 'field',
            fieldIndex,
            stockpileIndex,
            cardIndex,
            isLoose: false,
            position: { x: 0, y: 0, z: 0 } // TODO: Calculate 3D position
          })
        })
      })
    })
    
    return cards
  })
  
  /**
   * Get field season based on Illimat orientation
   */
  const getFieldSeason = (fieldIndex) => {
    if (!gameState.value) return 'Winter'
    return gameState.value.getFieldSeason(fieldIndex)
  }
  
  /**
   * Apply a move using WASM engine
   */
  const applyMove = async (playerId, move) => {
    if (!wasmEngine.value || !gameState.value) {
      throw new Error('Game engine not ready')
    }
    
    try {
      const moveObj = move instanceof Move ? move : Move.fromJSON(move)
      const updatedState = await wasmEngine.value.applyMove(gameState.value, playerId, moveObj)
      gameState.value = updatedState
      return { success: true, newState: updatedState }
    } catch (error) {
      console.error('Failed to apply move:', error)
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Validate a move using WASM engine
   */
  const validateMove = async (playerId, move) => {
    if (!wasmEngine.value || !gameState.value) {
      throw new Error('Game engine not ready')
    }
    
    try {
      const moveObj = move instanceof Move ? move : Move.fromJSON(move)
      const result = await wasmEngine.value.validateMove(gameState.value, playerId, moveObj)
      return { valid: result.is_valid, feedback: result.error_message || 'Move is valid' }
    } catch (error) {
      console.error('Failed to validate move:', error)
      return { valid: false, feedback: error.message }
    }
  }
  
  /**
   * Get legal moves using WASM engine
   */
  const getLegalMoves = async (playerId) => {
    if (!wasmEngine.value || !gameState.value) {
      return []
    }
    
    try {
      const moves = await wasmEngine.value.getLegalMoves(gameState.value, playerId)
      return moves
    } catch (error) {
      console.error('Error getting legal moves:', error)
      return []
    }
  }
  
  /**
   * Get AI move using WASM engine
   */
  const getAIMove = async (playerId, timeLimit = 3000) => {
    if (!wasmEngine.value || !gameState.value) {
      throw new Error('Game engine not ready')
    }
    
    try {
      const move = await wasmEngine.value.getBestMove(gameState.value, playerId, timeLimit)
      return move
    } catch (error) {
      console.error('Failed to get AI move:', error)
      throw error
    }
  }
  
  /**
   * Advance to next season (for manual testing)
   */
  const nextSeason = () => {
    if (!gameState.value) return
    gameState.value.illimatOrientation = (gameState.value.illimatOrientation + 1) % 4
  }
  
  /**
   * End current player's turn (for manual testing)
   */
  const endTurn = () => {
    if (!gameState.value) return
    gameState.value.currentPlayer = gameState.value.getNextPlayer()
  }
  
  return {
    // State
    gameState,
    gameConfig,
    isWasmReady,
    
    // Computed properties
    currentPlayer,
    currentPlayerHand,
    allFieldCards,
    
    // Game initialization
    initializeEngine,
    createNewGame,
    createTestGame,
    
    // Game actions
    applyMove,
    validateMove,
    getLegalMoves,
    getAIMove,
    nextSeason,
    endTurn,
    
    // Utility functions
    getFieldSeason
  }
}