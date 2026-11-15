<script setup>
import { BoxIcon, FolderSearchIcon } from '@modrinth/assets'
import { Combobox, defineMessages, IconButton, Input, Slider, useVIntl } from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, ref, watch } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { get, set } from '@/helpers/settings.ts'

import ContentStorageSettings from './content-storage.vue'

const { formatMessage } = useVIntl()
const settings = ref(await get())
const appSettings = useAppSettings()

const versionIsolationOptions = [
	{ value: 'on', label: '开启版本隔离' },
	{ value: 'off', label: '关闭版本隔离' },
]

const versionIsolation = computed({
	get: () => (settings.value.feature_flags?.version_isolation === false ? 'off' : 'on'),
	set: async (value) => {
		const enabled = value !== 'off'
		settings.value.feature_flags = { ...settings.value.feature_flags, version_isolation: enabled }
		await set(settings.value)
		appSettings.setFeatureFlag('version_isolation', enabled)
	},
})

const downloadSource = ref('mojang')

const downloadSourceOptions = computed(() => [
	{ value: 'mojang', label: formatMessage(messages.downloadSourceMojang) },
	{ value: 'bmclapi', label: formatMessage(messages.downloadSourceBmclapi) },
])

async function setDownloadSource(value) {
	downloadSource.value = value === 'bmclapi' ? 'bmclapi' : 'mojang'
	await invoke('plugin:settings|settings_set_version_download_source', {
		source: downloadSource.value,
	})
}

onMounted(async () => {
	try {
		downloadSource.value = await invoke('plugin:settings|settings_get_version_download_source')
	} catch {
		downloadSource.value = 'mojang'
	}
})

const messages = defineMessages({
	versionIsolationTitle: {
		id: 'app.settings.resource-management.version-isolation.title',
		defaultMessage: 'Version Isolation',
	},
	versionIsolationDescription: {
		id: 'app.settings.resource-management.version-isolation.description',
		defaultMessage:
			'Isolate all your Minecraft versions so that they do not interact with each other. If you need mods or modpacks, be sure to enable this feature.',
	},
	downloadSourceTitle: {
		id: 'app.settings.resource-management.download-source.title',
		defaultMessage: 'Version download source',
	},
	downloadSourceDescription: {
		id: 'app.settings.resource-management.download-source.description',
		defaultMessage:
			'Where Minecraft game files are downloaded from. BMCLAPI is a faster mirror for mainland China.',
	},
	downloadSourceMojang: {
		id: 'app.settings.resource-management.download-source.mojang',
		defaultMessage: 'Use Mojang',
	},
	downloadSourceBmclapi: {
		id: 'app.settings.resource-management.download-source.bmclapi',
		defaultMessage: 'Use BMCLAPI',
	},
	appDirectoryTitle: {
		id: 'app.settings.resource-management.app-directory.title',
		defaultMessage: 'App directory',
	},
	appDirectoryDescription: {
		id: 'app.settings.resource-management.app-directory.description',
		defaultMessage:
			'Where Alemona Launcher stores instances and other files. Changes take effect after restarting the app.',
	},
	selectAppDirectory: {
		id: 'app.settings.resource-management.app-directory.select',
		defaultMessage: 'Select a new app directory',
	},
	browseAppDirectory: {
		id: 'app.settings.resource-management.app-directory.browse',
		defaultMessage: 'Browse for an app directory',
	},
	maximumConcurrentDownloadsTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.title',
		defaultMessage: 'Maximum concurrent downloads',
	},
	maximumConcurrentDownloadsDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-downloads.description',
		defaultMessage:
			'Number of files the app can download at once. Lower this if downloads are unreliable on your connection. Requires an app restart.',
	},
	maximumConcurrentWritesTitle: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.title',
		defaultMessage: 'Maximum concurrent writes',
	},
	maximumConcurrentWritesDescription: {
		id: 'app.settings.resource-management.maximum-concurrent-writes.description',
		defaultMessage:
			'Number of files the app can write to disk at once. Lower this if you frequently encounter I/O errors. Requires an app restart.',
	},
})

watch(
	settings,
	async () => {
		const setSettings = JSON.parse(JSON.stringify(settings.value))

		if (!setSettings.custom_dir) {
			setSettings.custom_dir = null
		}

		await set(setSettings)
	},
	{ deep: true },
)

async function findLauncherDir() {
	const newDir = await open({
		multiple: false,
		directory: true,
		title: formatMessage(messages.selectAppDirectory),
	})

	if (newDir) {
		settings.value.custom_dir = newDir
	}
}
</script>

<template>
	<div class="flex flex-col gap-6">
		<ContentStorageSettings />
		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.versionIsolationTitle) }}
			</h2>
			<Combobox
				id="version-isolation"
				v-model="versionIsolation"
				:options="versionIsolationOptions"
				:aria-label="formatMessage(messages.versionIsolationTitle)"
				class="w-full"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.versionIsolationDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.downloadSourceTitle) }}
			</h2>
			<Combobox
				id="version-download-source"
				:model-value="downloadSource"
				:options="downloadSourceOptions"
				:aria-label="formatMessage(messages.downloadSourceTitle)"
				class="w-full"
				@update:model-value="setDownloadSource"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.downloadSourceDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.appDirectoryTitle) }}
			</h2>
			<Input
				id="appDir"
				v-model="settings.custom_dir"
				:icon="BoxIcon"
				type="text"
				wrapper-class="w-full"
			>
				<template #right>
					<IconButton
						v-tooltip="formatMessage(messages.browseAppDirectory)"
						:label="formatMessage(messages.browseAppDirectory)"
						class="ml-1.5"
						@click="findLauncherDir"
					>
						<FolderSearchIcon aria-hidden="true" />
					</IconButton>
				</template>
			</Input>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.appDirectoryDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="m-0 text-lg font-semibold text-contrast mt-4">
				{{ formatMessage(messages.maximumConcurrentDownloadsTitle) }}
			</h2>
			<Slider
				id="max-downloads"
				v-model="settings.max_concurrent_downloads"
				:min="1"
				:max="10"
				:step="1"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.maximumConcurrentDownloadsDescription) }}
			</p>
		</div>

		<div class="flex flex-col gap-2.5">
			<h2 class="mt-0 m-0 text-lg font-semibold text-contrast">
				{{ formatMessage(messages.maximumConcurrentWritesTitle) }}
			</h2>
			<Slider
				id="max-writes"
				v-model="settings.max_concurrent_writes"
				:min="1"
				:max="50"
				:step="1"
			/>
			<p class="m-0 leading-tight text-secondary">
				{{ formatMessage(messages.maximumConcurrentWritesDescription) }}
			</p>
		</div>
	</div>
</template>
