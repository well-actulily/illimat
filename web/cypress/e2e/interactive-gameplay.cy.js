describe('Interactive Gameplay System', () => {
  beforeEach(() => {
    cy.visit('/')
    cy.waitForWasm()
  })

  it('loads the game and initializes WASM', () => {
    cy.get('[data-cy="loading-screen"]').should('not.exist')
    cy.get('[data-cy="game-interface"]').should('be.visible')
    cy.get('[data-cy="scoring-banner"]').should('be.visible')
    cy.get('[data-cy="illimat-table"]').should('be.visible')
  })

  it('displays player scores and game state', () => {
    cy.get('[data-cy="scoring-banner"]').within(() => {
      cy.get('[data-cy="player-score"]').should('have.length.at.least', 2)
      cy.get('[data-cy="season-indicator"]').should('exist')
      cy.get('[data-cy="okus-count"]').should('exist')
    })
  })

  it('shows game fields with correct seasons', () => {
    cy.get('[data-cy="game-field"]').should('have.length', 4)
    
    // Each field should show season and restrictions
    cy.get('[data-cy="game-field"]').each(($field, index) => {
      cy.wrap($field).should('have.attr', 'data-season')
      cy.wrap($field).should('contain.text', 'Field') // Field name
    })
  })

  it('displays the Illimat center with okus tokens', () => {
    cy.get('[data-cy="illimat-center"]').should('be.visible')
    cy.get('[data-cy="okus-token"]').should('exist')
    cy.get('[data-cy="season-arrow"]').should('exist')
  })

  describe('Card Selection and Action Popup', () => {
    it('opens action popup when clicking a card', () => {
      cy.get('[data-cy="player-hand"] [data-cy="game-card"]')
        .first()
        .click()
      
      cy.get('[data-cy="action-popup"]').should('be.visible')
      cy.get('[data-cy="action-popup"]').within(() => {
        cy.get('[data-cy="selected-card"]').should('exist')
        cy.get('[data-cy="action-sow"]').should('exist')
        cy.get('[data-cy="action-harvest"]').should('exist')
        cy.get('[data-cy="action-stockpile"]').should('exist')
      })
    })

    it('closes action popup when clicking close button', () => {
      cy.selectCard(0)
      cy.get('[data-cy="action-popup"] [data-cy="close-btn"]').click()
      cy.get('[data-cy="action-popup"]').should('not.exist')
    })

    it('shows field targeting after selecting an action', () => {
      cy.selectCard(0)
      cy.chooseAction('sow')
      
      cy.get('[data-cy="field-targeting"]').should('be.visible')
      cy.get('[data-cy="field-option"]').should('have.length', 4)
    })
  })

  describe('Move Execution via WASM', () => {
    it('executes a sow move successfully', () => {
      const initialHandSize = 4 // Assuming standard hand size
      
      cy.makeMove(0, 'sow', 0)
      
      // Verify move was executed
      cy.get('[data-cy="action-popup"]').should('not.exist')
      
      // Verify game state changed
      cy.verifyGameState({
        // Hand should have one less card
        currentPlayerHandSize: initialHandSize - 1
      })
    })

    it('respects season restrictions', () => {
      // Find a winter field (no harvesting)
      cy.get('[data-cy="game-field"][data-season="winter"]').then(($winterField) => {
        const fieldIndex = $winterField.data('field-index')
        
        cy.selectCard(0)
        cy.chooseAction('harvest')
        
        // Winter field should be disabled for harvest
        cy.get(`[data-cy="field-option-${fieldIndex}"]`)
          .should('have.class', 'invalid')
          .should('be.disabled')
      })
    })

    it('validates moves through WASM backend', () => {
      // Try to harvest from empty field - should fail
      cy.makeMove(0, 'harvest', 0)
      
      // Should show error or return to action selection
      cy.get('[data-cy="error-message"]').should('exist')
        .and('contain.text', 'No matching cards')
    })
  })

  describe('Game State Management', () => {
    it('updates scores after successful moves', () => {
      const initialScore = 0
      
      // Perform a harvest move that should give points
      cy.makeMove(0, 'harvest', 1)
      
      // Verify score updated
      cy.get('[data-cy="player-score"][data-player="0"]')
        .should('not.contain.text', initialScore.toString())
    })

    it('advances to next player after turn', () => {
      cy.get('[data-cy="current-player"]').should('contain.text', 'Player 1')
      
      cy.makeMove(0, 'sow', 0)
      
      cy.get('[data-cy="current-player"]').should('contain.text', 'Player 2')
    })

    it('updates okus count when fields are cleared', () => {
      const initialOkusCount = 4
      
      // Find field with okus and clear it
      cy.get('[data-cy="game-field"][data-has-okus="true"]').then(($field) => {
        const fieldIndex = $field.data('field-index')
        
        // Execute move that clears the field
        cy.makeMove(0, 'harvest', fieldIndex)
        
        // Verify okus count decreased
        cy.get('[data-cy="okus-count"]')
          .should('not.contain.text', initialOkusCount.toString())
      })
    })
  })

  describe('Error Handling', () => {
    it('handles WASM errors gracefully', () => {
      // Force an invalid move by manipulating game state
      cy.window().then((win) => {
        // Simulate WASM error
        win.wasmEngine = null
      })
      
      cy.makeMove(0, 'sow', 0)
      
      cy.get('[data-cy="error-message"]')
        .should('be.visible')
        .and('contain.text', 'WASM not available')
    })

    it('validates card ownership', () => {
      // Try to play opponent's card
      cy.get('[data-cy="opponent-card"]').first().click()
      
      // Should not open action popup
      cy.get('[data-cy="action-popup"]').should('not.exist')
    })
  })

  describe('Performance', () => {
    it('loads WASM module within reasonable time', () => {
      cy.visit('/')
      
      // WASM should load within 5 seconds
      cy.waitForWasm()
      cy.get('[data-cy="game-interface"]', { timeout: 5000 })
        .should('be.visible')
    })

    it('handles rapid interactions without lag', () => {
      // Rapidly select and deselect cards
      for (let i = 0; i < 5; i++) {
        cy.selectCard(0)
        cy.get('[data-cy="action-popup"] [data-cy="close-btn"]').click()
      }
      
      // Should remain responsive
      cy.get('[data-cy="game-interface"]').should('be.visible')
    })
  })
})