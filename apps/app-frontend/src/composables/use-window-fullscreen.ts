import { getCurrentWindow } from '@tauri-apps/api/window'
import { ref } from 'vue'

const isFullscreen = ref(false)

/**
 * The window only ever changes fullscreen state from this app, so the state is
 * tracked locally instead of trusting `isFullscreen()`, which reports a stale
 * value right after `setFullscreen()` on Windows.
 */
let fullscreenState = false
let initialized = false

async function initialize() {
	if (initialized) return
	initialized = true
	try {
		if (await getCurrentWindow().isFullscreen()) {
			fullscreenState = true
			isFullscreen.value = true
		}
	} catch {
		// the window is not ready yet; keep the default state
	}
	window.addEventListener('keydown', onKeydown)
}

async function toggleFullscreen() {
	fullscreenState = !fullscreenState
	isFullscreen.value = fullscreenState
	try {
		await getCurrentWindow().setFullscreen(fullscreenState)
	} catch {
		fullscreenState = !fullscreenState
		isFullscreen.value = fullscreenState
	}
}

function onKeydown(event: KeyboardEvent) {
	if (event.key === 'F11') {
		event.preventDefault()
		void toggleFullscreen()
	}
}

/** F11 / shortcut fullscreen toggling for the launcher window. */
export function useWindowFullscreen() {
	void initialize()
	return { isFullscreen, toggleFullscreen }
}

/** Toggles the window between maximized and restored. */
export async function toggleMaximizeWindow() {
	try {
		await getCurrentWindow().toggleMaximize()
	} catch {
		// ignore: the window may be closing
	}
}
