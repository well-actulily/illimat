import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import path from 'path'

export default defineConfig({
    root: 'src',
    build: {
        outDir: '../dist',
        emptyOutDir: true,
    },
    server: {
        port: 3000,
        host: true,
    },
    plugins: [
        vue(),
        {
            name: 'wasm',
            generateBundle() {
                // Custom plugin to handle WASM files
                // Will be expanded when we add actual WASM integration
            }
        }
    ],
    resolve: {
        alias: {
            '@': path.resolve(__dirname, 'src')
        }
    },
    // Optimize for WebAssembly
    optimizeDeps: {
        exclude: ['../backend/pkg'] // Exclude WASM package from pre-bundling
    }
})