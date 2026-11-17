import { readonly, shallowRef } from 'vue'
import type { MinecraftCredential } from '@/models/astralrinth/authentication'

const accounts = shallowRef<readonly MinecraftCredential[] | null>(null)

export const minecraftAccounts = readonly(accounts)

export function updateMinecraftAccounts(value: MinecraftCredential[]) {
	accounts.value = [...value]
}
