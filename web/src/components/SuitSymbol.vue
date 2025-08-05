<template>
  <g class="suit-symbol" :class="symbolClasses">
    <!-- Dynamic suit symbol based on type -->
    <g v-if="suit === 'spring'" class="spring-symbol">
      <path
        :d="getSpringPath()"
        :fill="symbolColor"
        :stroke="strokeColor"
        :stroke-width="strokeWidth"
        class="spring-leaf"
      />
    </g>
    
    <g v-else-if="suit === 'summer'" class="summer-symbol">
      <circle
        :cx="size/2"
        :cy="size/2"
        :r="size/3"
        :fill="symbolColor"
        :stroke="strokeColor"
        :stroke-width="strokeWidth"
        class="summer-sun"
      />
      <g class="sun-rays">
        <line
          v-for="angle in sunRayAngles"
          :key="angle"
          :x1="size/2 + Math.cos(angle) * size/2.5"
          :y1="size/2 + Math.sin(angle) * size/2.5"
          :x2="size/2 + Math.cos(angle) * size/1.8"
          :y2="size/2 + Math.sin(angle) * size/1.8"
          :stroke="symbolColor"
          :stroke-width="strokeWidth * 0.7"
          stroke-linecap="round"
        />
      </g>
    </g>
    
    <g v-else-if="suit === 'autumn'" class="autumn-symbol">
      <path
        :d="getAutumnPath()"
        :fill="symbolColor"
        :stroke="strokeColor"
        :stroke-width="strokeWidth"
        class="autumn-leaf"
      />
    </g>
    
    <g v-else-if="suit === 'winter'" class="winter-symbol">
      <g class="snowflake">
        <line
          v-for="angle in snowflakeAngles"
          :key="angle"
          :x1="size/2"
          :y1="size/2"
          :x2="size/2 + Math.cos(angle) * size/2.5"
          :y2="size/2 + Math.sin(angle) * size/2.5"
          :stroke="symbolColor"
          :stroke-width="strokeWidth * 0.8"
          stroke-linecap="round"
        />
        <circle
          :cx="size/2"
          :cy="size/2"
          :r="size/8"
          :fill="symbolColor"
          class="snowflake-center"
        />
      </g>
    </g>
    
    <g v-else-if="suit === 'stars'" class="stars-symbol">
      <path
        :d="getStarPath()"
        :fill="symbolColor"
        :stroke="strokeColor"
        :stroke-width="strokeWidth"
        class="five-pointed-star"
      />
    </g>
    
    <!-- Fallback for unknown suits -->
    <g v-else class="unknown-symbol">
      <circle
        :cx="size/2"
        :cy="size/2"
        :r="size/3"
        fill="none"
        :stroke="symbolColor"
        :stroke-width="strokeWidth"
        stroke-dasharray="2,2"
      />
      <text
        :x="size/2"
        :y="size/2 + 2"
        text-anchor="middle"
        dominant-baseline="central"
        font-size="6"
        :fill="symbolColor"
      >
        ?
      </text>
    </g>
    
    <!-- Glowing effect -->
    <g v-if="glowing" class="glow-effect">
      <circle
        :cx="size/2"
        :cy="size/2"
        :r="size/1.5"
        fill="none"
        :stroke="glowColor"
        :stroke-width="0.5"
        opacity="0.6"
        class="suit-glow"
      />
    </g>
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  suit: {
    type: String,
    required: true,
    validator: value => ['spring', 'summer', 'autumn', 'winter', 'stars'].includes(value.toLowerCase())
  },
  size: {
    type: Number,
    default: 12
  },
  glowing: {
    type: Boolean,
    default: false
  }
})

// Computed properties
const symbolClasses = computed(() => [
  'suit-symbol',
  `suit-${props.suit.toLowerCase()}`,
  {
    'symbol-glowing': props.glowing
  }
])

const symbolColor = computed(() => {
  const colors = {
    spring: '#55cc00',    // Phosphor lime
    summer: '#ffaa00',    // Solar orange
    autumn: '#cc5500',    // Autumn rust
    winter: '#00ccaa',    // Phosphor cyan
    stars: '#aa00cc'      // Phosphor magenta
  }
  return colors[props.suit.toLowerCase()] || '#00ccaa'
})

const strokeColor = computed(() => {
  return props.glowing ? '#000' : 'none'
})

const strokeWidth = computed(() => {
  return props.glowing ? 0.5 : 0
})

const glowColor = computed(() => {
  return symbolColor.value
})

// Sun ray angles (8 rays)
const sunRayAngles = computed(() => {
  return Array.from({ length: 8 }, (_, i) => (i * Math.PI * 2) / 8)
})

// Snowflake angles (6 main branches)
const snowflakeAngles = computed(() => {
  return Array.from({ length: 6 }, (_, i) => (i * Math.PI * 2) / 6)
})

// Methods for path generation
const getSpringPath = () => {
  const s = props.size
  // Simple leaf shape
  return `M ${s/2} ${s/6} 
          Q ${s/6} ${s/3} ${s/4} ${s*2/3}
          Q ${s/2} ${s*5/6} ${s*3/4} ${s*2/3}
          Q ${s*5/6} ${s/3} ${s/2} ${s/6} Z`
}

const getAutumnPath = () => {
  const s = props.size
  // Maple leaf style
  return `M ${s/2} ${s/8}
          L ${s/4} ${s/3}
          L ${s/8} ${s/2}
          L ${s/3} ${s*2/3}
          L ${s/2} ${s*7/8}
          L ${s*2/3} ${s*2/3}
          L ${s*7/8} ${s/2}
          L ${s*3/4} ${s/3}
          L ${s/2} ${s/8} Z`
}

const getStarPath = () => {
  const s = props.size
  const cx = s / 2
  const cy = s / 2
  const outerRadius = s / 2.5
  const innerRadius = s / 5
  
  let path = ''
  
  for (let i = 0; i < 5; i++) {
    const outerAngle = (i * Math.PI * 2) / 5 - Math.PI / 2
    const innerAngle = ((i + 0.5) * Math.PI * 2) / 5 - Math.PI / 2
    
    const outerX = cx + Math.cos(outerAngle) * outerRadius
    const outerY = cy + Math.sin(outerAngle) * outerRadius
    const innerX = cx + Math.cos(innerAngle) * innerRadius
    const innerY = cy + Math.sin(innerAngle) * innerRadius
    
    if (i === 0) {
      path += `M ${outerX} ${outerY}`
    } else {
      path += ` L ${outerX} ${outerY}`
    }
    path += ` L ${innerX} ${innerY}`
  }
  
  return path + ' Z'
}
</script>

<style scoped>
.suit-symbol {
  transition: all 0.2s ease;
}

.symbol-glowing {
  filter: brightness(1.3);
}

.spring-leaf {
  animation: leafSway 3s ease-in-out infinite alternate;
}

.summer-sun {
  animation: sunGlow 2s ease-in-out infinite alternate;
}

.sun-rays {
  animation: rayRotate 4s linear infinite;
  transform-origin: 50% 50%;
}

.autumn-leaf {
  animation: leafFall 4s ease-in-out infinite alternate;
}

.snowflake {
  animation: snowflakeSparkle 2s ease-in-out infinite alternate;
}

.five-pointed-star {
  animation: starTwinkle 1.5s ease-in-out infinite alternate;
}

.suit-glow {
  animation: glowPulse 1s ease-in-out infinite alternate;
}

@keyframes leafSway {
  0% { transform: rotate(-2deg); }
  100% { transform: rotate(2deg); }
}

@keyframes sunGlow {
  0% { filter: brightness(1); }
  100% { filter: brightness(1.3); }
}

@keyframes rayRotate {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

@keyframes leafFall {
  0% { transform: translateY(-1px) rotate(-1deg); }
  100% { transform: translateY(1px) rotate(1deg); }
}

@keyframes snowflakeSparkle {
  0% { opacity: 0.8; }
  100% { opacity: 1; }
}

@keyframes starTwinkle {
  0% { opacity: 0.7; filter: brightness(1); }
  100% { opacity: 1; filter: brightness(1.4); }
}

@keyframes glowPulse {
  0% { opacity: 0.4; transform: scale(1); }
  100% { opacity: 0.8; transform: scale(1.1); }
}
</style>