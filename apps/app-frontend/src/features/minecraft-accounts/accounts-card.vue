<script setup lang="ts">
import { ref } from 'vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import type { Skin } from '@/helpers/skins'
const card = ref<InstanceType<typeof AccountsCard> | null>(null)
defineExpose({
	refreshValues: async () => {
		await card.value?.refreshValues()
	},
	setEquippedSkin: async (skin: Skin) => {
		await card.value?.setEquippedSkin(skin)
	},
	setLoginDisabled: (value: boolean) => card.value?.setLoginDisabled(value),
	login: async () => {
		card.value?.showAccountLoginModal()
	},
	get loginDisabled() {
		return card.value?.loginDisabled ?? false
	},
})
</script>
<template>
	<Suspense><AccountsCard ref="card" hide-card /></Suspense>
</template>
