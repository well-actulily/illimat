<script setup>
import GameMat from './GameMat.vue'
import GameField from './GameField.vue'
import IllimatDevice from './IllimatDevice.vue'

const props = defineProps({
  cameraRotation: {
    type: Number,
    default: 0
  },
  deviceRotation: {
    type: Number,
    default: 0
  }
})

// Center point for rotation
const centerX = 260
const centerY = 260

// Isometric scaling factor: cos(26.565°) ≈ 0.894427
const isometricScale = 0.894427
</script>

<template>
  <svg 
    id="scene" 
    xmlns="http://www.w3.org/2000/svg" 
    viewBox="0 0 520 465"
  >
    <g :transform="`scale(1 ${isometricScale})`">
      <g :transform="`rotate(${cameraRotation} ${centerX} ${centerY})`">
        <GameMat :camera-rotation="cameraRotation" :device-rotation="deviceRotation" />
      </g>
    </g>
  </svg>
</template>

<style>
.blk-fill { fill: #181a1b; }
.wht-fill { fill: #f9f2e7; }
.wht-stroke { fill: none; stroke: #f9f2e7; stroke-miterlimit: 10; }
.blk-stroke { fill: none; stroke: #181a1b; stroke-miterlimit: 10; }
.wht-fill-blk-stroke { fill: #f9f2e7; stroke: #181a1b; stroke-miterlimit: 10; }
.hvy-stroke { stroke-width: 5px; }
.fine-stroke { stroke-width: 0.5px; }
</style>