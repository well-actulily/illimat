import { defineConfig } from 'vite';

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
    // WebAssembly support
    plugins: [
        {
            name: 'wasm',
            generateBundle() {
                // Custom plugin to handle WASM files
                // Will be expanded when we add actual WASM integration
            }
        }
    ],
    // Optimize for WebAssembly
    optimizeDeps: {
        exclude: ['../backend/pkg'] // Exclude WASM package from pre-bundling
    }
});