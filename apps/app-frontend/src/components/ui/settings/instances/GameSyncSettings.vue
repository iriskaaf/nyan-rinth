<script setup lang="ts">
import { Checkbox, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, inject, onBeforeUnmount, onMounted, ref } from 'vue'
import { appSettingsModalContextKey } from '@/providers/app-settings-modal'
import { toError } from '@/helpers/errors'

type Instance = {
	id: string
	name: string
	game_version: string
	status: string
	excluded: boolean
}
type Option = { id: string; key: string; category: string; enabled: boolean; value: unknown }
type View = {
	last_error: string | null
	enabled: boolean
	source_instance: string | null
	instances: Instance[]
	options: Option[]
}
type Update = {
	enabled: boolean
	source_instance: string | null
	excluded_instances: string[]
	disabled_options: string[]
}
const messages = defineMessages({
	title: { id: 'nyanrinth.sync.title', defaultMessage: 'Game settings sync' },
	description: {
		id: 'nyanrinth.sync.description',
		defaultMessage:
			'Share selected game options between your instances. Changes saved in Minecraft become available in other instances.',
	},
	enabled: { id: 'nyanrinth.sync.enabled', defaultMessage: 'Sync game settings' },
	source: { id: 'nyanrinth.sync.source', defaultMessage: 'Starting instance' },
	choose: { id: 'nyanrinth.sync.choose', defaultMessage: 'Choose an instance' },
	sourceHelp: {
		id: 'nyanrinth.sync.source-help',
		defaultMessage:
			'Used when enabling sync or changing the starting instance. Launch and close the game once to create its settings.',
	},
	instances: { id: 'nyanrinth.sync.instances', defaultMessage: 'Participating instances' },
	options: { id: 'nyanrinth.sync.options', defaultMessage: 'Individual options' },
	search: { id: 'nyanrinth.sync.search', defaultMessage: 'Search by option key…' },
	limits: {
		id: 'nyanrinth.sync.limits',
		defaultMessage:
			'Uses the Modrinth 0.22.1 option catalogue. Snapshot versions and unknown mod options stay local. Fullscreen and window size remain controlled by the launcher. Server lists, resource packs and hotbars are not synced here.',
	},
	backups: {
		id: 'nyanrinth.sync.backups',
		defaultMessage:
			'Original files are backed up in game-settings-backups in the application directory. A running game receives updates on its next launch.',
	},
	ready: { id: 'nyanrinth.sync.status.ready', defaultMessage: 'Ready' },
	waiting: { id: 'nyanrinth.sync.status.waiting', defaultMessage: 'Waiting for first game launch' },
	running: { id: 'nyanrinth.sync.status.running', defaultMessage: 'Game is running' },
	unsupported: {
		id: 'nyanrinth.sync.status.unsupported',
		defaultMessage: 'Unsupported game version',
	},
	installing: { id: 'nyanrinth.sync.status.installing', defaultMessage: 'Instance is not ready' },
	unreadable: { id: 'nyanrinth.sync.status.unreadable', defaultMessage: 'Cannot read options.txt' },
	empty: { id: 'nyanrinth.sync.empty', defaultMessage: 'Create or import an instance first.' },
})
const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const modalContext = inject(appSettingsModalContextKey, null)
const view = ref(await invoke<View>('plugin:instance|game_settings_sync_get'))
function toUpdate(value: View): Update {
	return {
		enabled: value.enabled,
		source_instance: value.source_instance,
		excluded_instances: value.instances
			.filter((instance) => instance.excluded)
			.map((instance) => instance.id),
		disabled_options: value.options.filter((option) => !option.enabled).map((option) => option.id),
	}
}
function copyUpdate(value: Update): Update {
	return {
		...value,
		excluded_instances: [...value.excluded_instances],
		disabled_options: [...value.disabled_options],
	}
}
const original = ref(toUpdate(view.value))
const current = ref(copyUpdate(original.value))
const saving = ref(false)
const search = ref('')
const error = ref('')
const dirty = computed(() => JSON.stringify(original.value) !== JSON.stringify(current.value))
const options = computed(() =>
	view.value.options.filter((option) =>
		option.key.toLowerCase().includes(search.value.toLowerCase()),
	),
)
function toggle(list: string[], id: string, included: boolean) {
	const index = list.indexOf(id)
	if (included && index < 0) list.push(id)
	else if (!included && index >= 0) list.splice(index, 1)
}
async function save() {
	if (saving.value) return
	saving.value = true
	error.value = ''
	try {
		view.value = await invoke<View>('plugin:instance|game_settings_sync_update', {
			update: current.value,
		})
		original.value = toUpdate(view.value)
		current.value = copyUpdate(original.value)
	} catch (cause) {
		error.value = toError(cause).message
		handleError(cause)
	} finally {
		saving.value = false
	}
}
function reset() {
	current.value = copyUpdate(original.value)
}
onMounted(() =>
	modalContext?.registerUnsavedChangesController({
		hasChanges: () => dirty.value,
		getOriginal: () => original.value,
		getModified: () => current.value,
		isSaving: () => saving.value,
		reset,
		save,
	}),
)
onBeforeUnmount(() => modalContext?.registerUnsavedChangesController(null))
</script>

<template>
	<div class="flex flex-col gap-5">
		<div>
			<h2 class="m-0 text-xl font-semibold text-contrast">{{ formatMessage(messages.title) }}</h2>
			<p>{{ formatMessage(messages.description) }}</p>
		</div>
		<Checkbox
			v-model="current.enabled"
			:label="formatMessage(messages.enabled)"
			:disabled="saving"
		/>
		<template v-if="current.enabled">
			<label class="flex flex-col gap-2">
				<span class="font-semibold text-contrast">{{ formatMessage(messages.source) }}</span>
				<select
					v-model="current.source_instance"
					class="rounded-xl border-none bg-surface-3 p-3 text-primary"
					:disabled="saving"
				>
					<option :value="null" disabled>{{ formatMessage(messages.choose) }}</option>
					<option
						v-for="instance in view.instances"
						:key="instance.id"
						:value="instance.id"
						:disabled="
							instance.status !== 'ready' || current.excluded_instances.includes(instance.id)
						"
					>
						{{ instance.name }} · {{ instance.game_version }}
					</option>
				</select>
			</label>
			<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.sourceHelp) }}</p>
			<h3 class="m-0 text-lg text-contrast">{{ formatMessage(messages.instances) }}</h3>
			<p v-if="!view.instances.length">{{ formatMessage(messages.empty) }}</p>
			<div
				v-for="instance in view.instances"
				:key="instance.id"
				class="flex items-center justify-between gap-3 rounded-xl bg-surface-3 p-3"
			>
				<Checkbox
					:model-value="!current.excluded_instances.includes(instance.id)"
					:label="instance.name"
					:disabled="saving"
					@update:model-value="toggle(current.excluded_instances, instance.id, !$event)"
				/>
				<span class="text-right text-xs text-secondary"
					>{{ instance.game_version }} ·
					{{ formatMessage(messages[instance.status as 'ready']) }}</span
				>
			</div>
			<details class="rounded-xl bg-surface-3 p-3">
				<summary class="cursor-pointer font-semibold text-contrast">
					{{ formatMessage(messages.options) }}
				</summary>
				<input v-model="search" class="my-3 w-full" :placeholder="formatMessage(messages.search)" />
				<div class="flex flex-col gap-3">
					<Checkbox
						v-for="option in options"
						:key="option.id"
						:label="option.key"
						:model-value="!current.disabled_options.includes(option.id)"
						:disabled="saving"
						@update:model-value="toggle(current.disabled_options, option.id, !$event)"
					/>
				</div>
			</details>
		</template>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.limits) }}</p>
		<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.backups) }}</p>
		<p v-if="error || view.last_error" role="alert" class="text-red">
			{{ error || view.last_error }}
		</p>
	</div>
</template>
