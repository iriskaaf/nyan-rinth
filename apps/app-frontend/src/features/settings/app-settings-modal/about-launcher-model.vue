<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import {
	ACESFilmicToneMapping,
	AmbientLight,
	Box3,
	DirectionalLight,
	Group,
	Mesh,
	PerspectiveCamera,
	Scene,
	Vector3,
	WebGLRenderer,
} from 'three'
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js'

import modelUrl from '@/assets/nyanrinth.glb?url'

const host = ref<HTMLDivElement | null>(null)
let cleanup = () => {}

function disposeModel(model: Group) {
	model.traverse((object) => {
		if (!(object instanceof Mesh)) return
		object.geometry.dispose()
		for (const material of Array.isArray(object.material) ? object.material : [object.material]) {
			material.dispose()
		}
	})
}

onMounted(() => {
	if (!host.value) return
	let renderer: WebGLRenderer
	try {
		renderer = new WebGLRenderer({ alpha: true, antialias: true, powerPreference: 'low-power' })
	} catch {
		return
	}
	const element = host.value
	const scene = new Scene()
	const camera = new PerspectiveCamera(35, 1, 0.1, 100)
	const pivot = new Group()
	pivot.rotation.y = Math.PI + 0.45
	scene.add(pivot, new AmbientLight(0xffffff, 2))
	const keyLight = new DirectionalLight(0xffe4f2, 4)
	keyLight.position.set(-3, 4, -5)
	scene.add(keyLight)
	const rimLight = new DirectionalLight(0xa8caff, 3)
	rimLight.position.set(3, 2, 3)
	scene.add(rimLight)
	renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5))
	renderer.setClearColor(0x000000, 0)
	renderer.toneMapping = ACESFilmicToneMapping
	renderer.toneMappingExposure = 1.1
	element.append(renderer.domElement)
	let dragging = false
	let lastPointerX = 0
	let lastPointerY = 0
	let pitch = 0
	const onPointerDown = (event: PointerEvent) => {
		dragging = true
		lastPointerX = event.clientX
		lastPointerY = event.clientY
		element.setPointerCapture(event.pointerId)
		element.classList.add('is-dragging')
	}
	const onPointerMove = (event: PointerEvent) => {
		if (!dragging || !model) return
		pivot.rotation.y += (event.clientX - lastPointerX) * 0.012
		pitch = Math.max(-0.65, Math.min(0.65, pitch + (event.clientY - lastPointerY) * 0.008))
		pivot.rotation.x = pitch
		lastPointerX = event.clientX
		lastPointerY = event.clientY
		render()
	}
	const onPointerUp = (event: PointerEvent) => {
		if (!dragging) return
		dragging = false
		if (element.hasPointerCapture(event.pointerId)) element.releasePointerCapture(event.pointerId)
		element.classList.remove('is-dragging')
	}
	element.addEventListener('pointerdown', onPointerDown)
	element.addEventListener('pointermove', onPointerMove)
	element.addEventListener('pointerup', onPointerUp)
	element.addEventListener('pointercancel', onPointerUp)

	let disposed = false
	let model: Group | undefined
	let frame = 0
	let previous = 0
	let lastRender = 0
	const reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)')
	const render = () => renderer.render(scene, camera)
	const resize = () => {
		const { width, height } = element.getBoundingClientRect()
		if (!width || !height) return
		camera.aspect = width / height
		camera.position.set(0, 0.2, 5.7 / Math.min(camera.aspect, 1))
		camera.lookAt(0, 0, 0)
		camera.updateProjectionMatrix()
		renderer.setSize(width, height)
		render()
	}
	const observer = new ResizeObserver(resize)
	observer.observe(element)
	const animate = (now: number) => {
		frame = requestAnimationFrame(animate)
		if (now - lastRender < 1000 / 30) return
		if (!dragging) pivot.rotation.y += Math.min((now - previous) / 1000, 0.05) * 0.3
		previous = now
		lastRender = now
		render()
	}
	const updateAnimation = () => {
		cancelAnimationFrame(frame)
		previous = performance.now()
		if (!document.hidden && !reducedMotion.matches && model) {
			frame = requestAnimationFrame(animate)
		} else {
			render()
		}
	}
	document.addEventListener('visibilitychange', updateAnimation)
	reducedMotion.addEventListener('change', updateAnimation)
	cleanup = () => {
		if (disposed) return
		disposed = true
		cancelAnimationFrame(frame)
		element.removeEventListener('pointerdown', onPointerDown)
		element.removeEventListener('pointermove', onPointerMove)
		element.removeEventListener('pointerup', onPointerUp)
		element.removeEventListener('pointercancel', onPointerUp)
		observer.disconnect()
		document.removeEventListener('visibilitychange', updateAnimation)
		reducedMotion.removeEventListener('change', updateAnimation)
		if (model) disposeModel(model)
		renderer.dispose()
		renderer.forceContextLoss()
		renderer.domElement.remove()
	}
	void new GLTFLoader()
		.loadAsync(modelUrl)
		.then((gltf) => {
			if (disposed) {
				disposeModel(gltf.scene)
				return
			}
			model = gltf.scene
			const bounds = new Box3().setFromObject(model)
			const size = bounds.getSize(new Vector3())
			const center = bounds.getCenter(new Vector3())
			const scale = 2.6 / Math.max(size.x, size.y, size.z)
			model.scale.setScalar(scale)
			model.position.copy(center.multiplyScalar(-scale))
			pivot.add(model)
			resize()
			updateAnimation()
		})
		.catch(() => {
			cleanup()
		})
})

onBeforeUnmount(() => cleanup())
</script>

<template>
	<div ref="host" class="about-model" aria-hidden="true" />
</template>

<style scoped>
.about-model {
	position: absolute;
	inset: 0;
	pointer-events: auto;
	touch-action: none;
	cursor: grab;
}
.about-model.is-dragging {
	cursor: grabbing;
}
.about-model :deep(canvas) {
	display: block;
}
</style>
