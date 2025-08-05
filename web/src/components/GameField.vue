<template>
  <g class="game-field" :class="fieldClasses" @click="handleFieldClick"
     :data-cy="`game-field`"
     :data-field-index="fieldIndex"
     :data-season="season.toLowerCase()"
     :data-has-okus="hasOkus">
    <!-- Field boundary -->
    <rect
      :x="safeBounds.x"
      :y="safeBounds.y"
      :width="safeBounds.width"
      :height="safeBounds.height"
      :fill="fieldFill"
      :stroke="fieldStroke"
      :stroke-width="strokeWidth"
      class="field-boundary"
      :class="{ 
        clickable: isClickable, 
        valid: isValidTarget,
        invalid: isInvalidTarget 
      }"
      :rx="4"
    />
    
    <!-- Field label -->
    <text
      :x="safeBounds.x + safeBounds.width / 2"
      :y="safeBounds.y + 12"
      text-anchor="middle"
      :fill="labelColor"
      font-size="10"
      font-family="monospace"
      font-weight="bold"
      class="field-label"
    >
      {{ fieldLabel }}
    </text>
    
    <!-- Season indicator -->
    <circle
      :cx="safeBounds.x + safeBounds.width - 8"
      :cy="safeBounds.y + 8"
      r="4"
      :fill="seasonColor"
      :stroke="seasonStroke"
      stroke-width="1"
      class="season-indicator"
    />
    
    <!-- Okus indicator (if present) -->
    <circle
      v-if="hasOkus"
      :cx="safeBounds.x + 8"
      :cy="safeBounds.y + 8"
      r="3"
      fill="#666"
      stroke="#000"
      stroke-width="0.5"
      class="okus-indicator"
    />
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  field: {
    type: Object,
    required: true
  },
  fieldIndex: {
    type: Number,
    required: true
  },
  season: {
    type: String,
    required: true
  },
  bounds: {
    type: Object,
    default: () => ({ x: 0, y: 0, width: 50, height: 50 })
  },
  pilePositions: {
    type: Array,
    default: () => []
  },
  hasOkus: {
    type: Boolean,
    default: false
  },
  isClickable: {
    type: Boolean,
    default: false
  },
  isValidTarget: {
    type: Boolean,
    default: false
  },
  isInvalidTarget: {
    type: Boolean,
    default: false
  }
})

const emit = defineEmits(['field-click'])

// Computed properties
const safeBounds = computed(() => {
  return props.bounds || { x: 0, y: 0, width: 50, height: 50 }
})

const fieldClasses = computed(() => [
  'game-field',
  `field-${props.fieldIndex}`,
  `season-${props.season.toLowerCase()}`,
  {
    'has-okus': props.hasOkus,
    'has-cards': props.field.looseCards?.length > 0 || props.field.stockpiles?.length > 0
  }
])

const fieldLabel = computed(() => {
  const seasonName = props.season
  const restrictions = getSeasonRestrictions()
  return restrictions ? `${seasonName} (${restrictions})` : seasonName
})

const fieldFill = computed(() => {
  const alpha = props.hasOkus ? 0.15 : 0.08
  return `${seasonColor.value}${Math.round(alpha * 255).toString(16).padStart(2, '0')}`
})

const fieldStroke = computed(() => {
  return props.hasOkus ? seasonColor.value : '#4a4a4a'
})

const strokeWidth = computed(() => {
  return props.hasOkus ? 1.5 : 1
})

const labelColor = computed(() => {
  return seasonColor.value
})

const seasonColor = computed(() => {
  const colors = {
    winter: '#00ccaa',    // Phosphor cyan
    spring: '#55cc00',    // Phosphor lime
    summer: '#ffaa00',    // Solar orange
    autumn: '#cc5500'     // Autumn rust
  }
  return colors[props.season.toLowerCase()] || '#00ccaa'
})

const seasonStroke = computed(() => {
  return '#000'
})

// Methods
const getSeasonRestrictions = () => {
  const restrictions = {
    winter: 'no harvesting',
    spring: 'no stockpiling',
    summer: null,
    autumn: null
  }
  return restrictions[props.season.toLowerCase()]
}

const handleFieldClick = (event) => {
  if (!props.isClickable) return
  
  event.stopPropagation()
  emit('field-click', {
    fieldIndex: props.fieldIndex,
    fieldId: props.fieldIndex,
    season: props.season,
    bounds: safeBounds.value,
    hasOkus: props.hasOkus,
    cardCount: (props.field.looseCards?.length || 0) + (props.field.stockpiles?.length || 0),
    position: {
      x: event.offsetX,
      y: event.offsetY
    }
  })
}
</script>

<style scoped>
.game-field {
  transition: all 0.2s ease;
}

.field-boundary {
  transition: all 0.2s ease;
}

.field-boundary.clickable {
  cursor: pointer;
}

.field-boundary.clickable:hover {
  filter: brightness(1.2);
  stroke-width: 2;
}

.field-boundary.valid {
  stroke: #55cc00;
  stroke-width: 2;
  filter: drop-shadow(0 0 4px rgba(85, 204, 0, 0.4));
}

.field-boundary.invalid {
  stroke: #cc4400;
  stroke-width: 2;
  stroke-dasharray: 4,2;
  filter: drop-shadow(0 0 4px rgba(204, 68, 0, 0.4));
}

.game-field:hover .field-boundary {
  filter: brightness(1.2);
}

.game-field.has-cards .field-boundary {
  opacity: 0.9;
}

.game-field.has-okus .field-boundary {
  filter: brightness(1.3);
}

.field-label {
  pointer-events: none;
  filter: drop-shadow(1px 1px 1px rgba(0, 0, 0, 0.8));
}

.season-indicator {
  transition: all 0.2s ease;
}

.game-field:hover .season-indicator {
  filter: brightness(1.3);
  transform: scale(1.1);
}

.okus-indicator {
  animation: okusGlow 2s ease-in-out infinite alternate;
}

@keyframes okusGlow {
  0% { opacity: 0.7; }
  100% { opacity: 1; }
}
</style>