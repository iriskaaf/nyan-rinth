import { getVersion } from '@tauri-apps/api/app'
import { isTauri } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { fetch } from '@tauri-apps/plugin-http'

import { getOS, initUpdateLauncher, isDev } from '@/helpers/utils.js'

export type LauncherReleaseAsset = {
	name: string
	browser_download_url: string
	download_count: number
}

export type LauncherRelease = {
	tag_name: string
	name: string
	assets: LauncherReleaseAsset[]
}

// import.meta.env uses `vite.config.ts`
// Environments can be configured in `packages/app-lib/` directory.
export const LAUNCHER_REPOSITORY_URL = 'https://github.com/42WASD/AstralRinth'
export const LAUNCHER_RELEASES_URL = `${LAUNCHER_REPOSITORY_URL}releases`
export const LAUNCHER_LATEST_RELEASE_API = ''

export const isUpdateInstalling = ref(false)
export const isUpdateAvailable = ref(false)
export const latestLauncherReleases = ref<LauncherRelease | null>(null)
export const latestLauncherReleaseHttpStatus = ref<number | null>(null)

const currentOS = ref('')

const systems = ['macos', 'windows', 'linux'] as const
const osExtensions = {
	linux: ['.deb', '.rpm', '.AppImage'],
	macos: ['.dmg', '.pkg', '.app'],
	windows: ['.exe', '.msi'],
}

const isDeveloper = isTauri() && (await isDev())

const blacklistBeginPrefixes = [
	'dev',
	'nightly',
	'dirty',
	'dirty-dev',
	'dirty-nightly',
	'dirty_dev',
	'dirty_nightly',
]

export async function fetchRemote(): Promise<void> {
	isUpdateAvailable.value = false
	latestLauncherReleases.value = null
	latestLauncherReleaseHttpStatus.value = null
}

export async function downloadLatestRelease(
	selectedInstaller?: LauncherReleaseAsset | null,
): Promise<boolean> {
	if (!latestLauncherReleases.value) {
		return false
	}

	if (!currentOS.value) {
		currentOS.value = (await getOS()).toLowerCase()
	}

	const installer = selectedInstaller ?? null
	if (isDeveloper) {
		console.debug(installer)
	}
	if (!installer) {
		isUpdateInstalling.value = false
		return false
	}

	try {
		isUpdateInstalling.value = true
		return await initUpdateLauncher(installer.browser_download_url, installer.name, currentOS.value)
	} finally {
		isUpdateInstalling.value = false
	}
}

export function getAvailableInstallers(): LauncherReleaseAsset[] {
	if (!latestLauncherReleases.value) {
		return []
	}

	return getInstallers(resolveOperationalSystemExtension(), latestLauncherReleases.value.assets)
}

function getInstallers(os: string[], builds: LauncherReleaseAsset[]): LauncherReleaseAsset[] {
	return builds.filter((build) => {
		if (blacklistBeginPrefixes.some((prefix) => build.name.startsWith(prefix))) {
			return false
		}

		const matchesExtension = os.some((extension) => build.name.endsWith(extension))
		if (matchesExtension && isDeveloper) {
			console.debug(build.name, build.browser_download_url)
		}

		return matchesExtension
	})
}

function resolveOperationalSystemExtension(): string[] {
	try {
		switch (currentOS.value) {
			case 'macos':
				return osExtensions.macos
			case 'windows':
				return osExtensions.windows
			case 'linux':
				return osExtensions.linux
			default:
				throw new Error(String("Operational System can't be resolved"))
		}
	} catch (error) {
		console.error("Operational System can't be resolved")
		return []
	}
}

function normalizeVersion(version: string): string {
	return version.trim().replace(/^v/i, '')
}

function compareVersions(left: string, right: string): number {
	const leftParts = parseVersionParts(left)
	const rightParts = parseVersionParts(right)
	const maxLength = Math.max(leftParts.length, rightParts.length)

	for (let index = 0; index < maxLength; index += 1) {
		const leftPart = leftParts[index] ?? 0
		const rightPart = rightParts[index] ?? 0

		if (leftPart !== rightPart) {
			if (isDeveloper) {
				console.debug('Version parts differ at index', index, leftPart, rightPart)
			}
			return leftPart - rightPart
		}
	}

	if (isDeveloper) {
		console.debug('Version parts are equal', leftParts, rightParts)
	}

	return 0
}

function parseVersionParts(version: string): number[] {
	return normalizeVersion(version)
		.split(/[.-]/)
		.map((part) => Number.parseInt(part, 10))
		.filter((part) => !Number.isNaN(part))
}
