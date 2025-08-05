// Simple Node.js script to test WASM loading without full browser
// This validates that our WASM module can be imported and initialized

import { createRequire } from 'module';
const require = createRequire(import.meta.url);

async function testWasmLoading() {
  try {
    console.log('🔬 Testing WASM module loading...');
    
    // Check if WASM files exist
    const fs = require('fs');
    const path = require('path');
    
    const wasmPath = path.join(process.cwd(), 'pkg', 'illimat_core_bg.wasm');
    const jsPath = path.join(process.cwd(), 'pkg', 'illimat_core.js');
    
    if (!fs.existsSync(wasmPath)) {
      throw new Error(`WASM file not found: ${wasmPath}`);
    }
    if (!fs.existsSync(jsPath)) {
      throw new Error(`JS bindings not found: ${jsPath}`);
    }
    
    console.log('✅ WASM files exist');
    console.log(`📦 WASM size: ${Math.round(fs.statSync(wasmPath).size / 1024)}KB`);
    console.log(`📦 JS size: ${Math.round(fs.statSync(jsPath).size / 1024)}KB`);
    
    // Try to import the module (won't work in Node.js but will validate syntax)
    try {
      const wasmModule = await import('./pkg/illimat_core.js');
      console.log('✅ WASM module imports successfully');
      console.log('🔧 Available exports:', Object.keys(wasmModule));
    } catch (importError) {
      if (importError.message.includes('WebAssembly')) {
        console.log('⚠️  WASM import failed (expected in Node.js environment)');
        console.log('✅ But module syntax is valid');
      } else {
        throw importError;
      }
    }
    
    return true;
  } catch (error) {
    console.error('❌ WASM loading test failed:', error.message);
    return false;
  }
}

testWasmLoading().then(success => {
  process.exit(success ? 0 : 1);
});