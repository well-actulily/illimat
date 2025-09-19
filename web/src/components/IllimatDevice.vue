<script setup>
import { computed, ref } from 'vue'

const props = defineProps({
  cameraRotation: {
    type: Number,
    default: 0
  }
})

const deviceRotation = ref(45)
const currentLift = ref(0)
const animating = ref(false)

const nextSeason = () => {
  if (animating.value) return
  
  const currentAngle = deviceRotation.value
  // Season positions: 45, 135, 225, 315
  const nextAngle = (Math.round((currentAngle - 45) / 90) * 90 + 45 + 90) % 360
  animateTo(nextAngle)
}

const animateTo = (nextDeviceAngle) => {
  if (animating.value) return
  animating.value = true

  const start = performance.now()
  const liftDuration = 160
  const rotateDuration = 280
  const dropDuration = 120
  const settleDuration = 200
  const total = liftDuration + rotateDuration + dropDuration + settleDuration
  
  const angleStart = deviceRotation.value
  const angleEnd = nextDeviceAngle % 360
  const angleDelta = (angleEnd - angleStart + 360) % 360

  const frame = (t) => {
    const elapsed = t - start
    let angle = angleStart
    let lift = 0

    if (elapsed < liftDuration) {
      // Lift phase - reluctant upward movement
      const p = elapsed / liftDuration
      const reluctant = 1 - Math.pow(1 - p, 4)
      lift = -35 * reluctant
    } else if (elapsed < liftDuration + rotateDuration) {
      // Rotate phase - heavy rotation while lifted
      const p = (elapsed - liftDuration) / rotateDuration
      const heavy = p < 0.3 ? 2 * p * p : 1 - 0.5 * Math.pow(-2 * p + 2, 2)
      angle = angleStart + angleDelta * heavy
      lift = -35
    } else if (elapsed < liftDuration + rotateDuration + dropDuration) {
      // Drop phase - gravity-based fall
      const p = (elapsed - liftDuration - rotateDuration) / dropDuration
      const gravity = p * p * p * p
      lift = -35 + (35 * gravity)
      angle = angleEnd
    } else if (elapsed < total) {
      // Settle phase - bounce and settle
      const settleTime = elapsed - liftDuration - rotateDuration - dropDuration
      const p = settleTime / settleDuration
      if (p < 0.3) {
        const bounceP = p / 0.3
        lift = Math.sin(bounceP * Math.PI) * 4 * (1 - bounceP)
      } else {
        lift = 0
      }
      angle = angleEnd
    } else {
      // Animation complete
      deviceRotation.value = angleEnd
      currentLift.value = 0
      animating.value = false
      return
    }

    deviceRotation.value = angle
    currentLift.value = lift
    requestAnimationFrame(frame)
  }
  
  requestAnimationFrame(frame)
}

const handleClick = () => {
  nextSeason()
}

const deviceHeight = 30
const radius = 34

const translateX = computed(() => {
  const radians = (props.cameraRotation * Math.PI) / 180
  const offsetX = -radius * Math.sin(radians)
  
  // Apply lift using same transformation as screen-down offset
  const liftOffset = getScreenDownOffset(currentLift.value)
  
  return 260 + offsetX - 65 + liftOffset.x
})

const translateY = computed(() => {
  const radians = (props.cameraRotation * Math.PI) / 180
  const offsetY = -radius * Math.cos(radians)
  
  // Apply lift using same transformation as screen-down offset  
  const liftOffset = getScreenDownOffset(currentLift.value)
  
  return 260 + offsetY - 65 + liftOffset.y
})

// Define the four corners of the 130x130 Illimat device
const cornerSW = computed(() => (rotatePoint({ x: 0, y: 130 }, deviceRotation.value)))
const cornerNW = computed(() => (rotatePoint({ x: 0, y: 0 }, deviceRotation.value)))
const cornerNE = computed(() => (rotatePoint({ x: 130, y: 0 }, deviceRotation.value)))
const cornerSE = computed(() => (rotatePoint({ x: 130, y: 130 }, deviceRotation.value)))

// Helper function to rotate a point around center (65, 65)
const rotatePoint = (point, angle) => {
  const centerX = 65
  const centerY = 65
  const radians = (angle * Math.PI) / 180
  const cos = Math.cos(radians)
  const sin = Math.sin(radians)
  
  const x = point.x - centerX
  const y = point.y - centerY
  
  return {
    x: centerX + (x * cos - y * sin),
    y: centerY + (x * sin + y * cos)
  }
}

// Helper to calculate normalized combined rotation
const getCombinedRotation = () => {
  const rawRotation = props.cameraRotation + deviceRotation.value
  return ((rawRotation % 360) + 360) % 360
}

// Helper to select corner based on rotation and position offset
const getCornerByRotation = (positionOffset) => {
  const combinedRotation = getCombinedRotation()
  const corners = [cornerSW, cornerSE, cornerNE, cornerNW]
  const cornerLabels = ['SW', 'SE', 'NE', 'NW']
  
  let index
  if (combinedRotation >= 0 && combinedRotation < 90) index = positionOffset
  else if (combinedRotation >= 90 && combinedRotation < 180) index = (positionOffset + 1) % 4
  else if (combinedRotation >= 180 && combinedRotation < 270) index = (positionOffset + 2) % 4
  else index = (positionOffset + 3) % 4
  
  return { corner: corners[index].value, label: cornerLabels[index] }
}

// Visual position points and their labels
const leftPoint = computed(() => getCornerByRotation(0).corner)
const lowerPoint = computed(() => getCornerByRotation(1).corner)  
const rightPoint = computed(() => getCornerByRotation(2).corner)
const topPoint = computed(() => getCornerByRotation(3).corner)

const leftLabel = computed(() => getCornerByRotation(0).label)
const lowerLabel = computed(() => getCornerByRotation(1).label)
const rightLabel = computed(() => getCornerByRotation(2).label)
const topLabel = computed(() => getCornerByRotation(3).label)


// Helper function to calculate screen-space "down" offset in local coordinates
const getScreenDownOffset = (depthPixels) => {
  // Screen space "down" is (0, depthPixels)
  // We need to inverse transform this through camera rotation to get local offset
  const cameraRadians = (-props.cameraRotation * Math.PI) / 180 // negative to undo camera rotation
  const scaledDepth = depthPixels / 0.894 // account for isometric scaling
  
  return {
    x: -scaledDepth * Math.sin(cameraRadians),
    y: scaledDepth * Math.cos(cameraRadians)
  }
}

const cornerSWL = computed(() => {
  const offset = getScreenDownOffset(deviceHeight)
  return { x: cornerSW.value.x + offset.x, y: cornerSW.value.y + offset.y }
})

const cornerSEL = computed(() => {
  const offset = getScreenDownOffset(deviceHeight)
  return { x: cornerSE.value.x + offset.x, y: cornerSE.value.y + offset.y }
})

const cornerNWL = computed(() => {
  const offset = getScreenDownOffset(deviceHeight)
  return { x: cornerNW.value.x + offset.x, y: cornerNW.value.y + offset.y }
})

const cornerNEL = computed(() => {
  const offset = getScreenDownOffset(deviceHeight)
  return { x: cornerNE.value.x + offset.x, y: cornerNE.value.y + offset.y }
})

// Helper to get quadrilateral points based on combined rotation and side offset
const getSideQuadrilateral = (sideOffset) => {
  const combinedRotation = getCombinedRotation()
  const corners = [cornerSW, cornerSE, cornerNE, cornerNW]
  const cornersL = [cornerSWL, cornerSEL, cornerNEL, cornerNWL]
  
  let baseIndex
  if (combinedRotation >= 0 && combinedRotation < 90) baseIndex = 0
  else if (combinedRotation >= 90 && combinedRotation < 180) baseIndex = 1
  else if (combinedRotation >= 180 && combinedRotation < 270) baseIndex = 2
  else baseIndex = 3
  
  const startIndex = (baseIndex + sideOffset) % 4
  const endIndex = (startIndex + 1) % 4
  
  // Special case for first range - different point order
  if (combinedRotation >= 0 && combinedRotation < 90 && sideOffset === 0) {
    return [cornersL[startIndex].value, cornersL[endIndex].value, corners[endIndex].value, corners[startIndex].value]
  }
  
  return [corners[startIndex].value, corners[endIndex].value, cornersL[endIndex].value, cornersL[startIndex].value]
}

// Left and right side quadrilateral points based on combined rotation
const leftSidePoints = computed(() => getSideQuadrilateral(0))
const rightSidePoints = computed(() => getSideQuadrilateral(1))

// Calculate affine transformation matrix from rectangle to parallelogram
const calculateTransformMatrix = (quadPoints) => {
  const [P0, P1, _, P3] = quadPoints
  
  // Calculate the actual parallelogram dimensions
  const dx1 = P1.x - P0.x  // Right edge vector
  const dy1 = P1.y - P0.y
  const dx2 = P3.x - P0.x  // Up edge vector  
  const dy2 = P3.y - P0.y

  const symbolWidth = 130
  const symbolHeight = 34
  
  const a = dx1 / symbolWidth
  const b = dy1 / symbolWidth  
  const c = dx2 / symbolHeight
  const d = dy2 / symbolHeight
  const e = P0.x
  const f = P0.y
  
  return `matrix(${a} ${b} ${c} ${d} ${e} ${f})`
}

// Computed transforms for left and right side faces
const leftSideTransform = computed(() => {
  return calculateTransformMatrix(leftSidePoints.value)
})

const rightSideTransform = computed(() => {
  return calculateTransformMatrix(rightSidePoints.value)
})
</script>

<template>
  <g id="illimat" :transform="`translate(${translateX} ${translateY})`" @click="handleClick" style="cursor: pointer;">
    <defs>
      <symbol id="illimat-hatching">
        <polyline points="2 77 28.589 59.153 28.589 62.153 2 80 2 83 28.589 65.153 28.589 68.153 2 86 2 89 28.589 71.153 28.589 74.153 2 92 2 95 28.589 77.153 28.589 80.153 2 98 2 101 28.589 83.153 28.589 86.153 2 104 2 107" class="blk-stroke fine-stroke"/>
      </symbol>
      <symbol id="illimat-corner">
        <use href="#illimat-hatching" />
        <use href="#illimat-hatching" transform="scale(-1 1) translate(-130 0) rotate(-90 65 65)" />
        <polygon id="black_wedge" points="23 128 2 128 2 107 64.572 65 23 128" class="blk-fill"/>
        <g id="white-arrow">
          <polygon points="21.239 94.767 23 107 35.276 108.333 64.572 65 21.239 94.767" class="wht-fill"/>
        </g>
        <g id="white-wedge">
          <polygon points="21.222 94.399 63.676 65.903 35.52 108.494 21.222 94.399" class="wht-fill"/>
          <path d="M62.779,66.806l-27.3,41.297-13.863-13.667,41.164-27.63M64.572,65l-43.744,29.362,14.732,14.524,29.011-43.886h0Z" class="blk-fill"/>
        </g>
        <line id="center-line" x1="23.586" y1="106.417" x2="64.572" y2="65" class="blk-stroke fine-stroke" style="fill: #f9f2e7;"/>
        <g id="diamond-lines">
          <path d="M60.835,68.762l-29.347,35.732-7.847,1.842,1.563-7.907,35.631-29.667M64.572,65l-39.824,33.158-1.748,8.842,8.767-2.058,32.805-39.942h0Z" class="blk-fill"/>
        </g>
        <circle id="corner-circle" cx="15" cy="115" r="8" class="wht-fill"/>
      </symbol>
      <symbol id="season-circle">
        <circle cx="7" cy="65" r="6" class="wht-fill"/>
      </symbol>
      <symbol id="side-hatching">
        <polyline points="2.25 17 17.25 2 14.25 2 2.25 14 2.25 11 11.25 2 8.25 2 2.25 8 2.25 5 5.25 2" class="wht-stroke fine-stroke"/>
      </symbol>
      <symbol id="side-face" viewBox="0 0 130 34" width="130" height="34">
        <rect id="outer-rect" x="0" y="0" width="130" height="34" class="blk-fill"/>
        <rect id="inner-rect" x="2" y="2" width="126" height="30" class="wht-stroke fine-stroke"/>
        <use href="#side-hatching" />
        <use href="#side-hatching" transform="rotate(180 65 17)" />
        <use href="#side-hatching" transform="scale(-1 1) translate(-130 0) rotate(0 65 17)" />
        <use href="#side-hatching" transform="scale(-1 1) translate(-130 0) rotate(180 65 17)" />
      </symbol>
    </defs>
    <use 
      href="#side-face"
      :transform="leftSideTransform"
    />
    <use 
      href="#side-face"
      :transform="rightSideTransform"
    />
    
    <!-- Edge highlight lines -->
    <line 
      :x1="leftSidePoints[0].x" 
      :y1="leftSidePoints[0].y" 
      :x2="leftSidePoints[3].x" 
      :y2="leftSidePoints[3].y" 
      class="wht-stroke"
      stroke-width="1"
    />
    
    <line 
      :x1="rightSidePoints[1].x" 
      :y1="rightSidePoints[1].y" 
      :x2="rightSidePoints[2].x" 
      :y2="rightSidePoints[2].y" 
      class="wht-stroke"
      stroke-width="1"
    />
    
    <g id="top" :transform="`rotate(${deviceRotation} 65 65)`">
      <rect id="outer-rectangle" x="0" y="0" width="130" height="130" transform="translate(0 130) rotate(-90)" class="wht-fill"/>
      <rect id="inner-rectangle" x="2" y="2" width="126" height="126" transform="translate(0 130) rotate(-90)" class="blk-stroke fine-stroke"/>
      <rect id="black-bar-horizontal" x="0" y="56" width="130" height="18" class="blk-fill"/>
      <rect id="black-bar-vertical" x="56" y="0" width="18" height="130" class="blk-fill"/>
      <use href="#illimat-corner" />
      <use href="#illimat-corner" transform="rotate(90 65 65)" />
      <use href="#illimat-corner" transform="rotate(180 65 65)" />
      <use href="#illimat-corner" transform="rotate(270 65 65)" />
      <circle id="inner-circle" cx="64.572" cy="65" r="43.333" class="wht-fill-blk-stroke fine-stroke"/>
      <use href="#season-circle" />
      <use href="#season-circle" transform="rotate(90 65 65)" />
      <use href="#season-circle" transform="rotate(180 65 65)" />
      <use href="#season-circle" transform="rotate(270 65 65)" />
    </g>
  </g>
</template>