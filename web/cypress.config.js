import { defineConfig } from 'cypress'

export default defineConfig({
  e2e: {
    baseUrl: 'http://localhost:3000',
    supportFile: 'cypress/support/e2e.js',
    specPattern: 'cypress/e2e/**/*.cy.{js,jsx,ts,tsx}',
    video: false,
    screenshot: false,
    viewportWidth: 1280,
    viewportHeight: 720,
    setupNodeEvents(on, config) {
      // implement node event listeners here
    },
  },
  
  component: {
    devServer: {
      framework: 'vue',
      bundler: 'vite',
    },
  },
})