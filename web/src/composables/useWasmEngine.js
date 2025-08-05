/**
 * Vue composable for WebAssembly engine integration
 * Handles loading and interfacing with the Rust backend via Doris's WASM interface
 */

import { ref, onMounted } from 'vue'
import init, { WasmGameEngine, init_panic_hook } from '../../pkg/illimat_core.js'

export function useWasmEngine() {
  const wasmEngine = ref(null)
  const isReady = ref(false)
  const loadingMessage = ref('Initializing WebAssembly engine...')
  const loadingError = ref(null)
  
  /**
   * Load the WebAssembly module from Doris's compilation
   */
  const loadWasmModule = async () => {
    try {
      loadingMessage.value = '🦀 Loading Rust WebAssembly module...'
      
      // Initialize WASM module compiled by Doris
      await init()
      init_panic_hook()
      
      // Create game engine for 2 players by default
      wasmEngine.value = new WasmGameEngine(2)
      
      loadingMessage.value = '✅ WebAssembly module loaded successfully'
      isReady.value = true
      
      console.log('🚀 WASM engine ready for game logic')
      console.log('Game engine created:', wasmEngine.value)
      
    } catch (error) {
      console.error('Failed to load WASM module:', error)
      loadingError.value = error.message
      loadingMessage.value = '❌ Failed to load WebAssembly - using fallback mode'
      
      // Continue without WASM (fallback mode)
      isReady.value = true
    }
  }
  
  /**
   * Initialize a new game via WASM (already done in constructor)
   */
  const initializeGame = async (playerCount = 2, config = {}) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, using mock game initialization')
      return createMockGameState(playerCount)
    }
    
    try {
      // Game already initialized in constructor, just get current state
      const gameStateJson = wasmEngine.value.get_state_json()
      const gameState = JSON.parse(gameStateJson)
      console.log('Game initialized with state:', gameState)
      return gameState
    } catch (error) {
      console.error('WASM game initialization failed:', error)
      return createMockGameState(playerCount)
    }
  }
  
  /**
   * Get legal moves for current player via WASM (uses internal state)
   */
  const getLegalMoves = async () => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, returning empty legal moves')
      return []
    }
    
    try {
      const movesJson = wasmEngine.value.get_legal_moves_json()
      return JSON.parse(movesJson)
    } catch (error) {
      console.error('WASM get legal moves failed:', error)
      return []
    }
  }
  
  /**
   * Apply a sow move via WASM
   */
  const applySow = async (fieldId, cardRank, cardSuit) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, simulating sow move')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.apply_sow(playerId, fieldId, cardRank, cardSuit)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM sow move failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Apply a harvest move via WASM
   */
  const applyHarvest = async (fieldId, cardRank, cardSuit, targetCardsJson) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, simulating harvest move')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.apply_harvest(playerId, fieldId, cardRank, cardSuit, targetCardsJson)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM harvest move failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Apply a stockpile move via WASM
   */
  const applyStockpile = async (fieldId, cardRank, cardSuit, targetCardsJson) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, simulating stockpile move')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.apply_stockpile(playerId, fieldId, cardRank, cardSuit, targetCardsJson)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM stockpile move failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Validate moves via WASM
   */
  const validateSow = async (fieldId, cardRank, cardSuit) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, assuming sow is valid')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.validate_sow(playerId, fieldId, cardRank, cardSuit)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM sow validation failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  const validateHarvest = async (fieldId, cardRank, cardSuit, targetCardsJson) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, assuming harvest is valid')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.validate_harvest(playerId, fieldId, cardRank, cardSuit, targetCardsJson)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM harvest validation failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  const validateStockpile = async (fieldId, cardRank, cardSuit, targetCardsJson) => {
    if (!wasmEngine.value) {
      console.warn('WASM not available, assuming stockpile is valid')
      return { success: true }
    }
    
    try {
      const playerId = wasmEngine.value.get_current_player()
      const moveResponse = wasmEngine.value.validate_stockpile(playerId, fieldId, cardRank, cardSuit, targetCardsJson)
      
      return {
        success: moveResponse.success,
        error: moveResponse.error_message
      }
    } catch (error) {
      console.error('WASM stockpile validation failed:', error)
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Get current game state via WASM
   */
  const getCurrentGameState = async () => {
    if (!wasmEngine.value) {
      return createMockGameState(2)
    }
    
    try {
      const gameStateJson = wasmEngine.value.get_state_json()
      return JSON.parse(gameStateJson)
    } catch (error) {
      console.error('WASM get current state failed:', error)
      return createMockGameState(2)
    }
  }
  
  /**
   * Get game status via WASM
   */
  const getGameStatus = async () => {
    if (!wasmEngine.value) {
      return { isGameOver: false, winner: null }
    }
    
    try {
      return {
        isGameOver: wasmEngine.value.is_game_over(),
        winner: wasmEngine.value.get_winner(),
        currentPlayer: wasmEngine.value.get_current_player()
      }
    } catch (error) {
      console.error('WASM get game status failed:', error)
      return { isGameOver: false, winner: null }
    }
  }
  
  /**
   * Create mock game state for fallback mode
   */
  const createMockGameState = (playerCount) => {
    return {
      fields: Array(4).fill(null).map(() => ({ 
        loose_cards: [], 
        stockpiles: [] 
      })),
      players: Array(playerCount).fill(null).map(() => ({ 
        hand: [], 
        harvest: [], 
        score: 0 
      })),
      current_player: 0,
      illimat_rotation: 0,
      okus_positions: [true, true, true, true],
      game_phase: 'playing',
      moves_played: 0
    }
  }
  
  // Auto-load WASM on mount
  onMounted(() => {
    loadWasmModule()
  })
  
  return {
    // State
    wasmEngine,
    isReady,
    loadingMessage,
    loadingError,
    
    // Core functions
    loadWasmModule,
    initializeGame,
    getCurrentGameState,
    getGameStatus,
    
    // Move actions
    applySow,
    applyHarvest, 
    applyStockpile,
    
    // Move validation
    validateSow,
    validateHarvest,
    validateStockpile,
    
    // Game queries
    getLegalMoves,
    
    // Utilities
    createMockGameState
  }
}