<script setup>
import IllimatDevice from './IllimatDevice.vue'
import GameField from './GameField.vue'

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
</script>

<template>
  <g id="mat">
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
      width="129.999" 
      height="129.999" 
      transform="translate(627.696 260) rotate(135)" 
      class="wht-stroke hvy-stroke" 
    />
    <GameField position="SW" />
    <GameField position="NW" />
    <GameField position="NE" />
    <GameField position="SE" />
    <IllimatDevice :camera-rotation="cameraRotation" :device-rotation="deviceRotation" />
  </g>
</template>