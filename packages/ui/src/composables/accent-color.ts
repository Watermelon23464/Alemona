import { ref } from 'vue'

/**
 * Accent colours for the launcher.
 *
 * The defaults are read from the theme tokens at runtime (no hardcoded purple),
 * and the secondary colour is derived from the primary with the same shift the
 * default purple pair uses (hue +7°, saturation +11%, lightness +18%).
 */
const STORAGE_KEY = 'accent-color'
const STORAGE_SECONDARY_KEY = 'accent-color-secondary'
const STORAGE_AUTO_KEY = 'accent-color-auto'

export const primaryAccent = ref<string>('')
export const secondaryAccent = ref<string>('')
export const autoDeriveAccent = ref<boolean>(readAuto())

/** The colours the user actually picked, used as the base for theme adaptation. */
export const basePrimaryAccent = ref<string>('')
export const baseSecondaryAccent = ref<string>('')

function read(key: string, fallback: string): string {
	if (typeof localStorage === 'undefined') return fallback
	return localStorage.getItem(key) ?? fallback
}

function readAuto(): boolean {
	if (typeof localStorage === 'undefined') return true
	return localStorage.getItem(STORAGE_AUTO_KEY) !== 'false'
}

export function hexToRgb(hex: string): [number, number, number] {
	const value = hex.replace('#', '')
	const full =
		value.length === 3
			? value
					.split('')
					.map((character) => character + character)
					.join('')
			: value
	const number = Number.parseInt(full.slice(0, 6), 16)
	return [(number >> 16) & 255, (number >> 8) & 255, number & 255]
}

export function rgbToHex(r: number, g: number, b: number): string {
	const clamp = (input: number) => Math.max(0, Math.min(255, Math.round(input)))
	return `#${[clamp(r), clamp(g), clamp(b)]
		.map((part) => part.toString(16).padStart(2, '0'))
		.join('')}`
}

export function rgbToHsl(r: number, g: number, b: number): [number, number, number] {
	const red = r / 255
	const green = g / 255
	const blue = b / 255
	const max = Math.max(red, green, blue)
	const min = Math.min(red, green, blue)
	const delta = max - min
	const lightness = (max + min) / 2
	let hue = 0
	let saturation = 0
	if (delta !== 0) {
		saturation = delta / (1 - Math.abs(2 * lightness - 1))
		hue =
			max === red
				? 60 * (((green - blue) / delta) % 6)
				: max === green
					? 60 * ((blue - red) / delta + 2)
					: 60 * ((red - green) / delta + 4)
	}
	return [(hue + 360) % 360, saturation * 100, lightness * 100]
}

export function hslToHex(h: number, s: number, l: number): string {
	const saturation = Math.max(0, Math.min(100, s)) / 100
	const lightness = Math.max(0, Math.min(100, l)) / 100
	const chroma = (1 - Math.abs(2 * lightness - 1)) * saturation
	const hue = ((h % 360) + 360) % 360
	const x = chroma * (1 - Math.abs(((hue / 60) % 2) - 1))
	const m = lightness - chroma / 2
	const [r, g, b] =
		hue < 60
			? [chroma, x, 0]
			: hue < 120
				? [x, chroma, 0]
				: hue < 180
					? [0, chroma, x]
					: hue < 240
						? [0, x, chroma]
						: hue < 300
							? [x, 0, chroma]
							: [chroma, 0, x]
	return rgbToHex((r + m) * 255, (g + m) * 255, (b + m) * 255)
}

/** Same relationship the default theme purple pair uses. */
const HUE_SHIFT = 7
const SATURATION_SHIFT = 11
const LIGHTNESS_SHIFT = 18

export function deriveSecondary(primary: string): string {
	const [h, s, l] = rgbToHsl(...hexToRgb(primary))
	return hslToHex(h + HUE_SHIFT, s + SATURATION_SHIFT, l + LIGHTNESS_SHIFT)
}

export function derivePrimary(secondary: string): string {
	const [h, s, l] = rgbToHsl(...hexToRgb(secondary))
	return hslToHex(h - HUE_SHIFT, s - SATURATION_SHIFT, l - LIGHTNESS_SHIFT)
}

/** Same lightness delta the default purple pair uses between its light and dark variants. */
const THEME_VARIANT_LIGHTNESS = 18

/** Variant of a colour that matches the current theme brightness. */
export function variantForTheme(hex: string, dark: boolean): string {
	const [h, s, l] = rgbToHsl(...hexToRgb(hex))
	const next = dark ? l + THEME_VARIANT_LIGHTNESS : l - THEME_VARIANT_LIGHTNESS
	return hslToHex(h, s, Math.min(88, Math.max(26, next)))
}

export function applyAccentColors(primary: string, secondary: string): void {
	if (typeof document === 'undefined') return
	const root = document.documentElement
	const [r, g, b] = hexToRgb(primary)
	const [sr, sg, sb] = hexToRgb(secondary)
	// 600 is the deep variant used by light themes, 400 the light variant used by
	// dark themes, so `--color-brand` keeps resolving to a readable shade.
	const lightVariant = variantForTheme(primary, true)
	root.style.setProperty('--color-purple-600', primary)
	root.style.setProperty('--color-purple-400', lightVariant)
	root.style.setProperty('--color-purple-highlight', `rgba(${sr}, ${sg}, ${sb}, 0.25)`)
	root.style.setProperty('--color-brand-highlight', `rgba(${sr}, ${sg}, ${sb}, 0.25)`)
	root.style.setProperty('--color-brand-shadow', `rgba(${r}, ${g}, ${b}, 0.7)`)
	root.style.setProperty(
		'--loading-bar-gradient',
		`linear-gradient(to right, ${primary} 0%, ${secondary} 100%)`,
	)
}

export function clearAccentColors(): void {
	if (typeof document === 'undefined') return
	const root = document.documentElement
	for (const token of [
		'--color-brand-highlight',
		'--color-brand-shadow',
		'--loading-bar-gradient',
		'--color-purple-600',
		'--color-purple-400',
		'--color-purple-highlight',
	]) {
		root.style.removeProperty(token)
	}
}

let saveTimer: ReturnType<typeof setTimeout> | null = null

/** Records the colours currently in use, debounced while dragging. */
function saveToFile(): void {
	if (typeof window === 'undefined') return
	if (saveTimer) clearTimeout(saveTimer)
	saveTimer = setTimeout(() => {
		saveTimer = null
		void (async () => {
			try {
				const { invoke } = await import('@tauri-apps/api/core')
				await invoke('plugin:al-color|al_color_save', {
					primary: primaryAccent.value,
					secondary: secondaryAccent.value,
				})
			} catch {
				// running outside the desktop shell (web preview), nothing to save
			}
		})()
	}, 250)
}

function persist(key: string, value: string | null): void {
	if (typeof localStorage === 'undefined') return
	if (value === null) localStorage.removeItem(key)
	else localStorage.setItem(key, value)
}

export function setPrimaryAccent(hex: string): void {
	primaryAccent.value = hex
	basePrimaryAccent.value = hex
	if (autoDeriveAccent.value) secondaryAccent.value = deriveSecondary(hex)
	persist(STORAGE_KEY, hex)
	persist(STORAGE_SECONDARY_KEY, secondaryAccent.value)
	applyAccentColors(primaryAccent.value, secondaryAccent.value)
	saveToFile()
}

export function setSecondaryAccent(hex: string): void {
	secondaryAccent.value = hex
	baseSecondaryAccent.value = hex
	if (autoDeriveAccent.value) primaryAccent.value = derivePrimary(hex)
	persist(STORAGE_SECONDARY_KEY, hex)
	persist(STORAGE_KEY, primaryAccent.value)
	applyAccentColors(primaryAccent.value, secondaryAccent.value)
	saveToFile()
}

export function setAutoDeriveAccent(enabled: boolean): void {
	autoDeriveAccent.value = enabled
	persist(STORAGE_AUTO_KEY, String(enabled))
	if (enabled) setPrimaryAccent(primaryAccent.value)
}

/** Reads the live theme tokens so the "default" is whatever the theme uses. */
function toHex(value: string | undefined | null): string | null {
	if (!value) return null
	const trimmed = value.trim()
	if (trimmed.startsWith('#')) return trimmed.toLowerCase()
	const match = trimmed.match(/rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/)
	return match ? rgbToHex(Number(match[1]), Number(match[2]), Number(match[3])) : null
}

/** Reads a live theme token, so "default" is always the theme colour. */
function themeToken(names: string[]): string | null {
	if (typeof document === 'undefined') return null
	const style = getComputedStyle(document.documentElement)
	for (const name of names) {
		const parsed = toHex(style.getPropertyValue(name))
		if (parsed) return parsed
	}
	return null
}

export function themeDefaults(): { primary: string | null; secondary: string | null } {
	return {
		primary: themeToken(['--color-purple', '--color-brand']),
		secondary: themeToken([
			'--color-purple-400',
			'--color-brand-highlight',
			'--color-purple-highlight',
		]),
	}
}

export function initAccentColor(): void {
	const defaults = themeDefaults()
	if (defaults.primary) primaryAccent.value = defaults.primary
	if (defaults.secondary) secondaryAccent.value = defaults.secondary

	if (typeof localStorage === 'undefined') return
	const storedPrimary = localStorage.getItem(STORAGE_KEY)
	const storedSecondary = localStorage.getItem(STORAGE_SECONDARY_KEY)
	if (storedPrimary) primaryAccent.value = storedPrimary
	if (storedSecondary) secondaryAccent.value = storedSecondary
	basePrimaryAccent.value = primaryAccent.value
	baseSecondaryAccent.value = secondaryAccent.value
	if (storedPrimary || storedSecondary) {
		applyAccentColors(primaryAccent.value, secondaryAccent.value)
	}

	void loadFromFile({ primary: !!storedPrimary, secondary: !!storedSecondary })
}

interface StoredColours {
	primary: string
	secondary: string
	history?: { primary: string; secondary: string; saved_at?: string }[]
}

/**
 * Reads AL/cache/color.json as a fallback only: whatever the launcher is using
 * right now always wins, and the file just fills in colours we do not have.
 */
async function loadFromFile(current: { primary: boolean; secondary: boolean }): Promise<void> {
	try {
		const { invoke } = await import('@tauri-apps/api/core')
		const stored = await invoke<StoredColours | null>('plugin:al-color|al_color_load')
		if (!stored) return
		let changed = false
		if (!current.primary && stored.primary) {
			primaryAccent.value = stored.primary
			basePrimaryAccent.value = stored.primary
			changed = true
		}
		if (!current.secondary && stored.secondary) {
			secondaryAccent.value = stored.secondary
			baseSecondaryAccent.value = stored.secondary
			changed = true
		}
		if (changed) applyAccentColors(primaryAccent.value, secondaryAccent.value)
	} catch {
		// no stored colours or running in the browser
	}
}

/** Resets only the primary colour back to the theme value. */
export function resetPrimaryAccent(): void {
	persist(STORAGE_KEY, null)
	clearAccentColors()
	const defaults = themeDefaults()
	if (defaults.primary) primaryAccent.value = defaults.primary
	if (autoDeriveAccent.value && primaryAccent.value) {
		secondaryAccent.value = deriveSecondary(primaryAccent.value)
	}
	applyAccentColors(primaryAccent.value, secondaryAccent.value)
}

/** Resets only the secondary colour back to the theme value. */
export function resetSecondaryAccent(): void {
	persist(STORAGE_SECONDARY_KEY, null)
	clearAccentColors()
	const defaults = themeDefaults()
	if (defaults.secondary) secondaryAccent.value = defaults.secondary
	if (autoDeriveAccent.value && secondaryAccent.value) {
		primaryAccent.value = derivePrimary(secondaryAccent.value)
	}
	if (primaryAccent.value && secondaryAccent.value) {
		applyAccentColors(primaryAccent.value, secondaryAccent.value)
	} else {
		clearAccentColors()
	}
}

/** Applies theme-adapted colours without touching the user's baseline picks. */
export function applyAdaptedAccents(primary: string, secondary: string): void {
	if (primary) primaryAccent.value = primary
	if (secondary) secondaryAccent.value = secondary
	persist(STORAGE_KEY, primaryAccent.value)
	persist(STORAGE_SECONDARY_KEY, secondaryAccent.value)
	applyAccentColors(primaryAccent.value, secondaryAccent.value)
	saveToFile()
}

/** Adapts a colour to the current theme brightness. */
export function adaptToTheme(hex: string, dark: boolean): string {
	const [h, s, l] = rgbToHsl(...hexToRgb(hex))
	return hslToHex(h, s, dark ? Math.min(92, l + 14) : Math.max(28, l - 14))
}
