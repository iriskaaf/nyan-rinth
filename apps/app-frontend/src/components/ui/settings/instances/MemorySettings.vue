<script setup lang="ts">
import { Slider, injectNotificationManager } from '@modrinth/ui'
import { ref, watch } from 'vue'
import useMemorySlider from '@/composables/useMemorySlider'
import { get, set } from '@/helpers/settings'

const { handleError } = injectNotificationManager()
const settings = await get()
const memory = ref(settings.memory.maximum)
const { maxMemory, snapPoints } = await useMemorySlider()
let pending = Promise.resolve()
watch(memory, (value) => {
	const maximum = Math.max(512, Math.min(maxMemory.value, Math.round(value / 64) * 64))
	pending = pending
		.then(async () => {
			const latest = await get()
			latest.memory.maximum = maximum
			await set(latest)
		})
		.catch(handleError)
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<div>
			<h2 class="m-0 text-2xl font-semibold text-contrast">Память для Minecraft</h2>
			<p>Общий лимит для новых и существующих сборок без индивидуального переопределения.</p>
		</div>
		<div class="rounded-2xl bg-bg-raised p-6">
			<div class="mb-6 text-4xl font-bold text-brand">{{ (memory / 1024).toFixed(1) }} ГБ</div>
			<Slider
				id="nyan-memory"
				v-model="memory"
				:min="512"
				:max="maxMemory"
				:step="64"
				:snap-points="snapPoints"
				:snap-range="256"
				unit="МБ"
			/>
		</div>
		<p>
			На компьютере: {{ (maxMemory / 1024).toFixed(1) }} ГБ. Оставьте часть памяти Windows и другим
			приложениям. Изменения сохраняются автоматически и применяются при следующем запуске игры.
		</p>
		<p v-if="memory > maxMemory * 0.8" class="text-orange">
			Выделено больше 80% памяти компьютера. Это может замедлить систему.
		</p>
	</div>
</template>
