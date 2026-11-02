<script setup>
import {
	CheckIcon,
	CopyIcon,
	DropdownIcon,
	HammerIcon,
	LogInIcon,
	UpdatedIcon,
	WrenchIcon,
	XIcon,
} from '@modrinth/assets'
import {
	IntlFormatted,
	useVIntl,
	Button,
	ButtonLink,
	Collapsible,
	IconButton,
	injectNotificationManager,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { nyanMessages } from '@/helpers/ui-messages'

import { ChatIcon } from '@/assets/icons'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { handleSevereError } from '@/composables/use-error.js'
import { trackEvent } from '@/helpers/analytics'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { install_existing_instance } from '@/helpers/install'
import { cancel_directory_change } from '@/helpers/settings.ts'

const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()

const errorModal = ref()
const error = ref()
const closable = ref(true)
const errorCollapsed = ref(false)

const title = ref(formatMessage(nyanMessages.errorOccurred))
const errorType = ref('unknown')
const supportLink = ref('https://xorison.dev/product/astralrinth/support')
const metadata = ref({})

defineExpose({
	async show(errorVal, context, canClose = true, source = null) {
		console.log(errorVal, context, canClose, source)
		closable.value = canClose

		if (errorVal.message && errorVal.message.includes('Minecraft authentication error:')) {
			title.value = formatMessage(nyanMessages.minecraftSignInError)
			errorType.value = 'minecraft_auth'
			supportLink.value =
				'https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues'

			if (
				errorVal.message.includes('existing connection was forcibly closed') ||
				errorVal.message.includes('error sending request for url')
			) {
				metadata.value.network = true
			}
			if (errorVal.message.includes('because the target machine actively refused it')) {
				metadata.value.hostsFile = true
			}
		} else if (errorVal.message && errorVal.message.includes('Move directory error:')) {
			title.value = formatMessage(nyanMessages.moveDirectoryError)
			errorType.value = 'directory_move'
			supportLink.value = 'https://support.modrinth.com'

			if (errorVal.message.includes('directory is not writable')) {
				metadata.value.readOnly = true
			}

			if (errorVal.message.includes('Not enough space')) {
				metadata.value.notEnoughSpace = true
			}
		} else if (errorVal.message && errorVal.message.includes('No loader version selected for')) {
			title.value = formatMessage(nyanMessages.noLoader)
			errorType.value = 'no_loader_version'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value.instanceId = context.instanceId
		} else if (errorVal.code === 'external_auth_library_not_installed') {
			title.value = formatMessage(nyanMessages.authLibraryRequired)
			errorType.value = 'external_auth_library_not_installed'
			supportLink.value = 'https://xorison.dev/product/astralrinth/support'
			metadata.value = {}
		} else if (source === 'state_init') {
			title.value = formatMessage(nyanMessages.initializationError)
			errorType.value = 'state_init'
			supportLink.value = 'https://support.modrinth.com'
		} else {
			title.value = formatMessage(nyanMessages.errorOccurred)
			errorType.value = 'unknown'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value = {}
		}

		error.value = errorVal
		errorModal.value.show()
	},
})

const loadingMinecraft = ref(false)
async function loginMinecraft() {
	try {
		loadingMinecraft.value = true
		const loggedIn = await login_flow()

		if (loggedIn) {
			await set_default_user(loggedIn.profile.id).catch(handleError)
		}

		await trackEvent('AccountLogIn', { source: 'ErrorModal' })
		loadingMinecraft.value = false
		errorModal.value.hide()
	} catch (err) {
		loadingMinecraft.value = false
		handleSevereError(err)
	}
}

async function cancelDirectoryChange() {
	try {
		await cancel_directory_change()
		window.location.reload()
	} catch (err) {
		handleError(err)
	}
}

function retryDirectoryChange() {
	window.location.reload()
}

const loadingRepair = ref(false)
async function repairInstance() {
	loadingRepair.value = true
	try {
		await install_existing_instance(metadata.value.instanceId, false)
		errorModal.value.hide()
	} catch (err) {
		handleSevereError(err)
	}
	loadingRepair.value = false
}

const hasDebugInfo = computed(
	() =>
		errorType.value === 'directory_move' ||
		errorType.value === 'minecraft_auth' ||
		errorType.value === 'state_init' ||
		errorType.value === 'no_loader_version' ||
		errorType.value === 'external_auth_library_not_installed',
)

const debugInfo = computed(() => error.value.message ?? error.value ?? 'No error message.')

const copied = ref(false)

async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}
</script>

<template>
	<ModalWrapper ref="errorModal" :header="title" :closable="closable">
		<div class="modal-body max-w-[550px]">
			<div class="markdown-body">
				<template v-if="errorType === 'minecraft_auth'">
					<template v-if="metadata.network">
						<h3>{{ formatMessage(nyanMessages.networkIssues) }}</h3>
						<p>
							<IntlFormatted :message-id="nyanMessages.networkBody"
								><template #support="{ children }"
									><a
										href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_e71a5f805f"
										><component :is="() => children" /></a></template
							></IntlFormatted>
						</p>
					</template>
					<template v-else-if="metadata.hostsFile">
						<h3>{{ formatMessage(nyanMessages.networkIssues) }}</h3>
						<p>
							<IntlFormatted :message-id="nyanMessages.hostsBody"
								><template #support="{ children }"
									><a
										href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_d694a29256"
										><component :is="() => children" /></a></template
							></IntlFormatted>
						</p>
					</template>
					<template v-else>
						<h3>{{ formatMessage(nyanMessages.tryOtherMicrosoft) }}</h3>
						<p>{{ formatMessage(nyanMessages.checkMicrosoft) }}</p>
						<div class="cta-button">
							<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
								<LogInIcon /> {{ formatMessage(nyanMessages.tryOtherAccount) }}
							</button>
						</div>
						<h3>{{ formatMessage(nyanMessages.newGameAccount) }}</h3>
						<p>
							<IntlFormatted :message-id="nyanMessages.officialLauncherBody"
								><template #launcher="{ children }"
									><a href="https://www.minecraft.net/en-us/download"
										><component :is="() => children" /></a></template
							></IntlFormatted>
						</p>
					</template>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
							<LogInIcon /> {{ formatMessage(nyanMessages.trySigningInAgain) }}
						</button>
					</div>
				</template>
				<template v-if="errorType === 'directory_move'">
					<template v-if="metadata.readOnly">
						<h3>{{ formatMessage(nyanMessages.changePermissions) }}</h3>
						<p>{{ formatMessage(nyanMessages.permissionsBody) }}</p>
					</template>
					<template v-else-if="metadata.notEnoughSpace">
						<h3>{{ formatMessage(nyanMessages.notEnoughSpace) }}</h3>
						<p>{{ formatMessage(nyanMessages.spaceBody) }}</p>
					</template>
					<template v-else>
						<p>{{ formatMessage(nyanMessages.moveBody) }}</p>
					</template>

					<div class="cta-button">
						<button class="btn" @click="retryDirectoryChange">
							<UpdatedIcon /> {{ formatMessage(nyanMessages.retryMove) }}
						</button>
						<button class="btn btn-danger" @click="cancelDirectoryChange">
							<XIcon /> {{ formatMessage(nyanMessages.cancelMove) }}
						</button>
					</div>
				</template>
				<template v-else-if="errorType === 'state_init'">
					<p>{{ formatMessage(nyanMessages.initBody) }}</p>
					<p>{{ formatMessage(nyanMessages.initOptions) }}</p>
					<ul>
						<li>{{ formatMessage(nyanMessages.restartApp) }}</li>
						<li>{{ formatMessage(nyanMessages.redownloadApp) }}</li>
					</ul>
				</template>
				<template v-else-if="errorType === 'no_loader_version'">
					<p>{{ formatMessage(nyanMessages.loaderBody) }}</p>
					<p>{{ formatMessage(nyanMessages.repairBody) }}</p>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingRepair" @click="repairInstance">
							<HammerIcon /> {{ formatMessage(nyanMessages.repairInstance) }}
						</button>
					</div>
				</template>
				<template v-else-if="errorType === 'external_auth_library_not_installed'">
					<p>{{ debugInfo }}</p>
					<p>{{ formatMessage(nyanMessages.authLibraryBody) }}</p>
				</template>
				<template v-else>
					{{ debugInfo }}
				</template>
				<template v-if="hasDebugInfo">
					<div class="w-full h-[1px] bg-surface-5 mb-3"></div>
					<p>
						<IntlFormatted :message-id="nyanMessages.supportBody"
							><template #support="{ children }"
								><a :href="supportLink"><component :is="() => children" /></a></template
						></IntlFormatted>
					</p>
				</template>
			</div>
			<div class="flex items-center gap-2">
				<ButtonLink :href="supportLink" @click="errorModal.hide()"
					><ChatIcon /> {{ formatMessage(nyanMessages.getSupport) }}
				</ButtonLink>
				<Button v-if="closable" @click="errorModal.hide()"
					><XIcon /> {{ formatMessage(nyanMessages.close) }}
				</Button>
			</div>
			<template v-if="hasDebugInfo">
				<div class="flex flex-col gap-2">
					<div class="w-full h-[1px] bg-surface-5"></div>

					<div class="overflow-clip">
						<button
							class="flex items-center justify-between w-full bg-transparent border-0 py-4 cursor-pointer"
							@click="errorCollapsed = !errorCollapsed"
						>
							<span class="flex items-center gap-2 text-contrast font-extrabold m-0">
								<WrenchIcon class="h-4 w-4" /> {{ formatMessage(nyanMessages.debugInfo) }}
							</span>
							<DropdownIcon
								class="h-5 w-5 text-secondary transition-transform"
								:class="{ 'rotate-180': !errorCollapsed }"
							/>
						</button>
						<Collapsible :collapsed="errorCollapsed">
							<div
								class="p-3 bg-surface-2 rounded-2xl text-xs grid grid-cols-[1fr_auto] max-w-full items-start"
							>
								<div
									class="m-0 p-0 rounded-none bg-transparent text-sm font-mono break-words overflow-auto"
								>
									{{ debugInfo }}
								</div>
								<IconButton
									v-tooltip="formatMessage(nyanMessages.copyDebug)"
									:label="formatMessage(nyanMessages.copyDebug)"
									:disabled="copied"
									@click="copyToClipboard(debugInfo)"
								>
									<template v-if="copied"> <CheckIcon class="text-green" /> </template>
									<template v-else> <CopyIcon /> </template>
								</IconButton>
							</div>
						</Collapsible>
					</div>
				</div>
			</template>
		</div>
	</ModalWrapper>
</template>

<style>
.light-mode {
	--color-orange-bg: rgba(255, 163, 71, 0.2);
}

.dark-mode,
.oled-mode {
	--color-orange-bg: rgba(224, 131, 37, 0.2);
}
</style>

<style scoped lang="scss">
.cta-button {
	display: flex;
	align-items: center;
	justify-content: center;
	padding: 0.5rem;
	gap: 0.5rem;
}

.warning-banner {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: var(--gap-lg);
	background-color: var(--color-orange-bg);
	border: 2px solid var(--color-orange);
	border-radius: var(--radius-md);
	margin-bottom: 1rem;
}

.warning-banner__title {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	font-weight: 700;

	svg {
		color: var(--color-orange);
		height: 1.5rem;
		width: 1.5rem;
	}
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);
}

.markdown-body {
	overflow: auto;
}
</style>
