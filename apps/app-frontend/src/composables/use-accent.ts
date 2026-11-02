import { ref, watch } from 'vue'

export const DEFAULT_ACCENT = '#f472b6'
export const ACCENT_PRESETS = [
	{ id: 'pink', color: DEFAULT_ACCENT },
	{ id: 'sky', color: '#38bdf8' },
	{ id: 'green', color: '#4ade80' },
] as const
const stored = localStorage.getItem('nyanrinth.accent') ?? DEFAULT_ACCENT
export const accent = ref<string>(
	ACCENT_PRESETS.some((preset) => preset.color === stored) ? stored : DEFAULT_ACCENT,
)

watch(
	accent,
	(color) => {
		if (!/^#[\da-f]{6}$/i.test(color)) return
		localStorage.setItem('nyanrinth.accent', color)
		const r = parseInt(color.slice(1, 3), 16)
		const g = parseInt(color.slice(3, 5), 16)
		const b = parseInt(color.slice(5, 7), 16)
		const channels = [r, g, b].map((v) => {
			const s = v / 255
			return s <= 0.04045 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4
		})
		const luminance = channels[0] * 0.2126 + channels[1] * 0.7152 + channels[2] * 0.0722
		const contrast = luminance > 0.179 ? '#17121c' : '#ffffff'
		const vars: Record<string, string> = {
			'--color-brand': color,
			'--color-brand-highlight': `rgba(${r}, ${g}, ${b}, 0.18)`,
			'--color-brand-shadow': `rgba(${r}, ${g}, ${b}, 0.5)`,
			'--color-accent-contrast': contrast,
			'--color-brand-inverted': contrast,
			'--brand-gradient-bg': `linear-gradient(135deg, rgba(${r},${g},${b},0.12), transparent)`,
			'--brand-gradient-strong-bg': `linear-gradient(270deg, #18131c, rgba(${r},${g},${b},0.2))`,
			'--brand-gradient-border': `rgba(${r},${g},${b},0.16)`,
			'--loading-bar-gradient': `linear-gradient(to right, ${color}, ${color})`,
			'--modal-overlay-background': `linear-gradient(to bottom, rgba(${r}, ${g}, ${b}, 0.10), rgba(10, 10, 14, 0.92))`,
		}
		for (const [key, value] of Object.entries(vars))
			document.documentElement.style.setProperty(key, value)
	},
	{ immediate: true },
)
