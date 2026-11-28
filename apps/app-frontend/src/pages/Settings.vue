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
	UserIcon,
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
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

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
import AddMinecraftAccountView from '@/features/settings/privacy/add-minecraft-account-view.vue'
import MinecraftAccountsSettings from '@/features/settings/privacy/minecraft-accounts.vue'
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
const route = useRoute()
const router = useRouter()

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
	accountsBreadcrumb: {
		id: 'app.settings.tabs.minecraft-accounts',
		defaultMessage: 'Minecraft accounts',
	},
	addAccountBreadcrumb: {
		id: 'app.settings.accounts.add-page-title',
		defaultMessage: 'Add Minecraft account',
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
	privacy: { id: 'app.settings.categories.privacy', defaultMessage: 'Privacy and protection' },
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		category: tabCategories.display,
		content: AppearanceSettings,
		slug: 'appearance',
		icon: PaintbrushIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.features',
			defaultMessage: 'Features',
		}),
		category: tabCategories.display,
		content: FeaturesSettings,
		slug: 'features',
		icon: Settings2Icon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.behavior',
			defaultMessage: 'Behavior',
		}),
		category: tabCategories.display,
		content: BehaviorSettings,
		slug: 'behavior',
		icon: ToggleRightIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.language',
			defaultMessage: 'Language',
		}),
		category: tabCategories.display,
		content: LanguageSettings,
		slug: 'language',
		icon: LanguagesIcon,
		badge: commonMessages.beta,
	},
	{
		name: commonSettingsMessages.featureFlags,
		category: tabCategories.display,
		content: FeatureFlagSettings,
		slug: 'feature-flags',
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
		slug: 'java-and-memory',
		icon: CoffeeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.window',
			defaultMessage: 'Window',
		}),
		category: tabCategories.instanceDefaults,
		content: WindowSettings,
		slug: 'window',
		icon: WindowIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.launch-hooks',
			defaultMessage: 'Launch hooks',
		}),
		category: tabCategories.instanceDefaults,
		content: LaunchHooksSettings,
		slug: 'launch-hooks',
		icon: CodeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.syncing',
			defaultMessage: 'Syncing',
		}),
		category: tabCategories.instanceDefaults,
		content: SyncingSettings,
		slug: 'syncing',
		icon: RefreshCwIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resources',
			defaultMessage: 'Resources',
		}),
		category: tabCategories.system,
		content: ResourceManagementSettings,
		slug: 'resources',
		icon: GaugeIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.troubleshooting',
			defaultMessage: 'Troubleshooting',
		}),
		category: tabCategories.system,
		content: TroubleshootingSettings,
		slug: 'troubleshooting',
		icon: WrenchIcon,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.minecraft-accounts',
			defaultMessage: 'Minecraft accounts',
		}),
		category: tabCategories.privacy,
		content: MinecraftAccountsSettings,
		slug: 'minecraft-accounts',
		icon: UserIcon,
	},
]

const visibleTabs = computed(() => tabs.filter((tab) => !tab.developerOnly || appSettings.devMode))
const showingAddAccount = computed(() => route.path.startsWith('/settings/minecraft-accounts/add'))
const accountsTabIndex = computed(() =>
	visibleTabs.value.findIndex((tab) => String(tab.name.id).includes('minecraft-accounts')),
)
const activeTabIndex = ref(0)
const currentTab = computed(
	() => visibleTabs.value[Math.min(activeTabIndex.value, visibleTabs.value.length - 1)],
)
const paneDirection = ref<'left' | 'right'>('right')

watch(
	() => showingAddAccount.value,
	(showAdd) => {
		paneDirection.value = showAdd ? 'right' : 'left'
	},
)

const shownTab = computed(() =>
	showingAddAccount.value && accountsTabIndex.value >= 0
		? visibleTabs.value[accountsTabIndex.value]
		: currentTab.value,
)

const startsCategory = (index) =>
	!!visibleTabs.value[index]?.category &&
	visibleTabs.value[index]?.category?.id !== visibleTabs.value[index - 1]?.category?.id

watch(visibleTabs, (next) => {
	if (activeTabIndex.value > next.length - 1) activeTabIndex.value = 0
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

function isTabActive(tab: { slug?: string }): boolean {
	if (showingAddAccount.value) return tab.slug === 'minecraft-accounts'
	return visibleTabs.value[activeTabIndex.value]?.slug === tab.slug
}

function selectTab(index: number) {
	if (index === activeTabIndex.value) return
	if (!canLeaveCurrentTab()) return
	const tab = visibleTabs.value[index]
	if (!tab) return
	activeTabIndex.value = index
	void router.replace(`/settings/${tab.slug}`)
}

function applyRouteTab(): void {
	const slug = typeof route.params.tabId === 'string' ? route.params.tabId : null
	if (!slug) return
	const index = visibleTabs.value.findIndex((tab) => tab.slug === slug)
	if (index >= 0) activeTabIndex.value = index
}

watch(
	() => route.params.tabId,
	() => applyRouteTab(),
)

onMounted(() => applyRouteTab())

// entering the add page keeps the accounts tab highlighted
watch(showingAddAccount, (showAdd) => {
	if (showAdd && accountsTabIndex.value >= 0) activeTabIndex.value = accountsTabIndex.value
})

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

watch(
	() => [showingAddAccount.value, route.params.tabId],
	() => {
		const showAdd = showingAddAccount.value
		const root = breadcrumbManager.reset({
			slot: 'settings',
			id: () => 'settings',
			label: () => formatMessage(messages.title),
			to: () => '/settings',
		})
		const tab = visibleTabs.value[activeTabIndex.value]
		if (!showAdd && tab) {
			breadcrumbManager.push(
				{
					slot: 'settings-tab',
					id: () => `settings-${tab.slug}`,
					label: () => formatMessage(tab.name),
					to: () => `/settings/${tab.slug}`,
				},
				{ parent: root },
			)
		}
		if (showAdd) {
			const accounts = breadcrumbManager.push(
				{
					slot: 'settings-accounts',
					id: () => 'settings-accounts',
					label: () => formatMessage(messages.accountsBreadcrumb),
					to: () => '/settings/minecraft-accounts',
				},
				{ parent: root },
			)
			breadcrumbManager.push(
				{
					slot: 'settings-accounts-add',
					id: () => 'settings-accounts-add',
					label: () => formatMessage(messages.addAccountBreadcrumb),
				},
				{ parent: accounts },
			)
		}
	},
	{ immediate: true },
)

function leaveAddPage(): void {
	void router.push('/settings/minecraft-accounts')
}
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
								isTabActive(tab)
									? 'bg-[color-mix(in_srgb,var(--color-purple-400)_30%,transparent)] text-brand'
									: 'bg-transparent text-primary hover:bg-surface-4 hover:text-contrast'
							"
							@click="selectTab(index)"
						>
							<component
								:is="tab.icon"
								class="size-5 flex-shrink-0"
								:class="isTabActive(tab) ? 'text-brand' : ''"
							/>
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

			<div class="min-w-0 flex-1 rounded-2xl border border-solid border-surface-5 bg-surface-2">
				<div
					:key="showingAddAccount ? 'add-account' : String(shownTab?.slug ?? 'tab')"
					class="pane-enter px-6 pb-6 pt-6"
					:class="paneDirection === 'left' ? 'pane-from-left' : 'pane-from-right'"
				>
					<AddMinecraftAccountView
						v-if="showingAddAccount"
						@back="leaveAddPage"
						@created="leaveAddPage"
					/>
					<component :is="shownTab.content" v-else />
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
<style scoped>
@keyframes pane-in-right {
	from {
		opacity: 0;
		transform: translateX(22px);
	}
	to {
		opacity: 1;
		transform: translateX(0);
	}
}

@keyframes pane-in-left {
	from {
		opacity: 0;
		transform: translateX(-22px);
	}
	to {
		opacity: 1;
		transform: translateX(0);
	}
}

.pane-enter.pane-from-right {
	animation: pane-in-right 220ms cubic-bezier(0.4, 0, 0.2, 1);
}

.pane-enter.pane-from-left {
	animation: pane-in-left 220ms cubic-bezier(0.4, 0, 0.2, 1);
}
</style>
