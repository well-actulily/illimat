/**
 * Illimat Web Frontend - Vue 3 Entry Point
 * 
 * This module initializes the Vue 3 application with WebAssembly integration
 * and the 3D isometric rendering system.
 */

import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'

console.log('🃏 Illimat Vue 3 Frontend initializing...')

// Create Vue app with Pinia store
const app = createApp(App)
const pinia = createPinia()

app.use(pinia)

// Mount the application
app.mount('#app')

console.log('✨ Vue 3 Illimat app mounted successfully')