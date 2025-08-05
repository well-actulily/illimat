<template>
  <g class="illimat-center" @click="handleCenterClick">
    <!-- Illimat Box Base -->
    <rect 
      :x="centerX - boxSize/2" 
      :y="centerY - boxSize/2" 
      :width="boxSize" 
      :height="boxSize"
      class="illimat-box"
      :transform="`rotate(${illimatRotation} ${centerX} ${centerY})`"
    />
    
    <!-- Directional Arrow -->
    <polygon 
      :points="arrowPoints"
      class="season-arrow"
      :transform="`rotate(${illimatRotation} ${centerX} ${centerY})`"
    />
    
    <!-- Season Labels -->
    <g class="season-labels">
      <text 
        v-for="(season, index) in seasons" 
        :key="season.name"
        :x="season.labelX" 
        :y="season.labelY"
        :class="['season-label', { active: index === currentSeasonIndex }]"
      >
        {{ season.emoji }}
      </text>
    </g>
    
    <!-- Okus Tokens -->
    <g class="okus-tokens">
      <OkusToken
        v-for="(hasOkus, index) in okusPositions"
        :key="index"
        v-show="hasOkus"
        :x="getOkusPosition(index).x"
        :y="getOkusPosition(index).y"
        :size="okusSize"
      />
    </g>
    
    <!-- Rotation Indicator -->
    <circle
      :cx="centerX"
      :cy="centerY"
      :r="boxSize * 0.6"
      class="rotation-guide"
      fill="none"
      stroke="rgba(85, 204, 0, 0.3)"
      stroke-width="1"
      stroke-dasharray="2,2"
    />
  </g>
</template>

<script setup>
import { computed } from 'vue'
import OkusToken from './OkusToken.vue'

const props = defineProps({
  centerX: { type: Number, default: 150 },
  centerY: { type: Number, default: 150 },
  illimatRotation: { type: Number, default: 0 },
  okusPositions: { type: Array, default: () => [true, true, true, true] },
  interactive: { type: Boolean, default: true }
})

const emit = defineEmits(['rotate', 'okus-click'])

// Visual constants
const boxSize = 24
const okusSize = 6
const arrowSize = 8

// Season configuration
const seasons = computed(() => [
  { name: 'Winter', emoji: '❄️', labelX: props.centerX, labelY: props.centerY - 35 },
  { name: 'Spring', emoji: '🌱', labelX: props.centerX + 35, labelY: props.centerY },
  { name: 'Summer', emoji: '☀️', labelX: props.centerX, labelY: props.centerY + 35 },
  { name: 'Autumn', emoji: '🍂', labelX: props.centerX - 35, labelY: props.centerY }
])

const currentSeasonIndex = computed(() => 
  Math.floor(props.illimatRotation / 90) % 4
)

// Arrow pointing upward from center
const arrowPoints = computed(() => {
  const { centerX, centerY } = props
  const halfArrow = arrowSize / 2
  return `${centerX},${centerY - arrowSize} ${centerX - halfArrow},${centerY} ${centerX + halfArrow},${centerY}`
})

// Okus token positions around the center
const getOkusPosition = (index) => {
  const angle = (index * 90) * Math.PI / 180
  const radius = boxSize * 0.8
  return {
    x: props.centerX + Math.cos(angle - Math.PI/2) * radius,
    y: props.centerY + Math.sin(angle - Math.PI/2) * radius
  }
}

// Event handlers
const handleCenterClick = (event) => {
  if (!props.interactive) return
  
  event.stopPropagation()
  const newRotation = (props.illimatRotation + 90) % 360
  emit('rotate', newRotation)
}
</script>

<style scoped>
.illimat-center {
  cursor: pointer;
}

.illimat-center:not([interactive]) {
  cursor: default;
}

.illimat-box {
  fill: #1a0a1f;
  stroke: #55cc00;
  stroke-width: 2;
  transition: all 0.3s ease;
}

.illimat-box:hover {
  fill: #2a1a2f;
  stroke: #66dd11;
  filter: drop-shadow(0 0 8px rgba(85, 204, 0, 0.4));
}

.season-arrow {
  fill: #00ccaa;
  stroke: #00ccaa;
  stroke-width: 1;
  filter: drop-shadow(0 0 4px rgba(0, 204, 170, 0.6));
}

.season-labels {
  pointer-events: none;
  user-select: none;
}

.season-label {
  font-size: 14px;
  text-anchor: middle;
  dominant-baseline: middle;
  fill: #666;
  transition: all 0.3s ease;
}

.season-label.active {
  fill: #55cc00;
  font-size: 16px;
  filter: drop-shadow(0 0 6px rgba(85, 204, 0, 0.8));
}

.rotation-guide {
  opacity: 0;
  transition: opacity 0.3s ease;
}

.illimat-center:hover .rotation-guide {
  opacity: 1;
}

.okus-tokens {
  pointer-events: none;
}
</style>