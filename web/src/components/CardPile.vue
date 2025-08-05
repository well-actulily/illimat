<template>
  <g class="card-pile" :class="pileClasses">
    <GameCard
      v-for="(card, index) in visibleCards"
      :key="`${card.id}-${index}`"
      :card="card"
      :position="getCardPosition(index)"
      :size="cardSize"
      :draggable="draggable && index === cards.length - 1"
      :selected="selectedCard === card.id"
      @drag-start="handleCardDragStart"
      @card-select="handleCardSelect"
      @card-hover="handleCardHover"
    />
    
    <!-- Pile count indicator for stockpiles -->
    <g v-if="pileType === 'stockpile' && cards.length > maxVisibleCards" class="pile-count">
      <circle
        :cx="position.x + 12"
        :cy="position.y - 8"
        r="6"
        fill="#2a1a3a"
        stroke="#aa00cc"
        stroke-width="1"
        class="count-badge"
      />
      <text
        :x="position.x + 12"
        :y="position.y - 8"
        text-anchor="middle"
        dominant-baseline="central"
        font-size="8"
        font-family="monospace"
        font-weight="bold"
        fill="#aa00cc"
        class="count-text"
      >
        {{ cards.length }}
      </text>
    </g>
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
  position: {
    type: Object,
    required: true
  },
  pileType: {
    type: String,
    default: 'loose', // 'loose', 'stockpile', 'hand', 'harvest'
    validator: value => ['loose', 'stockpile', 'hand', 'harvest'].includes(value)
  },
  draggable: {
    type: Boolean,
    default: true
  },
  maxVisibleCards: {
    type: Number,
    default: 3
  }
})

const emit = defineEmits([
  'card-drag-start',
  'card-drag',
  'card-drop',
  'card-select',
  'card-hover'
])

// Component state
const selectedCard = ref(null)

// Computed properties
const pileClasses = computed(() => [
  'card-pile',
  `pile-${props.pileType}`,
  {
    'pile-empty': props.cards.length === 0,
    'pile-single': props.cards.length === 1,
    'pile-multiple': props.cards.length > 1,
    'pile-draggable': props.draggable
  }
])

const visibleCards = computed(() => {
  if (props.pileType === 'stockpile') {
    // For stockpiles, show only the top few cards
    return props.cards.slice(-props.maxVisibleCards)
  } else {
    // For loose cards, show all
    return props.cards
  }
})

const cardSize = computed(() => {
  const sizes = {
    loose: 'normal',
    stockpile: 'normal',
    hand: 'normal',
    harvest: 'small'
  }
  return sizes[props.pileType] || 'normal'
})

// Methods
const getCardPosition = (index) => {
  const basePos = { ...props.position }
  
  if (props.pileType === 'stockpile') {
    // Stack cards with slight offset for depth
    return {
      x: basePos.x + index * 0.5,
      y: basePos.y - index * 0.3,
      z: basePos.z + index * 0.1,
      rotation: basePos.rotation + (Math.random() - 0.5) * 2 // Slight random rotation
    }
  } else if (props.pileType === 'loose') {
    // Loose cards spread out more
    const spread = 8
    return {
      x: basePos.x + (index - visibleCards.value.length / 2) * spread,
      y: basePos.y + (Math.random() - 0.5) * 4,
      z: basePos.z,
      rotation: basePos.rotation + (Math.random() - 0.5) * 5
    }
  } else {
    // Default positioning
    return {
      x: basePos.x + index * 2,
      y: basePos.y,
      z: basePos.z + index * 0.1,
      rotation: basePos.rotation
    }
  }
}

// Event handlers
const handleCardDragStart = (event) => {
  emit('card-drag-start', {
    ...event,
    pileType: props.pileType,
    pilePosition: props.position
  })
}

const handleCardSelect = (event) => {
  selectedCard.value = selectedCard.value === event.card.id ? null : event.card.id
  emit('card-select', {
    ...event,
    pileType: props.pileType,
    selected: selectedCard.value === event.card.id
  })
}

const handleCardHover = (event) => {
  emit('card-hover', {
    ...event,
    pileType: props.pileType
  })
}
</script>

<style scoped>
.card-pile {
  transition: all 0.2s ease;
}

.pile-draggable {
  cursor: pointer;
}

.pile-empty {
  opacity: 0.5;
}

.pile-multiple .game-card:not(:last-child) {
  filter: brightness(0.9);
}

.pile-stockpile .game-card:not(:last-child) {
  filter: brightness(0.8);
}

.count-badge {
  animation: badgePulse 2s ease-in-out infinite alternate;
}

.count-text {
  pointer-events: none;
}

@keyframes badgePulse {
  0% { opacity: 0.8; }
  100% { opacity: 1; }
}
</style>