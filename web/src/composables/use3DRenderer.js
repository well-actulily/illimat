/**
 * Vue composable for 3D rendering integration
 */

import { ref, reactive, computed } from 'vue'
import { IllimatRenderer3D } from '@/services/IllimatRenderer3D.js'
import { SceneLayoutManager } from '@/services/UniversalPileEngine.js'

export function use3DRenderer() {
  const renderer3D = ref(null)
  const sceneManager = ref(null)
  const isInitialized = ref(false)
  
  // Reactive camera state
  const cameraState = reactive({
    angle: 90,
    illimatAngle: 0,
    isAnimating: false
  })
  
  // Reactive projection state
  const projectionState = reactive({
    projectedPoints: [],
    okusPositions: [],
    cubeFaces: {}
  })
  
  /**
   * Initialize the 3D renderer
   */
  const initializeRenderer = () => {
    renderer3D.value = new IllimatRenderer3D()
    sceneManager.value = new SceneLayoutManager(renderer3D.value)
    
    // Initial projection update
    updateProjections()
    isInitialized.value = true
    
    return renderer3D.value
  }
  
  /**
   * Update all 3D projections
   */
  const updateProjections = (lift = 0) => {
    if (!renderer3D.value) return
    
    renderer3D.value.updateProjections(cameraState.angle, cameraState.illimatAngle, lift)
    
    // Update reactive state
    projectionState.projectedPoints = [...renderer3D.value.projectedPoints]
    projectionState.okusPositions = [...renderer3D.value.okusPositions]
    projectionState.cubeFaces = renderer3D.value.getCubeFacePaths()
  }
  
  /**
   * Get 3D position for any world coordinate
   */
  const project3D = (x, y, z, lift = 0) => {
    if (!renderer3D.value) return { x: 0, y: 0 }
    return renderer3D.value.project3D(x, y, z, lift)
  }
  
  /**
   * Get field boundaries in 3D space
   */
  const getFieldBounds = (fieldIndex) => {
    if (!renderer3D.value) return null
    return renderer3D.value.getFieldBounds(fieldIndex)
  }
  
  /**
   * Animate season change with physics
   */
  const animateSeasonChange = (targetAngle) => {
    if (!renderer3D.value || cameraState.isAnimating) return Promise.resolve()
    
    cameraState.isAnimating = true
    
    return new Promise((resolve) => {
      renderer3D.value.animateSeasonChange(
        targetAngle,
        // onUpdate callback
        (animationState) => {
          cameraState.illimatAngle = animationState.angle
          projectionState.projectedPoints = [...animationState.projectedPoints]
          projectionState.okusPositions = [...animationState.okusPositions]
          projectionState.cubeFaces = renderer3D.value.getCubeFacePaths()
        },
        // onComplete callback
        () => {
          cameraState.isAnimating = false
          cameraState.illimatAngle = targetAngle
          resolve()
        }
      )
    })
  }
  
  /**
   * Set camera angle and update projections
   */
  const setCameraAngle = (angle) => {
    cameraState.angle = angle
    updateProjections()
  }
  
  /**
   * Set Illimat angle and update projections
   */
  const setIllimatAngle = (angle) => {
    cameraState.illimatAngle = angle
    updateProjections()
  }
  
  /**
   * Calculate complete scene layout
   */
  const calculateSceneLayout = (gameState) => {
    if (!sceneManager.value) return null
    return sceneManager.value.calculateSceneLayout(gameState)
  }
  
  /**
   * Get pile positions for a specific area
   */
  const getPilePositions = (pileCount, area, areaType = 'field') => {
    if (!sceneManager.value) return []
    return sceneManager.value.pileEngine.calculatePilePositions(pileCount, area, areaType)
  }
  
  // Computed properties for easy access
  const illimatFacePaths = computed(() => projectionState.cubeFaces)
  const okusScreenPositions = computed(() => projectionState.okusPositions)
  const cubeProjections = computed(() => projectionState.projectedPoints)
  
  return {
    // State
    isInitialized,
    cameraState,
    projectionState,
    
    // Core functions
    initializeRenderer,
    updateProjections,
    project3D,
    getFieldBounds,
    
    // Animation
    animateSeasonChange,
    
    // Controls
    setCameraAngle,
    setIllimatAngle,
    
    // Scene management
    calculateSceneLayout,
    getPilePositions,
    
    // Computed data
    illimatFacePaths,
    okusScreenPositions,
    cubeProjections,
    
    // Direct access for advanced usage
    renderer3D,
    sceneManager
  }
}