import { onScopeDispose } from 'vue'

/** Checks again after failures and releases its timer when the app unmounts. */
export function useUpdateChecks<T>(check: () => Promise<T>) {
	let active = false
	let disposed = false
	let timer: ReturnType<typeof setTimeout> | undefined

	let pending: Promise<T> | undefined

	function checkNow() {
		pending ??= check().finally(() => {
			pending = undefined
		})
		return pending
	}

	async function run() {
		try {
			await checkNow()
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

	return {
		checkNow,
		start() {
			if (active || disposed) return
			active = true
			void run()
		},
	}
}
