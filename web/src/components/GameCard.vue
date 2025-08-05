<template>
  <g 
    :class="cardClasses"
    :transform="cardTransform"
    @mousedown="handleMouseDown"
    @mouseover="handleMouseOver"
    @mouseout="handleMouseOut"
  >
    <!-- Card body -->
    <rect 
      :width="cardDimensions.width"
      :height="cardDimensions.height"
      :rx="cardRadius"
      :fill="cardFill"
      :stroke="cardStroke"
      :stroke-width="strokeWidth"
      class="card-body"
    />
    
    <!-- Mystical border for face cards -->
    <rect 
      v-if="isFaceCard"
      :x="1" :y="1"
      :width="cardDimensions.width - 2" 
      :height="cardDimensions.height - 2"
      :rx="cardRadius - 1"
      fill="none"
      :stroke="mysticalBorderColor"
      stroke-width="0.5"
      class="face-card-border"
      opacity="0.8"
    />
    
    <!-- Rank display -->
    <text 
      :x="rankPosition.x" 
      :y="rankPosition.y"
      :fill="rankColor"
      :font-size="rankFontSize"
      font-family="monospace"
      font-weight="bold"
      text-anchor="middle"
      dominant-baseline="central"
      class="card-rank"
    >
      {{ displayRank }}
    </text>
    
    <!-- Suit symbol -->
    <g :transform="`translate(${suitPosition.x}, ${suitPosition.y})`">
      <SuitSymbol 
        :suit="card.suit" 
        :size="suitSize"
        :glowing="isHovered || isSelected"
      />
    </g>
    
    <!-- Mystical glow effect -->
    <rect 
      v-if="isGlowing"
      :width="cardDimensions.width"
      :height="cardDimensions.height"
      :rx="cardRadius"
      fill="url(#cardGlow)"
      opacity="0.4"
      class="card-glow"
      pointer-events="none"
    />
    
    <!-- Selection indicator -->
    <rect 
      v-if="isSelected"
      :width="cardDimensions.width + 2"
      :height="cardDimensions.height + 2"
      :x="-1" :y="-1"
      :rx="cardRadius + 1"
      fill="none"
      stroke="#55cc00"
      stroke-width="2"
      class="selection-ring"
      opacity="0.8"
    />
  </g>
</template>

<script setup>
import { computed, ref } from 'vue'
import SuitSymbol from './SuitSymbol.vue'

const props = defineProps({
  card: {
    type: Object,
    required: true
  },
  position: {
    type: Object,
    default: () => ({ x: 0, y: 0, z: 0, rotation: 0 })
  },
  draggable: {
    type: Boolean,
    default: false
  },
  selected: {
    type: Boolean,
    default: false
  },
  size: {
    type: String,
    default: 'normal', // 'small', 'normal', 'large'
    validator: value => ['small', 'normal', 'large'].includes(value)
  }
})

const emit = defineEmits([
  'drag-start',
  'card-select',
  'card-hover'
])

// Component state
const isHovered = ref(false)
const isDragging = ref(false)

// Computed properties
const cardDimensions = computed(() => {
  const sizes = {
    small: { width: 12, height: 16 },
    normal: { width: 15, height: 20 },
    large: { width: 18, height: 24 }
  }
  return sizes[props.size]
})

const cardRadius = computed(() => {
  return cardDimensions.value.width * 0.1
})

const cardTransform = computed(() => {
  const pos = props.position
  return `translate(${pos.x}, ${pos.y}) rotate(${pos.rotation || 0})`
})

const cardClasses = computed(() => {
  return [
    'game-card',
    `card-${props.card.suit.toLowerCase()}`,
    {
      'card-draggable': props.draggable,
      'card-face': isFaceCard.value,
      'card-hovered': isHovered.value,
      'card-selected': props.selected,
      'card-dragging': isDragging.value
    }
  ]
})

const isFaceCard = computed(() => {
  return ['F', 'N', 'Q', 'K'].includes(props.card.rank)
})

const isSelected = computed(() => props.selected)

const isGlowing = computed(() => {
  return isHovered.value || props.selected || isDragging.value
})

const displayRank = computed(() => {
  const rankMap = {
    'F': 'F',  // Fool
    'N': 'N',  // Knight  
    'Q': 'Q',  // Queen
    'K': 'K',  // King
    'T': '10'  // Ten
  }
  return rankMap[props.card.rank] || props.card.rank
})

const cardFill = computed(() => {
  if (isFaceCard.value) {
    return 'url(#faceCardGradient)'
  }
  return getSeasonalCardColor()
})

const cardStroke = computed(() => {
  if (isHovered.value || props.selected) {
    return getSuitColor()
  }
  return '#4a4a4a'
})

const strokeWidth = computed(() => {
  return isHovered.value || props.selected ? 1.5 : 1
})

const mysticalBorderColor = computed(() => {
  return getSuitColor()
})

const rankColor = computed(() => {
  return getSuitColor()
})

const rankFontSize = computed(() => {
  return cardDimensions.value.width * 0.4
})

const rankPosition = computed(() => {
  return {
    x: cardDimensions.value.width * 0.2,
    y: cardDimensions.value.height * 0.25
  }
})

const suitPosition = computed(() => {
  return {
    x: cardDimensions.value.width * 0.75,
    y: cardDimensions.value.height * 0.7
  }
})

const suitSize = computed(() => {
  return cardDimensions.value.width * 0.3
})

// Methods
const getSuitColor = () => {
  const colors = {
    spring: '#55cc00',    // Phosphor lime
    summer: '#ffaa00',    // Solar orange
    autumn: '#cc5500',    // Autumn rust
    winter: '#00ccaa',    // Phosphor cyan
    stars: '#aa00cc'      // Phosphor magenta
  }
  return colors[props.card.suit.toLowerCase()] || '#00ccaa'
}

const getSeasonalCardColor = () => {
  const baseColors = {
    spring: 'rgba(85, 204, 0, 0.1)',
    summer: 'rgba(255, 170, 0, 0.1)', 
    autumn: 'rgba(204, 85, 0, 0.1)',
    winter: 'rgba(0, 204, 170, 0.1)',
    stars: 'rgba(170, 0, 204, 0.1)'
  }
  
  const base = baseColors[props.card.suit.toLowerCase()] || 'rgba(0, 204, 170, 0.1)'
  
  if (isHovered.value || props.selected) {
    return base.replace('0.1', '0.3')
  }
  
  return base
}

// Event handlers
const handleMouseDown = (event) => {
  if (!props.draggable) return
  
  event.preventDefault()
  isDragging.value = true
  
  emit('drag-start', {
    card: props.card,
    position: props.position,
    mouseEvent: event
  })
}

const handleMouseOver = () => {
  isHovered.value = true
  emit('card-hover', {
    card: props.card,
    hovered: true
  })
}

const handleMouseOut = () => {
  isHovered.value = false
  emit('card-hover', {
    card: props.card,
    hovered: false
  })
}
</script>

<style scoped>
.game-card {
  cursor: pointer;
  transition: all 0.2s ease;
}

.card-draggable {
  cursor: grab;
}

.card-dragging {
  cursor: grabbing;
  filter: brightness(1.3);
}

.card-hovered {
  filter: brightness(1.2);
}

.card-body {
  transition: all 0.2s ease;
}

.card-rank {
  pointer-events: none;
  filter: drop-shadow(1px 1px 1px rgba(0, 0, 0, 0.5));
}

.face-card-border {
  animation: mysticalPulse 2s ease-in-out infinite;
}

.card-glow {
  animation: glowPulse 1.5s ease-in-out infinite alternate;
}

.selection-ring {
  animation: selectionPulse 1s ease-in-out infinite alternate;
}

@keyframes mysticalPulse {
  0%, 100% { opacity: 0.6; }
  50% { opacity: 1; }
}

@keyframes glowPulse {
  0% { opacity: 0.2; }
  100% { opacity: 0.6; }
}

@keyframes selectionPulse {
  0% { opacity: 0.6; }
  100% { opacity: 1; }
}
</style>