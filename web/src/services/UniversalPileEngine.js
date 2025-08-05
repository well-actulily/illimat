/**
 * UniversalPileEngine - Intelligent pile positioning for all card collections
 * Replaces hardcoded field card positioning with general algorithms
 */

export class UniversalPileEngine {
  constructor(cardDimensions = { width: 15, height: 20 }) {
    this.cardDimensions = cardDimensions
    this.layouts = {
      'field': new FieldRowLayout(),      // For field loose cards & stockpiles
      'hand': new HandArcLayout(),        // For player hands  
      'harvest': new HarvestGridLayout(), // For player harvested cards
      'linear': new LinearLayout()        // Fallback for simple layouts
    }
  }
  
  /**
   * Calculate pile positions for any area type
   */
  calculatePilePositions(pileCount, area, areaType = 'field') {
    const layout = this.layouts[areaType]
    if (!layout) {
      console.warn(`Unknown area type: ${areaType}, using linear layout`)
      return this.layouts.linear.generate(pileCount, area, this.cardDimensions)
    }
    
    return layout.generate(pileCount, area, this.cardDimensions)
  }
}

/**
 * Field layout using 3, 5, 7, 9 row pattern for visual balance
 * Replaces the hardcoded if/else chains from PoC
 */
class FieldRowLayout {
  generate(pileCount, fieldBounds, cardSize) {
    if (pileCount === 0) return []
    
    const positions = []
    const rows = this.calculateOptimalRows(pileCount)
    let pileIndex = 0
    
    const rowSpacing = fieldBounds.height / (rows.length + 1)
    
    for (let rowIndex = 0; rowIndex < rows.length; rowIndex++) {
      const pilesInRow = rows[rowIndex]
      const rowY = fieldBounds.centerY - fieldBounds.height/2 + rowSpacing * (rowIndex + 1)
      
      for (let col = 0; col < pilesInRow; col++) {
        if (pileIndex >= pileCount) break
        
        // Center the row, then space cards evenly
        const rowWidth = Math.min(pilesInRow * cardSize.width * 1.2, fieldBounds.width * 0.9)
        const colSpacing = rowWidth / (pilesInRow + 1)
        const startX = fieldBounds.centerX - rowWidth/2
        const x = startX + colSpacing * (col + 1)
        
        positions.push({
          x, 
          y: rowY, 
          z: fieldBounds.centerZ,
          rotation: 0,
          pileIndex,
          row: rowIndex,
          col
        })
        pileIndex++
      }
    }
    
    return positions
  }
  
  /**
   * Calculate optimal row distribution using 3, 5, 7, 9 pattern
   */
  calculateOptimalRows(pileCount) {
    // Use odd numbers for visual balance: 3, 5, 7, 9
    const rowSizes = [3, 5, 7, 9]
    const rows = []
    let remaining = pileCount
    
    for (const maxSize of rowSizes) {
      if (remaining <= 0) break
      
      const cardsInThisRow = Math.min(remaining, maxSize)
      rows.push(cardsInThisRow)
      remaining -= cardsInThisRow
    }
    
    return rows
  }
}

/**
 * Hand layout with arc positioning for ergonomic card selection
 */
class HandArcLayout {
  generate(pileCount, handArea, cardSize) {
    if (pileCount === 0) return []
    
    const positions = []
    const centerX = handArea.centerX
    const centerY = handArea.centerY
    const radius = handArea.radius || 80
    
    // Calculate arc span based on card count (max 120 degrees)
    const maxArcSpan = Math.PI * 2/3 // 120 degrees
    const arcSpan = Math.min(maxArcSpan, pileCount * 0.3)
    
    for (let i = 0; i < pileCount; i++) {
      const angle = (i / Math.max(pileCount - 1, 1)) * arcSpan - arcSpan/2
      const x = centerX + radius * Math.sin(angle)
      const y = centerY + radius * Math.cos(angle)
      
      positions.push({
        x,
        y,
        z: handArea.centerZ || 0,
        rotation: angle * 180 / Math.PI, // Fan cards out
        pileIndex: i
      })
    }
    
    return positions
  }
}

/**
 * Harvest layout with compact grid for many collected cards
 */
class HarvestGridLayout {
  generate(pileCount, harvestArea, cardSize) {
    if (pileCount === 0) return []
    
    const positions = []
    const gridCols = Math.ceil(Math.sqrt(pileCount))
    const gridRows = Math.ceil(pileCount / gridCols)
    
    const cellWidth = harvestArea.width / gridCols
    const cellHeight = harvestArea.height / gridRows
    
    for (let i = 0; i < pileCount; i++) {
      const row = Math.floor(i / gridCols)
      const col = i % gridCols
      
      const x = harvestArea.centerX - harvestArea.width/2 + cellWidth * (col + 0.5)
      const y = harvestArea.centerY - harvestArea.height/2 + cellHeight * (row + 0.5)
      
      positions.push({
        x,
        y,
        z: harvestArea.centerZ || 0,
        rotation: 0,
        pileIndex: i,
        row,
        col
      })
    }
    
    return positions
  }
}

/**
 * Simple linear layout for basic arrangements
 */
class LinearLayout {
  generate(pileCount, area, cardSize) {
    if (pileCount === 0) return []
    
    const positions = []
    const spacing = Math.min(cardSize.width * 1.1, area.width / (pileCount + 1))
    const startX = area.centerX - (pileCount - 1) * spacing / 2
    
    for (let i = 0; i < pileCount; i++) {
      positions.push({
        x: startX + i * spacing,
        y: area.centerY,
        z: area.centerZ || 0,
        rotation: 0,
        pileIndex: i
      })
    }
    
    return positions
  }
}

/**
 * Scene layout manager - coordinates all four high-level elements:
 * Illimat board, okus tokens, piles, and playmat
 */
export class SceneLayoutManager {
  constructor(renderer3D) {
    this.renderer3D = renderer3D
    this.pileEngine = new UniversalPileEngine()
  }
  
  /**
   * Calculate complete scene layout
   */
  calculateSceneLayout(gameState) {
    const layout = {
      illimat: this.getIllimatLayout(),
      okus: this.getOkusLayout(gameState.okusPositions),
      piles: this.getPileLayouts(gameState),
      playmat: this.getPlaymatLayout()
    }
    
    return layout
  }
  
  getIllimatLayout() {
    return {
      center: { x: 0, y: 0, z: 0 },
      faces: this.renderer3D.getCubeFacePaths(),
      projectedPoints: this.renderer3D.projectedPoints
    }
  }
  
  getOkusLayout(okusState) {
    return {
      positions: this.renderer3D.okusPositions,
      visibility: okusState
    }
  }
  
  getPileLayouts(gameState) {
    const layouts = {}
    
    // Field piles (loose cards and stockpiles)
    for (let fieldIndex = 0; fieldIndex < 4; fieldIndex++) {
      const field = gameState.fields[fieldIndex]
      const fieldBounds = this.renderer3D.getFieldBounds(fieldIndex)
      
      layouts[`field_${fieldIndex}`] = this.pileEngine.calculatePilePositions(
        field.looseCards.length + field.stockpiles.length,
        fieldBounds,
        'field'
      )
    }
    
    // Player hands
    for (let playerIndex = 0; playerIndex < gameState.players.length; playerIndex++) {
      const hand = gameState.players[playerIndex].hand
      const handArea = this.getPlayerHandArea(playerIndex, gameState.players.length)
      
      layouts[`hand_${playerIndex}`] = this.pileEngine.calculatePilePositions(
        hand.length,
        handArea,
        'hand'
      )
    }
    
    // Player harvest areas
    for (let playerIndex = 0; playerIndex < gameState.players.length; playerIndex++) {
      const harvest = gameState.players[playerIndex].harvest
      const harvestArea = this.getPlayerHarvestArea(playerIndex, gameState.players.length)
      
      layouts[`harvest_${playerIndex}`] = this.pileEngine.calculatePilePositions(
        harvest.length,
        harvestArea,
        'harvest'
      )
    }
    
    return layouts
  }
  
  getPlayerHandArea(playerIndex, totalPlayers) {
    // Position hands around the perimeter
    const angle = (playerIndex / totalPlayers) * Math.PI * 2
    const distance = 200
    
    return {
      centerX: Math.cos(angle) * distance,
      centerY: Math.sin(angle) * distance,
      centerZ: 30,
      radius: 80,
      width: 100,
      height: 30
    }
  }
  
  getPlayerHarvestArea(playerIndex, totalPlayers) {
    // Position harvest areas in corners
    const positions = [
      { x: 120, y: -120 },  // Top-right
      { x: -120, y: -120 }, // Top-left
      { x: -120, y: 120 },  // Bottom-left
      { x: 120, y: 120 }    // Bottom-right
    ]
    
    const pos = positions[playerIndex % 4]
    
    return {
      centerX: pos.x,
      centerY: pos.y,
      centerZ: -10,
      width: 60,
      height: 40
    }
  }
  
  getPlaymatLayout() {
    return {
      bounds: {
        width: 400,
        height: 400,
        centerX: 0,
        centerY: 0,
        centerZ: -30
      }
    }
  }
}