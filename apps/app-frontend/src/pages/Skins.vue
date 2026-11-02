<script setup lang="ts">
import { Button, DropdownSelect, SkinPreviewRenderer } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, inject, onMounted, ref, watch, type Ref } from 'vue'
import ElyLoginModal from '@/components/ui/astralrinth/accounts/input/ElyLoginModal.vue'
import ElySkinConnectModal from '@/components/ui/skin/ElySkinConnectModal.vue'
import ElySkinThumbnail from '@/components/ui/skin/ElySkinThumbnail.vue'
import { users, get_default_user, set_default_user } from '@/helpers/auth'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'

type Skin = { id: number; skin_url: string; is_slim: boolean; tags: string[] }
type Catalog = { items: Skin[]; current: number; last: number }
type Preview = { skin: Skin; texture: string }
const accountsCard = inject<
	Ref<{
		refreshValues: () => Promise<void>
		setElySkinHead?: (user: string, texture: string) => Promise<void>
	} | null>
>('accountsCard')
const accounts = ref<MinecraftCredential[]>([])
const selected = ref('')
const account = computed(() => accounts.value.find((a) => a.profile.id === selected.value))
const equipped = ref({ texture: '', variant: 'classic' })
const pending = ref<Preview | null>(null)
const previewTexture = computed(() => pending.value?.texture ?? equipped.value.texture)
const previewVariant = computed(() =>
	(pending.value ? pending.value.skin.is_slim : equipped.value.variant === 'slim')
		? 'SLIM'
		: 'CLASSIC',
)
const catalog = ref<Catalog>({ items: [], current: 1, last: 1 })
const page = ref(1)
const tab = ref('Мои скины')
const collection = ref('Сохранённые')
const tag = ref('')
const model = ref('Все модели')
const loadingCatalog = ref(false)
const loadingSkin = ref(false)
const checkingSession = ref(false)
const connected = ref(false)
const busy = ref(false)
const error = ref('')
const catalogError = ref('')
const notice = ref('')
const loginModal = ref<InstanceType<typeof ElyLoginModal> | null>(null)
const connectModal = ref<InstanceType<typeof ElySkinConnectModal> | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
let previewRequest = 0
let catalogRequest = 0
let accountRequest = 0
function message(e: unknown) {
	return typeof e === 'string'
		? e
		: ((e as { message?: string }).message ?? 'Не удалось выполнить запрос Ely.by.')
}
async function loadCatalog() {
	const request = ++catalogRequest
	loadingCatalog.value = true
	catalogError.value = ''
	try {
		if (tab.value === 'Мои скины' && (!connected.value || !selected.value)) {
			catalog.value = { items: [], current: 1, last: 1 }
			return
		}
		const args = {
			page: page.value,
			model: model.value === 'Все модели' ? '' : 'slim',
			tags: tag.value.trim(),
		}
		const result =
			tab.value === 'Мои скины'
				? await invoke<Catalog>('plugin:auth|ely_skins_personal', {
						...args,
						user: selected.value,
						uploaded: collection.value === 'Мои загрузки',
					})
				: await invoke<Catalog>('plugin:auth|ely_skins_catalog', args)
		if (request === catalogRequest) catalog.value = result
	} catch (e) {
		if (request === catalogRequest) catalogError.value = message(e)
	} finally {
		if (request === catalogRequest) loadingCatalog.value = false
	}
}
async function refreshAccount() {
	const user = selected.value
	const request = ++accountRequest
	catalogRequest++
	catalog.value = { items: [], current: 1, last: 1 }
	previewRequest++
	pending.value = null
	error.value = ''
	notice.value = ''
	connected.value = false
	equipped.value = { texture: '', variant: 'classic' }
	if (!user || !account.value) return
	checkingSession.value = true
	const username = account.value.profile.name
	const results = await Promise.allSettled([
		invoke<{ texture: string; variant: string } | null>('plugin:auth|ely_skin_texture', {
			username,
		}),
		invoke<boolean>('plugin:auth|ely_skins_connected', { user }),
	])
	if (request !== accountRequest) return
	if (results[0].status === 'fulfilled' && results[0].value) equipped.value = results[0].value
	if (results[1].status === 'fulfilled') connected.value = results[1].value
	else error.value = message(results[1].reason)
	checkingSession.value = false
	if (tab.value === 'Мои скины') void loadCatalog()
}
async function loadAccounts(preferred?: string) {
	const all = (await users()) as MinecraftCredential[]
	accounts.value = all.filter((a) => a.account_type === 'elyby')
	const active = preferred ?? (await get_default_user())
	const next =
		accounts.value.find((a) => a.profile.id === active)?.profile.id ??
		accounts.value[0]?.profile.id ??
		''
	if (next === selected.value) await refreshAccount()
	else selected.value = next
}
async function authenticated(credentials: MinecraftCredential) {
	await set_default_user(credentials.profile.id)
	await accountsCard?.value?.refreshValues()
	await loadAccounts(credentials.profile.id)
}
async function choose(skin: Skin) {
	if (busy.value) return
	const request = ++previewRequest
	loadingSkin.value = true
	error.value = ''
	notice.value = ''
	try {
		const result = await invoke<Preview>('plugin:auth|ely_skins_preview', { skinId: skin.id })
		if (request === previewRequest) pending.value = result
	} catch (e) {
		if (request === previewRequest) error.value = message(e)
	} finally {
		if (request === previewRequest) loadingSkin.value = false
	}
}
function resetPreview() {
	previewRequest++
	pending.value = null
	loadingSkin.value = false
}
async function applySkin() {
	if (!pending.value || !connected.value || !account.value || busy.value) return
	const user = selected.value
	const preview = pending.value
	busy.value = true
	error.value = ''
	notice.value = ''
	try {
		await invoke('plugin:auth|ely_skins_apply', { user, skinId: preview.skin.id })
		equipped.value = {
			texture: preview.texture,
			variant: preview.skin.is_slim ? 'slim' : 'classic',
		}
		pending.value = null
		await accountsCard?.value?.setElySkinHead?.(user, preview.texture)
		notice.value =
			'Скин применён в Ely.by. Перезапустите Minecraft; обновление игрового кэша может занять около минуты.'
	} catch (e) {
		error.value = message(e)
	} finally {
		busy.value = false
	}
}
async function uploadFile(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	input.value = ''
	if (!file || busy.value || !connected.value) return
	if (file.size > 196608) {
		error.value = 'Размер PNG не должен превышать 192 КиБ.'
		return
	}
	busy.value = true
	error.value = ''
	notice.value = ''
	try {
		const skin = await invoke<Skin>('plugin:auth|ely_skins_upload', {
			user: selected.value,
			bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
		})
		const preview = await invoke<Preview>('plugin:auth|ely_skins_preview', { skinId: skin.id })
		previewRequest++
		pending.value = preview
		notice.value = 'PNG загружен в Ely.by. Проверьте предпросмотр и нажмите «Применить».'
	} catch (e) {
		error.value = message(e)
	} finally {
		busy.value = false
	}
}
async function disconnect() {
	if (busy.value) return
	busy.value = true
	try {
		await invoke('plugin:auth|ely_skins_disconnect', { user: selected.value })
		connected.value = false
		if (tab.value === 'Мои скины') void loadCatalog()
		notice.value = 'Управление скинами отключено. Игровой аккаунт остался в лаунчере.'
	} catch (e) {
		error.value = message(e)
	} finally {
		busy.value = false
	}
}
async function onConnected() {
	connected.value = true
	error.value = ''
	notice.value = 'Управление скинами подключено. Можно загружать PNG и применять скины.'
	void loadCatalog()
}
function search() {
	page.value = 1
	void loadCatalog()
}
function movePage(delta: number) {
	page.value += delta
	void loadCatalog()
}
watch(selected, refreshAccount)
watch([model, tab, collection], search)
onMounted(() => {
	void loadAccounts().catch((e) => {
		error.value = message(e)
	})
	void loadCatalog()
})
</script>

<template>
	<div class="mx-auto w-full max-w-[1440px] p-6 xl:p-8">
		<div class="mb-6 flex flex-wrap items-center justify-between gap-4">
			<div>
				<div class="mb-2 text-sm font-semibold text-brand">NYANRINTH / ELY.BY</div>
				<h1 class="m-0 text-3xl font-bold">Ваш скин. Во всех сборках.</h1>
			</div>
			<Button @click="loginModal?.show()">Добавить Ely.by</Button>
		</div>
		<div
			class="grid items-start gap-6 lg:grid-cols-[360px_minmax(0,1fr)] xl:grid-cols-[400px_minmax(0,1fr)]"
		>
			<div class="skin-preview-panel flex min-h-0 flex-col gap-3">
				<div class="skin-model relative shrink-0 rounded-3xl bg-surface-2 p-3">
					<SkinPreviewRenderer
						v-if="previewTexture"
						:key="`${selected}-${previewVariant}`"
						:texture-src="previewTexture"
						:fit="true"
						framing="modal"
						:lock-fit="false"
						:fit-padding="{ top: pending ? 0.22 : 0.08, right: 0.08, bottom: 0.2, left: 0.08 }"
						:variant="previewVariant"
						:initial-rotation="Math.PI / 8"
					/>
					<div v-else class="flex h-full items-center justify-center text-center text-secondary">
						{{
							account
								? checkingSession
									? 'Загружаем текущий скин…'
									: 'Скин не задан. Выберите его в каталоге.'
								: 'Войдите в Ely.by или выберите скин в каталоге.'
						}}
					</div>
					<span
						class="pointer-events-none absolute bottom-2 left-0 right-0 text-center text-xs text-secondary"
						>Перетащите, чтобы повернуть</span
					>
					<span
						v-if="pending"
						class="absolute left-4 top-4 rounded-full bg-brand-highlight px-3 py-1 text-sm font-semibold text-brand"
						>Предпросмотр · ещё не применён</span
					>
					<span
						v-if="loadingSkin"
						class="absolute bottom-4 left-4 rounded-lg bg-surface-3 px-3 py-2"
						>Загружаем…</span
					>
				</div>
				<div v-if="accounts.length" class="shrink-0 rounded-2xl bg-surface-2 p-3">
					<div class="mb-2 text-sm text-secondary">Аккаунт Ely.by</div>
					<DropdownSelect
						v-if="accounts.length > 1"
						v-model="selected"
						name="ely-skin-account"
						:disabled="busy"
						:options="accounts.map((a) => a.profile.id)"
						:display-name="
							(id: string) => accounts.find((a) => a.profile.id === id)?.profile.name ?? id
						"
					/>
					<div v-else class="flex items-center justify-between">
						<strong class="text-contrast">{{ account?.profile.name }}</strong
						><span class="rounded-full bg-surface-3 px-3 py-1 text-xs">Ely.by</span>
					</div>
				</div>
				<div v-if="account" class="shrink-0 rounded-2xl border border-solid border-surface-5 p-3">
					<template v-if="!connected"
						><p class="mt-0">
							{{
								checkingSession
									? 'Проверяем сессию…'
									: 'Подключите управление скинами один раз. Дальше всё работает здесь, без браузера.'
							}}
						</p>
						<Button
							type="colored"
							color="brand"
							:disabled="busy || checkingSession"
							@click="connectModal?.show()"
							>Подключить скины</Button
						></template
					>
					<template v-else
						><div class="mb-3 flex items-center justify-between gap-2">
							<span class="text-sm font-semibold text-brand">Управление подключено</span
							><Button size="sm" :disabled="busy" @click="disconnect">Отключить</Button>
						</div>
						<Button :disabled="busy" @click="fileInput?.click()">Загрузить PNG</Button
						><input
							ref="fileInput"
							type="file"
							class="hidden"
							accept="image/png,.png"
							@change="uploadFile"
						/>
						<p class="mb-0 text-xs text-secondary">
							64×64 или 64×32 · до 192 КиБ. Файл загрузится в Ely.by; применение — отдельной
							кнопкой.
						</p></template
					>
				</div>
				<div v-if="pending" class="flex flex-wrap items-center gap-3">
					<Button
						type="colored"
						color="brand"
						:disabled="busy || !connected || loadingSkin"
						@click="applySkin"
						>{{ busy ? 'Подождите…' : 'Применить скин' }}</Button
					><Button :disabled="busy" @click="resetPreview">Отмена</Button
					><span class="text-sm text-secondary">{{
						pending.skin.is_slim ? 'Alex · тонкие руки' : 'Steve · обычные руки'
					}}</span>
				</div>
				<p v-if="error" role="alert" class="m-0 text-red">{{ error }}</p>
				<p v-if="notice" role="status" class="m-0 text-sm text-brand">{{ notice }}</p>
			</div>
			<section class="min-w-0 rounded-3xl bg-surface-2 p-5">
				<div class="mb-4 flex flex-wrap items-center justify-between gap-3">
					<div class="flex flex-wrap gap-2" role="tablist" aria-label="Коллекция скинов">
						<button
							v-for="name in ['Мои скины', 'Каталог Ely.by']"
							:key="name"
							role="tab"
							:aria-selected="tab === name"
							class="cursor-pointer rounded-xl border-0 px-4 py-2 font-semibold"
							:class="tab === name ? 'bg-brand-highlight text-brand' : 'bg-surface-3 text-primary'"
							@click="tab = name"
						>
							{{ name }}
						</button>
					</div>
					<span class="text-sm text-secondary">{{ catalog.current }} / {{ catalog.last }}</span>
				</div>
				<DropdownSelect
					v-if="tab === 'Мои скины' && connected"
					v-model="collection"
					name="ely-my-collection"
					:options="['Сохранённые', 'Мои загрузки']"
					class="mb-4"
				/>
				<form
					v-if="tab === 'Каталог Ely.by' || connected"
					class="mb-5 flex flex-wrap gap-3"
					@submit.prevent="search"
				>
					<input
						v-model="tag"
						class="input min-w-[160px] flex-1"
						placeholder="Поиск по тегу, например cat"
						aria-label="Тег скина"
						maxlength="100"
					/><Button :disabled="loadingCatalog" @click="search">Найти</Button
					><DropdownSelect
						v-model="model"
						name="ely-skin-model"
						:options="['Все модели', 'Alex — тонкие руки']"
					/>
				</form>
				<p v-if="catalogError" role="alert" class="text-red">
					{{ catalogError }} <Button @click="loadCatalog">Повторить</Button>
				</p>
				<div v-if="loadingCatalog" class="py-20 text-center text-secondary">Загружаем каталог…</div>
				<div v-else class="grid grid-cols-2 gap-3 sm:grid-cols-3 xl:grid-cols-4">
					<button
						v-if="tab === 'Мои скины' && equipped.texture"
						class="cursor-pointer rounded-2xl border border-solid border-brand bg-brand-highlight p-3 text-primary"
						@click="resetPreview"
					>
						<ElySkinThumbnail :src="equipped.texture" :slim="equipped.variant === 'slim'" /><span
							class="mt-2 block text-sm font-semibold"
							>Текущий скин</span
						><span class="block text-xs text-secondary">{{
							equipped.variant === 'slim' ? 'Alex' : 'Steve'
						}}</span>
					</button>
					<button
						v-for="skin in catalog.items"
						:key="skin.id"
						:disabled="busy"
						class="cursor-pointer rounded-2xl border border-solid p-3 text-primary transition-colors hover:bg-surface-4 disabled:opacity-50"
						:class="
							pending?.skin.id === skin.id
								? 'border-brand bg-brand-highlight'
								: 'border-transparent bg-surface-3'
						"
						:aria-label="`Предпросмотр скина ${skin.id}`"
						:aria-pressed="pending?.skin.id === skin.id"
						@click="choose(skin)"
					>
						<ElySkinThumbnail :src="skin.skin_url" :slim="skin.is_slim" /><span
							class="mt-2 block truncate text-sm font-semibold"
							>{{ skin.tags[0] || `Скин #${skin.id}` }}</span
						><span class="block text-xs text-secondary">{{ skin.is_slim ? 'Alex' : 'Steve' }}</span>
					</button>
				</div>
				<p
					v-if="!loadingCatalog && !catalogError && !catalog.items.length"
					class="py-16 text-center text-secondary"
				>
					{{
						tab === 'Мои скины'
							? connected
								? 'В этой коллекции пока нет скинов. Загрузите PNG или выберите скин в общем каталоге.'
								: 'Подключите управление скинами, чтобы увидеть сохранённые скины и загрузки Ely.by.'
							: 'По этому тегу скины не найдены.'
					}}
				</p>
				<div v-if="catalog.last > 1" class="mt-5 flex items-center justify-between">
					<Button :disabled="loadingCatalog || catalog.current <= 1" @click="movePage(-1)"
						>Назад</Button
					><Button
						:disabled="loadingCatalog || catalog.current >= catalog.last"
						@click="movePage(1)"
						>Дальше</Button
					>
				</div>
			</section>
		</div>
		<ElyLoginModal ref="loginModal" @authenticated="authenticated" />
		<ElySkinConnectModal
			v-if="account"
			:key="account.profile.id"
			ref="connectModal"
			:user="account.profile.id"
			:username="account.profile.name"
			@connected="onConnected"
		/>
	</div>
</template>

<style scoped>
.skin-model :deep([data-skin-preview-debug='controls']) {
	display: none;
}
.skin-model {
	height: clamp(200px, calc(100dvh - 500px), 380px);
}
@media (min-width: 1024px) {
	.skin-preview-panel {
		position: sticky;
		top: 16px;
		max-height: calc(100dvh - 205px);
		overflow-y: auto;
		padding-right: 4px;
		scrollbar-width: thin;
	}
}
</style>
