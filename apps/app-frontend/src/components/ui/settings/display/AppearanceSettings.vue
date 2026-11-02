<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	defineMessages,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { type ColorTheme, useTheme } from '@/composables/use-theme.ts'
import { type AppSettings, get, set } from '@/helpers/settings.ts'
import { getOS } from '@/helpers/utils'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'

import { accent, ACCENT_PRESETS } from '@/composables/use-accent'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	accent: { id: 'nyanrinth.appearance.accent', defaultMessage: 'Accent color' },
	description: {
		id: 'nyanrinth.appearance.accent-description',
		defaultMessage: 'Choose a color. Changes are saved automatically.',
	},
	pink: { id: 'nyanrinth.appearance.pink', defaultMessage: 'Pink' },
	sky: { id: 'nyanrinth.appearance.sky', defaultMessage: 'Sky blue' },
	green: { id: 'nyanrinth.appearance.green', defaultMessage: 'Green' },
})

const theme = useTheme()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = inject(appSettingsModalContextKey, null)
const os = await getOS()
const settings = ref(await get())

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
	advancedRendering: boolean
	nativeDecorations: boolean
}

function getAppearanceSettingsState(settings: AppSettings): AppearanceSettingsState {
	return {
		theme: settings.theme,
		syncAcrossDevices: settings.sync_theme_across_devices,
		advancedRendering: settings.advanced_rendering,
		nativeDecorations: settings.native_decorations,
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	() => getAppearanceSettingsState(settings.value),
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const nextSettings: AppSettings = {
			...settings.value,
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: value.advancedRendering,
			native_decorations: value.nativeDecorations,
		}

		await set(nextSettings)
		settings.value = nextSettings
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = value.advancedRendering
	},
)

const themeOptions = computed(() =>
	theme.options.filter(
		(option) =>
			option !== 'retro' || settings.value.developer_mode || current.value.theme === 'retro',
	),
)

function setTheme(value: ColorTheme): void {
	current.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	current.value.advancedRendering = enabled
}

function setNativeDecorations(enabled: boolean): void {
	current.value.nativeDecorations = enabled
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal?.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	settingsModal?.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => theme.native),
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'MacOS'
			? {
					value: computed(() => current.value.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	updatePreferences,
})
</script>

<template>
	<div class="flex flex-col gap-6">
		<div class="flex flex-col gap-4 rounded-xl bg-surface-2 p-4">
			<div>
				<h3 class="m-0 text-lg text-contrast">{{ formatMessage(messages.accent) }}</h3>
				<p class="mb-0">{{ formatMessage(messages.description) }}</p>
			</div>
			<div class="grid grid-cols-3 gap-3" role="group" :aria-label="formatMessage(messages.accent)">
				<button
					v-for="preset in ACCENT_PRESETS"
					:key="preset.id"
					type="button"
					:aria-pressed="accent === preset.color"
					class="flex cursor-pointer items-center justify-center gap-2 rounded-xl border border-solid px-3 py-3 font-semibold"
					:class="
						accent === preset.color
							? 'border-brand bg-brand-highlight text-contrast'
							: 'border-surface-5 bg-surface-3 text-primary'
					"
					@click="accent = preset.color"
				>
					<span
						class="h-5 w-5 shrink-0 rounded-full"
						:style="{ background: preset.color }"
						aria-hidden="true"
					/>
					{{ formatMessage(messages[preset.id]) }}
				</button>
			</div>
		</div>
		<AppearanceSettingsLayout />
	</div>
</template>
