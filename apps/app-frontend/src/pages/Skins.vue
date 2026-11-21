<script setup lang="ts">
import {
	Button,
	ConfirmModal,
	DropdownSelect,
	IconButton,
	injectNotificationManager,
	NewModal,
	SkinPreviewRenderer,
} from '@modrinth/ui'
import { CheckIcon, ElyByIcon, MoveIcon, PlusIcon, RefreshCwIcon } from '@modrinth/assets'
import { invoke } from '@tauri-apps/api/core'
import { computed, inject, nextTick, onMounted, ref, watch, type Ref } from 'vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import ElySkinConnectModal from '@/components/ui/skin/ElySkinConnectModal.vue'
import ElySkinThumbnail from '@/components/ui/skin/ElySkinThumbnail.vue'
import ElySkinCard from '@/components/ui/skin/ElySkinCard.vue'
import steveSkin from '@/assets/skins/steve.png'
import alexSkin from '@/assets/skins/alex.png'
import { users, get_default_user } from '@/helpers/auth'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'
import { useRootBreadcrumb } from '@/shared/breadcrumbs/use-breadcrumb'

useRootBreadcrumb({
	slot: 'skins',
	id: 'skins',
	label: 'ELY.BY',
	to: '/skins',
	visual: { type: 'icon', component: ElyByIcon },
})

type Skin = { id: number; skin_url: string; is_slim: boolean; tags: string[] }
type Catalog = {
	items: Skin[]
	current: number
	last: number
	total_items?: number
	equipped_id?: number
}
type Preview = { skin: Skin; texture: string }
const accountsCard = inject<
	Ref<{
		refreshValues: () => Promise<void>
		setElySkinHead?: (user: string, texture: string) => Promise<void>
	} | null>
>('accountsCard')
const { handleError } = injectNotificationManager()
const accounts = ref<MinecraftCredential[]>([])
const headerAccountsCard = ref<InstanceType<typeof AccountsCard> | null>(null)
const selected = ref('')
const account = computed(() => accounts.value.find((a) => a.profile.id === selected.value))
const supportsElySkins = computed(() => account.value?.account_type === 'elyby')
const equipped = ref({ texture: '', variant: 'classic' })
const pending = ref<Preview | null>(null)
const previewNotApplied = computed(
	() =>
		!!pending.value &&
		pending.value.skin.id !== catalog.value.equipped_id &&
		(pending.value.texture !== equipped.value.texture ||
			pending.value.skin.is_slim !== (equipped.value.variant === 'slim')),
)
const previewTexture = computed(() => pending.value?.texture ?? equipped.value.texture)
const previewVariant = computed(() =>
	(pending.value ? pending.value.skin.is_slim : equipped.value.variant === 'slim')
		? 'SLIM'
		: 'CLASSIC',
)
const catalog = ref<Catalog>({ items: [], current: 1, last: 1 })
const page = ref(1)
const tab = ref('Мои скины')
const collection = ref('Мои загрузки')
const tag = ref('')
const loadingCatalog = ref(false)
const loadingSkin = ref(false)
const checkingSession = ref(false)
const connected = ref(false)
const busy = ref(false)
const catalogFailed = ref(false)
const savedSkins = ref(new Set<number>())
const connectModal = ref<InstanceType<typeof ElySkinConnectModal> | null>(null)
const fileInput = ref<HTMLInputElement | null>(null)
const uploadModal = ref<InstanceType<typeof NewModal> | null>(null)
const uploadDraft = ref<{ user: string; bytes: number[]; legacy: boolean } | null>(null)
const uploadModel = ref<'classic' | 'slim'>('classic')
const removeModal = ref<InstanceType<typeof ConfirmModal> | null>(null)
const removal = ref<{ skin: Skin; user: string; uploaded: boolean } | null>(null)
let previewRequest = 0
let catalogRequest = 0
let accountRequest = 0
function message(e: unknown) {
	return typeof e === 'string'
		? e
		: ((e as { message?: string }).message ?? 'Не удалось выполнить запрос Ely.by.')
}
async function loadCatalog() {
	if (!supportsElySkins.value) {
		catalog.value = { items: [], current: 1, last: 1 }
		return
	}
	const request = ++catalogRequest
	loadingCatalog.value = true
	catalogFailed.value = false
	try {
		if (tab.value === 'Мои скины' && (!connected.value || !selected.value)) {
			catalog.value = { items: [], current: 1, last: 1 }
			return
		}
		const args = {
			page: page.value,
			model: '',
			tags: tab.value === 'Каталог Ely.by' ? tag.value.trim() : '',
		}
		const result =
			tab.value === 'Мои скины'
				? await invoke<Catalog | null>('plugin:auth|ely_skins_personal', {
						...args,
						user: selected.value,
						uploaded: collection.value === 'Мои загрузки',
					})
				: await invoke<Catalog>('plugin:auth|ely_skins_catalog', args)
		if (request !== catalogRequest) return
		if (result === null) {
			connected.value = false
			catalog.value = { items: [], current: 1, last: 1 }
			return
		}
		catalog.value = result
	} catch (e) {
		if (request === catalogRequest) {
			catalogFailed.value = true
			handleError(new Error(message(e)))
		}
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
	savedSkins.value = new Set()
	connected.value = false
	checkingSession.value = false
	loadingCatalog.value = false
	loadingSkin.value = false
	equipped.value = { texture: '', variant: 'classic' }
	if (!user || !supportsElySkins.value) return
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
	else handleError(new Error(message(results[1].reason)))
	checkingSession.value = false
	void loadCatalog()
}
async function loadAccounts(preferred?: string) {
	const all = (await users()) as MinecraftCredential[]
	accounts.value = all
	const active = preferred ?? (await get_default_user())
	const next =
		accounts.value.find((a) => a.profile.id === active)?.profile.id ??
		accounts.value[0]?.profile.id ??
		''
	if (next === selected.value) await refreshAccount()
	else selected.value = next
}
async function accountChanged() {
	await accountsCard?.value?.refreshValues()
	await loadAccounts()
}
async function choose(skin: Skin) {
	if (busy.value) return
	const request = ++previewRequest
	loadingSkin.value = true
	try {
		const result = await invoke<Preview>('plugin:auth|ely_skins_preview', { skinId: skin.id })
		if (request === previewRequest) pending.value = result
	} catch (e) {
		if (request === previewRequest) handleError(new Error(message(e)))
	} finally {
		if (request === previewRequest) loadingSkin.value = false
	}
}
function resetPreview() {
	previewRequest++
	pending.value = null
	loadingSkin.value = false
}
async function applySkin(skin: Skin) {
	if (!connected.value || !account.value || busy.value) return
	const user = selected.value
	busy.value = true
	previewRequest++
	loadingSkin.value = false
	try {
		const preview = await invoke<Preview>('plugin:auth|ely_skins_preview', { skinId: skin.id })
		await invoke('plugin:auth|ely_skins_apply', { user, skinId: skin.id })
		equipped.value = {
			texture: preview.texture,
			variant: preview.skin.is_slim ? 'slim' : 'classic',
		}
		pending.value = null
		await accountsCard?.value?.setElySkinHead?.(user, preview.texture)
		await headerAccountsCard.value?.setElySkinHead(user, preview.texture)
		catalog.value.equipped_id = preview.skin.id
	} catch (e) {
		handleError(new Error(message(e)))
	} finally {
		busy.value = false
	}
}
function requestRemoval(skin: Skin) {
	if (busy.value || !connected.value || tab.value !== 'Мои скины') return
	removal.value = { skin, user: selected.value, uploaded: collection.value === 'Мои загрузки' }
	removeModal.value?.show()
}
async function saveSkin(skin: Skin) {
	if (!connected.value || !account.value || busy.value) return
	const user = selected.value
	busy.value = true
	try {
		await invoke('plugin:auth|ely_skins_save', { user, skinId: skin.id })
		if (user === selected.value) savedSkins.value.add(skin.id)
	} catch (e) {
		if (user === selected.value) handleError(new Error(message(e)))
	} finally {
		busy.value = false
	}
}
async function removeSkin() {
	const target = removal.value
	removal.value = null
	if (!target || busy.value || target.user !== selected.value) return
	busy.value = true
	try {
		await invoke<'ely_by' | 'local_only' | 'saved'>('plugin:auth|ely_skins_remove', {
			user: target.user,
			skinId: target.skin.id,
			uploaded: target.uploaded,
		})
		if (!target.uploaded) savedSkins.value.delete(target.skin.id)
		if (pending.value?.skin.id === target.skin.id) resetPreview()
		await loadCatalog()
		page.value = catalog.value.current
	} catch (e) {
		handleError(new Error(message(e)))
	} finally {
		busy.value = false
	}
}
function clearUploadDraft() {
	uploadDraft.value = null
}
async function selectUploadFile(event: Event) {
	const input = event.target as HTMLInputElement
	const file = input.files?.[0]
	input.value = ''
	if (!file || busy.value || !connected.value) return
	if (file.size > 196608) {
		handleError(new Error('Размер PNG не должен превышать 192 КиБ.'))
		return
	}
	const user = selected.value
	busy.value = true
	try {
		const bytes = new Uint8Array(await file.arrayBuffer())
		const header = new DataView(bytes.buffer)
		const height = bytes.length >= 24 ? header.getUint32(20) : 0
		if (
			bytes.length < 33 ||
			header.getUint32(0) !== 0x89504e47 ||
			header.getUint32(4) !== 0x0d0a1a0a ||
			header.getUint32(12) !== 0x49484452 ||
			header.getUint32(16) !== 64 ||
			![32, 64].includes(height)
		)
			throw new Error('Нужен PNG-скин размером 64×64 или 64×32 пикселя.')
		if (user !== selected.value || !connected.value) return
		clearUploadDraft()
		uploadDraft.value = {
			user,
			bytes: Array.from(bytes),
			legacy: height === 32,
		}
		uploadModel.value = 'classic'
		uploadModal.value?.show()
	} catch (e) {
		handleError(new Error(message(e)))
	} finally {
		busy.value = false
	}
}
async function uploadFile() {
	const draft = uploadDraft.value
	if (!draft || busy.value || !connected.value || draft.user !== selected.value) return
	busy.value = true
	resetPreview()
	try {
		const skin = await invoke<Skin>('plugin:auth|ely_skins_upload', {
			user: draft.user,
			bytes: draft.bytes,
			slim: uploadModel.value === 'slim',
		})
		tab.value = 'Мои скины'
		collection.value = 'Мои загрузки'
		tag.value = ''
		page.value = 1
		await loadCatalog()
		const request = ++previewRequest
		try {
			const preview = await invoke<Preview>('plugin:auth|ely_skins_preview', { skinId: skin.id })
			if (request === previewRequest) pending.value = preview
		} catch {
			pending.value = null
		}
		busy.value = false
		await nextTick()
		uploadModal.value?.hide()
	} catch (e) {
		handleError(new Error(message(e)))
	} finally {
		busy.value = false
	}
}
async function onConnected() {
	connected.value = true
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
watch(selected, () => {
	uploadModal.value?.hide()
	clearUploadDraft()
})
watch([tab, collection], search)
onMounted(() => {
	void loadAccounts().catch((e) => {
		handleError(new Error(message(e)))
	})
})
</script>

<template>
	<div class="skins-page mx-auto w-full max-w-[1440px] p-6">
		<div class="mb-6 flex shrink-0 flex-wrap items-center justify-between gap-4">
			<div>
				<h1 class="m-0 text-3xl font-bold">Аккаунт</h1>
			</div>
			<div
				class="skin-account-picker relative h-14 w-[280px] shrink-0"
				:inert="busy"
				:class="{ 'opacity-60': busy }"
			>
				<div class="absolute inset-x-0 top-0 z-30">
					<Suspense>
						<AccountsCard ref="headerAccountsCard" :inert="busy" compact @change="accountChanged" />
					</Suspense>
				</div>
			</div>
		</div>
		<div v-if="!supportsElySkins" class="flex grow items-center justify-center py-16">
			<section class="max-w-lg rounded-2xl bg-surface-2 p-8 text-center">
				<ElyByIcon class="mx-auto mb-4 size-12" aria-hidden="true" />
				<h2 class="m-0 mb-3 text-xl font-semibold">Скины Ely.by</h2>
				<p class="m-0 text-secondary">
					{{
						account?.account_type === 'offline'
							? 'Офлайн-аккаунты не поддерживают смену скинов. В игре используется стандартный скин Minecraft.'
							: account
								? 'Для управления скинами Ely.by выберите или добавьте аккаунт Ely.by.'
								: 'Добавьте игровой аккаунт, чтобы начать играть. Для смены скинов войдите в Ely.by.'
					}}
				</p>
			</section>
		</div>
		<div
			v-else
			class="skins-layout grid items-start gap-4 lg:grid-cols-[280px_minmax(0,1fr)] xl:grid-cols-[300px_minmax(0,1fr)]"
		>
			<div class="skin-preview-panel flex min-h-0 flex-col gap-3">
				<div class="skin-model relative rounded-3xl bg-surface-2 p-3">
					<SkinPreviewRenderer
						v-if="previewTexture"
						:key="`${selected}-${previewVariant}`"
						:texture-src="previewTexture"
						:fit="true"
						framing="modal"
						:lock-fit="false"
						:fit-padding="{ top: 0.16, right: 0.06, bottom: 0.14, left: 0.06 }"
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
					<span class="pointer-events-none absolute inset-x-4 top-5 flex justify-center"
						><span
							class="max-w-full truncate rounded-lg bg-surface-3 px-3 py-2 font-minecraft text-[28px] leading-none text-contrast"
							>{{ account?.profile.name }}</span
						></span
					>
					<span
						class="pointer-events-none absolute bottom-4 inset-x-2 flex items-center justify-center gap-2 text-xs text-secondary"
						><MoveIcon class="size-4" aria-hidden="true" />Перетащите, чтобы повернуть</span
					>
					<span
						v-if="loadingSkin"
						class="absolute bottom-4 left-4 rounded-lg bg-surface-3 px-3 py-2"
						>Загружаем…</span
					>
				</div>
				<div
					v-if="previewNotApplied && pending"
					class="flex flex-col gap-2 rounded-2xl bg-surface-2 p-3"
				>
					<span class="text-center text-sm text-secondary" role="status">Скин не применён</span>
					<Button
						type="colored"
						color="brand"
						:disabled="busy || !connected"
						@click="applySkin(pending.skin)"
					>
						<CheckIcon />Применить
					</Button>
				</div>
				<input
					ref="fileInput"
					type="file"
					class="hidden"
					accept="image/png,.png"
					@change="selectUploadFile"
				/>
			</div>
			<section class="skin-catalog min-w-0 rounded-3xl bg-surface-2 p-5">
				<div class="skin-toolbar mb-4 flex items-center gap-2">
					<div class="skin-tabs flex shrink-0 gap-2" role="tablist" aria-label="Коллекция скинов">
						<button
							v-for="name in ['Мои скины', 'Каталог Ely.by']"
							:key="name"
							role="tab"
							:aria-selected="tab === name"
							class="cursor-pointer rounded-xl border-0 h-10 px-3 py-2 text-sm font-semibold"
							:class="tab === name ? 'bg-brand-highlight text-brand' : 'bg-surface-3 text-primary'"
							@click="tab = name"
						>
							{{ name }}
						</button>
					</div>
					<DropdownSelect
						v-if="tab === 'Мои скины' && connected"
						v-model="collection"
						name="ely-my-collection"
						:options="['Мои загрузки', 'Сохранённые']"
						class="skin-collection-select !w-36 !min-w-0 shrink-0 text-sm"
					/>
					<div class="skin-toolbar-status ml-auto flex shrink-0 items-center gap-2">
						<span
							class="skin-count whitespace-nowrap text-right text-sm tabular-nums text-secondary"
							:aria-label="`Скинов: ${loadingCatalog ? 'загрузка' : (catalog.total_items ?? catalog.items.length)}`"
							><span class="skin-count-label">Скинов: </span
							>{{ loadingCatalog ? '…' : (catalog.total_items ?? catalog.items.length) }}</span
						>
						<IconButton
							label="Обновить скины"
							class="!size-10 shrink-0"
							:disabled="busy || loadingCatalog"
							@click="loadCatalog"
							><RefreshCwIcon :class="{ 'animate-spin': loadingCatalog }"
						/></IconButton>
					</div>
				</div>

				<form
					v-if="tab === 'Каталог Ely.by'"
					class="mb-5 flex flex-wrap gap-3"
					@submit.prevent="search"
				>
					<input
						v-model="tag"
						class="input min-w-[160px] flex-1"
						placeholder="Поиск по тегу, например cat"
						aria-label="Тег скина"
						maxlength="100"
					/><Button native-type="submit" :disabled="loadingCatalog">Найти</Button>
				</form>
				<div v-if="loadingCatalog" class="py-20 text-center text-secondary">Загружаем каталог…</div>
				<div v-else class="skin-grid grid gap-3">
					<button
						v-if="tab === 'Мои скины' && collection === 'Мои загрузки' && account"
						class="skin-upload-card flex min-w-0 cursor-pointer flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-surface-5 bg-transparent p-3 text-primary transition-colors hover:border-brand hover:bg-brand-highlight disabled:cursor-default disabled:opacity-40"
						:disabled="busy || checkingSession"
						@click="connected ? fileInput?.click() : connectModal?.show()"
					>
						<PlusIcon class="size-8" aria-hidden="true" /><span class="font-semibold"
							>Добавить скин</span
						>
					</button>
					<button
						v-if="tab === 'Мои скины' && equipped.texture && !connected"
						class="skin-current-card relative flex min-w-0 cursor-pointer items-center justify-center overflow-hidden rounded-2xl border border-solid border-brand bg-surface-3 p-3 text-primary"
						@click="resetPreview"
					>
						<ElySkinThumbnail :src="equipped.texture" :slim="equipped.variant === 'slim'" /><span
							class="absolute right-2 top-2 flex size-6 items-center justify-center rounded-full bg-white text-black"
							aria-label="Скин используется"
							><CheckIcon class="size-4"
						/></span>
					</button>
					<ElySkinCard
						v-for="skin in catalog.items"
						:key="skin.id"
						:skin="skin"
						:selected="pending?.skin.id === skin.id"
						:equipped="skin.id === catalog.equipped_id"
						:personal="tab === 'Мои скины'"
						:connected="connected"
						:busy="busy"
						:saved="savedSkins.has(skin.id)"
						@preview="choose(skin)"
						@apply="applySkin(skin)"
						@remove="requestRemoval(skin)"
						@save="saveSkin(skin)"
					/>
				</div>
				<p
					v-if="!loadingCatalog && !catalogFailed && !catalog.items.length"
					class="py-16 text-center text-secondary"
				>
					{{
						tab === 'Мои скины'
							? connected
								? collection === 'Сохранённые'
									? 'Сохранённых скинов пока нет. Добавьте их звёздочкой в каталоге Ely.by.'
									: 'В этой коллекции пока нет скинов. Загрузите PNG или выберите скин в общем каталоге.'
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
		<NewModal
			ref="uploadModal"
			header="Выберите тип скина"
			max-width="480px"
			:disable-close="busy"
			@hide="clearUploadDraft"
		>
			<form v-if="uploadDraft" class="flex flex-col gap-4" @submit.prevent="uploadFile">
				<div class="grid grid-cols-2 gap-3" role="radiogroup" aria-label="Тип скина">
					<label
						v-for="model in ['classic', 'slim'] as const"
						:key="model"
						class="flex min-w-0 cursor-pointer flex-col items-center gap-2 rounded-2xl border border-solid p-3 focus-within:outline focus-within:outline-2 focus-within:outline-brand"
						:class="[
							uploadModel === model
								? 'border-brand bg-brand-highlight'
								: 'border-surface-5 bg-surface-3',
							{ 'cursor-default opacity-40': uploadDraft.legacy && model === 'slim' },
						]"
					>
						<input
							v-model="uploadModel"
							type="radio"
							name="upload-model"
							:value="model"
							:disabled="busy || (uploadDraft.legacy && model === 'slim')"
							class="sr-only"
						/>
						<ElySkinThumbnail
							:src="model === 'slim' ? alexSkin : steveSkin"
							:slim="model === 'slim'"
						/>
						<span class="font-semibold text-contrast">{{
							model === 'slim' ? 'Alex' : 'Steve'
						}}</span>
						<span class="text-sm text-secondary">{{
							model === 'slim' ? 'Руки 3 пикселя' : 'Руки 4 пикселя'
						}}</span>
					</label>
				</div>
				<p v-if="uploadDraft.legacy" class="m-0 text-sm text-secondary">
					Формат 64×32 поддерживает только Steve. Для Alex нужен PNG 64×64.
				</p>

				<div class="flex justify-end gap-2">
					<Button :disabled="busy" @click="uploadModal?.hide()">Отмена</Button>
					<Button native-type="submit" type="colored" color="brand" :disabled="busy">{{
						busy ? 'Загружаем…' : 'Загрузить'
					}}</Button>
				</div>
			</form>
		</NewModal>
		<ConfirmModal
			ref="removeModal"
			title="Удалить скин?"
			:description="
				removal?.uploaded
					? 'Ваша загрузка будет удалена из Ely.by. Если Ely.by указал другого автора, из лаунчера будет убрана только карточка.'
					: 'Скин будет убран из сохранённых Ely.by. Его можно снова найти в каталоге.'
			"
			proceed-label="Удалить"
			@proceed="removeSkin"
		/>
		<ElySkinConnectModal
			v-if="account && supportsElySkins"
			:key="account.profile.id"
			ref="connectModal"
			:user="account.profile.id"
			:username="account.profile.name"
			@connected="onConnected"
		/>
	</div>
</template>

<style scoped>
.skin-catalog {
	container-type: inline-size;
}
.skin-count {
	min-width: 5ch;
}
.skin-toolbar {
	min-width: 0;
}
.skin-grid {
	grid-template-columns: repeat(auto-fill, minmax(min(150px, 100%), 1fr));
	align-items: start;
}
.skin-upload-card,
.skin-current-card {
	aspect-ratio: 1;
}
.skin-current-card :deep(canvas) {
	width: 100%;
	height: auto;
}
.skin-count-label {
	display: none;
}
@container (max-width: 560px) {
	.skin-tabs {
		gap: 0.5rem;
	}
	.skin-tabs button {
		padding-inline: 0.5rem;
		font-size: 0.75rem;
		white-space: nowrap;
	}
	.skin-collection-select {
		width: 8rem !important;
		font-size: 0.75rem;
	}
	.skin-toolbar-status .skin-count {
		display: none;
	}
}

.skin-collection-select :deep(.selected) {
	gap: 0.5rem;
	padding-inline: 0.75rem;
	white-space: nowrap;
}

@container (max-width: 400px) {
	.skin-toolbar,
	.skin-tabs {
		gap: 0.5rem;
	}
	.skin-tabs button {
		padding-inline: 0.25rem;
		font-size: 0.6875rem;
	}
	.skin-collection-select {
		width: 6.5rem !important;
		font-size: 0.6875rem;
	}
	.skin-collection-select :deep(.selected) {
		gap: 0.25rem;
		padding-inline: 0.375rem;
	}
	.skin-toolbar-status :deep(button) {
		width: 2rem !important;
	}
}

.skin-model :deep([data-skin-preview-debug='controls']) {
	display: none;
}
.skin-model {
	height: clamp(200px, calc(100dvh - 500px), 380px);
}
@media (min-width: 1024px) and (min-height: 600px) {
	.skins-page {
		display: flex;
		height: 100%;
		min-height: 0;
		flex-direction: column;
	}
	.skins-layout {
		flex: 1;
		min-height: 0;
		align-items: stretch;
	}
	.skin-preview-panel {
		min-height: 0;
	}
	.skin-model {
		flex: 1;
		height: auto;
		min-height: 140px;
	}
	.skin-catalog {
		min-height: 0;
		overflow-y: auto;
		scrollbar-width: thin;
	}
}
</style>
