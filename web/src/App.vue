<template>
  <div id="app" class="app" data-cy="app">
    <!-- Loading screen while WebAssembly initializes -->
    <div v-if="!wasmReady" class="loading-screen" data-cy="loading-screen">
      <div class="splash">
        <h1 class="title">ILLIMAT</h1>
        <p class="subtitle">Mystical Card Game</p>
        <div class="loading-indicator">
          <div class="loading-spinner"></div>
          <p class="status">{{ loadingMessage }}</p>
        </div>
      </div>
    </div>

    <!-- Main game interface -->  
    <div v-else class="game-interface" data-cy="game-interface">
      <!-- Scoring Banner -->
      <div class="scoring-banner" data-cy="scoring-banner">
        <div class="player-score" v-for="(player, index) in gameState?.players || []" :key="index" 
             :class="{ active: index === gameState?.currentPlayer }"
             :data-cy="`player-score`"
             :data-player="index">
          <span class="player-name">P{{ index + 1 }}</span>
          <span class="score">{{ player.score || 0 }}</span>
          <span class="cards">{{ player.handSize || 0 }}🂠</span>
        </div>
        <div class="game-info">
          <span class="season" data-cy="season-indicator">{{ currentSeasonText }}</span>
          <span class="okus" data-cy="okus-count">{{ okusCount }}🪙</span>
        </div>
      </div>

      <IllimatTable 
        :show-debug="showDebug"
        data-cy="illimat-table"
        @card-drag="handleCardDrag"
        @season-change="handleSeasonChange"
        @move-attempt="handleMoveAttempt"
      />
      
      <div class="ui-overlay">
        <div class="debug-controls" v-if="showDebugControls">
          <label>
            <input type="checkbox" v-model="showDebug" />
            Show Debug Overlay
          </label>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useGameState } from '@/composables/useGameState.js'
import { useWasmEngine } from '@/composables/useWasmEngine.js'
import IllimatTable from '@/components/IllimatTable.vue'

// Composables
const { gameState, currentPlayerHand } = useGameState()
const { wasmEngine, isReady: wasmReady, loadingMessage } = useWasmEngine()

// Component state
const showDebug = ref(false)
const showDebugControls = ref(import.meta.env.DEV) // Only in development

// Computed properties
const currentPlayerText = computed(() => {
  if (!gameState.value) return 'Initializing...'
  const playerNum = gameState.value.currentPlayer + 1
  const handSize = currentPlayerHand.value?.length || 0
  return `Player ${playerNum} (${handSize} cards)`
})

const currentSeasonText = computed(() => {
  if (!gameState.value) return ''
  const seasons = ['❄️', '🌱', '☀️', '🍂']
  const seasonIndex = Math.floor(gameState.value.illimatRotation / 90) % 4
  return seasons[seasonIndex]
})

const okusCount = computed(() => {
  if (!gameState.value) return 0
  return gameState.value.okusPositions?.filter(Boolean).length || 4
})

// Event handlers
const handleCardDrag = (event) => {
  console.log('Card drag event:', event)
  // TODO: Implement card drag logic
}

const handleSeasonChange = (event) => {
  console.log('Season change:', event)
  // Season change already handled by the table component
}

const handleMoveAttempt = (event) => {
  console.log('Move attempt:', event)
  // TODO: Validate and apply move via WASM engine
}

// Lifecycle
onMounted(() => {
  console.log('🎮 Illimat App mounted, waiting for WASM...')
})
</script>

<style scoped>
.app {
  width: 100vw;
  height: 100vh;
  background: #222;
  color: #eee;
  font-family: sans-serif;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 1em;
}

.loading-screen {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 100%;
  height: 100%;
}

.splash {
  text-align: center;
}

.title {
  font-size: 2rem;
  font-weight: normal;
  margin: 0;
  color: #eee;
}

.subtitle {
  color: #aaa;
  font-size: 1rem;
  margin: 1rem 0 2rem 0;
}

.loading-indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
}

.loading-spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(0, 204, 170, 0.3);
  border-top: 3px solid #00ccaa;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

.status {
  color: #55cc00;
  font-size: 0.9rem;
  margin: 0;
}

.game-interface {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.scoring-banner {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 3rem;
  background: linear-gradient(90deg, rgba(10, 10, 15, 0.95) 0%, rgba(20, 10, 25, 0.95) 100%);
  border-bottom: 2px solid #55cc00;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 1rem;
  font-family: 'Courier New', monospace;
  z-index: 200;
}

.player-score {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.25rem 0.75rem;
  border: 1px solid rgba(85, 204, 0, 0.3);
  border-radius: 4px;
  background: rgba(5, 5, 5, 0.7);
  transition: all 0.3s ease;
}

.player-score.active {
  border-color: #55cc00;
  background: rgba(85, 204, 0, 0.1);
  box-shadow: 0 0 10px rgba(85, 204, 0, 0.3);
}

.player-name {
  color: #55cc00;
  font-weight: bold;
  font-size: 0.9rem;
}

.score {
  color: #00ccaa;
  font-weight: bold;
  min-width: 2ch;
  text-align: right;
}

.cards {
  color: #aa00cc;
  font-size: 0.8rem;
}

.game-info {
  display: flex;
  align-items: center;
  gap: 1rem;
  color: #aaa;
  font-size: 0.9rem;
}

.season {
  font-size: 1.2rem;
}

.okus {
  color: #ffaa00;
}

.ui-overlay {
  position: absolute;
  top: 4rem;
  left: 1rem;
  right: 1rem;
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  pointer-events: none;
  z-index: 100;
}

.game-status {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  background: rgba(10, 10, 15, 0.8);
  padding: 1rem;
  border: 1px solid #55cc00;
  border-radius: 8px;
  pointer-events: auto;
}

.current-player {
  color: #55cc00;
  font-weight: 600;
}

.season-indicator {
  color: #aa00cc;
  font-size: 0.9rem;
}

.debug-controls {
  background: rgba(10, 10, 15, 0.8);
  padding: 0.5rem;
  border: 1px solid #666;
  border-radius: 4px;
  pointer-events: auto;
}

.debug-controls label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #666;
  font-size: 0.8rem;
  cursor: pointer;
}

.debug-controls input[type="checkbox"] {
  accent-color: #aa00cc;
}

@keyframes phosphorGlow {
  0% { 
    text-shadow: 0 0 10px #00ccaa; 
  }
  100% { 
    text-shadow: 0 0 20px #00ccaa, 0 0 30px #aa00cc, 0 0 40px #55cc00; 
  }
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}
</style>