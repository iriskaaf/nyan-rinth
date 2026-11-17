import { invoke } from '@tauri-apps/api/core'

const cache = new Map<string, { head: string | null; expires: number }>()
const inFlight = new Map<string, Promise<string | null>>()

export async function skinTextureHead(texture: string): Promise<string> {
	const image = new Image()
	image.src = texture
	await image.decode()
	const canvas = document.createElement('canvas')
	canvas.width = canvas.height = 64
	const context = canvas.getContext('2d')!
	context.imageSmoothingEnabled = false
	context.drawImage(image, 8, 8, 8, 8, 0, 0, 64, 64)
	context.drawImage(image, 40, 8, 8, 8, 0, 0, 64, 64)
	return canvas.toDataURL('image/png')
}

export function rememberElyHead(username: string, head: string) {
	cache.set(username.toLowerCase(), { head, expires: Date.now() + 60_000 })
}

export async function loadElyHead(username: string): Promise<string | null> {
	const cached = cache.get(username.toLowerCase())
	if (cached && cached.expires > Date.now()) return cached.head
	const skin = await invoke<{ texture: string } | null>('plugin:auth|ely_skin_texture', {
		username,
	})
	if (!skin) return null
	const head = await skinTextureHead(skin.texture)
	rememberElyHead(username, head)
	return head
}
