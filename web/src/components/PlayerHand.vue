<template>
  <g class="player-hand" :class="handClasses">
    <!-- Hand background -->
    <ellipse
      :cx="position.centerX"
      :cy="position.centerY"
      :rx="position.radius + 10"
      :ry="20"
      :fill="handBackground"
      :stroke="handStroke"
      :stroke-width="strokeWidth"
      class="hand-background"
      opacity="0.3"
    />
    
    <!-- Player label -->
    <text
      :x="position.centerX"
      :y="position.centerY - position.radius - 15"
      text-anchor="middle"
      font-size="10"
      font-family="monospace"
      font-weight="bold"
      :fill="labelColor"
      class="player-label"
    >
      {{ playerLabel }}
    </text>
    
    <!-- Cards in hand -->
    <GameCard
      v-for="(card, index) in cards"
      :key="`hand-${playerIndex}-${card.id}`"
      :card="card"
      :position="getCardPosition(index)"
      :size="cardSize"
      :draggable="draggable && isCurrent"
      :selected="selectedCards.includes(card.id)"
      @drag-start="handleCardDragStart"
      @card-select="handleCardSelect"
      @card-hover="handleCardHover"
    />
    
    <!-- Current player indicator -->
    <circle
      v-if="isCurrent"
      :cx="position.centerX + position.radius + 15"
      :cy="position.centerY"
      r="4"
      fill="#55cc00"
      stroke="#000"
      stroke-width="1"
      class="current-player-indicator"
    />
  </g>
</template>

<script setup>
import { computed, ref } from 'vue'
import GameCard from './GameCard.vue'

const props = defineProps({
  cards: {
    type: Array,
    required: true
  },
  playerIndex: {
    type: Number,
    required: true
  },
  isCurrent: {
    type: Boolean,
    default: false
  },
  position: {
    type: Object,
    required: true
  },
  draggable: {
    type: Boolean,
    default: true
  },
  maxSelection: {
    type: Number,
    default: 1
  }
})

const emit = defineEmits([
  'card-select',
  'card-drag-start',
  'card-hover',
  'hand-action'
])

// Component state
const selectedCards = ref([])

// Computed properties
const handClasses = computed(() => [
  'player-hand',
  `player-${props.playerIndex}`,
  {
    'hand-current': props.isCurrent,
    'hand-empty': props.cards.length === 0,
    'hand-full': props.cards.length >= 8
  }
])

const playerLabel = computed(() => {
  const baseLabel = `Player ${props.playerIndex + 1}`
  const cardCount = props.cards.length
  return `${baseLabel} (${cardCount})`
})

const handBackground = computed(() => {
  return props.isCurrent ? 'rgba(85, 204, 0, 0.1)' : 'rgba(170, 0, 204, 0.05)'
})

const handStroke = computed(() => {
  return props.isCurrent ? '#55cc00' : '#aa00cc'
})

const strokeWidth = computed(() => {
  return props.isCurrent ? 1.5 : 1
})

const labelColor = computed(() => {
  return props.isCurrent ? '#55cc00' : '#aa00cc'
})

const cardSize = computed(() => {
  if (props.cards.length > 6) return 'small'
  return 'normal'
})

// Methods
const getCardPosition = (index) => {
  if (props.cards.length === 0) return { x: 0, y: 0, rotation: 0 }
  
  const totalCards = props.cards.length
  const maxArcSpan = Math.PI * 2/3 // 120 degrees max
  const arcSpan = Math.min(maxArcSpan, totalCards * 0.25)
  const startAngle = -arcSpan / 2
  
  const angle = totalCards === 1 ? 0 : startAngle + (index / (totalCards - 1)) * arcSpan
  const radius = props.position.radius
  
  const x = props.position.centerX + radius * Math.sin(angle)
  const y = props.position.centerY + radius * Math.cos(angle)
  const rotation = angle * 180 / Math.PI
  
  return {
    x,
    y,
    z: props.position.centerZ || 0,
    rotation
  }
}

// Event handlers
const handleCardSelect = (event) => {
  const cardId = event.card.id
  
  if (selectedCards.value.includes(cardId)) {
    // Deselect card
    selectedCards.value = selectedCards.value.filter(id => id !== cardId)
  } else {
    // Select card
    if (selectedCards.value.length >= props.maxSelection) {
      // Replace selection if at max
      selectedCards.value = [cardId]
    } else {
      selectedCards.value.push(cardId)
    }
  }
  
  emit('card-select', {
    card: event.card,
    playerIndex: props.playerIndex,
    selected: selectedCards.value.includes(cardId),
    selectedCards: [...selectedCards.value]
  })
}

const handleCardDragStart = (event) => {
  // Auto-select dragged card
  if (!selectedCards.value.includes(event.card.id)) {
    selectedCards.value = [event.card.id]
  }
  
  emit('card-drag-start', {
    ...event,
    playerIndex: props.playerIndex,
    selectedCards: [...selectedCards.value]
  })
}

const handleCardHover = (event) => {
  emit('card-hover', {
    ...event,
    playerIndex: props.playerIndex
  })
}

// Public methods (for parent component)
const clearSelection = () => {
  selectedCards.value = []
}

const selectCard = (cardId) => {
  if (!selectedCards.value.includes(cardId)) {
    selectedCards.value.push(cardId)
  }
}

defineExpose({
  clearSelection,
  selectCard,
  selectedCards
})
</script>

<style scoped>
.player-hand {
  transition: all 0.3s ease;
}

.hand-background {
  transition: all 0.3s ease;
}

.hand-current .hand-background {
  filter: brightness(1.2);
}

.hand-empty .hand-background {
  opacity: 0.1;
}

.player-label {
  pointer-events: none;
  filter: drop-shadow(1px 1px 1px rgba(0, 0, 0, 0.8));
}

.current-player-indicator {
  animation: currentPlayerPulse 1.5s ease-in-out infinite alternate;
}

@keyframes currentPlayerPulse {
  0% { 
    opacity: 0.7; 
    transform: scale(1);
  }
  100% { 
    opacity: 1; 
    transform: scale(1.2);
  }
}

/* Responsive adjustments */
.hand-full .game-card {
  transform: scale(0.9);
}

.player-hand:hover .hand-background {
  opacity: 0.5;
}
</style>