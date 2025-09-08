<script setup>
import { ref } from 'vue'
import IllimatScene from './components/IllimatScene.vue'

const cameraRotation = ref(0)
const deviceRotation = ref(45)
</script>

<template>
  <div id="app">
    <div class="game-container">
      <div class="game-scene">
        <IllimatScene :camera-rotation="cameraRotation" :device-rotation="deviceRotation" />
      </div>
      <div class="game-ui">
        <h1>Illimat Game</h1>
        <div class="controls">
          <label for="camera-slider">Camera Rotation: {{ cameraRotation }}°</label>
          <input 
            id="camera-slider"
            type="range" 
            min="0" 
            max="360" 
            step="1"
            v-model.number="cameraRotation"
          />
          <label for="device-slider">Device Rotation: {{ deviceRotation }}°</label>
          <input 
            id="device-slider"
            type="range" 
            min="0" 
            max="360" 
            step="1"
            v-model.number="deviceRotation"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

#app {
  font-family: Avenir, Helvetica, Arial, sans-serif;
  color: #f9f2e7;
  background-color: #181a1b;
  height: 100vh;
  overflow: hidden;
}

.game-container {
  display: flex;
  height: 100vh;
}

.game-scene {
  flex: 0 0 auto;
  height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: #181a1b;
}

.game-ui {
  flex: 1;
  padding: 20px;
  overflow-y: auto;
}

.controls {
  margin: 20px 0;
}

.controls label {
  display: block;
  margin-bottom: 10px;
  font-weight: bold;
}

.controls input[type="range"] {
  width: 100%;
  margin-bottom: 10px;
}

svg {
  height: calc(100vh - 40px);
  width: auto;
  max-width: none;
}
</style>