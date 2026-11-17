import { type AbstractWebNotificationManager, provideTags } from '@modrinth/ui'
import { ref } from 'vue'

import { get_game_versions, get_loaders } from '@/helpers/tags'

export function setupTagsProvider(_notificationManager: AbstractWebNotificationManager) {
	const gameVersions = ref([])
	const loaders = ref([])

	function initialize() {
		get_game_versions()
			.then((v) => {
				gameVersions.value = v
			})
			.catch((error) => console.warn('Failed to initialize game version tags:', error))
		get_loaders()
			.then((v) => {
				loaders.value = v
			})
			.catch((error) => console.warn('Failed to initialize loader tags:', error))
	}

	provideTags({ gameVersions, loaders })

	return { initialize }
}
