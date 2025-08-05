<template>
  <div class="player-hand-mobile">
    <!-- Player hand cards fanned out skeumorphically -->
    <div class="hand-container">
      <div
        v-for="(card, index) in playerHand"
        :key="card.id"
        class="hand-card"
        :class="{ 
          'selected': selectedCardId === card.id,
          'can-play': canPlayCard(card)
        }"
        :style="getCardStyle(index, playerHand.length)"
        @click="selectCard(card)"
      >
        <!-- Card background with suit color -->
        <div 
          class="card-face" 
          :style="{ backgroundColor: getCardColor(card) }"
        >
          <!-- Card rank and suit -->
          <div class="card-rank">{{ card.getRankDisplay() }}</div>
          <div class="card-suit">{{ card.getSuitSymbol() }}</div>
          <div class="card-value">{{ card.getGameValue() }}</div>
        </div>
      </div>
    </div>

    <!-- Action popup when card is selected -->
    <div 
      v-if="selectedCard" 
      class="action-popup"
      :style="getPopupPosition()"
    >
      <button 
        class="action-btn sow"
        @click="selectAction('sow')"
        :disabled="!canSow(selectedCard)"
      >
        💧 Sow
      </button>
      <button 
        class="action-btn harvest"
        @click="selectAction('harvest')"
        :disabled="!canHarvest(selectedCard)"
      >
        🌾 Harvest
      </button>
      <button 
        class="action-btn stockpile"
        @click="selectAction('stockpile')"
        :disabled="!canStockpile(selectedCard)"
      >
        📚 Stockpile
      </button>
    </div>

    <!-- Instruction text -->
    <div class="hand-instructions">
      <div v-if="!selectedCard" class="instruction">
        Tap a card to play
      </div>
      <div v-else-if="!selectedAction" class="instruction">
        Choose an action
      </div>
      <div v-else class="instruction">
        Tap a field to {{ selectedAction }}
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed } from 'vue'

const props = defineProps({
  playerHand: {
    type: Array,
    default: () => []
  },
  gameState: {
    type: Object,
    default: null
  },
  currentPlayer: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['card-selected', 'action-selected', 'move-ready'])

// Component state
const selectedCardId = ref(null)
const selectedAction = ref(null)

// Computed properties
const selectedCard = computed(() => {
  return props.playerHand.find(card => card.id === selectedCardId.value) || null
})

// Card styling for skeumorphic fan layout
const getCardStyle = (index, totalCards) => {
  const maxSpread = 60 // degrees
  const cardWidth = 60 // pixels
  const cardSpacing = Math.min(15, 180 / Math.max(totalCards - 1, 1)) // adaptive spacing
  
  // Calculate fan spread
  const centerIndex = (totalCards - 1) / 2
  const angleOffset = (index - centerIndex) * cardSpacing
  const angle = Math.max(-maxSpread/2, Math.min(maxSpread/2, angleOffset))
  
  // Calculate position with slight arc
  const radius = 200
  const x = Math.sin(angle * Math.PI / 180) * radius
  const y = Math.cos(angle * Math.PI / 180) * radius - radius + 20
  
  // Stacking order - center cards on top
  const zIndex = totalCards - Math.abs(index - centerIndex)
  
  return {
    transform: `translate(${x}px, ${y}px) rotate(${angle}deg)`,
    zIndex: zIndex,
    '--hover-lift': selectedCardId.value === props.playerHand[index]?.id ? '-10px' : '0px'
  }
}

// Card colors based on suit
const suitColors = {
  'Spring': '#4ade80',   // Bright green
  'Summer': '#fbbf24',   // Bright yellow  
  'Autumn': '#fb923c',   // Bright orange
  'Winter': '#60a5fa',   // Bright blue
  'Stars': '#f3f4f6'     // Light gray
}

const getCardColor = (card) => {
  return suitColors[card.suit] || '#f3f4f6'
}

// Card selection logic
const selectCard = (card) => {
  if (!props.currentPlayer) return
  
  if (selectedCardId.value === card.id) {
    // Deselect if clicking same card
    selectedCardId.value = null
    selectedAction.value = null
  } else {
    // Select new card
    selectedCardId.value = card.id
    selectedAction.value = null
    emit('card-selected', card)
  }
}

// Action selection
const selectAction = (action) => {
  if (!selectedCard.value) return
  
  selectedAction.value = action
  emit('action-selected', {
    card: selectedCard.value,
    action: action
  })
}

// Move validation (simplified - will connect to game engine later)
const canPlayCard = (card) => {
  return props.currentPlayer
}

const canSow = (card) => {
  // Can always sow (discard) a card
  return true
}

const canHarvest = (card) => {
  // Can harvest if card value matches cards in any field
  // TODO: Connect to actual game logic
  return true
}

const canStockpile = (card) => {
  // Can stockpile if can combine with other cards
  // TODO: Connect to actual game logic  
  return true
}

// Popup positioning above selected card
const getPopupPosition = () => {
  if (!selectedCard.value) return {}
  
  const selectedIndex = props.playerHand.findIndex(card => card.id === selectedCardId.value)
  const cardStyle = getCardStyle(selectedIndex, props.playerHand.length)
  
  // Parse transform to get card position
  const match = cardStyle.transform.match(/translate\(([^,]+),\s*([^)]+)\)/)
  if (match) {
    const x = parseFloat(match[1])
    const y = parseFloat(match[2])
    
    return {
      left: `calc(50% + ${x}px - 60px)`, // Center popup over card
      bottom: `${Math.abs(y) + 120}px`   // Position above card
    }
  }
  
  return {
    left: '50%',
    bottom: '120px',
    transform: 'translateX(-50%)'
  }
}

// Clear selection when component unmounts or turn changes
const clearSelection = () => {
  selectedCardId.value = null
  selectedAction.value = null
}

// Expose methods to parent
defineExpose({
  clearSelection,
  selectedCard,
  selectedAction
})
</script>

<style scoped>
.player-hand-mobile {
  position: fixed;
  bottom: 20px;
  right: 20px;
  width: 300px;
  height: 160px;
  pointer-events: none;
}

.hand-container {
  position: relative;
  width: 100%;
  height: 100%;
  pointer-events: auto;
}

.hand-card {
  position: absolute;
  width: 60px;
  height: 90px;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
  transform-origin: center bottom;
}

.hand-card:hover {
  transform: translate(var(--hover-lift, 0)) rotate(var(--angle, 0deg)) translateY(-5px);
}

.hand-card.selected {
  transform: translate(var(--hover-lift, 0)) rotate(var(--angle, 0deg)) translateY(-15px) scale(1.1);
  filter: drop-shadow(0 8px 16px rgba(0, 0, 0, 0.3));
}

.card-face {
  width: 100%;
  height: 100%;
  border: 2px solid #374151;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: space-around;
  font-family: 'system-ui', monospace;
  font-weight: bold;
  text-shadow: 1px 1px 2px rgba(0, 0, 0, 0.3);
  box-shadow: 
    0 2px 4px rgba(0, 0, 0, 0.1),
    inset 0 1px 2px rgba(255, 255, 255, 0.2);
}

.card-rank {
  font-size: 12px;
  color: #1f2937;
}

.card-suit {
  font-size: 20px;
  line-height: 1;
}

.card-value {
  font-size: 10px;
  color: #6b7280;
}

/* Special styling for light backgrounds */
.card-face[style*="backgroundColor: #fbbf24"],
.card-face[style*="backgroundColor: #f3f4f6"] {
  color: #1f2937;
}

.card-face[style*="backgroundColor: #fbbf24"] .card-rank,
.card-face[style*="backgroundColor: #f3f4f6"] .card-rank {
  color: #1f2937;
}

.action-popup {
  position: absolute;
  display: flex;
  gap: 8px;
  padding: 8px;
  background: rgba(15, 23, 42, 0.95);
  border: 1px solid #475569;
  border-radius: 12px;
  backdrop-filter: blur(8px);
  pointer-events: auto;
  z-index: 1000;
}

.action-btn {
  padding: 8px 12px;
  background: #1e293b;
  border: 1px solid #475569;
  border-radius: 8px;
  color: #e2e8f0;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.action-btn:hover:not(:disabled) {
  background: #334155;
  border-color: #64748b;
  transform: translateY(-1px);
}

.action-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn.sow { border-left-color: #06b6d4; }
.action-btn.harvest { border-left-color: #eab308; }
.action-btn.stockpile { border-left-color: #8b5cf6; }

.hand-instructions {
  position: absolute;
  bottom: -30px;
  left: 50%;
  transform: translateX(-50%);
  pointer-events: none;
}

.instruction {
  font-size: 12px;
  color: #94a3b8;
  text-align: center;
  background: rgba(15, 23, 42, 0.8);
  padding: 4px 12px;
  border-radius: 12px;
  white-space: nowrap;
}

/* Mobile optimization */
@media (max-width: 768px) {
  .player-hand-mobile {
    bottom: 10px;
    right: 10px;
    width: 280px;
  }
  
  .hand-card {
    width: 50px;
    height: 75px;
  }
  
  .action-popup {
    gap: 6px;
    padding: 6px;
  }
  
  .action-btn {
    padding: 6px 10px;
    font-size: 11px;
  }
}

/* Landscape mobile optimization */
@media (max-height: 500px) {
  .player-hand-mobile {
    bottom: 5px;
    right: 5px;
    height: 120px;
    width: 250px;
  }
  
  .hand-card {
    width: 45px;
    height: 65px;
  }
}
</style>