<template>
  <div class="illimat-container">
    <svg 
      ref="gameScene" 
      class="illimat-scene" 
      width="300" 
      height="300" 
      viewBox="0 0 300 300"
      @click="handleSceneClick"
    >
      <!-- Game fields -->
      <GameField
        v-for="(field, index) in gameState?.fields || []"
        :key="index"
        :field="field"
        :field-index="index"
        :season="getFieldSeason(index)"
        :bounds="getFieldBounds(index)"
        :has-okus="hasFieldOkus(index)"
        :is-clickable="fieldInteractionMode"
        :is-valid-target="isValidFieldTarget(index)"
        :is-invalid-target="isInvalidFieldTarget(index)"
        @field-click="handleFieldClick"
      />
      
      <!-- Game cards -->
      <GameCard
        v-for="card in allVisibleCards"
        :key="`${card.id}-${card.fieldIndex}`"
        :card="card"
        :position="getCardPosition(card)"
        :draggable="isCardDraggable(card)"
        :selected="selectedCard?.id === card.id"
        @click="handleCardClick"
      />
      
      <!-- Illimat center -->
      <IllimatCenter
        :center-x="150"
        :center-y="150"
        :illimat-rotation="cameraState.illimatAngle"
        :okus-positions="okusState"
        @rotate="setIllimatAngle"
      />
      
      <!-- Reference elements (hidden unless debugging) -->
      <g v-if="showDebug" id="referenceLines"></g>
      <g id="cards"></g>
      <polygon id="top" fill="#fff" stroke="#000" stroke-width="0.5" opacity="0"/>
      <polygon id="left" stroke="#000" stroke-width="0.5" opacity="0"/>
      <polygon id="right" stroke="#000" stroke-width="0.5" opacity="0"/>
      <g id="okuses"></g>
    </svg>
    
    <div class="controls">
      <label for="cameraSlider">Camera:</label>
      <input type="range" id="cameraSlider" min="0" max="359" :value="cameraState.angle" @input="setCameraAngle($event.target.value)" />
      <span>{{ Math.round(cameraState.angle) }}°</span>
    </div>

    <div class="controls">
      <label for="illimatSlider">Illimat:</label>
      <input type="range" id="illimatSlider" min="0" max="359" :value="cameraState.illimatAngle" @input="setIllimatAngle($event.target.value)" />
      <span>{{ Math.round(cameraState.illimatAngle) }}°</span>
      <button @click="animateNextSeason" :disabled="cameraState.isAnimating">Next Season</button>
    </div>

    <div class="controls">
      <label>Okus Count:</label>
      <input type="range" min="2" max="4" :value="okusCount" @input="updateOkusCount($event.target.value)" />
      <span>{{ okusCount }}</span>
      <span>Okuses:</span>
      <button v-for="(active, index) in okusState" :key="index" class="okus-btn" :class="{ active, disabled: index >= okusCount }" @click="toggleOkus(index)">
        {{ String.fromCharCode(65 + index) }}
      </button>
    </div>

    <div class="controls">
      <label>Field Cards:</label>
      <span v-for="(field, index) in (gameState?.fields || [])" :key="index">
        F{{ index }}: {{ field.looseCards?.length || 0 }}
      </span>
      <button @click="addRandomCard">Add Random Card</button>
    </div>

    <div class="debug">
      Total angle: {{ Math.round((cameraState.angle + cameraState.illimatAngle) % 360) }}° | 
      Quadrant: {{ Math.floor(((cameraState.angle + cameraState.illimatAngle + 45) % 360) / 90) }} | 
      Camera: {{ Math.round(cameraState.angle) }}° | 
      Illimat: {{ Math.round(cameraState.illimatAngle) }}°
    </div>

    <!-- Action popup -->
    <ActionPopup
      :visible="showActionPopup"
      :selected-card="selectedCard"
      :position="actionPopupPosition"
      :available-actions="availableActions"
      :game-state="gameState"
      @close="closeActionPopup"
      @action-confirm="handleActionConfirm"
    />

    <!-- Mobile player hand overlay -->
    <PlayerHandMobile
      :playerHand="currentPlayerHand"
      :gameState="gameState"
      :currentPlayer="true"
      @card-selected="handleCardSelected"
      @action-selected="handleActionSelected"
      @move-ready="handleMoveReady"
    />
  </div>
</template>

<script setup>
import { ref, computed, onMounted, watch, nextTick } from 'vue'
import { useGameState } from '@/composables/useGameState.js'
import { useWasmEngine } from '@/composables/useWasmEngine.js'
import { Card } from '@/domain/types.js'
import GameField from './GameField.vue'
import GameCard from './GameCard.vue'
import IllimatCenter from './IllimatCenter.vue'
import PlayerHandMobile from './PlayerHandMobile.vue'
import ActionPopup from './ActionPopup.vue'

// Props
const props = defineProps({
  showDebug: { type: Boolean, default: false }
})

const emit = defineEmits(['card-drag', 'season-change', 'move-attempt'])

// Composables
const { gameState, currentPlayerHand } = useGameState()
const { 
  isReady: wasmReady, 
  applySow, 
  applyHarvest, 
  applyStockpile,
  validateSow,
  validateHarvest,
  validateStockpile,
  getCurrentGameState
} = useWasmEngine()

// Interactive state
const selectedCard = ref(null)
const showActionPopup = ref(false)
const actionPopupPosition = ref({ x: 0, y: 0 })
const fieldInteractionMode = ref(false)
const targetAction = ref(null)

// Direct port of Lily's PoC logic
const W = 100, D = 100, H = 17
const offset = [150, 150]
let cameraAngle = 90
let illimatAngle = 0
let animating = false

let maxOkuses = 4
let okusState = [true, true, true, true]
let cardCounts = [1, 0, 0, 0]

let illimatCenter = { x: 0, y: 0 }
let fieldCenters = {
  left: { x: -1.25, y: 0 },
  right: { x: 1.25, y: 0 },
  front: { x: 0, y: -1.25 },
  back: { x: 0, y: 1.25 }
}

const ISO_X_FACTOR = Math.sqrt(3) / 2
const ISO_Y_FACTOR = 0.5
const Z_SCALE = 1.3

const cubeVertices = [
  [-W/2, -D/2,  H], [ W/2, -D/2,  H],
  [ W/2,  D/2,  H], [-W/2,  D/2,  H],
  [-W/2, -D/2, -H], [ W/2, -D/2, -H],
  [ W/2,  D/2, -H], [-W/2,  D/2, -H],
]

const projectedPoints = new Array(8)
for (let i = 0; i < 8; i++) {
  projectedPoints[i] = [0, 0]
}

const okusPositions = new Array(4)
for (let i = 0; i < 4; i++) {
  okusPositions[i] = [0, 0]
}

// Computed properties for interactive gameplay
const allVisibleCards = computed(() => {
  if (!gameState.value?.field_cards) return []
  
  const cards = []
  gameState.value.field_cards.forEach((fieldCards, fieldIndex) => {
    fieldCards.forEach((card, cardIndex) => {
      cards.push({
        ...card,
        id: `field-${fieldIndex}-${cardIndex}`,
        fieldIndex,
        cardIndex
      })
    })
  })
  
  return cards
})

const availableActions = computed(() => {
  if (!selectedCard.value || !wasmReady.value) return []
  
  const actions = []
  
  // Sow (always available)
  actions.push({
    type: 'sow',
    label: 'Sow',
    icon: '💧',
    enabled: true,
    tooltip: 'Discard card to field'
  })
  
  // Harvest (season dependent)
  actions.push({
    type: 'harvest', 
    label: 'Harvest',
    icon: '🌾',
    enabled: true,
    tooltip: 'Collect matching cards'
  })
  
  // Stockpile (season dependent)
  actions.push({
    type: 'stockpile',
    label: 'Stockpile', 
    icon: '📚',
    enabled: true,
    tooltip: 'Combine cards for future harvest'
  })
  
  return actions
})

// Methods for interactive gameplay
const getFieldSeason = (fieldIndex) => {
  const seasons = ['winter', 'spring', 'summer', 'autumn']
  const rotation = cameraState.value?.illimatAngle || 0
  const seasonIndex = (Math.floor(rotation / 90) + fieldIndex) % 4
  return seasons[seasonIndex]
}

const getFieldBounds = (fieldIndex) => {
  // Return field boundaries based on 3D projection - simplified for now
  const fieldSize = 40
  const positions = [
    { x: 75, y: 75 },   // Top-left field
    { x: 225, y: 75 },  // Top-right field  
    { x: 225, y: 225 }, // Bottom-right field
    { x: 75, y: 225 }   // Bottom-left field
  ]
  
  const pos = positions[fieldIndex] || positions[0]
  return {
    x: pos.x - fieldSize/2,
    y: pos.y - fieldSize/2,
    width: fieldSize,
    height: fieldSize
  }
}

const hasFieldOkus = (fieldIndex) => {
  return okusState[fieldIndex] || false
}

const isValidFieldTarget = (fieldIndex) => {
  if (!fieldInteractionMode.value || !targetAction.value) return false
  
  const season = getFieldSeason(fieldIndex)
  const restrictions = {
    winter: { harvest: false },
    spring: { stockpile: false },
    summer: {},
    autumn: { sow: false }
  }
  
  const seasonRestrictions = restrictions[season] || {}
  return seasonRestrictions[targetAction.value] !== false
}

const isInvalidFieldTarget = (fieldIndex) => {
  if (!fieldInteractionMode.value || !targetAction.value) return false
  return !isValidFieldTarget(fieldIndex)
}

const getCardPosition = (card) => {
  // Simple positioning for now - would use 3D projection in full implementation
  const fieldBounds = getFieldBounds(card.fieldIndex)
  const cardOffset = card.cardIndex * 5
  
  return {
    corners: [
      [fieldBounds.x + cardOffset, fieldBounds.y + cardOffset],
      [fieldBounds.x + cardOffset + 20, fieldBounds.y + cardOffset], 
      [fieldBounds.x + cardOffset + 20, fieldBounds.y + cardOffset + 28],
      [fieldBounds.x + cardOffset, fieldBounds.y + cardOffset + 28]
    ]
  }
}

const isCardDraggable = (card) => {
  // Cards in current player's hand are draggable
  return gameState.value?.current_player === 0 // Simplified for now
}

// Event handlers
const handleSceneClick = (event) => {
  // Close action popup when clicking elsewhere
  if (showActionPopup.value) {
    closeActionPopup()
  }
}

const handleCardClick = (cardEvent) => {
  console.log('Card clicked:', cardEvent)
  
  // Only handle clicks for cards in current player's hand
  if (!isCardDraggable(cardEvent.card)) return
  
  selectedCard.value = cardEvent.card
  showActionPopup.value = true
  
  // Position popup near the card
  actionPopupPosition.value = {
    x: cardEvent.event?.clientX || 150,
    y: cardEvent.event?.clientY || 150
  }
}

const handleFieldClick = (fieldEvent) => {
  console.log('Field clicked:', fieldEvent)
  
  if (!fieldInteractionMode.value || !selectedCard.value || !targetAction.value) return
  
  // Execute the move
  executeMove(targetAction.value, selectedCard.value, fieldEvent.fieldIndex)
}

const closeActionPopup = () => {
  showActionPopup.value = false
  selectedCard.value = null
  fieldInteractionMode.value = false
  targetAction.value = null
}

const handleActionConfirm = async (actionData) => {
  console.log('Action confirmed:', actionData)
  
  try {
    await executeMove(actionData.action, actionData.card, actionData.fieldIndex)
    closeActionPopup()
  } catch (error) {
    console.error('Move execution failed:', error)
    // Show error to user
  }
}

const executeMove = async (action, card, fieldIndex) => {
  if (!wasmReady.value) {
    console.warn('WASM not ready, cannot execute move')
    return
  }
  
  try {
    let result
    const cardRank = card.rank || 1
    const cardSuit = ['Spring', 'Summer', 'Autumn', 'Winter', 'Stars'].indexOf(card.suit) || 0
    
    switch (action) {
      case 'sow':
        result = await applySow(fieldIndex, cardRank, cardSuit)
        break
      case 'harvest':
        result = await applyHarvest(fieldIndex, cardRank, cardSuit, '[]') // Empty target cards for now
        break
      case 'stockpile':
        result = await applyStockpile(fieldIndex, cardRank, cardSuit, '[]') // Empty target cards for now
        break
      default:
        throw new Error(`Unknown action: ${action}`)
    }
    
    if (result.success) {
      console.log('Move executed successfully')
      // Refresh game state
      const newState = await getCurrentGameState()
      // Update UI state
      emit('move-attempt', { success: true, action, card, fieldIndex })
    } else {
      console.error('Move failed:', result.error)
      // Show error to user
    }
    
  } catch (error) {
    console.error('Move execution error:', error)
    throw error
  }
}

const okusOffsets = [
  [1.5, -1], [-2, 0.5], [1, 1.5], [-0.5, -2]
]

const greekMappings = [
  { alpha: 3, beta: 2, gamma: 1, delta: 0, epsilon: 7, zeta: 6, eta: 5 },
  { alpha: 2, beta: 1, gamma: 0, delta: 3, epsilon: 6, zeta: 5, eta: 4 },
  { alpha: 1, beta: 0, gamma: 3, delta: 2, epsilon: 5, zeta: 4, eta: 7 },
  { alpha: 0, beta: 3, gamma: 2, delta: 1, epsilon: 4, zeta: 7, eta: 6 },
]

const seasonalColors = [
  ['#3af', '#3f6'], ['#3f6', '#fd4'], ['#fd4', '#f93'], ['#f93', '#3af'],
]

// Additional reactive state  
const gameScene = ref(null)
const cameraState = ref({ angle: cameraAngle, illimatAngle: illimatAngle, isAnimating: animating })
const okusCount = ref(maxOkuses)

// Direct DOM manipulation like Lily's PoC
const updateProjections = (cameraAngle, illimatAngle, lift = 0) => {
  const cameraRad = cameraAngle * Math.PI / 180
  const illimatRad = illimatAngle * Math.PI / 180
  
  const illimatX = illimatCenter.x * W
  const illimatY = illimatCenter.y * W
  
  for (let i = 0; i < 8; i++) {
    const [x, y, z] = cubeVertices[i]
    
    const illimatRotX = x * Math.cos(illimatRad) - y * Math.sin(illimatRad) + illimatX
    const illimatRotY = x * Math.sin(illimatRad) + y * Math.cos(illimatRad) + illimatY
    
    const finalX = illimatRotX * Math.cos(cameraRad) - illimatRotY * Math.sin(cameraRad)
    const finalY = illimatRotX * Math.sin(cameraRad) + illimatRotY * Math.cos(cameraRad)
    
    projectedPoints[i][0] = (finalX - finalY) * ISO_X_FACTOR + offset[0]
    projectedPoints[i][1] = (finalX + finalY) * ISO_Y_FACTOR - (z * Z_SCALE) + lift + offset[1]
  }

  const circleRadius = W / 4
  for (let i = 0; i < 4; i++) {
    const theta = (i / Math.max(maxOkuses, 1)) * 2 * Math.PI
    const baseX = circleRadius * Math.cos(theta)
    const baseY = circleRadius * Math.sin(theta)
    const [offsetX, offsetY] = okusOffsets[i]
    const x = baseX + offsetX
    const y = baseY + offsetY
    const z = H + 3
    
    const illimatRotX = x * Math.cos(illimatRad) - y * Math.sin(illimatRad) + illimatX
    const illimatRotY = x * Math.sin(illimatRad) + y * Math.cos(illimatRad) + illimatY
    
    const finalX = illimatRotX * Math.cos(cameraRad) - illimatRotY * Math.sin(cameraRad)
    const finalY = illimatRotX * Math.sin(cameraRad) + illimatRotY * Math.cos(cameraRad)
    
    okusPositions[i][0] = (finalX - finalY) * ISO_X_FACTOR + offset[0]
    okusPositions[i][1] = (finalX + finalY) * ISO_Y_FACTOR - (z * Z_SCALE) + lift + offset[1]
  }
}

const pointsToString = (indices) => {
  let result = ''
  for (let i = 0; i < indices.length; i++) {
    const [x, y] = projectedPoints[indices[i]]
    if (i > 0) result += ' '
    result += Math.round(x * 10) / 10 + ',' + Math.round(y * 10) / 10
  }
  return result
}

const getSeasonalMapping = (cameraAngle, illimatAngle) => {
  const totalAngle = cameraAngle + illimatAngle
  const q = Math.floor(((totalAngle + 45) % 360) / 90)
  const g = greekMappings[q]
  const colors = seasonalColors[q]
  
  return { g, colors }
}

const update = (cameraAngle, illimatAngle, lift = 0) => {
  updateProjections(cameraAngle, illimatAngle, lift)
  
  const { g, colors } = getSeasonalMapping(cameraAngle, illimatAngle)
  
  if (!gameScene.value) return
  
  const topElement = gameScene.value.querySelector('#top')
  const leftElement = gameScene.value.querySelector('#left') 
  const rightElement = gameScene.value.querySelector('#right')
  
  if (topElement) topElement.setAttribute("points", pointsToString([0, 1, 2, 3]))
  if (leftElement) {
    leftElement.setAttribute("points", pointsToString([g.alpha, g.epsilon, g.zeta, g.beta]))
    leftElement.setAttribute("fill", colors[0])
  }
  if (rightElement) {
    rightElement.setAttribute("points", pointsToString([g.zeta, g.eta, g.gamma, g.beta]))
    rightElement.setAttribute("fill", colors[1])
  }
  
  updateOkuses()
  updateCards(cameraAngle, illimatAngle)
}

const updateOkuses = () => {
  if (!gameScene.value) return
  const okusesGroup = gameScene.value.querySelector('#okuses')
  if (!okusesGroup) return
  
  okusesGroup.innerHTML = ''
  for (let i = 0; i < maxOkuses; i++) {
    if (okusState[i]) {
      const [x, y] = okusPositions[i]
      const circle = document.createElementNS("http://www.w3.org/2000/svg", "circle")
      circle.setAttribute("cx", x.toFixed(1))
      circle.setAttribute("cy", y.toFixed(1))
      circle.setAttribute("r", "4")
      circle.setAttribute("fill", "#666")
      circle.setAttribute("stroke", "#000")
      circle.setAttribute("stroke-width", "0.5")
      okusesGroup.appendChild(circle)
    }
  }
}

const updateCards = (cameraAngle, illimatAngle) => {
  if (!gameScene.value) return
  const cardsGroup = gameScene.value.querySelector('#cards')
  if (!cardsGroup) return
  
  cardsGroup.innerHTML = ''
  
  const fieldZ = -H
  const isAligned = Math.abs(illimatAngle % 90) < 0.5 || Math.abs(illimatAngle % 90) > 89.5
  const illimatQuadrant = Math.floor(((illimatAngle + 45 + 90) % 360) / 90)
  const cameraRad = cameraAngle * Math.PI / 180
  
  let fieldColors = ['#fff', '#fff', '#fff', '#fff']
  
  if (isAligned) {
    const allSeasonColors = ['#3af', '#3f6', '#fd4', '#f93']
    fieldColors[0] = allSeasonColors[(illimatQuadrant + 0) % 4]
    fieldColors[1] = allSeasonColors[(illimatQuadrant + 3) % 4]
    fieldColors[2] = allSeasonColors[(illimatQuadrant + 2) % 4]
    fieldColors[3] = allSeasonColors[(illimatQuadrant + 1) % 4]
  }
  
  const fields = [
    { name: 'right', center: fieldCenters.right, color: fieldColors[0], index: 0 },
    { name: 'back', center: fieldCenters.back, color: fieldColors[1], index: 1 },
    { name: 'left', center: fieldCenters.left, color: fieldColors[2], index: 2 },
    { name: 'front', center: fieldCenters.front, color: fieldColors[3], index: 3 }
  ]
  
  fields.forEach((field) => {
    drawFieldAndCards(field, cameraRad, fieldZ)
  })
}

const drawFieldAndCards = (field, cameraRad, fieldZ) => {
  if (!gameScene.value) return
  const cardsGroup = gameScene.value.querySelector('#cards')
  if (!cardsGroup) return
  
  const centerX = field.center.x * W
  const centerY = field.center.y * W
  
  let corners
  if (field.index === 0) { // Right field
    corners = [
      [centerX - W/2, centerY - W/2, fieldZ],
      [centerX + W, centerY - W/2, fieldZ],
      [centerX + W, centerY + W/2, fieldZ],
      [centerX - W/2, centerY + W/2, fieldZ],
    ]
  } else if (field.index === 1) { // Back field
    corners = [
      [centerX - W/2, centerY - W/2, fieldZ],
      [centerX + W/2, centerY - W/2, fieldZ],
      [centerX + W/2, centerY + W, fieldZ],
      [centerX - W/2, centerY + W, fieldZ],
    ]
  } else if (field.index === 2) { // Left field
    corners = [
      [centerX - W, centerY - W/2, fieldZ],
      [centerX + W/2, centerY - W/2, fieldZ],
      [centerX + W/2, centerY + W/2, fieldZ],
      [centerX - W, centerY + W/2, fieldZ],
    ]
  } else { // Front field
    corners = [
      [centerX - W/2, centerY - W, fieldZ],
      [centerX + W/2, centerY - W, fieldZ],
      [centerX + W/2, centerY + W/2, fieldZ],
      [centerX - W/2, centerY + W/2, fieldZ],
    ]
  }
  
  const projectedCorners = corners.map(([x, y, z]) => {
    const finalX = x * Math.cos(cameraRad) - y * Math.sin(cameraRad)
    const finalY = x * Math.sin(cameraRad) + y * Math.cos(cameraRad)
    return [
      (finalX - finalY) * ISO_X_FACTOR + offset[0],
      (finalX + finalY) * ISO_Y_FACTOR - (z * Z_SCALE) + offset[1]
    ]
  })
  
  const fieldElement = document.createElementNS("http://www.w3.org/2000/svg", "polygon")
  const pointsStr = projectedCorners.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ")
  fieldElement.setAttribute("points", pointsStr)
  fieldElement.setAttribute("fill", "none")
  fieldElement.setAttribute("stroke", field.color)
  fieldElement.setAttribute("stroke-width", "1.5")
  cardsGroup.appendChild(fieldElement)
  
  const fieldCenter = projectedCorners.reduce((acc, corner) => [
    acc[0] + corner[0] / 4,
    acc[1] + corner[1] / 4
  ], [0, 0])
  
  const label = document.createElementNS("http://www.w3.org/2000/svg", "text")
  label.setAttribute("x", fieldCenter[0].toFixed(1))
  label.setAttribute("y", fieldCenter[1].toFixed(1))
  label.setAttribute("text-anchor", "middle")
  label.setAttribute("dominant-baseline", "middle")
  label.setAttribute("fill", "#fff")
  label.setAttribute("font-size", "12")
  label.setAttribute("font-weight", "bold")
  label.setAttribute("stroke", "#000")
  label.setAttribute("stroke-width", "0.5")
  label.textContent = field.index.toString()
  cardsGroup.appendChild(label)
  
  // Draw cards in this field
  drawCardsInField(field.index, field.center, cameraRad, fieldZ)
}

const drawCardsInField = (fieldIndex, fieldCenter, cameraRad, fieldZ) => {
  // Get cards from game state using domain types
  const fieldCards = gameState.value?.fields[fieldIndex]?.looseCards || []
  const cardCount = fieldCards.length
  let fieldCardWidth = W * 0.4
  let fieldCardHeight = fieldCardWidth * (30/42)
  
  if (fieldIndex === 1 || fieldIndex === 3) {
    [fieldCardWidth, fieldCardHeight] = [fieldCardHeight, fieldCardWidth]
  }
  
  if (!gameScene.value) return
  const cardsGroup = gameScene.value.querySelector('#cards')
  if (!cardsGroup) return
  
  for (let i = 0; i < cardCount; i++) {
    let cardY, cardX
    
    if (fieldIndex === 0) { // Right field - cards are landscape, use fieldCardHeight for Y positioning
      const fieldCenterY = fieldCenter.y * W
      const fieldTopY = fieldCenterY - W/2
      const fieldBottomY = fieldCenterY + W/2
      if (i === 0) cardY = fieldCenterY, cardX = fieldCenter.x * W - 25
      else if (i === 1) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W - 25
      else if (i === 2) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W - 25
      else if (i === 3) cardY = fieldCenterY, cardX = fieldCenter.x * W + 25
      else if (i === 4) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W + 25
      else if (i === 5) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W + 25
      else if (i === 6) cardY = fieldCenterY, cardX = fieldCenter.x * W + 75
      else if (i === 7) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W + 75
      else if (i === 8) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W + 75
    } else if (fieldIndex === 1) { // Back field
      if (i === 0) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W - 25
      else if (i === 1) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W - 25
      else if (i === 2) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W - 25
      else if (i === 3) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W + 25
      else if (i === 4) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W + 25
      else if (i === 5) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W + 25
      else if (i === 6) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W + 75
      else if (i === 7) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W + 75
      else if (i === 8) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W + 75
    } else if (fieldIndex === 2) { // Left field - cards are landscape, use fieldCardHeight for Y positioning
      const fieldCenterY = fieldCenter.y * W
      const fieldTopY = fieldCenterY - W/2
      const fieldBottomY = fieldCenterY + W/2
      if (i === 0) cardY = fieldCenterY, cardX = fieldCenter.x * W + 25
      else if (i === 1) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W + 25
      else if (i === 2) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W + 25
      else if (i === 3) cardY = fieldCenterY, cardX = fieldCenter.x * W - 25
      else if (i === 4) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W - 25
      else if (i === 5) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W - 25
      else if (i === 6) cardY = fieldCenterY, cardX = fieldCenter.x * W - 75
      else if (i === 7) cardY = fieldTopY + fieldCardHeight/2, cardX = fieldCenter.x * W - 75
      else if (i === 8) cardY = fieldBottomY - fieldCardHeight/2, cardX = fieldCenter.x * W - 75
    } else if (fieldIndex === 3) { // Front field
      if (i === 0) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W + 25
      else if (i === 1) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W + 25
      else if (i === 2) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W + 25
      else if (i === 3) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W - 25
      else if (i === 4) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W - 25
      else if (i === 5) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W - 25
      else if (i === 6) cardX = fieldCenter.x * W, cardY = fieldCenter.y * W - 75
      else if (i === 7) cardX = fieldCenter.x * W - W/2 + fieldCardWidth/2, cardY = fieldCenter.y * W - 75
      else if (i === 8) cardX = fieldCenter.x * W + W/2 - fieldCardWidth/2, cardY = fieldCenter.y * W - 75
    }
    
    const cardCorners = [
      [cardX - fieldCardWidth/2, cardY - fieldCardHeight/2, fieldZ],
      [cardX + fieldCardWidth/2, cardY - fieldCardHeight/2, fieldZ],
      [cardX + fieldCardWidth/2, cardY + fieldCardHeight/2, fieldZ],
      [cardX - fieldCardWidth/2, cardY + fieldCardHeight/2, fieldZ],
    ]
    
    const projectedCardCorners = cardCorners.map(([x, y, z]) => {
      const finalX = x * Math.cos(cameraRad) - y * Math.sin(cameraRad)
      const finalY = x * Math.sin(cameraRad) + y * Math.cos(cameraRad)
      return [
        (finalX - finalY) * ISO_X_FACTOR + offset[0],
        (finalX + finalY) * ISO_Y_FACTOR - (z * Z_SCALE) + offset[1]
      ]
    })
    
    // Get the actual card data if available
    const currentCard = fieldCards[i]
    const suitColors = {
      'Spring': '#3f6',   // Green  
      'Summer': '#fd4',   // Yellow
      'Autumn': '#f93',   // Orange
      'Winter': '#3af',   // Blue
      'Stars': '#fff'     // White
    }
    
    // Create card background
    const cardElement = document.createElementNS("http://www.w3.org/2000/svg", "polygon")
    const cardPointsStr = projectedCardCorners.map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`).join(" ")
    cardElement.setAttribute("points", cardPointsStr)
    cardElement.setAttribute("fill", currentCard ? suitColors[currentCard.suit] || "#fff" : "#fff")
    cardElement.setAttribute("stroke", "#000")
    cardElement.setAttribute("stroke-width", "2")
    cardsGroup.appendChild(cardElement)
    
    // Add card text if we have card data
    if (currentCard) {
      const cardCenter = projectedCardCorners.reduce((acc, corner) => [
        acc[0] + corner[0] / 4,
        acc[1] + corner[1] / 4
      ], [0, 0])
      
      const cardText = document.createElementNS("http://www.w3.org/2000/svg", "text")
      cardText.setAttribute("x", cardCenter[0].toFixed(1))
      cardText.setAttribute("y", cardCenter[1].toFixed(1))
      cardText.setAttribute("text-anchor", "middle")
      cardText.setAttribute("dominant-baseline", "middle")
      cardText.setAttribute("fill", currentCard.suit === 'Summer' || currentCard.suit === 'Stars' ? "#000" : "#fff")
      cardText.setAttribute("font-size", "8")
      cardText.setAttribute("font-weight", "bold")
      cardText.setAttribute("font-family", "monospace")
      
      // Use Card object methods for display
      const displayText = currentCard.getRankDisplay ? 
        `${currentCard.getRankDisplay()}${currentCard.getSuitSymbol ? currentCard.getSuitSymbol() : ''}` :
        `${currentCard.rank}${currentCard.suit ? currentCard.suit.charAt(0) : ''}`
      
      cardText.textContent = displayText
      cardsGroup.appendChild(cardText)
    }
  }
}

const setCameraAngle = (angle) => {
  cameraAngle = +angle
  cameraState.value.angle = cameraAngle
  update(cameraAngle, illimatAngle, 0)
}

const setIllimatAngle = (angle) => {
  illimatAngle = +angle
  cameraState.value.illimatAngle = illimatAngle
  update(cameraAngle, illimatAngle, 0)
}

const animateTo = (nextIllimatAngle) => {
  if (animating) return
  animating = true
  cameraState.value.isAnimating = true

  const start = performance.now()
  const liftDuration = 160
  const rotateDuration = 280
  const dropDuration = 120
  const settlesDuration = 200
  const total = liftDuration + rotateDuration + dropDuration + settlesDuration
  const angleStart = illimatAngle
  const angleEnd = nextIllimatAngle % 360
  const angleDelta = (angleEnd - angleStart + 360) % 360

  function frame(t) {
    const elapsed = t - start
    let angle = angleStart
    let lift = 0

    if (elapsed < liftDuration) {
      const p = elapsed / liftDuration
      const reluctant = 1 - Math.pow(1 - p, 4)
      lift = -35 * reluctant
    } else if (elapsed < liftDuration + rotateDuration) {
      const p = (elapsed - liftDuration) / rotateDuration
      const heavy = p < 0.3 ? 2 * p * p : 1 - 0.5 * Math.pow(-2 * p + 2, 2)
      angle = angleStart + angleDelta * heavy
      lift = -35
    } else if (elapsed < liftDuration + rotateDuration + dropDuration) {
      const p = (elapsed - liftDuration - rotateDuration) / dropDuration
      const gravity = p * p * p * p
      lift = -35 + (35 * gravity)
      angle = angleEnd
    } else if (elapsed < total) {
      const settleTime = elapsed - liftDuration - rotateDuration - dropDuration
      const p = settleTime / settlesDuration
      if (p < 0.3) {
        const bounceP = p / 0.3
        lift = Math.sin(bounceP * Math.PI) * 4 * (1 - bounceP)
      } else {
        lift = 0
      }
      angle = angleEnd
    } else {
      illimatAngle = angleEnd
      cameraState.value.illimatAngle = illimatAngle
      animating = false
      cameraState.value.isAnimating = false
      update(cameraAngle, illimatAngle, 0)
      return
    }

    cameraState.value.illimatAngle = angle
    update(cameraAngle, angle, lift)
    requestAnimationFrame(frame)
  }
  
  requestAnimationFrame(frame)
}

const animateNextSeason = () => {
  const next = (Math.round(illimatAngle / 90) * 90 + 90) % 360
  animateTo(next)
}

const updateOkusCount = (count) => {
  maxOkuses = +count
  okusCount.value = maxOkuses
  update(cameraAngle, illimatAngle, 0)
}

const toggleOkus = (index) => {
  if (index < maxOkuses) {
    okusState[index] = !okusState[index]
    update(cameraAngle, illimatAngle, 0)
  }
}

const addRandomCard = () => {
  if (!gameState.value) return
  
  const suits = ['Spring', 'Summer', 'Autumn', 'Winter', 'Stars']
  const ranks = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'N', 'Q', 'K', 'F']
  const values = [2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1]
  
  const randomField = Math.floor(Math.random() * 4)
  const randomRankIndex = Math.floor(Math.random() * ranks.length)
  const randomSuit = suits[Math.floor(Math.random() * suits.length)]
  
  // Create a proper Card domain object
  const newCard = new Card({
    id: Date.now(),
    rank: ranks[randomRankIndex],
    suit: randomSuit,
    value: values[randomRankIndex]
  })
  
  gameState.value.fields[randomField].looseCards.push(newCard)
  update(cameraAngle, illimatAngle, 0)
}

// Mobile hand interaction handlers
const handleCardSelected = (card) => {
  console.log('Card selected:', card)
  // TODO: Show card details or highlight playable fields
}

const handleActionSelected = ({ card, action }) => {
  console.log('Action selected:', action, 'for card:', card)
  // TODO: Highlight valid target fields for this action
}

const handleMoveReady = ({ card, action, field }) => {
  console.log('Move ready:', { card, action, field })
  // TODO: Execute move through game engine
}

onMounted(async () => {
  // Initialize game state with test data
  try {
    await createTestGame()
  } catch (error) {
    console.error('Failed to create test game:', error)
  }
  
  nextTick(() => {
    update(cameraAngle, illimatAngle, 0)
  })
})
</script>

<style scoped>
.illimat-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.5em;
}

.illimat-scene {
  border: 1px solid #444;
  overflow: visible;
}

.controls {
  margin-top: 0.5em;
  display: flex;
  gap: 0.5em;
  align-items: center;
  flex-wrap: wrap;
}

.controls input[type=range] {
  width: 100px;
}

.controls button {
  background: #444;
  color: #eee;
  border: 1px solid #666;
  padding: 0.4em 1em;
  cursor: pointer;
}

.okus-btn {
  padding: 0.3em 0.6em;
  min-width: 2em;
}

.okus-btn.active {
  background: #666;
  border-color: #888;
}

.okus-btn.disabled {
  background: #333;
  color: #666;
  border-color: #444;
  cursor: not-allowed;
}

.controls label {
  font-size: 0.9em;
  color: #eee;
}

.controls span {
  font-size: 0.9em;
  min-width: 1.5em;
  text-align: center;
  color: #eee;
}

.debug {
  margin-top: 1em;
  font-size: 0.8em;
  color: #aaa;
}
</style>