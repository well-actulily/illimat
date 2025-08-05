/**
 * Vue composable for game state management
 */

import { ref, computed, reactive } from 'vue'

export function useGameState() {
  // Mock game state structure (will integrate with WASM later)
  const gameState = ref({
    fields: [
      { looseCards: [], stockpiles: [] },
      { looseCards: [], stockpiles: [] },
      { looseCards: [], stockpiles: [] },
      { looseCards: [], stockpiles: [] }
    ],
    players: [
      { hand: [], harvest: [], score: 0 },
      { hand: [], harvest: [], score: 0 },
      { hand: [], harvest: [], score: 0 },
      { hand: [], harvest: [], score: 0 }
    ],
    currentPlayer: 0,
    illimatRotation: 0,
    okusPositions: [true, true, true, true], // Which fields have okus
    scores: [0, 0, 0, 0],
    gamePhase: 'playing' // 'playing', 'ended'
  })
  
  // Game configuration
  const gameConfig = reactive({
    playerCount: 2,
    enableAI: false,
    difficulty: 'medium'
  })
  
  /**
   * Create a new game with sample data for testing
   */
  const createTestGame = () => {
    // Sample cards for testing the pile system
    const sampleCards = [
      { id: 1, rank: '5', suit: 'Spring', value: 5 },
      { id: 2, rank: 'Q', suit: 'Stars', value: 12 },
      { id: 3, rank: '8', suit: 'Autumn', value: 8 },
      { id: 4, rank: 'F', suit: 'Winter', value: 1 },
      { id: 5, rank: '3', suit: 'Summer', value: 3 },
      { id: 6, rank: 'K', suit: 'Spring', value: 13 },
      { id: 7, rank: '7', suit: 'Winter', value: 7 },
      { id: 8, rank: 'N', suit: 'Stars', value: 11 }
    ]
    
    gameState.value = {
      fields: [
        { 
          looseCards: [sampleCards[0], sampleCards[1]], 
          stockpiles: [
            { cards: [sampleCards[2], sampleCards[3]] }
          ] 
        },
        { 
          looseCards: [sampleCards[4]], 
          stockpiles: [] 
        },
        { 
          looseCards: [], 
          stockpiles: [
            { cards: [sampleCards[5]] }
          ] 
        },
        { 
          looseCards: [sampleCards[6], sampleCards[7]], 
          stockpiles: [] 
        }
      ],
      players: [
        { 
          hand: [sampleCards[0], sampleCards[1], sampleCards[2], sampleCards[3]], 
          harvest: [sampleCards[4]], 
          score: 2 
        },
        { 
          hand: [sampleCards[4], sampleCards[5], sampleCards[6]], 
          harvest: [sampleCards[7]], 
          score: 1 
        },
        { hand: [], harvest: [], score: 0 },
        { hand: [], harvest: [], score: 0 }
      ],
      currentPlayer: 0,
      illimatRotation: 0,
      okusPositions: [true, false, true, false],
      scores: [2, 1, 0, 0],
      gamePhase: 'playing'
    }
  }
  
  /**
   * Get current player data
   */
  const currentPlayer = computed(() => {
    return gameState.value.players[gameState.value.currentPlayer]
  })
  
  /**
   * Get current player's hand
   */
  const currentPlayerHand = computed(() => {
    return currentPlayer.value?.hand || []
  })
  
  /**
   * Get all visible cards (for rendering)
   */
  const allVisibleCards = computed(() => {
    const cards = []
    
    // Field cards
    gameState.value.fields.forEach((field, fieldIndex) => {
      field.looseCards.forEach((card, cardIndex) => {
        cards.push({
          ...card,
          location: 'field',
          fieldIndex,
          cardIndex,
          isLoose: true
        })
      })
      
      field.stockpiles.forEach((stockpile, stockpileIndex) => {
        stockpile.cards.forEach((card, cardIndex) => {
          cards.push({
            ...card,
            location: 'field',
            fieldIndex,
            stockpileIndex,
            cardIndex,
            isLoose: false
          })
        })
      })
    })
    
    // Player hands
    gameState.value.players.forEach((player, playerIndex) => {
      player.hand.forEach((card, cardIndex) => {
        cards.push({
          ...card,
          location: 'hand',
          playerIndex,
          cardIndex
        })
      })
    })
    
    return cards
  })
  
  /**
   * Get field season based on Illimat rotation
   */
  const getFieldSeason = (fieldIndex) => {
    const seasons = ['Winter', 'Spring', 'Summer', 'Autumn']
    const rotation = Math.floor(gameState.value.illimatRotation / 90) % 4
    return seasons[(fieldIndex + rotation) % 4]
  }
  
  /**
   * Apply a move (placeholder for WASM integration)
   */
  const applyMove = async (playerId, move) => {
    try {
      // TODO: Integrate with WASM backend
      console.log('Applying move:', { playerId, move })
      
      // For now, just simulate move success
      return { success: true, newState: gameState.value }
    } catch (error) {
      return { success: false, error: error.message }
    }
  }
  
  /**
   * Validate a move (placeholder for WASM integration)
   */
  const validateMove = async (playerId, move) => {
    try {
      // TODO: Integrate with WASM backend
      console.log('Validating move:', { playerId, move })
      
      // For now, just simulate validation
      return { valid: true, feedback: 'Move is valid' }
    } catch (error) {
      return { valid: false, feedback: error.message }
    }
  }
  
  /**
   * Get legal moves (placeholder for WASM integration)
   */
  const getLegalMoves = async (playerId) => {
    try {
      // TODO: Integrate with WASM backend
      console.log('Getting legal moves for player:', playerId)
      
      // For now, return empty array
      return []
    } catch (error) {
      console.error('Error getting legal moves:', error)
      return []
    }
  }
  
  /**
   * Advance to next season
   */
  const nextSeason = () => {
    gameState.value.illimatRotation = (gameState.value.illimatRotation + 90) % 360
  }
  
  /**
   * End current player's turn
   */
  const endTurn = () => {
    gameState.value.currentPlayer = (gameState.value.currentPlayer + 1) % gameConfig.playerCount
  }
  
  return {
    // State
    gameState,
    gameConfig,
    
    // Computed properties
    currentPlayer,
    currentPlayerHand,
    allVisibleCards,
    
    // Game actions
    createTestGame,
    applyMove,
    validateMove,
    getLegalMoves,
    nextSeason,
    endTurn,
    
    // Utility functions
    getFieldSeason
  }
}