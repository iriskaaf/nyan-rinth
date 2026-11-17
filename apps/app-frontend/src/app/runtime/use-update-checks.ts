import { onScopeDispose } from 'vue'

/** Checks again after failures and releases its timer when the app unmounts. */
export function useUpdateChecks(check: () => Promise<void>) {
	let active = false
	let disposed = false
	let timer: ReturnType<typeof setTimeout> | undefined

	async function run() {
		try {
			await check()
		} catch (error) {
			console.warn('Failed to check for launcher updates:', error)
		} finally {
			if (!disposed) timer = setTimeout(run, 5 * 60 * 1000)
		}
	}

	onScopeDispose(() => {
		disposed = true
		clearTimeout(timer)
	})

	return () => {
		if (active || disposed) return
		active = true
		void run()
	}
}
