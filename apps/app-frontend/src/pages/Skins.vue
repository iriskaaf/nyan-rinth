<script setup lang="ts">
import { Button, DropdownSelect, SkinPreviewRenderer } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, inject, onMounted, onUnmounted, ref, watch, type Ref } from 'vue'
import ElyLoginModal from '@/components/ui/astralrinth/accounts/input/ElyLoginModal.vue'
import { users, get_default_user, set_default_user } from '@/helpers/auth'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'

const accountsCard = inject<Ref<{ refreshValues: () => Promise<void> } | null>>('accountsCard')
const accounts = ref<MinecraftCredential[]>([])
const selected = ref('')
const texture = ref('')
const loading = ref(false)
const error = ref('')
const variant = ref<'classic' | 'slim'>('classic')
const loginModal = ref<InstanceType<typeof ElyLoginModal> | null>(null)
const account = computed(() => accounts.value.find((a) => a.profile.id === selected.value))
let request = 0
async function refresh() {
	const current = ++request
	loading.value = true
	error.value = ''
	try {
		const all = (await users()) as MinecraftCredential[]
		accounts.value = all.filter((a) => a.account_type === 'elyby')
		if (!accounts.value.some((a) => a.profile.id === selected.value)) {
			const active = await get_default_user()
			selected.value =
				accounts.value.find((a) => a.profile.id === active)?.profile.id ??
				accounts.value[0]?.profile.id ??
				''
		}
		if (!account.value) {
			texture.value = ''
			return
		}
		const skin = await invoke<{ texture: string; variant: 'classic' | 'slim' } | null>(
			'plugin:auth|ely_skin_texture',
			{ username: account.value.profile.name },
		)
		if (current === request) {
			texture.value = skin?.texture ?? ''
			variant.value = skin?.variant ?? 'classic'
		}
	} catch {
		if (current === request)
			error.value = 'Не удалось обновить скин Ely.by. Проверьте подключение и повторите попытку.'
	} finally {
		if (current === request) loading.value = false
	}
}
async function authenticated(credentials: MinecraftCredential) {
	await set_default_user(credentials.profile.id)
	await accountsCard?.value?.refreshValues()
	selected.value = credentials.profile.id
	await refresh()
}
async function openCatalog(upload = false) {
	try {
		await invoke('plugin:auth|open_ely_skins', { upload })
	} catch {
		error.value = 'Не удалось открыть каталог Ely.by.'
	}
}
watch(selected, () => {
	texture.value = ''
	void refresh()
})
onMounted(() => {
	void refresh()
	window.addEventListener('focus', refresh)
})
onUnmounted(() => {
	request++
	window.removeEventListener('focus', refresh)
})
</script>

<template>
	<div class="mx-auto w-full max-w-6xl p-8">
		<div class="mb-8 flex items-center justify-between gap-4">
			<div>
				<div class="mb-2 text-sm font-semibold text-brand">NYANRINTH / ELY.BY</div>
				<h1 class="m-0 text-3xl font-bold">Ваш скин. Во всех сборках.</h1>
			</div>
			<Button @click="loginModal?.show()">Добавить Ely.by</Button>
		</div>
		<div class="grid grid-cols-2 gap-8">
			<div class="relative h-[520px] rounded-3xl bg-bg-raised p-6">
				<SkinPreviewRenderer
					v-if="texture"
					:texture-src="texture"
					:variant="variant === 'slim' ? 'SLIM' : 'CLASSIC'"
					:key="`${selected}-${variant}`"
					:nametag="account?.profile.name ?? ''"
					:initial-rotation="Math.PI / 8"
				/>
				<div v-else class="flex h-full items-center justify-center text-center text-lg">
					{{
						loading
							? 'Загружаем скин…'
							: account
								? 'Скин ещё не выбран. Откройте каталог Ely.by.'
								: 'Войдите в Ely.by, чтобы увидеть свой скин.'
					}}
				</div>
			</div>
			<div class="flex flex-col justify-center gap-6">
				<div class="rounded-2xl border border-solid border-brand p-6">
					<h2 class="mt-0 text-xl">Скины Ely.by</h2>
					<p>
						Выберите скин в каталоге или загрузите PNG в свой аккаунт Ely.by. Каталог откроется в
						отдельном окне NyanRinth; при первом открытии войдите в тот же аккаунт.
					</p>
					<div class="flex flex-wrap gap-3">
						<Button type="colored" color="brand" @click="openCatalog()">Выбрать скин</Button>
						<Button @click="openCatalog(true)">Загрузить PNG</Button>
					</div>
				</div>
				<div v-if="accounts.length" class="flex flex-col gap-2">
					<span class="text-sm font-semibold text-secondary">Аккаунт Ely.by</span>
					<DropdownSelect
						v-if="accounts.length > 1"
						v-model="selected"
						name="ely-preview-account"
						:options="accounts.map((a) => a.profile.id)"
						:display-name="
							(id: string) => accounts.find((a) => a.profile.id === id)?.profile.name ?? id
						"
						class="w-full"
					/>
					<div v-else class="flex items-center gap-3 rounded-xl bg-surface-3 px-4 py-3">
						<span
							class="flex size-9 items-center justify-center rounded-lg bg-brand-highlight font-bold text-brand"
							>{{ account?.profile.name.slice(0, 1).toUpperCase() }}</span
						>
						<span class="font-semibold text-contrast">{{ account?.profile.name }}</span>
						<span class="ml-auto rounded-full bg-surface-4 px-2.5 py-1 text-xs text-secondary"
							>Ely.by</span
						>
					</div>
				</div>
				<div v-if="account" class="flex items-center gap-3">
					<Button :disabled="loading" @click="refresh">{{
						loading ? 'Обновляем…' : 'Обновить скин'
					}}</Button>
					<span>{{ variant === 'slim' ? 'Модель Alex' : 'Модель Steve' }}</span>
				</div>
				<p v-if="error" role="alert" class="text-red">{{ error }}</p>
				<p class="m-0">
					Для игры выберите аккаунт Ely.by в меню аккаунтов. Система скинов подключится
					автоматически при запуске Vanilla, Fabric, Forge, NeoForge и Quilt. После смены скина
					перезапустите игру; обновление кэша Ely.by может занять около минуты.
				</p>
				<p class="m-0 text-sm">
					На старых версиях до 1.7.6 скины могут требовать отдельного патча. Другие игроки увидят
					скин, если их клиент или сервер поддерживает Ely.by.
				</p>
			</div>
		</div>
		<ElyLoginModal ref="loginModal" @authenticated="authenticated" />
	</div>
</template>
