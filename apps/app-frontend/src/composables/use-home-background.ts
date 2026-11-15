import { ref } from 'vue'

const MODE_KEY = 'alemona.home-background-mode'
const IMAGE_KEY = 'alemona.home-background-image'

export type HomeBackgroundMode = 'news' | 'custom' | 'none'

const stored = localStorage.getItem(MODE_KEY)
export const homeBackgroundMode = ref<HomeBackgroundMode>(
	stored === 'custom' || stored === 'none' ? stored : 'news',
)
export const homeBackgroundImage = ref<string | null>(localStorage.getItem(IMAGE_KEY))

function persist() {
	localStorage.setItem(MODE_KEY, homeBackgroundMode.value)
	if (homeBackgroundImage.value) localStorage.setItem(IMAGE_KEY, homeBackgroundImage.value)
	else localStorage.removeItem(IMAGE_KEY)
}

export function useHomeBackground() {
	return { homeBackgroundMode, homeBackgroundImage, persist }
}

export function setCustomBackground(dataUrl: string | null) {
	homeBackgroundImage.value = dataUrl
	homeBackgroundMode.value = dataUrl ? 'custom' : 'none'
	persist()
}

export function useNewsBackground(enabled: boolean) {
	homeBackgroundMode.value = enabled ? 'news' : homeBackgroundImage.value ? 'none' : 'none'
	persist()
}

const WELCOME_KEY = 'alemona.display-welcome'

/** Welcome block on the home page, enabled by default. */
export const displayWelcome = ref(localStorage.getItem(WELCOME_KEY) !== 'false')

export function setDisplayWelcome(value: boolean) {
	displayWelcome.value = value
	localStorage.setItem(WELCOME_KEY, value ? 'true' : 'false')
}
