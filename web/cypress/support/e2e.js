// Cypress support file for e2e tests
// This file is processed and loaded automatically before your test files

import './commands'

// Disable uncaught exception handling for WASM loading
Cypress.on('uncaught:exception', (err, runnable) => {
  // Don't fail tests on WASM loading issues during development
  if (err.message.includes('WASM') || err.message.includes('WebAssembly')) {
    return false
  }
  return true
})

// Global test configuration
beforeEach(() => {
  // Wait for WASM to load before each test
  cy.visit('/')
  cy.get('[data-cy="app"]', { timeout: 10000 })
  cy.window().its('wasmReady', { timeout: 15000 }).should('exist')
})