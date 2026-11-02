<script setup lang="ts">
import { onBeforeUnmount, ref, watch } from 'vue'
const props = defineProps<{ src: string; slim: boolean }>()
const canvas = ref<HTMLCanvasElement | null>(null)
let revision = 0
watch(
	[() => props.src, () => props.slim, canvas],
	() => {
		const current = ++revision
		const image = new Image()
		image.onload = () => {
			if (current !== revision || !canvas.value) return
			const ctx = canvas.value.getContext('2d')
			if (!ctx || image.naturalWidth !== 64) return
			ctx.clearRect(0, 0, 128, 128)
			ctx.imageSmoothingEnabled = false
			const arm = props.slim ? 3 : 4
			const part = (sx: number, sy: number, w: number, h: number, x: number, y: number) =>
				ctx.drawImage(image, sx, sy, w, h, x * 4, y * 4, w * 4, h * 4)
			part(8, 8, 8, 8, 12, 0)
			part(20, 20, 8, 12, 12, 8)
			part(44, 20, arm, 12, 12 - arm, 8)
			part(4, 20, 4, 12, 12, 20)
			if (image.naturalHeight === 64) {
				part(36, 52, arm, 12, 20, 8)
				part(20, 52, 4, 12, 16, 20)
				part(20, 36, 8, 12, 12, 8)
				part(44, 36, arm, 12, 12 - arm, 8)
				part(52, 52, arm, 12, 20, 8)
				part(4, 36, 4, 12, 12, 20)
				part(4, 52, 4, 12, 16, 20)
			} else {
				part(44, 20, arm, 12, 20, 8)
				part(4, 20, 4, 12, 16, 20)
			}
			part(40, 8, 8, 8, 12, 0)
		}
		image.src = props.src
	},
	{ immediate: true },
)
onBeforeUnmount(() => {
	revision++
})
</script>
<template>
	<canvas ref="canvas" width="128" height="128" class="mx-auto block size-32" aria-hidden="true" />
</template>
