<script setup lang="ts">
import { nyanMessages } from '@/helpers/ui-messages'
import { useVIntl } from '@modrinth/ui'

import { DropdownIcon, FolderOpenIcon, PlusIcon } from '@modrinth/assets'
import { Button, injectNotificationManager, TeleportOverflowMenu } from '@modrinth/ui'
import { open } from '@tauri-apps/plugin-dialog'
import { useRouter } from 'vue-router'

import { add_project_from_path } from '@/helpers/instance'

const { handleError } = injectNotificationManager()

const props = defineProps({
	instance: {
		type: Object,
		required: true,
	},
})

const router = useRouter()

const handleAddContentFromFile = async () => {
	const newProject = await open({ multiple: true })
	if (!newProject) return

	for (const project of newProject) {
		await add_project_from_path(props.instance.id, project.path ?? project).catch(handleError)
	}
}

const handleSearchContent = async () => {
	await router.push({
		path: `/browse/${props.instance.loader === 'vanilla' ? 'resourcepack' : 'mod'}`,
		query: { i: props.instance.id },
	})
}

const { formatMessage } = useVIntl()
</script>

<template>
	<div class="joined-buttons">
		<Button @click="handleSearchContent">
			<PlusIcon /> {{ formatMessage(nyanMessages.installContent) }}
		</Button>
		<TeleportOverflowMenu
			:label="formatMessage(nyanMessages.moreOptions)"
			:options="[
				{
					id: 'from_file',
					label: 'Add from file',
					action: handleAddContentFromFile,
				},
			]"
			class="!w-auto !px-2.5 !rounded-xl"
		>
			<DropdownIcon />
			<template #from_file>
				<FolderOpenIcon />
				<span class="no-wrap"> {{ formatMessage(nyanMessages.fromFile) }} </span>
			</template>
		</TeleportOverflowMenu>
	</div>
</template>
