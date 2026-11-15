<script setup lang="ts">
import {
	AppearanceSettingsLayout,
	defineMessages,
	injectAuth,
	injectUserPreferences,
	provideAppearanceSettings,
	useSavable,
	useVIntl,
} from '@modrinth/ui'
import { platform } from '@tauri-apps/plugin-os'
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { homeBackgroundImage } from '@/composables/use-home-background.ts'
import { type ColorTheme, isDarkTheme, useTheme } from '@/composables/use-theme.ts'
import {
	DEFAULT_WELCOME_TEMPLATE,
	setWelcomeTemplate,
	welcomeTemplate,
} from '@/composables/use-welcome-template.ts'
import { injectAppSettingsModal } from '@/features/settings/app-settings-modal/context'
import { type AppSettings, get, set } from '@/helpers/settings.ts'

const theme = useTheme()
const appSettings = useAppSettings()
const auth = injectAuth()
const { updatePreferences } = injectUserPreferences()
const settingsModal = injectAppSettingsModal()
const os = platform()

type AppearanceSettingsState = {
	theme: ColorTheme
	syncAcrossDevices: boolean
	advancedRendering: boolean
	nativeDecorations: boolean
	displayWelcome: boolean
	backgroundLatestNews: boolean
	backgroundCustom: boolean
	backgroundImage: string | null
	newsMenu: boolean
}

function getAppearanceSettingsState(): AppearanceSettingsState {
	return {
		theme: theme.preferred,
		syncAcrossDevices: theme.syncAcrossDevices,
		advancedRendering: theme.advancedRendering,
		nativeDecorations: appSettings.nativeDecorations,
		displayWelcome: appSettings.getFeatureFlag('home_display_welcome'),
		backgroundLatestNews: appSettings.getFeatureFlag('home_background_latest_news'),
		backgroundCustom: appSettings.getFeatureFlag('home_background_custom'),
		backgroundImage: homeBackgroundImage.value,
		newsMenu: appSettings.getFeatureFlag('home_news_menu'),
	}
}

const { saved, current, changes, saving, hasChanges, reset, save } = useSavable(
	getAppearanceSettingsState,
	async (appearanceChanges) => {
		const value = current.value
		if (
			value.syncAcrossDevices &&
			auth.user.value &&
			(appearanceChanges.theme !== undefined || appearanceChanges.syncAcrossDevices !== undefined)
		) {
			await updatePreferences({
				appearance: value.theme === 'system' ? { auto: true } : { auto: false, theme: value.theme },
			})
		}

		const settings = await get()
		const nextSettings: AppSettings = {
			...settings,
			theme: value.theme,
			sync_theme_across_devices: value.syncAcrossDevices,
			advanced_rendering: value.advancedRendering,
			native_decorations: value.nativeDecorations,
			feature_flags: {
				...settings.feature_flags,
				home_display_welcome: value.displayWelcome,
				home_background_latest_news: value.backgroundLatestNews,
				home_background_custom: value.backgroundCustom,
				home_news_menu: value.newsMenu,
			},
		}

		await set(nextSettings)
		if (isDarkTheme(value.theme)) {
			theme.preferredDark = value.theme
		}
		theme.preferred = value.theme
		theme.syncAcrossDevices = value.syncAcrossDevices
		theme.advancedRendering = value.advancedRendering
		appSettings.nativeDecorations = value.nativeDecorations
		appSettings.setFeatureFlag('home_display_welcome', value.displayWelcome)
		appSettings.setFeatureFlag('home_background_latest_news', value.backgroundLatestNews)
		appSettings.setFeatureFlag('home_background_custom', value.backgroundCustom)
		appSettings.setFeatureFlag('home_news_menu', value.newsMenu)
		if (value.backgroundImage) {
			homeBackgroundImage.value = value.backgroundImage
			localStorage.setItem('alemona.home-background-image', value.backgroundImage)
		}
	},
)

const themeOptions = computed(() =>
	theme.options.filter(
		(option) => option !== 'retro' || appSettings.devMode || current.value.theme === 'retro',
	),
)

const preferredDarkTheme = computed(() =>
	isDarkTheme(current.value.theme) ? current.value.theme : theme.preferredDark,
)

function setTheme(value: ColorTheme): void {
	current.value.theme = value
}

function setSyncAcrossDevices(enabled: boolean): void {
	current.value.syncAcrossDevices = enabled
}

function setAdvancedRendering(enabled: boolean): void {
	current.value.advancedRendering = enabled
}

function setNativeDecorations(enabled: boolean): void {
	current.value.nativeDecorations = enabled
}

function setDisplayWelcome(enabled: boolean): void {
	current.value.displayWelcome = enabled
}

function setBackgroundLatestNews(enabled: boolean): void {
	current.value.backgroundLatestNews = enabled
	if (enabled) current.value.backgroundCustom = false
}

function setBackgroundCustom(enabled: boolean): void {
	current.value.backgroundCustom = enabled
	if (enabled) current.value.backgroundLatestNews = false
}

watch(
	[() => current.value.theme, () => saved.value.theme],
	([selectedTheme, savedTheme]) => {
		theme.preview = selectedTheme === savedTheme ? null : selectedTheme
	},
	{ immediate: true },
)

async function saveAppearanceSettings(): Promise<void> {
	try {
		await save()
	} catch {
		return
	}
}

onMounted(() => {
	settingsModal.registerUnsavedChangesController({
		hasChanges: () => hasChanges.value,
		getOriginal: () => saved.value,
		getModified: () => changes.value,
		isSaving: () => saving.value,
		reset,
		save: saveAppearanceSettings,
	})
})

onBeforeUnmount(() => {
	theme.preview = null
	settingsModal.registerUnsavedChangesController(null)
})

provideAppearanceSettings({
	deferPersistence: true,
	theme: {
		current: computed(() => current.value.theme),
		options: themeOptions,
		system: computed(() => (theme.native === 'light' ? 'light' : preferredDarkTheme.value)),
		preferredDark: preferredDarkTheme,
		set: setTheme,
		syncAcrossDevices: {
			value: computed(() => current.value.syncAcrossDevices),
			set: setSyncAcrossDevices,
		},
		syncDisabled: computed(() => !auth.user.value),
	},
	advancedRendering: {
		value: computed(() => current.value.advancedRendering),
		set: setAdvancedRendering,
	},
	nativeDecorations:
		os !== 'macos'
			? {
					value: computed(() => current.value.nativeDecorations),
					set: setNativeDecorations,
				}
			: undefined,
	newsMenu: {
		value: computed(() => current.value.newsMenu),
		set: (enabled: boolean) => {
			current.value.newsMenu = enabled
		},
	},
	homeBackground: {
		displayWelcome: {
			value: computed(() => current.value.displayWelcome),
			set: setDisplayWelcome,
		},
		welcomeTemplate: {
			value: computed(() => welcomeTemplate.value),
			set: (value: string) => setWelcomeTemplate(value),
			defaultValue: computed(() => DEFAULT_WELCOME_TEMPLATE),
		},
		latestNews: {
			value: computed(() => current.value.backgroundLatestNews),
			set: setBackgroundLatestNews,
		},
		customImage: {
			value: computed(() => current.value.backgroundCustom),
			set: setBackgroundCustom,
		},
	},
	updatePreferences,
})

const { formatMessage } = useVIntl()
const pickerMessages = defineMessages({
	hint: {
		id: 'settings.display.background-images.custom.description',
		defaultMessage: 'Choose your own PNG image as the homepage background.',
	},
})

const customImageInput = ref<HTMLInputElement | null>(null)
const dropHint = computed(() => formatMessage(pickerMessages.hint))

function readImageFile(file: File | null | undefined) {
	if (!file) return
	if (file.type !== 'image/png' && !file.name.toLowerCase().endsWith('.png')) return
	const reader = new FileReader()
	reader.onload = () => {
		if (typeof reader.result !== 'string') return
		const image = new Image()
		image.onload = () => {
			const maxWidth = 1920
			const scale = Math.min(1, maxWidth / image.width)
			const canvas = document.createElement('canvas')
			canvas.width = Math.round(image.width * scale)
			canvas.height = Math.round(image.height * scale)
			canvas.getContext('2d')?.drawImage(image, 0, 0, canvas.width, canvas.height)
			current.value.backgroundImage = canvas.toDataURL('image/png')
			current.value.backgroundCustom = true
			current.value.backgroundLatestNews = false
		}
		image.src = reader.result
	}
	reader.readAsDataURL(file)
}

function onDrop(event: DragEvent) {
	event.preventDefault()
	readImageFile(event.dataTransfer?.files?.[0])
}
</script>

<template>
	<AppearanceSettingsLayout>
		<template #background-image-picker>
			<section class="mt-8 border-0 border-t border-solid border-divider pt-6">
				<div
					class="flex min-h-24 cursor-pointer items-center justify-center rounded-xl border-2 border-dashed border-surface-4 bg-surface-3 p-4 text-center text-sm text-secondary"
					@click="customImageInput?.click()"
					@dragover.prevent
					@drop="onDrop"
				>
					<img
						v-if="current.backgroundImage"
						:src="current.backgroundImage"
						alt=""
						class="mb-2 max-h-40 rounded-lg object-cover"
					/>
					<span v-else>{{ dropHint }}</span>
				</div>
				<input
					ref="customImageInput"
					type="file"
					accept="image/png"
					class="hidden"
					@change="readImageFile(($event.target as HTMLInputElement).files?.[0])"
				/>
			</section>
		</template>
	</AppearanceSettingsLayout>
</template>
