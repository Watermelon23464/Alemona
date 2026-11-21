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
	commonMessages,
	commonSettingsMessages,
	defineMessage,
	defineMessages,
	UnsavedChangesPopup,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

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
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

defineOptions({ name: 'SettingsPage' })

const { formatMessage } = useVIntl()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	title: { id: 'app.settings.title', defaultMessage: 'Settings' },
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

const selected = ref(0)
const currentTab = computed(() => tabs[selected.value])
const startsCategory = (index) =>
	!!tabs[index]?.category && tabs[index]?.category?.id !== tabs[index - 1]?.category?.id

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
					<template v-for="(tab, index) in tabs" :key="index">
						<div
							v-if="startsCategory(index) && tab.category"
							class="shrink-0 truncate pb-1.5 text-sm font-extrabold uppercase leading-5 text-secondary"
							:class="{ 'mt-3': index > 0 }"
						>
							{{ formatMessage(tab.category) }}
						</div>
						<button
							v-if="!tab.developerOnly || true"
							type="button"
							class="flex min-w-0 shrink-0 cursor-pointer items-center gap-2 rounded-[14px] border-none px-4 py-2.5 text-left text-base font-semibold leading-5 no-underline transition-all active:scale-[0.94]"
							:class="
								selected === index
									? 'bg-[color-mix(in_srgb,var(--color-brand)_30%,transparent)] text-brand'
									: 'bg-transparent text-primary hover:bg-surface-4 hover:text-contrast'
							"
							@click="selectTab(index)"
						>
							<component :is="tab.icon" class="size-5 flex-shrink-0" />
							<span class="min-w-0 flex-1 truncate">{{ formatMessage(tab.name) }}</span>
							<span
								v-if="tab.badge"
								class="shrink-0 rounded-full bg-brand-highlight px-1.5 py-0.5 text-xs font-bold text-brand-green"
							>
								{{ formatMessage(tab.badge) }}
							</span>
						</button>
					</template>
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
