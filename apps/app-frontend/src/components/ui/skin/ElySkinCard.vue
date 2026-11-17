<script setup lang="ts">
import { CheckIcon, StarIcon, TrashIcon } from '@modrinth/assets'
import ElySkinThumbnail from './ElySkinThumbnail.vue'

defineProps<{
	skin: { id: number; skin_url: string; is_slim: boolean; tags: string[] }
	selected: boolean
	equipped: boolean
	personal: boolean
	connected: boolean
	busy: boolean
	saved?: boolean
}>()
defineEmits<{ preview: []; apply: []; remove: []; save: [] }>()
</script>

<template>
	<div
		class="skin-card group relative min-w-0 overflow-hidden rounded-2xl border border-solid transition-colors hover:bg-surface-4"
		:class="
			selected || equipped ? 'border-brand bg-brand-highlight' : 'border-transparent bg-surface-3'
		"
	>
		<button
			class="skin-card-preview absolute inset-0 flex w-full cursor-pointer flex-col items-center justify-center rounded-2xl border-0 bg-transparent p-3 text-primary disabled:opacity-50"
			:disabled="busy"
			:aria-label="`Предпросмотр скина ${skin.id}`"
			:aria-pressed="selected"
			@click="$emit('preview')"
		>
			<ElySkinThumbnail class="skin-card-thumbnail" :src="skin.skin_url" :slim="skin.is_slim" />
			<span v-if="!personal" class="mt-2 block w-full truncate text-sm font-semibold">{{
				skin.tags[0] || `Скин #${skin.id}`
			}}</span>
		</button>
		<span
			v-if="equipped"
			class="absolute right-2 top-2 flex size-6 items-center justify-center rounded-full bg-white text-black"
			aria-label="Скин используется"
			title="Скин используется"
			><CheckIcon class="size-4"
		/></span>
		<div class="pointer-events-none absolute inset-x-0 bottom-0 flex justify-center p-2">
			<div
				v-if="connected && (!equipped || !personal)"
				class="skin-card-actions pointer-events-auto flex items-center justify-center gap-2 rounded-xl bg-surface-2 p-1 shadow-lg"
			>
				<button
					v-if="!equipped"
					class="flex size-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-brand text-black disabled:cursor-default disabled:opacity-40"
					:disabled="busy || equipped"
					:aria-label="`Применить скин ${skin.id}`"
					:title="equipped ? 'Скин уже используется' : 'Применить'"
					@click="$emit('apply')"
				>
					<CheckIcon class="size-5" />
				</button>
				<button
					v-if="!personal"
					class="flex size-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-surface-5 text-primary disabled:cursor-default disabled:opacity-40"
					:disabled="busy || saved"
					:aria-label="saved ? 'Скин сохранён' : 'Добавить скин в сохранённые'"
					:title="saved ? 'Скин сохранён' : 'Добавить в сохранённые'"
					@click="$emit('save')"
				>
					<StarIcon class="size-5" :class="{ 'fill-current text-brand': saved }" />
				</button>
				<button
					v-if="personal"
					class="flex size-8 cursor-pointer items-center justify-center rounded-lg border-0 bg-surface-5 text-red disabled:cursor-default disabled:opacity-40"
					:disabled="busy || equipped"
					:aria-label="`Удалить скин ${skin.id}`"
					:title="equipped ? 'Сначала примените другой скин' : 'Удалить'"
					@click="$emit('remove')"
				>
					<TrashIcon class="size-5" />
				</button>
			</div>
		</div>
	</div>
</template>

<style scoped>
.skin-card {
	aspect-ratio: 1;
}
.skin-card-preview :deep(canvas) {
	width: 100%;
	height: 100%;
	min-height: 0;
	object-fit: contain;
}
.skin-card-thumbnail {
	flex: 1;
	max-height: 100%;
}
@media (hover: hover) {
	.skin-card-actions {
		opacity: 0;
		pointer-events: none;
		transition: opacity 150ms ease;
	}
	.skin-card:hover .skin-card-actions,
	.skin-card:focus-within .skin-card-actions {
		opacity: 1;
		pointer-events: auto;
	}
}
@media (prefers-reduced-motion: reduce) {
	.skin-card-actions {
		transition: none;
	}
}
</style>
