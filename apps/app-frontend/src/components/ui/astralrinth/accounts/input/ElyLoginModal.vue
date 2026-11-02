<script setup lang="ts">
import { Button, NewModal, injectNotificationManager } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { ref } from 'vue'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'

const emit = defineEmits<{ authenticated: [credentials: MinecraftCredential] }>()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const username = ref('')
const password = ref('')
const totp = ref('')
const busy = ref(false)
const error = ref('')
const connectingSkins = ref(false)
const { addNotification } = injectNotificationManager()

async function login() {
	if (busy.value) return
	busy.value = true
	error.value = ''
	try {
		const credentials = await invoke<MinecraftCredential>('plugin:auth|ely_login', {
			username: username.value,
			password: password.value,
			totp: totp.value || null,
		})
		connectingSkins.value = true
		try {
			const connected = await invoke<boolean>('plugin:auth|ely_skins_connected', {
				user: credentials.profile.id,
			})
			if (!connected) {
				await invoke('plugin:auth|ely_skins_connect', {
					user: credentials.profile.id,
					login: username.value,
					password: password.value,
					totp: totp.value || null,
				})
			}
		} catch (e) {
			addNotification({
				type: 'warning',
				title: 'Вход выполнен, но управление скинами не подключилось',
				text: `${typeof e === 'string' ? e : ((e as { message?: string }).message ?? 'Ely.by недоступен.')} Повторите подключение в разделе «Скины».`,
			})
		}
		password.value = ''
		totp.value = ''
		emit('authenticated', credentials)
		modal.value?.hide()
	} catch (e) {
		error.value =
			typeof e === 'string'
				? e
				: ((e as { message?: string }).message ??
					'Не удалось войти в Ely.by. Проверьте данные и подключение.')
	} finally {
		clearSecrets()
		connectingSkins.value = false
		busy.value = false
	}
}

function show() {
	error.value = ''
	password.value = ''
	totp.value = ''
	modal.value?.show()
}
function clearSecrets() {
	password.value = ''
	totp.value = ''
}
defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		header="Вход в Ely.by"
		max-width="480px"
		:disable-close="busy"
		@hide="clearSecrets"
	>
		<form class="flex flex-col gap-4 p-6" @submit.prevent="login">
			<p class="m-0">Один вход для игры и управления скинами Ely.by во всех ваших сборках.</p>
			<label class="flex flex-col gap-2"
				>Почта или никнейм
				<input v-model="username" class="input" autocomplete="username" required :disabled="busy" />
			</label>
			<label class="flex flex-col gap-2"
				>Пароль
				<input
					v-model="password"
					class="input"
					type="password"
					autocomplete="current-password"
					required
					:disabled="busy"
				/>
			</label>
			<label class="flex flex-col gap-2"
				>Код 2FA, если включён
				<input
					v-model="totp"
					class="input"
					inputmode="numeric"
					autocomplete="one-time-code"
					maxlength="6"
					pattern="[0-9]{6}"
					:disabled="busy"
				/>
			</label>
			<p v-if="error" role="alert" class="m-0 text-red">{{ error }}</p>
			<p class="m-0 text-sm">Пароль отправляется только в Ely.by по HTTPS и не сохраняется.</p>
			<Button
				type="colored"
				color="brand"
				:disabled="busy || !username.trim() || !password"
				@click="login"
			>
				{{ connectingSkins ? 'Подключаем скины…' : busy ? 'Входим…' : 'Войти' }}
			</Button>
		</form>
	</NewModal>
</template>
