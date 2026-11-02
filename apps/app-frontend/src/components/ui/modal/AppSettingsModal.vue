<script setup lang="ts">
import {
	AstralRinthLogo,
	BadgeCheckIcon,
	CoffeeIcon,
	DownloadIcon,
	GameIcon,
	GaugeIcon,
	HeartHandshakeIcon,
	LanguagesIcon,
	PaintbrushIcon,
	RefreshCwIcon,
	Settings2Icon,
	ShieldIcon,
	SpinnerIcon,
	ToggleRightIcon,
	UserIcon,
} from '@modrinth/assets'
import {
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	NewModal,
	ProgressBar,
	TabbedModal,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, provide, ref, watch } from 'vue'

import LauncherUpdateModal from '@/components/ui/astralrinth/LauncherUpdateModal.vue'
import ExternalAuthLibrarySettings from '@/components/ui/settings/astralrinth/ExternalAuthLibrarySettings.vue'
import UpdateSettings from '@/components/ui/settings/astralrinth/UpdateSettings.vue'
import {
	isUpdateAvailable,
	isUpdateInstalling,
	latestLauncherReleases,
} from '@/helpers/astralrinth/update'
import PrivacySettings from '@/components/ui/settings/account/PrivacySettings.vue'
import ProfileSettings from '@/components/ui/settings/account/ProfileSettings.vue'
import SocialSettings from '@/components/ui/settings/account/SocialSettings.vue'
import AppearanceSettings from '@/components/ui/settings/display/AppearanceSettings.vue'
import BehaviorSettings from '@/components/ui/settings/display/BehaviorSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/display/FeatureFlagSettings.vue'
import LanguageSettings from '@/components/ui/settings/display/LanguageSettings.vue'
import GameSyncSettings from '@/components/ui/settings/instances/GameSyncSettings.vue'
import MemorySettings from '@/components/ui/settings/instances/MemorySettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/instances/DefaultInstanceSettings.vue'
import JavaSettings from '@/components/ui/settings/instances/JavaSettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/instances/ResourceManagementSettings.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import {
	appSettingsModalContextKey,
	type UnsavedChangesController,
} from '@/providers/app-settings-modal'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'

// TODO: Apply COMPONENT_STRUCTURE.md here and extract out common setting option components
const appSettings = useAppSettings()

const { formatMessage } = useVIntl()

const aboutModal = ref<InstanceType<typeof NewModal> | null>(null)
const launcherUpdateModal = ref<InstanceType<typeof LauncherUpdateModal> | null>(null)

const tabCategories = defineMessages({
	display: {
		id: 'settings.sidebar.label.display',
		defaultMessage: 'Display',
	},
	account: {
		id: 'settings.sidebar.label.account',
		defaultMessage: 'Account',
	},
	instances: {
		id: 'app.settings.sidebar.label.instances',
		defaultMessage: 'Instances',
	},
	astralrinth: {
		id: 'astralrinth.app.settings.sidebar.label.astralrinth',
		defaultMessage: 'NyanRinth',
	},
})

const tabs = [
	{
		name: defineMessage({ id: 'nyanrinth.settings.memory', defaultMessage: 'Оперативная память' }),
		category: tabCategories.astralrinth,
		icon: GaugeIcon,
		content: MemorySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.display,
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.display,
		icon: Settings2Icon,
		content: BehaviorSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		category: tabCategories.display,
		icon: LanguagesIcon,
		content: LanguageSettings,
		badge: commonMessages.beta,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		icon: ToggleRightIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
	{
		name: commonSettingsMessages.profile,
		category: tabCategories.account,
		icon: UserIcon,
		content: ProfileSettings,
	},
	{
		name: commonSettingsMessages.social,
		category: tabCategories.account,
		icon: HeartHandshakeIcon,
		content: SocialSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		category: tabCategories.account,
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({ id: 'nyanrinth.sync.title', defaultMessage: 'Game settings sync' }),
		category: tabCategories.instances,
		icon: RefreshCwIcon,
		content: GameSyncSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default game options',
		}),
		category: tabCategories.instances,
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		category: tabCategories.instances,
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		category: tabCategories.instances,
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'astralrinth.app.settings.tabs.external-auth-libraries',
			defaultMessage: 'Authentication libraries',
		}),
		category: tabCategories.astralrinth,
		icon: DownloadIcon,
		content: ExternalAuthLibrarySettings,
	},
	{
		name: defineMessage({
			id: 'astralrinth.app.settings.tabs.updates',
			defaultMessage: 'Updates',
		}),
		category: tabCategories.astralrinth,
		icon: RefreshCwIcon,
		content: UpdateSettings,
		developerOnly: true,
	},
]

const availableTabs = computed(() =>
	tabs.filter((tab) => !tab.developerOnly || appSettings.devMode),
)

const modal = ref<InstanceType<typeof TabbedModal> | null>(null)
const unsavedChangesPopup = ref<{ nudge: () => void } | null>(null)
const unsavedChangesController = ref<UnsavedChangesController | null>(null)
const emptyUnsavedChangesState: Record<string, unknown> = {}
const originalUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getOriginal() ?? emptyUnsavedChangesState,
)
const modifiedUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getModified() ?? emptyUnsavedChangesState,
)
const savingUnsavedChanges = computed(() => unsavedChangesController.value?.isSaving() ?? false)
const hasUnsavedChanges = computed(
	() =>
		(unsavedChangesController.value?.hasChanges() ?? false) ||
		(unsavedChangesController.value?.isSaving() ?? false),
)

function canLeaveCurrentTab(): boolean {
	if (
		!unsavedChangesController.value?.hasChanges() &&
		!unsavedChangesController.value?.isSaving()
	) {
		return true
	}
	unsavedChangesPopup.value?.nudge()
	return false
}

function close(): boolean {
	return modal.value?.hide() ?? false
}

function registerUnsavedChangesController(controller: UnsavedChangesController | null): void {
	unsavedChangesController.value = controller
}

provide(appSettingsModalContextKey, {
	close,
	registerUnsavedChangesController,
})

function resetUnsavedChanges(): void {
	unsavedChangesController.value?.reset()
}

function saveUnsavedChanges(): void {
	void unsavedChangesController.value?.save()
}

function show() {
	modal.value?.show()
}

function showUpdateModal() {
	modal.value?.show()
	void launcherUpdateModal.value?.show()
}

function showProfile(): void {
	const profileTabIndex = availableTabs.value.findIndex((tab) => tab.content === ProfileSettings)
	if (profileTabIndex >= 0) {
		modal.value?.setTab(profileTabIndex)
	}
	modal.value?.show()
}

defineExpose({ show, showProfile, showUpdateModal })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
	updateInstalling: {
		id: 'astralrinth.app.settings.update-installing',
		defaultMessage: 'Installing update...',
	},
	updatesInstalled: {
		id: 'astralrinth.app.settings.updates-installed',
		defaultMessage: 'Updates are already installed',
	},
	viewUpdateInfo: {
		id: 'astralrinth.app.settings.view-update-info',
		defaultMessage: 'View update info',
	},
	appVersion: {
		id: 'app.settings.app-version',
		defaultMessage: 'NyanRinth App {version}',
	},
	macos: {
		id: 'app.settings.operating-system.macos',
		defaultMessage: 'macOS',
	},
	developerModeButtonLabel: {
		id: 'nyanrinth.about.title',
		defaultMessage: 'About launcher',
	},
})
</script>

<template>
	<TabbedModal
		ref="modal"
		:tabs="availableTabs"
		:width="'min(928px, calc(95vw - 10rem))'"
		:before-hide="canLeaveCurrentTab"
		:before-tab-change="canLeaveCurrentTab"
		:floating-action-bar-shown="hasUnsavedChanges"
	>
		<template #title>
			<span class="text-2xl font-semibold text-contrast">
				{{ formatMessage(commonMessages.settingsLabel) }}
				<span class="ml-2 rounded-full bg-brand-highlight px-2 py-1 text-xs text-brand">Alpha</span>
			</span>
		</template>
		<template #floating-action-bar>
			<UnsavedChangesPopup
				ref="unsavedChangesPopup"
				:original="originalUnsavedChangesState"
				:modified="modifiedUnsavedChangesState"
				:saving="savingUnsavedChanges"
				inline
				@reset="resetUnsavedChanges"
				@save="saveUnsavedChanges"
			/>
		</template>
		<template #footer>
			<div class="mt-auto text-secondary text-sm">
				<div class="mb-3">
					<template v-if="progress > 0 && progress < 1">
						<p class="m-0 mb-2">
							{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
						</p>
						<ProgressBar :progress="progress" />
					</template>
				</div>
				<div class="flex items-center gap-3">
					<button
						:aria-label="formatMessage(messages.developerModeButtonLabel)"
						class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
						@click="aboutModal?.show()"
					>
						<AstralRinthLogo aria-hidden="true" class="w-6 h-6" />
					</button>
					<div class="max-w-[200px]">
						<p class="m-0">
							{{ formatMessage(messages.appVersion, { version }) }}
						</p>
						<p class="m-0">
							<span v-if="osPlatform === 'macos'">{{ formatMessage(messages.macos) }}</span>
							<span v-else class="capitalize">{{ osPlatform }}</span>
							{{ osVersion }}
						</p>
					</div>
					<div
						v-if="isUpdateAvailable"
						class="w-8 h-8 cursor-pointer hover:brightness-75 neon-icon pulse shrink-0"
					>
						<template v-if="isUpdateInstalling">
							<SpinnerIcon
								class="size-6 animate-spin"
								v-tooltip.bottom="formatMessage(messages.updateInstalling)"
							/>
						</template>
						<template v-else>
							<DownloadIcon
								class="size-6"
								v-tooltip.bottom="formatMessage(messages.viewUpdateInfo)"
								@click="showUpdateModal()"
							/>
						</template>
					</div>
					<BadgeCheckIcon
						v-else-if="latestLauncherReleases"
						class="size-7 shrink-0 text-green"
						v-tooltip.bottom="formatMessage(messages.updatesInstalled)"
					/>
				</div>
			</div>
		</template>
	</TabbedModal>

	<LauncherUpdateModal ref="launcherUpdateModal" :version="version" />
	<NewModal ref="aboutModal" :header="formatMessage(messages.developerModeButtonLabel)">
		<div class="flex max-w-lg flex-col gap-5">
			<div class="flex items-center gap-4">
				<AstralRinthLogo class="size-14" aria-hidden="true" />
				<div>
					<h2 class="m-0 text-2xl font-semibold text-contrast">NyanRinth</h2>
					<span class="text-brand">Alpha · {{ version }}</span>
				</div>
			</div>
			<dl class="m-0 grid grid-cols-[auto_1fr] gap-x-6 gap-y-3">
				<dt>Разработчик</dt>
				<dd class="m-0 text-contrast">iriska</dd>
				<dt>Версия исходного пакета</dt>
				<dd class="m-0 text-contrast">0.18.201 (AstralRinth)</dd>
				<dt>Основа</dt>
				<dd class="m-0">AstralRinth · beta · 69e9ef4</dd>
				<dt>Лицензия</dt>
				<dd class="m-0">GPL-3.0</dd>
				<dt>Система</dt>
				<dd class="m-0">{{ osPlatform }} {{ osVersion }}</dd>
			</dl>
			<p class="m-0 text-sm text-secondary">
				Ранняя тестовая версия. Возможны ошибки и изменения интерфейса.
			</p>
		</div>
	</NewModal>
</template>

<style lang="scss" scoped>
@import '../../../../../../packages/assets/styles/astralrinth/neon-icon.scss';
</style>
