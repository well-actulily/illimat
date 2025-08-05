/**
 * IllimatRenderer3D - 3D Isometric Projection Engine
 * Extracted from illimat-game-view-poc.html with Vue integration
 */

export class IllimatRenderer3D {
  constructor() {
    // Core projection constants (from PoC)
    this.W = 100  // Width/height of the cube
    this.H = 20   // Height/depth of the cube
    this.ISO_X_FACTOR = 0.8
    this.ISO_Y_FACTOR = 0.4
    this.Z_SCALE = 0.8
    
    // Camera and view state
    this.cameraAngle = 90
    this.illimatAngle = 0
    this.viewBox = { width: 300, height: 300 }
    this.centerOffset = { x: 150, y: 150 }
    
    // Cube geometry (from PoC)
    this.cubeVertices = [
      [-this.W/2, -this.W/2, -this.H],  // Bottom face
      [ this.W/2, -this.W/2, -this.H],
      [ this.W/2,  this.W/2, -this.H],
      [-this.W/2,  this.W/2, -this.H],
      [-this.W/2, -this.W/2,  this.H],  // Top face
      [ this.W/2, -this.W/2,  this.H],
      [ this.W/2,  this.W/2,  this.H],
      [-this.W/2,  this.W/2,  this.H]
    ]
    
    // Field center positions (from PoC)
    this.fieldCenters = [
      { x: 1, y: 0 },    // Right field
      { x: 0, y: -1 },   // Top field  
      { x: -1, y: 0 },   // Left field
      { x: 0, y: 1 }     // Bottom field
    ]
    
    // Okus positioning
    this.maxOkuses = 4
    this.okusOffsets = [
      [0, 0], [15, 0], [-15, 0], [0, 15]
    ]
    
    this.projectedPoints = new Array(8).fill(null).map(() => [0, 0])
    this.okusPositions = new Array(4).fill(null).map(() => [0, 0])
  }
  
  /**
   * Core 3D projection function (from PoC)
   * Transforms 3D world coordinates to 2D screen coordinates
   */
  project3D(x, y, z, lift = 0) {
    const cameraRad = this.cameraAngle * Math.PI / 180
    const illimatRad = this.illimatAngle * Math.PI / 180
    
    // Apply Illimat rotation
    const illimatRotX = x * Math.cos(illimatRad) - y * Math.sin(illimatRad)
    const illimatRotY = x * Math.sin(illimatRad) + y * Math.cos(illimatRad)
    
    // Apply camera rotation
    const finalX = illimatRotX * Math.cos(cameraRad) - illimatRotY * Math.sin(cameraRad)
    const finalY = illimatRotX * Math.sin(cameraRad) + illimatRotY * Math.cos(cameraRad)
    
    // Isometric projection
    return {
      x: (finalX - finalY) * this.ISO_X_FACTOR + this.centerOffset.x,
      y: (finalX + finalY) * this.ISO_Y_FACTOR - (z * this.Z_SCALE) + lift + this.centerOffset.y
    }
  }
  
  /**
   * Update all projections (from PoC updateProjections function)
   */
  updateProjections(cameraAngle, illimatAngle, lift = 0) {
    this.cameraAngle = cameraAngle
    this.illimatAngle = illimatAngle
    
    const cameraRad = cameraAngle * Math.PI / 180
    const illimatRad = illimatAngle * Math.PI / 180
    
    const illimatX = 0  // Center of cube
    const illimatY = 0
    
    // Update cube vertices
    for (let i = 0; i < 8; i++) {
      const [x, y, z] = this.cubeVertices[i]
      
      const illimatRotX = x * Math.cos(illimatRad) - y * Math.sin(illimatRad) + illimatX
      const illimatRotY = x * Math.sin(illimatRad) + y * Math.cos(illimatRad) + illimatY
      
      const finalX = illimatRotX * Math.cos(cameraRad) - illimatRotY * Math.sin(cameraRad)
      const finalY = illimatRotX * Math.sin(cameraRad) + illimatRotY * Math.cos(cameraRad)
      
      this.projectedPoints[i][0] = (finalX - finalY) * this.ISO_X_FACTOR + this.centerOffset.x
      this.projectedPoints[i][1] = (finalX + finalY) * this.ISO_Y_FACTOR - (z * this.Z_SCALE) + lift + this.centerOffset.y
    }
    
    // Update okus positions
    const circleRadius = this.W / 4
    for (let i = 0; i < 4; i++) {
      const theta = (i / Math.max(this.maxOkuses, 1)) * 2 * Math.PI
      const baseX = circleRadius * Math.cos(theta)
      const baseY = circleRadius * Math.sin(theta)
      const [offsetX, offsetY] = this.okusOffsets[i]
      const x = baseX + offsetX
      const y = baseY + offsetY
      const z = this.H + 3
      
      const illimatRotX = x * Math.cos(illimatRad) - y * Math.sin(illimatRad) + illimatX
      const illimatRotY = x * Math.sin(illimatRad) + y * Math.cos(illimatRad) + illimatY
      
      const finalX = illimatRotX * Math.cos(cameraRad) - illimatRotY * Math.sin(cameraRad)
      const finalY = illimatRotX * Math.sin(cameraRad) + illimatRotY * Math.cos(cameraRad)
      
      this.okusPositions[i][0] = (finalX - finalY) * this.ISO_X_FACTOR + this.centerOffset.x
      this.okusPositions[i][1] = (finalX + finalY) * this.ISO_Y_FACTOR - (z * this.Z_SCALE) + lift + this.centerOffset.y
    }
  }
  
  /**
   * Get field boundary in 3D space
   */
  getFieldBounds(fieldIndex) {
    const fieldCenter = this.fieldCenters[fieldIndex]
    const fieldCenterX = fieldCenter.x * this.W
    const fieldCenterY = fieldCenter.y * this.W
    const fieldZ = -this.H
    
    return {
      centerX: fieldCenterX,
      centerY: fieldCenterY,
      centerZ: fieldZ,
      width: this.W,
      height: this.W,
      // Project to screen coordinates
      screenBounds: this.project3D(fieldCenterX, fieldCenterY, fieldZ)
    }
  }
  
  /**
   * Get SVG path data for Illimat cube faces
   */
  getCubeFacePaths() {
    const faces = {
      // Bottom face (visible)
      bottom: [0, 1, 2, 3],
      // Top face (visible)  
      top: [4, 5, 6, 7],
      // Side faces
      front: [0, 1, 5, 4],
      right: [1, 2, 6, 5],
      back: [2, 3, 7, 6],
      left: [3, 0, 4, 7]
    }
    
    const paths = {}
    for (const [faceName, indices] of Object.entries(faces)) {
      const points = indices.map(i => {
        const [x, y] = this.projectedPoints[i]
        return `${Math.round(x * 10) / 10},${Math.round(y * 10) / 10}`
      }).join(' ')
      paths[faceName] = points
    }
    
    return paths
  }
  
  /**
   * Physics-based season change animation (from PoC)
   */
  animateSeasonChange(targetAngle, onUpdate, onComplete) {
    const startAngle = this.illimatAngle
    const angleDiff = targetAngle - startAngle
    const duration = 2000 // 2 seconds
    const startTime = performance.now()
    
    const animate = (currentTime) => {
      const elapsed = currentTime - startTime
      const progress = Math.min(elapsed / duration, 1)
      
      // Physics-based easing: lift-rotate-drop-settle
      let lift = 0
      let currentAngle = startAngle
      
      if (progress < 0.3) {
        // Lift phase
        const liftProgress = progress / 0.3
        lift = -20 * Math.sin(liftProgress * Math.PI)
      } else if (progress < 0.7) {
        // Rotate phase
        const rotateProgress = (progress - 0.3) / 0.4
        const easeRotate = 0.5 - 0.5 * Math.cos(rotateProgress * Math.PI)
        currentAngle = startAngle + angleDiff * easeRotate
        lift = -20
      } else {
        // Drop and settle phase
        const dropProgress = (progress - 0.7) / 0.3
        const easeDrop = 1 - Math.pow(1 - dropProgress, 3)
        currentAngle = targetAngle
        lift = -20 * (1 - easeDrop)
        
        // Settle bounce
        if (dropProgress > 0.8) {
          const bounceProgress = (dropProgress - 0.8) / 0.2
          lift += 3 * Math.sin(bounceProgress * Math.PI * 2) * (1 - bounceProgress)
        }
      }
      
      this.updateProjections(this.cameraAngle, currentAngle, lift)
      
      if (onUpdate) {
        onUpdate({
          angle: currentAngle,
          lift,
          progress,
          projectedPoints: this.projectedPoints,
          okusPositions: this.okusPositions
        })
      }
      
      if (progress < 1) {
        requestAnimationFrame(animate)
      } else {
        this.illimatAngle = targetAngle
        if (onComplete) onComplete()
      }
    }
    
    requestAnimationFrame(animate)
  }
}