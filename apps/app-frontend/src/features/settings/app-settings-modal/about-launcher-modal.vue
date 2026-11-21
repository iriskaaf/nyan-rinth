<script setup lang="ts">
import { CodeIcon, DownloadIcon, LayersIcon, RefreshCwIcon, UserIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, NewModal, useVIntl } from '@modrinth/ui'
import { defineAsyncComponent, ref } from 'vue'

import {
	exportDebugInfo,
	exportingDebugInfo,
} from '@/components/ui/download-manager/debug-info-export'

import { appUpdateState, checkForAppUpdate, type AppUpdateCheckResult } from '@/providers/app-update'
import NyanRinthIcon from '@/shared/components/nyanrinth-icon.vue'

defineProps<{ version?: string }>()

const AboutLauncherModel = defineAsyncComponent(() => import('./about-launcher-model.vue'))
const visible = ref(false)
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const { updatesEnabled, availableUpdate } = appUpdateState
const checkingUpdate = ref(false)
const updateResult = ref<AppUpdateCheckResult | 'error' | null>(null)
const messages = defineMessages({
	checkUpdate: {
		id: 'nyanrinth.about.check-update',
		defaultMessage: 'Check for updates',
	},
	checkingUpdate: {
		id: 'nyanrinth.about.checking-update',
		defaultMessage: 'Checking for updates…',
	},
	current: {
		id: 'nyanrinth.about.update-current',
		defaultMessage: 'NyanRinth is up to date.',
	},
	updateAvailable: {
		id: 'nyanrinth.about.update-available',
		defaultMessage: 'Version {version} is available.',
	},
	updateError: {
		id: 'nyanrinth.about.update-error',
		defaultMessage: 'Could not check for updates. Try again.',
	},
	downloadLog: { id: 'nyanrinth.about.download-log', defaultMessage: 'Download log' },
	title: { id: 'nyanrinth.about.title', defaultMessage: 'About launcher' },
	description: { id: 'nyanrinth.about.description', defaultMessage: 'Your Minecraft launcher' },
	developer: { id: 'nyanrinth.about.developer', defaultMessage: 'Developer' },
	base: { id: 'nyanrinth.about.base', defaultMessage: 'Base' },
	commit: { id: 'nyanrinth.about.commit', defaultMessage: 'Base commit' },
})

async function checkUpdate() {
	if (checkingUpdate.value) return
	checkingUpdate.value = true
	updateResult.value = null
	try {
		updateResult.value = await checkForAppUpdate()
	} catch (error) {
		console.warn('Manual launcher update check failed:', error)
		updateResult.value = 'error'
	} finally {
		checkingUpdate.value = false
	}
}

async function downloadLog() {
	await exportDebugInfo(formatMessage(messages.downloadLog)).catch(handleError)
}

defineExpose({ show: () => modal.value?.show() })
</script>

<template>
	<NewModal
		ref="modal"
		:header="formatMessage(messages.title)"
		hide-header
		merge-header
		no-padding
		width="520px"
		max-width="520px"
		@show="visible = true"
		@hide="visible = false"
	>
		<div class="overflow-hidden rounded-2xl">
			<header
				class="about-hero relative flex min-h-[300px] items-end overflow-hidden px-7 pb-7 pt-10"
			>
				<div class="about-model-background absolute">
					<AboutLauncherModel v-if="visible" />
				</div>
				<div class="about-hero-shade pointer-events-none absolute inset-0" />
				<div class="relative flex items-center gap-5">
					<div class="about-icon flex size-20 shrink-0 items-center justify-center rounded-3xl">
						<NyanRinthIcon aria-hidden="true" class="size-16" />
					</div>
					<div class="min-w-0">
						<h2 class="m-0 text-3xl font-extrabold tracking-tight text-contrast">NyanRinth</h2>
						<p class="mb-3 mt-1 text-sm text-secondary">
							{{ formatMessage(messages.description) }}
						</p>
						<span
							class="inline-flex rounded-lg bg-brand-highlight px-2.5 py-1 text-xs font-semibold text-brand"
						>
							v{{ version ?? '…' }}
						</span>
					</div>
				</div>
			</header>
			<div class="flex flex-col gap-5 bg-surface-2 p-7">
				<dl class="m-0 grid grid-cols-2 gap-3">
					<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-4">
						<dt class="flex items-center gap-2 text-xs font-medium text-secondary">
							<UserIcon aria-hidden="true" class="size-4" />
							{{ formatMessage(messages.developer) }}
						</dt>
						<dd class="m-0 mt-2 font-semibold text-contrast">iriska</dd>
					</div>
					<div class="rounded-xl border border-solid border-surface-5 bg-surface-3 p-4">
						<dt class="flex items-center gap-2 text-xs font-medium text-secondary">
							<LayersIcon aria-hidden="true" class="size-4" />
							{{ formatMessage(messages.base) }}
						</dt>
						<dd class="m-0 mt-2 font-semibold text-contrast">
							Modrinth <span class="text-xs font-normal text-secondary">v0.23.1</span>
						</dd>
					</div>
				</dl>
				<div v-if="updatesEnabled" class="flex flex-col gap-2">
					<Button class="w-full" :disabled="checkingUpdate" @click="checkUpdate">
						<RefreshCwIcon aria-hidden="true" :class="{ 'animate-spin': checkingUpdate }" />
						{{ formatMessage(checkingUpdate ? messages.checkingUpdate : messages.checkUpdate) }}
					</Button>
					<p v-if="updateResult && updateResult !== 'disabled'" class="m-0 text-sm text-secondary" role="status">
						<template v-if="updateResult === 'available'">
							{{ formatMessage(messages.updateAvailable, { version: availableUpdate?.version }) }}
						</template>
						<template v-else-if="updateResult === 'current'">
							{{ formatMessage(messages.current) }}
						</template>
						<template v-else>
							{{ formatMessage(messages.updateError) }}
						</template>
					</p>
				</div>
				<Button class="w-full" :disabled="exportingDebugInfo" @click="downloadLog">
					<DownloadIcon aria-hidden="true" />
					{{ formatMessage(messages.downloadLog) }}
				</Button>
				<div class="min-w-0 border-0 border-t border-solid border-surface-5 pt-4">
					<p class="m-0 mb-2 flex items-center gap-2 text-xs text-secondary">
						<CodeIcon aria-hidden="true" class="size-4" />
						{{ formatMessage(messages.commit) }}
					</p>
					<code class="block break-all text-xs text-secondary"
						>e949f1b2e73853e83ffdc9541bdb5aa5d5a660f1</code
					>
				</div>
			</div>
		</div>
	</NewModal>
</template>

<style scoped>
.about-hero {
	background:
		radial-gradient(ellipse at top right, var(--color-brand-highlight), transparent 75%),
		var(--surface-3);
}

.about-model-background {
	left: 24%;
	top: -10%;
	bottom: -5%;
	width: 82%;
}
.about-hero-shade {
	background:
		linear-gradient(0deg, var(--surface-3) 0%, transparent 70%),
		linear-gradient(90deg, var(--surface-3), transparent 48%);
}
.about-icon {
	background: var(--color-brand-highlight);
	box-shadow: 0 8px 32px var(--color-brand-highlight);
}
</style>
