<template>
  <div class="illimat-container">
    <svg 
      ref="gameScene" 
      class="illimat-scene" 
      viewBox="0 0 300 300"
      @mousemove="handleMouseMove"
      @mouseup="handleMouseUp"
    >
      <!-- Scene layers in Z-order: playmat → board → piles → tokens -->
      
      <!-- Playmat (background) -->
      <g class="playmat-layer">
        <rect 
          x="0" y="0" 
          width="300" height="300" 
          :fill="playmatFill" 
          class="playmat-surface"
        />
      </g>
      
      <!-- Illimat board faces -->
      <g class="illimat-board-layer">
        <polygon 
          v-for="(path, faceName) in illimatFacePaths"
          :key="faceName"
          :points="path"
          :fill="getFaceFill(faceName)"
          :stroke="getFaceStroke(faceName)"
          :class="`cube-face cube-face-${faceName}`"
        />
      </g>
      
      <!-- Field boundaries and seasonal styling -->
      <g class="fields-layer">
        <GameField
          v-for="(field, fieldIndex) in gameState.fields"
          :key="`field-${fieldIndex}`"
          :field="field"
          :field-index="fieldIndex"
          :season="getFieldSeason(fieldIndex)"
          :bounds="getFieldBounds(fieldIndex)"
          :pile-positions="getPilePositionsForField(fieldIndex)"
        />
      </g>
      
      <!-- Card piles (loose cards and stockpiles) -->
      <g class="piles-layer">
        <CardPile
          v-for="pile in allPiles"
          :key="pile.id"
          :cards="pile.cards"
          :position="pile.position"
          :pile-type="pile.type"
          :draggable="pile.draggable"
          @card-drag-start="handleCardDragStart"
          @card-drag="handleCardDrag"
          @card-drop="handleCardDrop"
        />
      </g>
      
      <!-- Okus tokens -->
      <g class="okus-layer">
        <OkusToken
          v-for="(position, index) in okusScreenPositions"
          :key="`okus-${index}`"
          :position="{ x: position[0], y: position[1] }"
          :visible="gameState.okusPositions[index]"
          :field-index="index"
        />
      </g>
      
      <!-- Player hands (positioned around perimeter) -->
      <g class="hands-layer">
        <PlayerHand
          v-for="(player, playerIndex) in gameState.players"
          :key="`hand-${playerIndex}`"
          :cards="player.hand"
          :player-index="playerIndex"
          :is-current="playerIndex === gameState.currentPlayer"
          :position="getHandPosition(playerIndex)"
          @card-select="handleCardSelect"
        />
      </g>
      
      <!-- Debug overlay (if enabled) -->
      <g v-if="showDebug" class="debug-layer">
        <DebugOverlay
          :projections="cubeProjections"
          :camera-angle="cameraState.angle"
          :illimat-angle="cameraState.illimatAngle"
        />
      </g>
    </svg>
    
    <!-- UI Controls -->
    <div class="game-controls">
      <div class="camera-controls">
        <label>Camera: 
          <input 
            type="range" 
            min="0" 
            max="360" 
            :value="cameraState.angle"
            @input="setCameraAngle($event.target.value)"
          />
          {{ Math.round(cameraState.angle) }}°
        </label>
        
        <label>Season: 
          <input 
            type="range" 
            min="0" 
            max="360" 
            step="90"
            :value="cameraState.illimatAngle"
            @input="setIllimatAngle($event.target.value)"
          />
          {{ getSeasonName() }}
        </label>
        
        <button 
          @click="animateNextSeason"
          :disabled="cameraState.isAnimating"
        >
          Next Season
        </button>
      </div>
      
      <div class="game-info">
        <span>Player {{ gameState.currentPlayer + 1 }}'s turn</span>
        <span>Hand: {{ currentPlayerHand.length }} cards</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch } from 'vue'
import { use3DRenderer } from '@/composables/use3DRenderer.js'
import { useGameState } from '@/composables/useGameState.js'
import GameField from './GameField.vue'
import CardPile from './CardPile.vue'
import OkusToken from './OkusToken.vue'
import PlayerHand from './PlayerHand.vue'
import DebugOverlay from './DebugOverlay.vue'

const props = defineProps({
  showDebug: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits([
  'card-drag',
  'season-change',
  'move-attempt'
])

// Composables
const {
  initializeRenderer,
  cameraState,
  projectionState,
  illimatFacePaths,
  okusScreenPositions,
  cubeProjections,
  setCameraAngle,
  setIllimatAngle,
  animateSeasonChange,
  getFieldBounds,
  calculateSceneLayout,
  getPilePositions
} = use3DRenderer()

const {
  gameState,
  currentPlayerHand,
  getFieldSeason,
  createTestGame,
  nextSeason
} = useGameState()

// Component state
const gameScene = ref(null)
const dragState = ref({
  isDragging: false,
  draggedCard: null,
  startPosition: null
})

// Computed properties
const playmatFill = computed(() => {
  return 'url(#mystical-playmat-gradient)'
})

const allPiles = computed(() => {
  const piles = []
  
  // Field piles
  gameState.value.fields.forEach((field, fieldIndex) => {
    const fieldBounds = getFieldBounds(fieldIndex)
    if (!fieldBounds) return
    
    const positions = getPilePositions(
      field.looseCards.length + field.stockpiles.length,
      fieldBounds,
      'field'
    )
    
    let positionIndex = 0
    
    // Loose cards
    field.looseCards.forEach((card, cardIndex) => {
      piles.push({
        id: `field-${fieldIndex}-loose-${cardIndex}`,
        cards: [card],
        position: positions[positionIndex++],
        type: 'loose',
        draggable: true
      })
    })
    
    // Stockpiles
    field.stockpiles.forEach((stockpile, stockpileIndex) => {
      piles.push({
        id: `field-${fieldIndex}-stockpile-${stockpileIndex}`,
        cards: stockpile.cards,
        position: positions[positionIndex++],
        type: 'stockpile',
        draggable: true
      })
    })
  })
  
  return piles
})

// Methods
const getFaceFill = (faceName) => {
  const fills = {
    bottom: '#2a1a3a',
    top: '#3a2a4a', 
    front: '#4a3a5a',
    right: '#5a4a6a',
    back: '#3a2a4a',
    left: '#4a3a5a'
  }
  return fills[faceName] || '#2a1a3a'
}

const getFaceStroke = (faceName) => {
  return '#6a5a7a'
}

const getPilePositionsForField = (fieldIndex) => {
  const field = gameState.value.fields[fieldIndex]
  const fieldBounds = getFieldBounds(fieldIndex)
  if (!fieldBounds) return []
  
  const totalPiles = field.looseCards.length + field.stockpiles.length
  return getPilePositions(totalPiles, fieldBounds, 'field')
}

const getHandPosition = (playerIndex) => {
  // Position hands around the perimeter
  const angle = (playerIndex / gameState.value.players.length) * Math.PI * 2
  const distance = 120
  
  return {
    centerX: 150 + Math.cos(angle) * distance,
    centerY: 150 + Math.sin(angle) * distance,
    radius: 60
  }
}

const getSeasonName = () => {
  const seasons = ['Winter', 'Spring', 'Summer', 'Autumn']
  const index = Math.floor(cameraState.illimatAngle / 90) % 4
  return seasons[index]
}

const animateNextSeason = async () => {
  const currentAngle = cameraState.illimatAngle
  const nextAngle = currentAngle + 90
  
  await animateSeasonChange(nextAngle)
  nextSeason()
  
  emit('season-change', {
    from: currentAngle,
    to: nextAngle,
    season: getSeasonName()
  })
}

// Event handlers
const handleCardDragStart = (event) => {
  dragState.value = {
    isDragging: true,
    draggedCard: event.card,
    startPosition: event.position
  }
  
  emit('card-drag', {
    type: 'start',
    card: event.card,
    position: event.position
  })
}

const handleCardDrag = (event) => {
  if (!dragState.value.isDragging) return
  
  emit('card-drag', {
    type: 'move',
    card: dragState.value.draggedCard,
    position: event.position
  })
}

const handleCardDrop = (event) => {
  if (!dragState.value.isDragging) return
  
  emit('card-drag', {
    type: 'end',
    card: dragState.value.draggedCard,
    startPosition: dragState.value.startPosition,
    endPosition: event.position
  })
  
  dragState.value = {
    isDragging: false,
    draggedCard: null,
    startPosition: null
  }
}

const handleMouseMove = (event) => {
  if (dragState.value.isDragging) {
    const rect = gameScene.value.getBoundingClientRect()
    const x = ((event.clientX - rect.left) / rect.width) * 300
    const y = ((event.clientY - rect.top) / rect.height) * 300
    
    handleCardDrag({ position: { x, y } })
  }
}

const handleMouseUp = () => {
  if (dragState.value.isDragging) {
    handleCardDrop({ position: { x: 0, y: 0 } })
  }
}

const handleCardSelect = (event) => {
  console.log('Card selected:', event.card)
}

// Lifecycle
onMounted(() => {
  initializeRenderer()
  createTestGame()
})

// Watch for game state changes and update projections
watch(
  [() => cameraState.angle, () => cameraState.illimatAngle],
  () => {
    // Projections are automatically updated by the composable
  }
)
</script>

<style scoped>
.illimat-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  width: 100%;
  height: 100vh;
  background: linear-gradient(135deg, #0a0a0f 0%, #1a0a1f 100%);
  color: #00ccaa;
  font-family: 'Courier New', monospace;
}

.illimat-scene {
  width: min(90vw, 90vh);
  height: min(90vw, 90vh);
  border: 2px solid #aa00cc;
  background: radial-gradient(circle at center, #1a0a1f 0%, #050505 100%);
  overflow: visible;
  cursor: crosshair;
}

.cube-face {
  stroke-width: 1;
  opacity: 0.8;
  transition: fill 0.2s ease;
}

.cube-face:hover {
  opacity: 1;
  filter: brightness(1.2);
}

.playmat-surface {
  opacity: 0.3;
}

.game-controls {
  display: flex;
  gap: 2rem;
  align-items: center;
  flex-wrap: wrap;
  padding: 1rem;
  background: rgba(16, 10, 20, 0.8);
  border: 1px solid #55cc00;
  border-radius: 8px;
}

.camera-controls {
  display: flex;
  gap: 1rem;
  align-items: center;
  flex-wrap: wrap;
}

.camera-controls label {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  color: #55cc00;
}

.camera-controls input[type="range"] {
  width: 100px;
  accent-color: #aa00cc;
}

.camera-controls button {
  background: #2a1a3a;
  color: #00ccaa;
  border: 1px solid #aa00cc;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.camera-controls button:hover {
  background: #3a2a4a;
  box-shadow: 0 0 10px rgba(170, 0, 204, 0.5);
}

.camera-controls button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.game-info {
  display: flex;
  gap: 1rem;
  color: #55cc00;
}

/* SVG gradients */
.illimat-scene defs {
  display: none;
}
</style>

<style>
/* Global SVG gradient definitions */
svg defs {
  position: absolute;
  width: 0;
  height: 0;
}

.mystical-gradient {
  --phosphor-cyan: #00ccaa;
  --phosphor-magenta: #aa00cc;
  --phosphor-lime: #55cc00;
  --phosphor-violet: #5500aa;
}
</style>