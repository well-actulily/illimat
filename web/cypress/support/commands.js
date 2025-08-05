// Custom Cypress commands for Illimat testing

// Wait for WASM engine to be ready
Cypress.Commands.add('waitForWasm', () => {
  cy.window().should('have.property', 'wasmReady', true)
})

// Select a card from player hand
Cypress.Commands.add('selectCard', (cardIndex = 0) => {
  cy.get('[data-cy="player-hand"] [data-cy="game-card"]')
    .eq(cardIndex)
    .click()
})

// Choose an action from the popup
Cypress.Commands.add('chooseAction', (action) => {
  cy.get('[data-cy="action-popup"]').should('be.visible')
  cy.get(`[data-cy="action-${action}"]`).click()
})

// Select a target field
Cypress.Commands.add('selectField', (fieldIndex) => {
  cy.get(`[data-cy="field-${fieldIndex}"]`).click()
})

// Complete a full move: card → action → field
Cypress.Commands.add('makeMove', (cardIndex, action, fieldIndex) => {
  cy.selectCard(cardIndex)
  cy.chooseAction(action)
  cy.selectField(fieldIndex)
})

// Verify game state updates
Cypress.Commands.add('verifyGameState', (expectedChanges) => {
  cy.window().its('gameState').then((gameState) => {
    Object.keys(expectedChanges).forEach((key) => {
      expect(gameState).to.have.property(key, expectedChanges[key])
    })
  })
})