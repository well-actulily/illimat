<script setup>
import { ref } from 'vue'
import IllimatDevice from './IllimatDevice.vue'
import GameField from './GameField.vue'

const props = defineProps({
  cameraRotation: {
    type: Number,
    default: 0
  }
})

const emit = defineEmits(['update:cameraRotation'])

const isDragging = ref(false)
const startAngle = ref(0)
const startRotation = ref(0)

// Get center point of the mat (260, 260 in SVG coordinates)
const centerX = 260
const centerY = 260

const getAngleFromEvent = (event) => {
  // Get SVG element to convert coordinates
  const svg = event.target.closest('svg')
  if (!svg) return 0
  
  // Get client coordinates from mouse or touch
  const clientX = event.clientX || (event.touches && event.touches[0]?.clientX) || 0
  const clientY = event.clientY || (event.touches && event.touches[0]?.clientY) || 0
  
  // Convert screen coordinates to SVG coordinates
  const rect = svg.getBoundingClientRect()
  const svgX = ((clientX - rect.left) / rect.width) * 520 // SVG viewBox width
  const svgY = ((clientY - rect.top) / rect.height) * 465 // SVG viewBox height
  
  // Calculate angle from center to mouse/touch position
  const deltaX = svgX - centerX
  const deltaY = svgY - centerY
  
  return Math.atan2(deltaY, deltaX) * (180 / Math.PI)
}

const handlePointerDown = (event) => {
  // Don't start dragging if clicking on the Illimat device
  if (event.target.closest('#illimat')) {
    return
  }
  
  isDragging.value = true
  startAngle.value = getAngleFromEvent(event)
  startRotation.value = props.cameraRotation
  
  // Prevent default to avoid text selection
  event.preventDefault()
}

const handlePointerMove = (event) => {
  if (!isDragging.value) return
  
  const currentAngle = getAngleFromEvent(event)
  const angleDelta = currentAngle - startAngle.value
  
  // Apply rotation delta to starting rotation
  let newRotation = startRotation.value + angleDelta
  
  // Normalize angle to 0-360 range
  newRotation = ((newRotation % 360) + 360) % 360
  
  emit('update:cameraRotation', newRotation)
  
  event.preventDefault()
}

const handlePointerUp = () => {
  if (isDragging.value) {
    isDragging.value = false
    
    // Snap to nearest 15-degree interval
    const currentRotation = props.cameraRotation
    const snapAngle = Math.round(currentRotation / 15) * 15
    
    // Animate to snap position
    animateToAngle(snapAngle)
  }
}

const animateToAngle = (targetAngle) => {
  const start = performance.now()
  const duration = 200 // 200ms snap animation
  const startAngle = props.cameraRotation
  const angleDelta = ((targetAngle - startAngle + 180) % 360) - 180 // Shortest path
  
  const animate = (timestamp) => {
    const elapsed = timestamp - start
    const progress = Math.min(elapsed / duration, 1)
    
    // Ease out cubic for smooth deceleration
    const easeProgress = 1 - Math.pow(1 - progress, 3)
    
    const currentAngle = startAngle + (angleDelta * easeProgress)
    const normalizedAngle = ((currentAngle % 360) + 360) % 360
    
    emit('update:cameraRotation', normalizedAngle)
    
    if (progress < 1) {
      requestAnimationFrame(animate)
    }
  }
  
  requestAnimationFrame(animate)
}


// Add event listeners for both mouse and touch
const matElement = ref(null)
</script>

<template>
  <g id="mat" 
     @mousedown="handlePointerDown"
     @mousemove="handlePointerMove" 
     @mouseup="handlePointerUp"
     @touchstart="handlePointerDown"
     @touchmove="handlePointerMove"
     @touchend="handlePointerUp">
    <rect 
      id="mat-bg" 
      x="0" 
      y="0" 
      width="520" 
      height="520" 
      transform="translate(520 0) rotate(90)" 
      class="blk-fill" 
    />
    <g id="outer-square">
      <path 
        d="M495,25v470H25V25h470M500,20H20v480h480V20h0Z" 
        class="wht-fill" 
      />
    </g>
    <rect 
      id="illimat-space" 
      x="195" 
      y="195" 
      width="130" 
      height="130" 
      transform="translate(627.696 260) rotate(135)" 
      class="wht-stroke hvy-stroke" 
    />
    <GameField position="SW" />
    <GameField position="NW" />
    <GameField position="NE" />
    <GameField position="SE" />
    <IllimatDevice :camera-rotation="cameraRotation" />
  </g>
</template>