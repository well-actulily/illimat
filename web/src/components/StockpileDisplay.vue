<template>
  <g class="stockpile-display" @click="handleStockpileClick">
    <!-- Stockpile Base -->
    <rect 
      :x="x - width/2" 
      :y="y - height/2" 
      :width="width" 
      :height="height"
      class="stockpile-base"
      :class="{ interactable: canHarvest, invalid: !isValid }"
    />
    
    <!-- Component Cards (stacked visual) -->
    <g class="component-cards">
      <rect
        v-for="(card, index) in displayCards"
        :key="`${card.id}-${index}`"
        :x="x - cardWidth/2 + index * stackOffset"
        :y="y - cardHeight/2 - index * stackOffset"
        :width="cardWidth"
        :height="cardHeight"
        :class="['stockpile-card', `suit-${card.suit.toLowerCase()}`]"
        :style="{ zIndex: index }"
      />
    </g>
    
    <!-- Total Value Display -->
    <g class="value-display">
      <circle
        :cx="x + width/2 - 8"
        :cy="y - height/2 + 8"
        r="10"
        class="value-badge"
      />
      <text
        :x="x + width/2 - 8"
        :y="y - height/2 + 8"
        class="value-text"
      >
        {{ totalValue }}
      </text>
    </g>
    
    <!-- Card Count (if more than visible) -->
    <g v-if="cards.length > maxDisplayCards" class="count-display">
      <rect
        :x="x - width/2 + 2"
        :y="y + height/2 - 12"
        width="16"
        height="10"
        class="count-badge"
        rx="2"
      />
      <text
        :x="x - width/2 + 10"
        :y="y + height/2 - 7"
        class="count-text"
      >
        {{ cards.length }}
      </text>
    </g>
    
    <!-- Harvest indicator -->
    <g v-if="canHarvest" class="harvest-indicator">
      <circle
        :cx="x"
        :cy="y + height/2 + 8"
        r="6"
        class="harvest-glow"
      />
      <text
        :x="x"
        :y="y + height/2 + 8"
        class="harvest-icon"
      >
        🌾
      </text>
    </g>
    
    <!-- Invalid indicator -->
    <g v-if="!isValid" class="invalid-indicator">
      <line
        :x1="x - width/2"
        :y1="y - height/2"
        :x2="x + width/2"
        :y2="y + height/2"
        class="invalid-strike"
      />
      <line
        :x1="x + width/2"
        :y1="y - height/2"
        :x2="x - width/2"
        :y2="y + height/2"
        class="invalid-strike"
      />
    </g>
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  x: { type: Number, required: true },
  y: { type: Number, required: true },
  cards: { type: Array, required: true },
  canHarvest: { type: Boolean, default: false },
  maxDisplayCards: { type: Number, default: 3 },
  interactive: { type: Boolean, default: true }
})

const emit = defineEmits(['stockpile-click', 'harvest-attempt'])

// Visual constants
const width = 40
const height = 28
const cardWidth = 16
const cardHeight = 20
const stackOffset = 1.5

// Computed properties  
const totalValue = computed(() => 
  props.cards.reduce((sum, card) => sum + card.gameValue, 0)
)

const isValid = computed(() => 
  props.cards.length > 1 && totalValue.value <= 14
)

const displayCards = computed(() => 
  props.cards.slice(-props.maxDisplayCards)
)

// Event handlers
const handleStockpileClick = (event) => {
  if (!props.interactive) return
  
  event.stopPropagation()
  
  if (props.canHarvest && isValid.value) {
    emit('harvest-attempt', {
      stockpile: props.cards,
      totalValue: totalValue.value,
      position: { x: props.x, y: props.y }
    })
  } else {
    emit('stockpile-click', {
      stockpile: props.cards,
      totalValue: totalValue.value,
      position: { x: props.x, y: props.y }
    })
  }
}
</script>

<style scoped>
.stockpile-display {
  cursor: default;
  transition: all 0.3s ease;
}

.stockpile-display.interactable {
  cursor: pointer;
}

.stockpile-base {
  fill: rgba(20, 10, 30, 0.8);
  stroke: #666;
  stroke-width: 1.5;
  stroke-dasharray: 3,2;
  transition: all 0.3s ease;
}

.stockpile-base.interactable {
  stroke: #55cc00;
  stroke-dasharray: none;
}

.stockpile-base.interactable:hover {
  stroke: #66dd11;
  stroke-width: 2;
  filter: drop-shadow(0 0 6px rgba(85, 204, 0, 0.4));
}

.stockpile-base.invalid {
  stroke: #cc4400;
  stroke-dasharray: 2,2;
}

.stockpile-card {
  fill: #1a1a2a;
  stroke-width: 1;
  transition: all 0.2s ease;
}

.stockpile-card.suit-spring { stroke: #4a8c2a; }
.stockpile-card.suit-summer { stroke: #cc8800; }
.stockpile-card.suit-autumn { stroke: #aa4400; }
.stockpile-card.suit-winter { stroke: #0066cc; }
.stockpile-card.suit-stars { stroke: #888888; }

.value-badge {
  fill: #2a1a3a;
  stroke: #aa00cc;
  stroke-width: 1.5;
}

.value-text {
  fill: #aa00cc;
  font-size: 10px;
  font-weight: bold;
  text-anchor: middle;
  dominant-baseline: middle;
  font-family: 'Courier New', monospace;
}

.count-badge {
  fill: rgba(10, 10, 20, 0.9);
  stroke: #666;
  stroke-width: 1;
}

.count-text {
  fill: #aaa;
  font-size: 7px;
  font-family: monospace;
  text-anchor: middle;
  dominant-baseline: middle;
}

.harvest-glow {
  fill: rgba(85, 204, 0, 0.3);
  stroke: #55cc00;
  stroke-width: 2;
  animation: harvestPulse 2s ease-in-out infinite;
}

.harvest-icon {
  font-size: 8px;
  text-anchor: middle;
  dominant-baseline: middle;
  pointer-events: none;
}

.invalid-strike {
  stroke: #cc4400;
  stroke-width: 2;
  opacity: 0.8;
}

@keyframes harvestPulse {
  0%, 100% { 
    opacity: 0.6;
    r: 5;
  }
  50% { 
    opacity: 1;
    r: 7;
  }
}
</style>