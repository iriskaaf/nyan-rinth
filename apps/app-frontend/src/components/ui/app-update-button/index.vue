<template>
	<IconButton
		v-if="isUpdateVisible"
		v-tooltip.right="updateLabel"
		:label="updateLabel"
		type="colored"
		color="brand"
		class="!size-12 shrink-0 self-center !rounded-full"
		:disabled="downloading"
		:aria-busy="downloading"
		@click="handleUpdateClick"
	>
		<RefreshCwIcon v-if="finishedDownloading" />
		<DownloadIcon v-else />
	</IconButton>
</template>

<script setup lang="ts">
import { DownloadIcon, RefreshCwIcon } from '@modrinth/assets'
import { IconButton, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import {
	appUpdateState,
	downloadAvailableAppUpdate,
	installAvailableAppUpdate,
} from '@/providers/app-update'

const { formatMessage } = useVIntl()

const messages = defineMessages({
	update: {
		id: 'app.action-bar.update',
		defaultMessage: 'Update',
	},
	downloadingUpdate: {
		id: 'app.action-bar.downloading-update',
		defaultMessage: 'Downloading update',
	},
	reloadToUpdate: {
		id: 'app.action-bar.reload-to-update',
		defaultMessage: 'Reload to update',
	},
})

const {
	downloading,
	downloadPercent,
	finishedDownloading,
	isVisible: isUpdateVisible,
	restarting,
} = appUpdateState

const updateLabel = computed(() => {
	if (downloading.value) {
		return `${formatMessage(messages.downloadingUpdate)} ${downloadPercent.value}%`
	}
	return formatMessage(finishedDownloading.value ? messages.reloadToUpdate : messages.update)
})

async function handleUpdateClick() {
	if (downloading.value || restarting.value) return
	if (finishedDownloading.value) {
		await installAvailableAppUpdate()
	} else {
		await downloadAvailableAppUpdate()
	}
}
</script>
