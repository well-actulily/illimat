<script setup>
import { computed } from 'vue'

const props = defineProps({
  rank: {
    type: [String, Number],
    required: true,
    validator: (value) => {
      const validRanks = ['F', '2', '3', '4', '5', '6', '7', '8', '9', '10', 'N', 'Q', 'K']
      return validRanks.includes(String(value))
    }
  },
  suit: {
    type: String,
    required: true,
    validator: (value) => ['hearts', 'diamonds', 'clubs', 'spades'].includes(value)
  },
  state: {
    type: String,
    default: 'face-up',
    validator: (value) => ['face-up', 'face-down'].includes(value)
  }
})

// Convert rank to number for pip display
const rankNumber = computed(() => {
  const rank = String(props.rank)
  if (rank === 'F') return 1
  if (rank === 'N') return 11
  if (rank === 'Q') return 12
  if (rank === 'K') return 13
  return parseInt(rank)
})

// Determine which pips to show (only for ranks 2-10)
const showPips = computed(() => {
  return rankNumber.value >= 2 && rankNumber.value <= 10
})

// Get array of pip indices to display
const activePips = computed(() => {
  if (!showPips.value) return []
  
  const pips = []
  for (let i = 1; i <= rankNumber.value; i++) {
    pips.push(i)
  }
  return pips
})

// Suit colors
const suitColor = computed(() => {
  return ['hearts', 'diamonds'].includes(props.suit) ? '#dc2626' : '#181a1b'
})

// Rank display text
const rankText = computed(() => {
  return String(props.rank).toUpperCase()
})
</script>

<template>
  <g class="game-card" :class="{ 'face-down': state === 'face-down' }">
    <!-- Card background -->
    <rect 
      id="card-face" 
      width="200" 
      height="280" 
      rx="8" 
      ry="8" 
      :style="state === 'face-down' ? 'fill: #4b5563;' : 'fill: #fff;'"
    />
    
    <!-- Card back pattern (shown when face-down) -->
    <g v-if="state === 'face-down'" id="card-back">
      <rect x="10" y="10" width="180" height="260" rx="4" ry="4" style="fill: #6b7280; stroke: #374151; stroke-width: 2;"/>
      <pattern id="card-back-pattern" patternUnits="userSpaceOnUse" width="20" height="20">
        <rect width="20" height="20" style="fill: #6b7280;"/>
        <circle cx="10" cy="10" r="3" style="fill: #9ca3af;"/>
      </pattern>
      <rect x="15" y="15" width="170" height="250" style="fill: url(#card-back-pattern);"/>
    </g>
    
    <!-- Card front (shown when face-up) -->
    <g v-else id="card-front">
      <!-- Pips (for ranks 1-10) -->
      <g v-if="showPips" id="pips">
        <rect 
          v-for="pip in activePips" 
          :key="`pip-${pip}`"
          :id="`pip-${pip}`"
          :x="pip === 1 || pip === 10 ? 88 : (pip % 2 === 0 ? 71 : 105)"
          :y="43 + Math.floor((pip - 1) / 2) * 34"
          width="24" 
          height="24" 
          :transform="pip <= 5 ? '' : 'translate(200 450) rotate(180)'"
          :style="`fill: ${suitColor};`"
        />
      </g>
      
      <!-- Suit and rank display -->
      <g id="suit-rank">
        <line 
          id="rank-underln" 
          x1="24" 
          y1="33" 
          x2="34" 
          y2="33" 
          style="fill: none; stroke: #231f20; stroke-miterlimit: 10; stroke-width: .75px;"
        />
        <text 
          id="rank" 
          transform="translate(17.259 28.997) scale(.819 1)" 
          :style="`fill: ${suitColor}; font-family: 'EB Garamond', serif; font-size: 27.931px; font-weight: 600;`"
        >
          <tspan x="0" y="0">{{ rankText }}</tspan>
        </text>
        <rect 
          id="suit" 
          x="13" 
          y="38" 
          width="32" 
          height="32" 
          :style="`fill: ${suitColor};`"
        />
      </g>
      
      <!-- Face cards get special treatment -->
      <g v-if="!showPips" id="face-card">
        <text 
          transform="translate(100 150)" 
          text-anchor="middle" 
          :style="`fill: ${suitColor}; font-family: 'EB Garamond', serif; font-size: 48px; font-weight: 700;`"
        >
          {{ rankText }}
        </text>
        <rect 
          x="84" 
          y="160" 
          width="32" 
          height="32" 
          :style="`fill: ${suitColor};`"
        />
      </g>
    </g>
  </g>
</template>

<style scoped>
.game-card {
  cursor: pointer;
  transition: transform 0.2s ease;
}

.game-card:hover {
  transform: scale(1.05);
}

.face-down {
  opacity: 0.9;
}
</style>