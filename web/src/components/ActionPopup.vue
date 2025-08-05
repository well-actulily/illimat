<template>
  <div v-if="visible" class="action-popup" :style="popupStyle" @click.stop data-cy="action-popup">
    <div class="popup-header">
      <span class="selected-card" data-cy="selected-card">{{ selectedCardDisplay }}</span>
      <button class="close-btn" @click="handleClose" data-cy="close-btn">×</button>
    </div>
    
    <div class="action-buttons">
      <button 
        v-for="action in availableActions" 
        :key="action.type"
        class="action-btn"
        :class="[`action-${action.type}`, { disabled: !action.enabled }]"
        :disabled="!action.enabled"
        @click="handleActionSelect(action)"
        :data-cy="`action-${action.type}`"
      >
        <span class="action-icon">{{ action.icon }}</span>
        <span class="action-label">{{ action.label }}</span>
        <span v-if="action.tooltip" class="action-tooltip">{{ action.tooltip }}</span>
      </button>
    </div>

    <div v-if="showFieldTargeting" class="field-targeting" data-cy="field-targeting">
      <div class="targeting-header">Choose field:</div>
      <div class="field-options">
        <button
          v-for="(field, index) in targetFields"
          :key="index"
          class="field-btn"
          :class="{ 
            valid: field.valid, 
            invalid: !field.valid,
            selected: selectedField === index 
          }"
          :disabled="!field.valid"
          @click="handleFieldSelect(index)"
          :data-cy="`field-option-${index}`"
          data-cy="field-option"
        >
          <span class="field-season">{{ field.seasonEmoji }}</span>
          <span class="field-name">{{ field.name }}</span>
          <div v-if="!field.valid" class="field-restriction">{{ field.restriction }}</div>
        </button>
      </div>
    </div>

    <div v-if="selectedAction && selectedField !== null" class="action-confirm">
      <button class="confirm-btn" @click="handleConfirm">
        {{ confirmText }}
      </button>
    </div>
  </div>
</template>

<script setup>
import { computed, ref, watch } from 'vue'

const props = defineProps({
  visible: { type: Boolean, default: false },
  selectedCard: { type: Object, default: null },
  position: { type: Object, default: () => ({ x: 0, y: 0 }) },
  availableActions: { type: Array, default: () => [] },
  gameState: { type: Object, default: null }
})

const emit = defineEmits(['close', 'action-confirm'])

// Component state
const selectedAction = ref(null)
const selectedField = ref(null)
const showFieldTargeting = ref(false)

// Reset state when popup visibility changes
watch(() => props.visible, (visible) => {
  if (!visible) {
    selectedAction.value = null
    selectedField.value = null
    showFieldTargeting.value = false
  }
})

// Computed properties
const popupStyle = computed(() => ({
  left: `${props.position.x}px`,
  top: `${props.position.y}px`,
  transform: 'translate(-50%, -100%)'
}))

const selectedCardDisplay = computed(() => {
  if (!props.selectedCard) return ''
  const rank = props.selectedCard.getRankDisplay?.() || props.selectedCard.rank
  const suit = props.selectedCard.getSuitSymbol?.() || props.selectedCard.suit
  return `${rank}${suit}`
})

const targetFields = computed(() => {
  if (!props.gameState?.fields) return []
  
  return props.gameState.fields.map((field, index) => {
    const season = getFieldSeason(index)
    const seasonName = season.charAt(0).toUpperCase() + season.slice(1)
    const restrictions = getSeasonRestrictions(season, selectedAction.value?.type)
    
    return {
      index,
      name: `${seasonName} Field`,
      season,
      seasonEmoji: getSeasonEmoji(season),
      valid: restrictions.valid,
      restriction: restrictions.reason,
      cardCount: field.cards?.length || 0
    }
  })
})

const confirmText = computed(() => {
  if (!selectedAction.value || selectedField.value === null) return ''
  
  const action = selectedAction.value.type
  const field = targetFields.value[selectedField.value]
  
  return `${action.charAt(0).toUpperCase() + action.slice(1)} in ${field.name}`
})

// Methods
const getFieldSeason = (fieldIndex) => {
  // Get season from illimat rotation (0°=Winter, 90°=Spring, 180°=Summer, 270°=Autumn)
  const rotation = props.gameState?.illimat_orientation || 0
  const seasonIndex = (Math.floor(rotation / 90) + fieldIndex) % 4
  const seasons = ['winter', 'spring', 'summer', 'autumn']
  return seasons[seasonIndex]
}

const getSeasonEmoji = (season) => {
  const emojis = { winter: '❄️', spring: '🌱', summer: '☀️', autumn: '🍂' }
  return emojis[season] || '?'
}

const getSeasonRestrictions = (season, actionType) => {
  if (!actionType) return { valid: true, reason: '' }
  
  const restrictions = {
    winter: { harvest: 'No harvesting in Winter' },
    spring: { stockpile: 'No stockpiling in Spring' },
    summer: {},
    autumn: { sow: 'No sowing in Autumn (can stockpile)' }
  }
  
  const seasonRestrictions = restrictions[season] || {}
  const restriction = seasonRestrictions[actionType]
  
  return {
    valid: !restriction,
    reason: restriction || ''
  }
}

// Event handlers
const handleClose = () => {
  emit('close')
}

const handleActionSelect = (action) => {
  if (!action.enabled) return
  
  selectedAction.value = action
  showFieldTargeting.value = true
  selectedField.value = null
}

const handleFieldSelect = (fieldIndex) => {
  const field = targetFields.value[fieldIndex]
  if (!field.valid) return
  
  selectedField.value = fieldIndex
}

const handleConfirm = () => {
  if (!selectedAction.value || selectedField.value === null) return
  
  emit('action-confirm', {
    action: selectedAction.value.type,
    card: props.selectedCard,
    fieldIndex: selectedField.value,
    field: targetFields.value[selectedField.value]
  })
}
</script>

<style scoped>
.action-popup {
  position: absolute;
  background: linear-gradient(135deg, rgba(10, 10, 15, 0.95) 0%, rgba(20, 10, 25, 0.95) 100%);
  border: 2px solid #55cc00;
  border-radius: 8px;
  padding: 1rem;
  min-width: 200px;
  z-index: 1000;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(4px);
}

.popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 0.75rem;
  padding-bottom: 0.5rem;
  border-bottom: 1px solid #333;
}

.selected-card {
  color: #55cc00;
  font-weight: bold;
  font-family: monospace;
  font-size: 1.1rem;
}

.close-btn {
  background: none;
  border: none;
  color: #aaa;
  font-size: 1.2rem;
  cursor: pointer;
  padding: 0.25rem;
  line-height: 1;
}

.close-btn:hover {
  color: #fff;
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  margin-bottom: 1rem;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.75rem;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid #555;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.action-btn:hover:not(.disabled) {
  background: rgba(85, 204, 0, 0.2);
  border-color: #55cc00;
}

.action-btn.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-sow { border-left: 3px solid #aa00cc; }
.action-harvest { border-left: 3px solid #55cc00; }
.action-stockpile { border-left: 3px solid #00ccaa; }

.action-icon {
  font-size: 1.2rem;
}

.action-label {
  font-weight: bold;
  flex: 1;
}

.action-tooltip {
  font-size: 0.8rem;
  color: #aaa;
}

.field-targeting {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #333;
}

.targeting-header {
  color: #aaa;
  font-size: 0.9rem;
  margin-bottom: 0.5rem;
}

.field-options {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.5rem;
}

.field-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 0.5rem;
  background: rgba(0, 0, 0, 0.4);
  border: 1px solid #555;
  border-radius: 4px;
  color: #fff;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
  font-size: 0.8rem;
}

.field-btn.valid:hover {
  background: rgba(85, 204, 0, 0.2);
  border-color: #55cc00;
}

.field-btn.invalid {
  opacity: 0.5;
  cursor: not-allowed;
  border-color: #cc4400;
}

.field-btn.selected {
  background: rgba(85, 204, 0, 0.3);
  border-color: #55cc00;
  border-width: 2px;
}

.field-season {
  font-size: 1.2rem;
  margin-bottom: 0.25rem;
}

.field-name {
  font-weight: bold;
  margin-bottom: 0.25rem;
}

.field-restriction {
  font-size: 0.7rem;
  color: #cc4400;
  text-align: center;
}

.action-confirm {
  margin-top: 1rem;
  padding-top: 1rem;
  border-top: 1px solid #333;
}

.confirm-btn {
  width: 100%;
  padding: 0.75rem;
  background: linear-gradient(45deg, #55cc00, #66dd11);
  border: none;
  border-radius: 4px;
  color: #000;
  font-weight: bold;
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: inherit;
}

.confirm-btn:hover {
  background: linear-gradient(45deg, #66dd11, #77ee22);
  transform: translateY(-1px);
  box-shadow: 0 4px 8px rgba(85, 204, 0, 0.3);
}
</style>