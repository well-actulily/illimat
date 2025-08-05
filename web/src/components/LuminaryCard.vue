<template>
  <g class="luminary-card" @click="handleClick">
    <!-- Card Base -->
    <rect 
      :x="x - cardWidth/2" 
      :y="y - cardHeight/2" 
      :width="cardWidth" 
      :height="cardHeight"
      :class="['card-base', { revealed: isRevealed, claimed: isClaimed }]"
      :transform="cardTransform"
    />
    
    <!-- Face-down Back -->
    <g v-if="!isRevealed" class="card-back">
      <rect 
        :x="x - cardWidth/2 + 2" 
        :y="y - cardHeight/2 + 2" 
        :width="cardWidth - 4" 
        :height="cardHeight - 4"
        class="back-pattern"
        :transform="cardTransform"
      />
      <text 
        :x="x" 
        :y="y"
        class="back-text"
        :transform="cardTransform"
      >
        ✦
      </text>
    </g>
    
    <!-- Face-up Front -->
    <g v-else class="card-front">
      <!-- Luminary Icon -->
      <text 
        :x="x" 
        :y="y - 8"
        class="luminary-icon"
        :transform="cardTransform"
      >
        {{ luminaryIcon }}
      </text>
      
      <!-- Luminary Name -->
      <text 
        :x="x" 
        :y="y + 8"
        class="luminary-name"
        :transform="cardTransform"
      >
        {{ luminaryName }}
      </text>
      
      <!-- Active Effect Indicator -->
      <circle
        v-if="hasActiveEffect"
        :cx="x + cardWidth/2 - 4"
        :cy="y - cardHeight/2 + 4"
        r="3"
        class="effect-indicator"
        :transform="cardTransform"
      />
    </g>
    
    <!-- Mystical Aura (when claimed) -->
    <circle
      v-if="isClaimed"
      :cx="x"
      :cy="y"
      :r="cardWidth * 0.8"
      class="mystical-aura"
      fill="none"
      :transform="cardTransform"
    />
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  x: { type: Number, required: true },
  y: { type: Number, required: true },
  luminary: { type: Object, required: true }, // { name, type, revealed, claimed, hasActiveEffect }
  size: { type: String, default: 'normal' }, // 'small', 'normal', 'large'
  rotation: { type: Number, default: 0 },
  interactive: { type: Boolean, default: true }
})

const emit = defineEmits(['click', 'reveal', 'claim'])

// Card dimensions based on size
const cardDimensions = computed(() => {
  const sizes = {
    small: { width: 20, height: 28 },
    normal: { width: 24, height: 32 },
    large: { width: 28, height: 38 }
  }
  return sizes[props.size] || sizes.normal
})

const cardWidth = computed(() => cardDimensions.value.width)
const cardHeight = computed(() => cardDimensions.value.height)

// Luminary state
const isRevealed = computed(() => props.luminary.revealed)
const isClaimed = computed(() => props.luminary.claimed)
const hasActiveEffect = computed(() => props.luminary.hasActiveEffect)

// Visual representation
const luminaryIcons = {
  'The Maiden': '👸',
  'The Changeling': '🎭',
  'The River': '🌊',
  'The Children': '👶',
  'The Forest Queen': '🌲',
  'The Rake': '🍂',
  'The Union': '🤝',
  'The Newborn': '🌟'
}

const luminaryIcon = computed(() => 
  luminaryIcons[props.luminary.name] || '✦'
)

const luminaryName = computed(() => {
  const name = props.luminary.name || 'Unknown'
  return name.length > 8 ? name.substring(0, 8) + '...' : name
})

const cardTransform = computed(() => 
  props.rotation ? `rotate(${props.rotation} ${props.x} ${props.y})` : ''
)

// Event handlers
const handleClick = (event) => {
  if (!props.interactive) return
  
  event.stopPropagation()
  emit('click', props.luminary)
  
  if (!isRevealed.value) {
    emit('reveal', props.luminary)
  } else if (isRevealed.value && !isClaimed.value) {
    emit('claim', props.luminary)
  }
}
</script>

<style scoped>
.luminary-card {
  cursor: pointer;
  transition: all 0.3s ease;
}

.luminary-card:not([interactive]) {
  cursor: default;
}

.card-base {
  fill: #1a0a1f;
  stroke: #aa00cc;
  stroke-width: 1.5;
  transition: all 0.3s ease;
}

.card-base.revealed {
  fill: #2a1a3f;
  stroke: #cc00dd;
  stroke-width: 2;
}

.card-base.claimed {
  fill: #3a2a4f;
  stroke: #dd11ee;
  stroke-width: 2.5;
  filter: drop-shadow(0 0 8px rgba(170, 0, 204, 0.6));
}

.card-base:hover {
  stroke: #dd11ee;
  filter: drop-shadow(0 0 6px rgba(170, 0, 204, 0.4));
}

.back-pattern {
  fill: url(#luminaryPattern);
  stroke: none;
}

.back-text {
  fill: #aa00cc;
  font-size: 16px;
  text-anchor: middle;
  dominant-baseline: middle;
  opacity: 0.7;
}

.luminary-icon {
  font-size: 14px;
  text-anchor: middle;
  dominant-baseline: middle;
  fill: #dd11ee;
  filter: drop-shadow(0 0 4px rgba(221, 17, 238, 0.6));
}

.luminary-name {
  font-size: 6px;
  text-anchor: middle;
  dominant-baseline: middle;
  fill: #aa00cc;
  font-family: 'Courier New', monospace;
  font-weight: bold;
}

.effect-indicator {
  fill: #55cc00;
  opacity: 0.8;
  animation: effectPulse 2s ease-in-out infinite;
}

.mystical-aura {
  stroke: #dd11ee;
  stroke-width: 2;
  opacity: 0.4;
  animation: auraPulse 3s ease-in-out infinite;
}

@keyframes effectPulse {
  0%, 100% { 
    opacity: 0.4;
    r: 2;
  }
  50% { 
    opacity: 1;
    r: 4;
  }
}

@keyframes auraPulse {
  0%, 100% { 
    opacity: 0.2;
    stroke-width: 1;
  }
  50% { 
    opacity: 0.6;
    stroke-width: 3;
  }
}

/* Define gradient pattern for card backs */
defs {
  background: linear-gradient(45deg, 
    rgba(170, 0, 204, 0.1) 0%, 
    rgba(85, 204, 0, 0.1) 50%, 
    rgba(170, 0, 204, 0.1) 100%);
}
</style>