<template>
	<Transition name="splash-fade" @after-leave="onAfterLeave">
		<div v-if="!doneLoading" class="splash-screen" :class="{ 'splash-light': theme.active === 'light' }">
			<div class="app-logo-wrapper" data-tauri-drag-region>
				<div class="app-logo">
					<NyanRinthIcon aria-hidden="true" class="app-icon" />
					<span>NyanRinth</span>
				</div>
				<ProgressBar class="loading-bar" :progress="Math.min(loadingProgress, 100)" />
				<span v-if="message">{{ message }}</span>
			</div>
			<div class="gradient-bg" data-tauri-drag-region></div>
			<div class="cube-bg"></div>
			<div class="base-bg"></div>
		</div>
	</Transition>
</template>

<script setup>
import { injectLoadingState } from '@modrinth/ui'
import { onMounted, onScopeDispose, ref, watch } from 'vue'

import NyanRinthIcon from '@/shared/components/nyanrinth-icon.vue'

import ProgressBar from '@/components/ui/ProgressBar.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useTheme } from '@/composables/use-theme.ts'
import { debugStartup } from '@/helpers/startup-debug'

const theme = useTheme()

const doneLoading = ref(false)
const loadingProgress = ref(0)
const message = ref()
let progressTimer
let directoryMoveActive = false

onScopeDispose(() => clearTimeout(progressTimer))

const MIN_DISPLAY_MS = 500
const mountedAt = Date.now()

const loading = injectLoadingState()
onMounted(() => debugStartup('Splash mounted'))

function onAfterLeave() {
	debugStartup('Splash fade completed', { displayedMs: Date.now() - mountedAt })
	loading.setEnabled(true)
}

watch(
	[loading.barEnabled, loading.pending],
	([barEnabled, pending], _previous, onCleanup) => {
		clearTimeout(progressTimer)
		debugStartup('Splash loading state changed', { barEnabled, pending })
		if (barEnabled) {
			return
		}

		if (pending) {
			if (!directoryMoveActive) loadingProgress.value = 0
			fakeLoadingIncrease()
			return
		}

		const elapsed = Date.now() - mountedAt
		const delay = Math.max(0, MIN_DISPLAY_MS - elapsed)
		debugStartup('Splash dismissal scheduled', { delayMs: delay, displayedMs: elapsed })

		loadingProgress.value = 100
		const dismissalTimer = setTimeout(() => {
			if (loading.pending.value) {
				debugStartup('Splash dismissal deferred: new loading work')
				return
			}
			doneLoading.value = true
			debugStartup('Splash fade started', { displayedMs: Date.now() - mountedAt })
		}, delay)
		onCleanup(() => clearTimeout(dismissalTimer))
	},
	{ immediate: true },
)

function fakeLoadingIncrease() {
	if (
		!directoryMoveActive &&
		!doneLoading.value &&
		loading.pending.value &&
		loadingProgress.value < 95
	) {
		progressTimer = setTimeout(() => {
			loadingProgress.value = Math.min(loadingProgress.value + 2, 95)
			fakeLoadingIncrease()
		}, 5)
	}
}

useAppEvent('loading', (e) => {
	if (e.event.type === 'directory_move') {
		directoryMoveActive = true
		clearTimeout(progressTimer)
		loadingProgress.value = Math.max(0, Math.min(100 * (e.fraction ?? 1), 100))
		message.value = 'Updating app directory...'
	}
})
</script>

<style scoped lang="scss">
.splash-screen {
	position: fixed;
	inset: 0;
	z-index: 10000;

	--splash-tint-top: var(--color-brand-highlight);
	--splash-tint-bottom: transparent;
	--splash-cube-image: url('@/assets/loading/cube.png');

	&.splash-light {
		--splash-cube-image: url('@/assets/loading/cube-light.webp');
	}
}

.splash-fade-leave-active {
	transition: opacity 0.3s ease-in-out;
}

.splash-fade-leave-to {
	opacity: 0;
}

.app-logo-wrapper {
	position: absolute;
	height: 100vh;
	width: 100%;

	display: flex;
	flex-direction: column;
	justify-content: center;
	align-items: center;

	gap: 1rem;
	color: var(--color-contrast);

	z-index: 9998;
}

.app-logo {
	display: flex;
	align-items: center;
	gap: 0.75rem;
	font-size: 2rem;
	font-weight: 800;
}

.app-icon {
	width: 3rem;
	height: 3rem;
}

.loading-bar {
	max-width: 20rem;
}

.gradient-bg {
	position: absolute;
	height: 100vh;
	width: 100vw;
	background:
		linear-gradient(180deg, var(--splash-tint-top) 0%, var(--splash-tint-bottom) 97.29%),
		linear-gradient(0deg, var(--splash-overlay), var(--splash-overlay));
	z-index: 9997;
}

.cube-bg {
	position: absolute;

	left: 50%;
	top: 50%;
	transform: translate(-50%, -50%);

	width: 180vw;
	height: 180vh;
	background-color: var(--color-bg);

	z-index: 9996;

	&::after {
		content: '';
		position: absolute;
		inset: 0;
		background: var(--splash-cube-image) center no-repeat;
		background-size: contain;
		opacity: var(--splash-cube-opacity);
		mix-blend-mode: var(--splash-cube-blend);
	}
}

.base-bg {
	position: absolute;
	top: 0;
	left: 0;
	width: 100%;
	height: 100%;
	background: var(--color-bg);
	z-index: 9995;
}
</style>
