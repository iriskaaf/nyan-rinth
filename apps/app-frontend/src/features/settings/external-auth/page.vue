<script setup lang="ts">
import NyanRinthIcon from '@/shared/components/nyanrinth-icon.vue'

defineProps<{ title: string; description: string }>()
</script>
<template>
	<div class="flex min-w-0 flex-col gap-5">
		<header class="rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5">
			<div class="flex items-center gap-4">
				<NyanRinthIcon aria-hidden="true" class="size-10 shrink-0" />
				<div>
					<h2 class="m-0 text-xl">{{ title }}</h2>
					<p class="mb-0 text-secondary">{{ description }}</p>
				</div>
				<div class="ml-auto"><slot name="actions" /></div>
			</div>
		</header>
		<main><slot /></main>
		<footer v-if="$slots.footer"><slot name="footer" /></footer>
	</div>
</template>
