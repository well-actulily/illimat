<template>
  <g v-if="visible" class="okus-token" :class="tokenClasses">
    <!-- Okus body -->
    <circle
      :cx="position.x"
      :cy="position.y"
      :r="tokenRadius"
      :fill="tokenFill"
      :stroke="tokenStroke"
      :stroke-width="strokeWidth"
      class="okus-body"
    />
    
    <!-- Inner mystical core -->
    <circle
      :cx="position.x"
      :cy="position.y"
      :r="tokenRadius * 0.6"
      :fill="coreFill"
      class="okus-core"
      opacity="0.8"
    />
    
    <!-- Field index marker -->
    <text
      :x="position.x"
      :y="position.y + 1"
      text-anchor="middle"
      dominant-baseline="central"
      font-size="6"
      font-family="monospace"
      font-weight="bold"
      :fill="textColor"
      class="field-marker"
    >
      {{ fieldIndex }}
    </text>
    
    <!-- Mystical aura -->
    <circle
      :cx="position.x"
      :cy="position.y"
      :r="tokenRadius + 2"
      fill="none"
      :stroke="auraColor"
      stroke-width="0.5"
      class="mystical-aura"
      opacity="0.4"
    />
  </g>
</template>

<script setup>
import { computed } from 'vue'

const props = defineProps({
  position: {
    type: Object,
    required: true
  },
  visible: {
    type: Boolean,
    default: true
  },
  fieldIndex: {
    type: Number,
    required: true
  },
  active: {
    type: Boolean,
    default: false
  },
  size: {
    type: String,
    default: 'normal', // 'small', 'normal', 'large'
    validator: value => ['small', 'normal', 'large'].includes(value)
  }
})

// Computed properties
const tokenClasses = computed(() => [
  'okus-token',
  `field-${props.fieldIndex}`,
  `size-${props.size}`,
  {
    'token-active': props.active,
    'token-inactive': !props.active
  }
])

const tokenRadius = computed(() => {
  const sizes = {
    small: 3,
    normal: 4,
    large: 5
  }
  return sizes[props.size]
})

const tokenFill = computed(() => {
  return props.active ? '#4a4a4a' : '#666'
})

const tokenStroke = computed(() => {
  return props.active ? '#aa00cc' : '#888'
})

const strokeWidth = computed(() => {
  return props.active ? 1 : 0.5
})

const coreFill = computed(() => {
  if (props.active) {
    return 'url(#okus-active-gradient)'
  }
  return '#333'
})

const textColor = computed(() => {
  return props.active ? '#aa00cc' : '#aaa'
})

const auraColor = computed(() => {
  return props.active ? '#aa00cc' : '#666'
})
</script>

<style scoped>
.okus-token {
  transition: all 0.3s ease;
}

.okus-body {
  transition: all 0.3s ease;
}

.token-active .okus-body {
  filter: brightness(1.2);
}

.token-active .mystical-aura {
  animation: okusAura 2s ease-in-out infinite alternate;
}

.field-marker {
  pointer-events: none;
  filter: drop-shadow(1px 1px 1px rgba(0, 0, 0, 0.5));
}

.okus-core {
  animation: coreGlow 3s ease-in-out infinite alternate;
}

@keyframes okusAura {
  0% { 
    opacity: 0.2; 
    transform: scale(1);
  }
  100% { 
    opacity: 0.6; 
    transform: scale(1.1);
  }
}

@keyframes coreGlow {
  0% { opacity: 0.6; }
  100% { opacity: 1; }
}
</style>

<!-- Add gradient definition for active okus -->
<style>
svg defs {
  position: absolute;
  width: 0;
  height: 0;
}
</style>