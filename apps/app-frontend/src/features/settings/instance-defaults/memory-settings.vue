<script setup lang="ts">
import { Slider } from '@modrinth/ui'
import useMemorySlider from '@/composables/useMemorySlider'
import { useDefaultInstanceSettings } from './use-default-instance-settings'
const { settings } = useDefaultInstanceSettings(
	(value) => ({ memory: { ...value.memory } }),
	(value) => ({ memory: { ...value.memory } }),
)
const { maxMemory, snapPoints } = useMemorySlider()
</script>
<template>
	<div v-if="settings && maxMemory" class="flex flex-col gap-6">
		<h2 class="m-0">Память для Minecraft</h2>
		<p>Общий лимит для сборок без индивидуального переопределения.</p>
		<div class="text-4xl font-bold text-brand">
			{{ (settings.memory.maximum / 1024).toFixed(1) }} ГБ
		</div>
		<Slider
			id="nyan-memory"
			v-model="settings.memory.maximum"
			:min="512"
			:max="maxMemory"
			:step="64"
			:snap-points="snapPoints"
			:snap-range="256"
			unit="МБ"
		/>
		<p>Изменения сохраняются автоматически и применяются при следующем запуске игры.</p>
	</div>
</template>
