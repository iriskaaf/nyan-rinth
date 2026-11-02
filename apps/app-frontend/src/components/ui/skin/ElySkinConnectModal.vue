<script setup lang="ts">
import { Button, NewModal } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { nextTick, ref } from 'vue'

const props = defineProps<{ user: string; username: string }>()
const emit = defineEmits<{ connected: [] }>()
const modal = ref<InstanceType<typeof NewModal> | null>(null)
const login = ref('')
const password = ref('')
const totp = ref('')
const busy = ref(false)
const error = ref('')
function clearSecrets() {
	password.value = ''
	totp.value = ''
}
function show() {
	clearSecrets()
	login.value = props.username
	error.value = ''
	modal.value?.show()
}
async function connect() {
	if (busy.value) return
	const user = props.user
	busy.value = true
	error.value = ''
	try {
		await invoke('plugin:auth|ely_skins_connect', {
			user,
			login: login.value,
			password: password.value,
			totp: totp.value || null,
		})
		clearSecrets()
		busy.value = false
		await nextTick()
		modal.value?.hide()
		emit('connected')
	} catch (e) {
		error.value =
			typeof e === 'string'
				? e
				: ((e as { message?: string }).message ??
					'Не удалось подключить управление скинами Ely.by.')
	} finally {
		busy.value = false
	}
}
defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		header="Подключить скины Ely.by"
		max-width="480px"
		:disable-close="busy"
		@hide="clearSecrets"
	>
		<form class="flex flex-col gap-4 p-6" @submit.prevent="connect">
			<p class="m-0">
				Один раз войдите в <strong>{{ username }}</strong
				>, чтобы загружать и применять скины прямо здесь. Окно браузера не откроется.
			</p>
			<label class="flex flex-col gap-2"
				>Почта или никнейм<input
					v-model="login"
					class="input"
					autocomplete="username"
					:disabled="busy"
					required
			/></label>
			<label class="flex flex-col gap-2"
				>Пароль<input
					v-model="password"
					class="input"
					type="password"
					autocomplete="current-password"
					:disabled="busy"
					required
			/></label>
			<label class="flex flex-col gap-2"
				>Код 2FA, если включён<input
					v-model="totp"
					class="input"
					inputmode="numeric"
					autocomplete="one-time-code"
					pattern="[0-9]{6}"
					maxlength="6"
					:disabled="busy"
			/></label>
			<p v-if="error" role="alert" class="m-0 text-red">{{ error }}</p>
			<p class="m-0 text-sm text-secondary">
				Данные отправляются только Ely.by по HTTPS. Пароль не сохраняется. Сессия сайта хранится
				зашифрованной средствами Windows и привязана к этому аккаунту.
			</p>
			<Button
				native-type="submit"
				type="colored"
				color="brand"
				:disabled="busy || !login.trim() || !password"
				>{{ busy ? 'Подключаем…' : 'Подключить управление скинами' }}</Button
			>
		</form>
	</NewModal>
</template>
