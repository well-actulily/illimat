<template>
  <g class="debug-overlay">
    <!-- Projection points -->
    <g class="projection-points">
      <circle
        v-for="(point, index) in projections"
        :key="index"
        :cx="point[0]"
        :cy="point[1]"
        r="2"
        fill="#ff0000"
        class="projection-point"
      />
    </g>
    
    <!-- Camera info -->
    <g class="debug-info">
      <rect
        x="10" y="10"
        width="180" height="60"
        fill="rgba(0, 0, 0, 0.8)"
        stroke="#55cc00"
        stroke-width="1"
        rx="4"
      />
      
      <text x="15" y="25" font-size="8" fill="#55cc00" font-family="monospace">
        Camera: {{ Math.round(cameraAngle) }}°
      </text>
      <text x="15" y="35" font-size="8" fill="#55cc00" font-family="monospace">
        Illimat: {{ Math.round(illimatAngle) }}°
      </text>
      <text x="15" y="45" font-size="8" fill="#55cc00" font-family="monospace">
        Season: {{ getCurrentSeason() }}
      </text>
      <text x="15" y="55" font-size="8" fill="#55cc00" font-family="monospace">
        Quadrant: {{ getQuadrant() }}
      </text>
    </g>
    
    <!-- Grid lines -->
    <g class="debug-grid" opacity="0.3">
      <defs>
        <pattern id="grid" width="20" height="20" patternUnits="userSpaceOnUse">
          <path d="M 20 0 L 0 0 0 20" fill="none" stroke="#666" stroke-width="0.5"/>
        </pattern>
      </defs>
      <rect width="300" height="300" fill="url(#grid)" />
    </g>
    
    <!-- Center crosshair -->
    <g class="debug-crosshair" opacity="0.5">
      <line x1="150" y1="140" x2="150" y2="160" stroke="#ff0000" stroke-width="1"/>
      <line x1="140" y1="150" x2="160" y2="150" stroke="#ff0000" stroke-width="1"/>
    </g>
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  projections: {
    type: Array,
    default: () => []
  },
  cameraAngle: {
    type: Number,
    default: 90
  },
  illimatAngle: {
    type: Number,
    default: 0
  }
})

// Methods
const getCurrentSeason = () => {
  const seasons = ['Winter', 'Spring', 'Summer', 'Autumn']
  const index = Math.floor(props.illimatAngle / 90) % 4
  return seasons[index]
}

const getQuadrant = () => {
  const totalAngle = (props.cameraAngle + props.illimatAngle) % 360
  return Math.floor(((totalAngle + 45) % 360) / 90)
}
</script>

<style scoped>
.debug-overlay {
  pointer-events: none;
  opacity: 0.7;
}

.projection-point {
  animation: debugPulse 2s ease-in-out infinite alternate;
}

@keyframes debugPulse {
  0% { opacity: 0.5; }
  100% { opacity: 1; }
}
</style>