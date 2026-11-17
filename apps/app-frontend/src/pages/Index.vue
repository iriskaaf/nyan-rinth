<script setup lang="ts">
import { LogInIcon, PlayIcon, PlusIcon } from '@modrinth/assets'
import { Button, defineMessages, injectNotificationManager, useVIntl } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, inject, onActivated, ref, type Ref } from 'vue'

import { minecraftAccounts } from '@/composables/use-minecraft-accounts'
import type AccountsCard from '@/components/ui/AccountsCard.vue'
import ContextMenu from '@/components/ui/context-menu/index.vue'
import LibrarySection from '@/components/ui/library/index.vue'
import WelcomeScreen from '@/components/ui/WelcomeScreen.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { useAppEvent } from '@/composables/use-app-event'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { toError } from '@/helpers/errors'
import { list } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'
import { useRootBreadcrumb } from '@/providers/breadcrumbs'
import { injectOnboardingChecklist } from '@/providers/onboarding-checklist'

defineOptions({
	name: 'LibraryPage',
})

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const { hasCreatedInstance, isReady } = injectOnboardingChecklist()
const showCreationModal = inject<() => void>('showCreationModal')
const pageOptions = ref<InstanceType<typeof ContextMenu>>()
const appSettings = useAppSettings()
const accountsCard = inject<Ref<InstanceType<typeof AccountsCard> | null>>('accountsCard')
const needsAccount = computed(
	() => minecraftAccounts.value !== null && minecraftAccounts.value.length === 0,
)

const messages = defineMessages({
	signInTitle: {
		id: 'nyanrinth.welcome.sign-in-title',
		defaultMessage: 'Sign in',
	},
	signInDescription: {
		id: 'nyanrinth.welcome.sign-in-description',
		defaultMessage: 'Add a game account to start playing in NyanRinth.',
	},
	signIn: {
		id: 'nyanrinth.welcome.sign-in',
		defaultMessage: 'Sign in',
	},
	home: {
		id: 'app.navigation.home',
		defaultMessage: 'Home',
	},
	newInstance: {
		id: 'app.library.context-menu.create-instance',
		defaultMessage: 'New instance',
	},
})

const homeBreadcrumb = useRootBreadcrumb({
	slot: 'root',
	id: 'home',
	label: formatMessage(messages.home),
	to: '/',
	visual: { type: 'icon', component: PlayIcon },
})
onActivated(homeBreadcrumb.reset)

const instances = ref<GameInstance[]>([])
let latestInstanceFetch = 0

const recentInstances = computed(() =>
	instances.value
		.slice()
		.sort((a, b) => dayjs(b.last_played ?? b.created).diff(dayjs(a.last_played ?? a.created))),
)

async function fetchInstances() {
	const fetchId = ++latestInstanceFetch
	try {
		const nextInstances = await list()
		if (fetchId === latestInstanceFetch) {
			instances.value = nextInstances
		}
	} catch (error: unknown) {
		if (fetchId === latestInstanceFetch) {
			handleError(toError(error))
		}
	}
}

if (hasCreatedInstance.value) {
	await fetchInstances()
}

useAppEvent('instance', fetchInstances)
useAppEvent('instance_groups_changed', fetchInstances)

function openPageContextMenu(event: MouseEvent) {
	if (
		!(event.target instanceof HTMLElement) ||
		!event.target.hasAttribute('data-library-page-background')
	) {
		return
	}

	event.preventDefault()
	event.stopPropagation()
	pageOptions.value?.showMenu(event, {}, [{ name: 'new_instance' }])
}

function handlePageOption({ option }: { option: string }) {
	if (option === 'new_instance') {
		showCreationModal?.()
	}
}
</script>

<template>
	<div v-if="isReady && needsAccount" class="flex min-h-full items-center justify-center p-6">
		<section
			class="flex w-full max-w-md flex-col items-center gap-4 rounded-2xl border border-solid border-surface-5 bg-button-bg p-8 text-center"
			aria-labelledby="welcome-sign-in-title"
		>
			<LogInIcon class="size-10 text-brand" aria-hidden="true" />
			<h1 id="welcome-sign-in-title" class="m-0 text-2xl font-semibold text-contrast">
				{{ formatMessage(messages.signInTitle) }}
			</h1>
			<p class="m-0 text-secondary">{{ formatMessage(messages.signInDescription) }}</p>
			<Button type="colored" color="brand" size="lg" @click="accountsCard?.showAccountLoginModal()">
				<LogInIcon />
				{{ formatMessage(messages.signIn) }}
			</Button>
		</section>
	</div>
	<WelcomeScreen v-else-if="isReady && !hasCreatedInstance" />
	<div
		v-else-if="isReady"
		data-library-page-background
		class="flex flex-col gap-3 p-6"
		@contextmenu="openPageContextMenu"
	>
		<RecentWorldsList
			v-if="recentInstances?.length > 0 && appSettings.getFeatureFlag('worlds_in_home')"
			:recent-instances="recentInstances"
		/>
		<LibrarySection :instances="instances" />
		<ContextMenu ref="pageOptions" @option-clicked="handlePageOption">
			<template #new_instance> <PlusIcon /> {{ formatMessage(messages.newInstance) }} </template>
		</ContextMenu>
	</div>
</template>
