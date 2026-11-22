<script setup lang="ts">
import {
	CodeIcon,
	CoffeeIcon,
	GaugeIcon,
	LanguagesIcon,
	PaintbrushIcon,
	RefreshCwIcon,
	Settings2Icon,
	ToggleRightIcon,
	WindowIcon,
	WrenchIcon,
} from '@modrinth/assets'
import {
	applyAdaptedAccents,
	Button,
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	injectNotificationManager,
	NewModal,
	primaryAccent,
	secondaryAccent,
	UnsavedChangesPopup,
	useVIntl,
	variantForTheme,
} from '@modrinth/ui'
import { useMutation, useQuery, useQueryClient } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { computed, nextTick, ref, watch } from 'vue'

import AlLogo from '@/components/ui/al-logo.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { useTheme } from '@/composables/use-theme.ts'
import {
	provideAppSettingsModal,
	type UnsavedChangesController,
} from '@/features/settings/app-settings-modal/context'
import AppearanceSettings from '@/features/settings/display/appearance-settings.vue'
import BehaviorSettings from '@/features/settings/display/behavior-settings.vue'
import FeatureFlagSettings from '@/features/settings/display/feature-flag-settings.vue'
import FeaturesSettings from '@/features/settings/display/features-settings.vue'
import LanguageSettings from '@/features/settings/display/language-settings.vue'
import JavaAndMemorySettings from '@/features/settings/instance-defaults/java-and-memory-settings/index.vue'
import LaunchHooksSettings from '@/features/settings/instance-defaults/launch-hooks-settings.vue'
import WindowSettings from '@/features/settings/instance-defaults/window-settings.vue'
import SyncingSettings from '@/features/settings/syncing/syncing-settings.vue'
import ResourceManagementSettings from '@/features/settings/system/resource-management-settings/index.vue'
import TroubleshootingSettings from '@/features/settings/system/troubleshooting-settings.vue'
import { appSettingsKeys, appSettingsQueryOptions, set as setSettings } from '@/helpers/settings.ts'
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

defineOptions({ name: 'SettingsPage' })

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const appSettings = useAppSettings()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	title: { id: 'app.settings.title', defaultMessage: 'Settings' },
	themePromptTitle: {
		id: 'app.settings.theme-prompt.title',
		defaultMessage: 'Switch to a better-suited colour?',
	},
	themePromptBody: {
		id: 'app.settings.theme-prompt.body',
		defaultMessage:
			'Against a different background the accent colour can adapt automatically so it never clashes.',
	},
	themePromptPrimary: {
		id: 'app.settings.theme-prompt.primary',
		defaultMessage: 'Accent',
	},
	themePromptSecondary: {
		id: 'app.settings.theme-prompt.secondary',
		defaultMessage: 'Secondary',
	},
	themePromptConfirm: {
		id: 'app.settings.theme-prompt.confirm',
		defaultMessage: 'Adapt my accent colour',
	},
})

const tabCategories = defineMessages({
	display: { id: 'app.settings.categories.display', defaultMessage: 'Display' },
	instanceDefaults: {
		id: 'app.settings.categories.instance-defaults',
		defaultMessage: 'Instance defaults',
	},
	system: { id: 'app.settings.categories.system', defaultMessage: 'System' },
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.display,
		content: AppearanceSettings,
		icon: PaintbrushIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.features',
			defaultMessage: 'Features',
		}),
		category: tabCategories.display,
		content: FeaturesSettings,
		icon: Settings2Icon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.display,
		content: BehaviorSettings,
		icon: ToggleRightIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		category: tabCategories.display,
		content: LanguageSettings,
		icon: LanguagesIcon,
		badge: commonMessages.beta,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		content: FeatureFlagSettings,
		icon: ToggleRightIcon,
		developerOnly: true,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-and-memory',
			defaultMessage: 'Java and memory',
		}),
		category: tabCategories.instanceDefaults,
		content: JavaAndMemorySettings,
		icon: CoffeeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.window',
			defaultMessage: 'Window',
		}),
		category: tabCategories.instanceDefaults,
		content: WindowSettings,
		icon: WindowIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.launch-hooks',
			defaultMessage: 'Launch hooks',
		}),
		category: tabCategories.instanceDefaults,
		content: LaunchHooksSettings,
		icon: CodeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.syncing',
			defaultMessage: 'Syncing',
		}),
		category: tabCategories.instanceDefaults,
		content: SyncingSettings,
		icon: RefreshCwIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resources',
			defaultMessage: 'Resources',
		}),
		category: tabCategories.system,
		content: ResourceManagementSettings,
		icon: GaugeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.troubleshooting',
			defaultMessage: 'Troubleshooting',
		}),
		category: tabCategories.system,
		content: TroubleshootingSettings,
		icon: WrenchIcon,
	},
]

const visibleTabs = computed(() => tabs.filter((tab) => !tab.developerOnly || appSettings.devMode))
const selected = ref(0)
const currentTab = computed(
	() => visibleTabs.value[Math.min(selected.value, visibleTabs.value.length - 1)],
)
const startsCategory = (index) =>
	!!visibleTabs.value[index]?.category &&
	visibleTabs.value[index]?.category?.id !== visibleTabs.value[index - 1]?.category?.id

watch(visibleTabs, (next) => {
	if (selected.value > next.length - 1) selected.value = 0
})

const unsavedChangesPopup = ref<{ nudge: () => void } | null>(null)
const unsavedChangesController = ref<UnsavedChangesController | null>(null)
const emptyUnsavedChangesState: Record<string, unknown> = {}
const originalUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getOriginal() ?? emptyUnsavedChangesState,
)
const modifiedUnsavedChangesState = computed(
	() => unsavedChangesController.value?.getModified() ?? emptyUnsavedChangesState,
)
const savingUnsavedChanges = computed(() => unsavedChangesController.value?.isSaving() ?? false)
const hasUnsavedChanges = computed(
	() =>
		(unsavedChangesController.value?.hasChanges() ?? false) ||
		(unsavedChangesController.value?.isSaving() ?? false),
)

function canLeaveCurrentTab(): boolean {
	if (
		!unsavedChangesController.value?.hasChanges() &&
		!unsavedChangesController.value?.isSaving()
	) {
		return true
	}
	unsavedChangesPopup.value?.nudge()
	return false
}

function selectTab(index: number) {
	if (index === selected.value) return
	if (!canLeaveCurrentTab()) return
	selected.value = index
}

function resetUnsavedChanges() {
	unsavedChangesController.value?.reset()
}

function saveUnsavedChanges() {
	void unsavedChangesController.value?.save()
}

const queryClient = useQueryClient()
const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const { data: appInfo } = useQuery({
	queryKey: ['app-info'],
	queryFn: async () => ({
		version: await getVersion(),
		osPlatform: getOsPlatform(),
		osVersion: getOsVersion(),
	}),
	staleTime: Infinity,
})

const developerModeMutation = useMutation({
	mutationKey: appSettingsKeys.update,
	scope: { id: 'app-settings' },
	mutationFn: async (enabled: boolean) => {
		const settings = await queryClient.fetchQuery(appSettingsQueryOptions())
		const nextSettings = { ...settings, developer_mode: enabled }
		await setSettings(nextSettings)
		return nextSettings
	},
	onMutate: () => queryClient.cancelQueries({ queryKey: appSettingsKeys.all }),
	onSuccess: (settings) => {
		queryClient.setQueryData(appSettingsKeys.all, settings)
		appSettings.devMode = settings.developer_mode
	},
	onError: handleError,
	onSettled: () => queryClient.invalidateQueries({ queryKey: appSettingsKeys.all }),
})

function devModeCount() {
	if (developerModeMutation.isPending.value) return
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		devModeCounter.value = 0
		developerModeMutation.mutate(!appSettings.devMode)
	}
}

const theme = useTheme()
const themePrompt = ref(false)
const themePromptRef = ref<{ show?: () => void } | null>(null)

// Every theme switch asks whether the accent colours should adapt, no matter
// whether they were customised before.
watch(
	() => theme.preferred,
	() => {
		themePrompt.value = true
		nextTick(() => themePromptRef.value?.show?.())
	},
)

/** Shifts the colour lightness so it keeps contrast on the active theme. */
/** Adapts with the same lightness delta the default purple pair uses. */
function adaptAccent(hex: string, dark: boolean): string {
	return variantForTheme(hex, dark)
}

const themePromptPreview = computed(() => {
	const dark = theme.active !== 'light'
	return {
		primaryFrom: primaryAccent.value,
		primaryTo: primaryAccent.value ? adaptAccent(primaryAccent.value, dark) : '',
		secondaryFrom: secondaryAccent.value,
		secondaryTo: secondaryAccent.value ? adaptAccent(secondaryAccent.value, dark) : '',
	}
})

function adaptAccentToTheme() {
	try {
		const dark = theme.active !== 'light'
		const nextPrimary = primaryAccent.value ? adaptAccent(primaryAccent.value, dark) : ''
		const nextSecondary = secondaryAccent.value ? adaptAccent(secondaryAccent.value, dark) : ''
		applyAdaptedAccents(nextPrimary, nextSecondary)
	} catch (error) {
		handleError(error as Error)
	} finally {
		themePrompt.value = false
	}
}

provideAppSettingsModal({
	close: () => true,
	registerUnsavedChangesController: (controller) => {
		unsavedChangesController.value = controller
	},
})

breadcrumbManager.reset({
	slot: 'settings',
	id: () => 'settings',
	label: () => formatMessage(messages.title),
})
</script>

<template>
	<div class="flex w-full flex-col gap-4 p-4 sm:p-6">
		<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>

		<div class="flex items-start gap-4">
			<div
				class="flex w-[250px] shrink-0 flex-col gap-2 rounded-[20px] border border-solid border-surface-5 bg-surface-3 p-4"
			>
				<div class="flex flex-col gap-1">
					<template v-for="(tab, index) in visibleTabs" :key="index">
						<div
							v-if="startsCategory(index) && tab.category"
							class="shrink-0 truncate pb-1.5 text-sm font-extrabold uppercase leading-5 text-secondary"
							:class="{ 'mt-3': index > 0 }"
						>
							{{ formatMessage(tab.category) }}
						</div>
						<button
							type="button"
							class="flex min-w-0 shrink-0 cursor-pointer items-center gap-2 rounded-[14px] border-none px-4 py-2.5 text-left text-base font-semibold leading-5 no-underline transition-all active:scale-[0.94]"
							:class="
								selected === index
									? 'bg-[color-mix(in_srgb,var(--color-purple-400)_30%,transparent)] text-brand'
									: 'bg-transparent text-primary hover:bg-surface-4 hover:text-contrast'
							"
							@click="selectTab(index)"
						>
							<component :is="tab.icon" class="size-5 flex-shrink-0" />
							<span class="min-w-0 flex-1 truncate">{{ formatMessage(tab.name) }}</span>
							<span
								v-if="tab.badge"
								class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand"
							>
								{{ formatMessage(tab.badge) }}
							</span>
						</button>
					</template>
				</div>

				<div class="mt-auto flex flex-col gap-2 pt-2 text-xs text-secondary">
					<p v-if="appSettings.devMode" class="m-0 font-semibold text-brand">
						{{ formatMessage(developerModeEnabled) }}
					</p>
					<div class="flex items-center gap-2">
						<button
							type="button"
							:aria-label="'Alemona Launcher'"
							:disabled="developerModeMutation.isPending.value"
							class="cursor-pointer border-none bg-transparent p-0"
							@click="devModeCount"
						>
							<AlLogo class="size-7" />
						</button>
						<div v-if="appInfo" class="max-w-[150px] leading-4">
							<p class="m-0">v{{ appInfo.version }}</p>
							<p class="m-0 capitalize">
								{{ appInfo.osPlatform === 'macos' ? 'macOS' : appInfo.osPlatform }}
								{{ appInfo.osVersion }}
							</p>
						</div>
					</div>
				</div>
			</div>

			<div
				class="relative min-w-0 flex-1 overflow-hidden rounded-[20px] border border-solid border-surface-5 bg-surface-2"
			>
				<div class="overscroll-contain">
					<div class="flow-root min-h-full px-6 pb-6 pt-6">
						<component :is="currentTab.content" />
					</div>
				</div>
			</div>
		</div>

		<NewModal
			v-if="themePrompt"
			ref="themePromptRef"
			:header="formatMessage(messages.themePromptTitle)"
			max-width="520px"
			:on-hide="() => undefined"
			:on-after-hide="() => (themePrompt = false)"
		>
			<p class="m-0 text-sm leading-6 text-primary">
				{{ formatMessage(messages.themePromptBody) }}
			</p>
			<div class="flex flex-col gap-2">
				<div class="flex items-center gap-2 text-xs text-secondary">
					<span class="w-16 shrink-0">{{ formatMessage(messages.themePromptPrimary) }}</span>
					<span
						class="size-4 shrink-0 rounded-sm border border-solid border-surface-5"
						:style="{ background: themePromptPreview.primaryFrom }"
					/>
					<span>→</span>
					<span
						class="size-4 shrink-0 rounded-sm border border-solid border-surface-5"
						:style="{ background: themePromptPreview.primaryTo }"
					/>
				</div>
				<div class="flex items-center gap-2 text-xs text-secondary">
					<span class="w-16 shrink-0">{{ formatMessage(messages.themePromptSecondary) }}</span>
					<span
						class="size-4 shrink-0 rounded-sm border border-solid border-surface-5"
						:style="{ background: themePromptPreview.secondaryFrom }"
					/>
					<span>→</span>
					<span
						class="size-4 shrink-0 rounded-sm border border-solid border-surface-5"
						:style="{ background: themePromptPreview.secondaryTo }"
					/>
				</div>
			</div>
			<template #actions>
				<div class="ml-auto flex w-full items-center justify-end gap-2">
					<Button type="colored" color="brand" @click="adaptAccentToTheme">
						{{ formatMessage(messages.themePromptConfirm) }}
					</Button>
				</div>
			</template>
		</NewModal>

		<div v-if="hasUnsavedChanges" class="sticky bottom-4 z-10">
			<UnsavedChangesPopup
				ref="unsavedChangesPopup"
				:original="originalUnsavedChangesState"
				:modified="modifiedUnsavedChangesState"
				:saving="savingUnsavedChanges"
				inline
				@reset="resetUnsavedChanges"
				@save="saveUnsavedChanges"
			/>
		</div>
	</div>
</template>
