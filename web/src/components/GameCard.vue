<template>
  <g class="game-card" :class="{ 'is-draggable': draggable, 'is-selected': selected }" 
     @click="handleClick" @mousedown="handleMouseDown"
     data-cy="game-card">
    <!-- Card background -->
    <polygon
      :points="cardPoints"
      :fill="cardFill"
      :stroke="cardStroke"
      :stroke-width="strokeWidth"
      class="card-background"
    />
    
    <!-- Card rank and suit -->
    <text
      v-if="showDetails"
      :x="textPosition.x"
      :y="textPosition.y"
      :font-size="fontSize"
      :fill="textColor"
      text-anchor="middle"
      dominant-baseline="middle"
      class="card-text"
    >
      {{ cardDisplay }}
    </text>
    
    <!-- Card value indicator (small number) -->
    <text
      v-if="showValue"
      :x="valuePosition.x"
      :y="valuePosition.y"
      :font-size="valueFontSize"
      :fill="valueColor"
      text-anchor="middle"
      dominant-baseline="middle"
      class="card-value"
    >
      {{ card.getGameValue() }}
    </text>
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  card: {
    type: Object,
    required: true
  },
  position: {
    type: Object,
    required: true,
    // Expected: { corners: [[x,y], [x,y], [x,y], [x,y]] }
  },
  draggable: {
    type: Boolean,
    default: false
  },
  selected: {
    type: Boolean,
    default: false
  },
  showDetails: {
    type: Boolean,
    default: true
  },
  showValue: {
    type: Boolean,
    default: true
  },
  size: {
    type: String,
    default: 'normal', // 'small', 'normal', 'large'
  }
})

const emit = defineEmits(['click', 'drag-start', 'drag-end'])

// Card visual properties based on suit
const suitColors = {
  'Spring': '#3f6',   // Green  
  'Summer': '#fd4',   // Yellow
  'Autumn': '#f93',   // Orange
  'Winter': '#3af',   // Blue
  'Stars': '#fff'     // White
}

const cardPoints = computed(() => {
  if (!props.position.corners) return '0,0 10,0 10,15 0,15'
  return props.position.corners
    .map(([x, y]) => `${x.toFixed(1)},${y.toFixed(1)}`)
    .join(' ')
})

const cardFill = computed(() => {
  if (props.selected) return '#fff'
  return suitColors[props.card.suit] || '#fff'
})

const cardStroke = computed(() => {
  if (props.selected) return '#ff0'
  if (props.draggable) return '#888'
  return '#000'
})

const strokeWidth = computed(() => {
  if (props.selected) return '3'
  if (props.draggable) return '2'
  return '1'
})

const textColor = computed(() => {
  // Use contrasting color on light backgrounds
  const lightSuits = ['Summer', 'Stars']
  return lightSuits.includes(props.card.suit) ? '#000' : '#fff'
})

const valueColor = computed(() => {
  return props.selected ? '#000' : '#fff'
})

const fontSize = computed(() => {
  const sizes = { small: '8', normal: '10', large: '14' }
  return sizes[props.size] || sizes.normal
})

const valueFontSize = computed(() => {
  const sizes = { small: '6', normal: '8', large: '10' }
  return sizes[props.size] || sizes.normal
})

// Calculate text position (center of card)
const textPosition = computed(() => {
  if (!props.position.corners) return { x: 5, y: 7.5 }
  
  const corners = props.position.corners
  const centerX = corners.reduce((sum, [x]) => sum + x, 0) / 4
  const centerY = corners.reduce((sum, [, y]) => sum + y, 0) / 4
  
  return { x: centerX, y: centerY }
})

// Calculate value position (top-left corner)
const valuePosition = computed(() => {
  if (!props.position.corners) return { x: 2, y: 3 }
  
  const corners = props.position.corners
  const minX = Math.min(...corners.map(([x]) => x))
  const minY = Math.min(...corners.map(([, y]) => y))
  
  return { x: minX + 3, y: minY + 5 }
})

// Card display text (rank + suit symbol)
const cardDisplay = computed(() => {
  const rank = props.card.getRankDisplay()
  const suit = props.card.getSuitSymbol()
  
  // For face cards, show both rank and suit
  if (props.card.isFaceCard()) {
    return `${rank.charAt(0)}${suit}`
  }
  
  // For number cards, show value and suit
  return `${props.card.getGameValue()}${suit}`
})

// Event handlers
const handleClick = (event) => {
  event.stopPropagation()
  emit('click', { 
    card: props.card, 
    cardId: props.card.id,
    suit: props.card.suit,
    rank: props.card.rank,
    gameValue: props.card.getGameValue(),
    event 
  })
}

const handleMouseDown = (event) => {
  if (props.draggable) {
    event.stopPropagation()
    emit('drag-start', { 
      card: props.card, 
      cardId: props.card.id,
      position: props.position,
      event 
    })
  }
}
</script>

<style scoped>
.game-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.game-card.is-draggable {
  cursor: grab;
}

.game-card.is-draggable:active {
  cursor: grabbing;
}

.game-card.is-selected {
  filter: brightness(1.2);
}

.card-background {
  transition: all 0.2s ease;
}

.game-card:hover .card-background {
  filter: brightness(1.1);
}

.card-text {
  font-family: monospace;
  font-weight: bold;
  pointer-events: none;
  text-shadow: 1px 1px 1px rgba(0, 0, 0, 0.8);
}

.card-value {
  font-family: monospace;
  font-weight: bold;
  pointer-events: none;
  text-shadow: 1px 1px 1px rgba(0, 0, 0, 0.8);
}
</style>