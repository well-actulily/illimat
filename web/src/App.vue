<script setup>
import IllimatScene from './components/IllimatScene.vue'
</script>

<template>
  <div id="app">
    <div class="game-container">
      <div class="game-scene">
        <IllimatScene />
      </div>
      <div class="game-ui">
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
  padding: clamp(0.75rem, 2vmin, 2rem);
  background: #181a1b;
}

.game-ui {
  flex: 1;
  padding: clamp(0.75rem, 2vmin, 2rem);
  overflow-y: auto;
}

#scene {
  height: calc(100vh - 2 * clamp(0.75rem, 2vmin, 2rem));
  width: auto;
  max-width: none;
}
</style>