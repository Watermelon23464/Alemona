<script setup lang="ts">
import { PencilIcon, RotateClockwiseIcon, UploadIcon, XIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { computed, onMounted, ref, useTemplateRef, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import IconEditorModal from '@/components/ui/instance_settings/icon-editor-modal/index.vue'
import McVersionIcon from '@/components/ui/mc-version-icon.vue'
import { symbolBytes, versionIconConfig, versionIconSymbol } from '@/composables/mc-version-icons'
import {
	createMinecraftInstance,
	fetchLoaderVersions,
	fetchMcVersions,
	installLiteLoader,
	installOptifine,
	installOptifineStandalone,
	type LiteLoaderBuild,
	type LoaderVersion,
	type McVersionKind,
	optifineHasForge,
	type OptifineVersion,
	resolveForgeVersion,
} from '@/composables/mc-versions'
import { cache_generated_icon, edit_icon, list as listInstances } from '@/helpers/instance'
import type { InstanceIconConfig } from '@/helpers/types'
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

const props = defineProps<{ version: string }>()
const router = useRouter()
const route = useRoute()
const { handleError } = injectNotificationManager()
const { formatMessage } = useVIntl()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	typeRelease: { id: 'app.downloads.filters.release', defaultMessage: 'Releases' },
	typeSnapshot: { id: 'app.downloads.filters.snapshot', defaultMessage: 'Snapshots' },
	typeOld: { id: 'app.downloads.filters.old', defaultMessage: 'Old versions' },
	typeFools: { id: 'app.downloads.filters.fools', defaultMessage: 'April Fools' },
	upload: { id: 'app.downloads.version.upload', defaultMessage: 'Upload' },
	randomize: { id: 'app.downloads.version.randomize', defaultMessage: 'Randomize' },
	custom: { id: 'app.downloads.version.custom', defaultMessage: 'Custom icon' },
	name: { id: 'app.downloads.version.name', defaultMessage: 'Instance name' },
	start: { id: 'app.downloads.version.start', defaultMessage: 'Start download' },
	starting: { id: 'app.downloads.version.starting', defaultMessage: 'Creating...' },
	loaders: { id: 'app.downloads.version.loaders', defaultMessage: 'Loaders' },
	loadersHint: {
		id: 'app.downloads.version.loaders-hint',
		defaultMessage: 'Choose a loader version to set up a modded instance.',
	},
	loadingLoaders: {
		id: 'app.downloads.version.loading-loaders',
		defaultMessage: 'Looking for loader versions...',
	},
	available: { id: 'app.downloads.version.available', defaultMessage: '{count} versions' },
	unavailable: {
		id: 'app.downloads.version.unavailable',
		defaultMessage: 'No versions for this Minecraft version',
	},
	useVanilla: { id: 'app.downloads.version.use-vanilla', defaultMessage: 'Use Vanilla' },
	removeForge: {
		id: 'app.downloads.version.remove-forge',
		defaultMessage: 'Remove Forge',
	},
	optifineNeedsForge: {
		id: 'app.downloads.version.optifine-needs-forge',
		defaultMessage:
			'This OptiFine build only works on vanilla, so no Forge can be added on top of it.',
	},
	optifineNeedsLegacy: {
		id: 'app.downloads.version.optifine-needs-legacy',
		defaultMessage:
			'Standalone OptiFine only works for Minecraft 1.12.2 and older. Pick a build with a Forge version.',
	},
	liteloaderMissing: {
		id: 'app.downloads.version.liteloader-missing',
		defaultMessage: 'This LiteLoader build could not be resolved.',
	},
})

const LOADERS: { id: string; label: string; kind: McVersionKind }[] = [
	{ id: 'fabric', label: 'Fabric', kind: 'fabric' },
	{ id: 'forge', label: 'Forge', kind: 'forge' },
	{ id: 'neoforge', label: 'NeoForge', kind: 'neoforge' },
	{ id: 'quilt', label: 'Quilt', kind: 'quilt' },
	{ id: 'optifine', label: 'OptiFine', kind: 'optifine' },
	{ id: 'liteloader', label: 'LiteLoader', kind: 'liteloader' },
]

const versionKind = ref<McVersionKind>('release')
const versionDate = ref('')
const loaderVersions = ref<Record<string, LoaderVersion[]>>({})
const loaderCounts = computed<Record<string, number>>(() =>
	Object.fromEntries(
		Object.entries(loaderVersions.value).map(([loader, versions]) => [loader, versions.length]),
	),
)
const loadingLoaders = ref(true)
const installing = ref(false)

const generatedIcon = ref<{ path: string; config: InstanceIconConfig } | null>(null)
const uploadedIcon = ref<string | null>(null)
const iconPreview = ref<string | null>(null)
const iconEditor = useTemplateRef<InstanceType<typeof IconEditorModal>>('iconEditor')

const name = ref(props.version)
const existingNames = ref<Set<string>>(new Set())

/** Names that are already in the library, keyed lowercased for comparison. */
async function refreshExistingNames() {
	try {
		const instances = await listInstances()
		existingNames.value = new Set(instances.map((instance) => instance.name.toLowerCase()))
	} catch {
		existingNames.value = new Set()
	}
}

/** Appends the re-download count when the name is already taken. */
function uniqueName(candidate: string) {
	const base = candidate.trim()
	if (!base) return base
	if (!existingNames.value.has(base.toLowerCase())) return base
	let count = 1
	let renamed = `${base} (${count})`
	while (existingNames.value.has(renamed.toLowerCase())) {
		count += 1
		renamed = `${base} (${count})`
	}
	return renamed
}

const selectedLoader = computed(() => {
	const loader = route.query.loader
	return typeof loader === 'string' && LOADERS.some((entry) => entry.id === loader) ? loader : null
})
const selectedLoaderVersion = computed(() => {
	const version = route.query.loaderVersion
	return typeof version === 'string' && version ? version : null
})
const selectedLoaderLabel = computed(
	() => LOADERS.find((entry) => entry.id === selectedLoader.value)?.label ?? '',
)

/** The OptiFine build picked on the loader page, if any. */
const selectedOptifine = computed<OptifineVersion | null>(() => {
	if (selectedLoader.value !== 'optifine' || !selectedLoaderVersion.value) return null
	const entry = (loaderVersions.value.optifine ?? []).find(
		(candidate) => candidate.id === selectedLoaderVersion.value,
	)
	return entry?.optifine ?? null
})
/** Forge stacked on top of OptiFine, picked by hand on the Forge page. */
const stackedForge = computed<string | null>(() => {
	const forge = route.query.forge
	return typeof forge === 'string' && forge ? forge : null
})

/** Forge version an OptiFine build is meant to run with, used as the fold hint. */
const matchingForge = computed<string | null>(() => {
	const optifine = selectedOptifine.value
	if (!optifine) return null
	return resolveForgeVersion(optifine, loaderVersions.value.forge ?? [])
})

/** LiteLoader build picked on the loader page, if any. */
const selectedLiteLoader = computed<LiteLoaderBuild | null>(() => {
	if (selectedLoader.value !== 'liteloader' || !selectedLoaderVersion.value) return null
	const entry = (loaderVersions.value.liteloader ?? []).find(
		(candidate) => candidate.id === selectedLoaderVersion.value,
	)
	return entry?.liteloader ?? null
})

/** OptiFine standalone needs launchwrapper, which only old versions ship with. */
const legacyVersion = computed(() => {
	const match = props.version.match(/^1\.(\d+)/)
	return Boolean(match) && Number(match?.[1]) <= 12
})

const selectionLabel = computed(() => {
	if (selectedLoader.value === 'optifine') {
		const optifine = selectedOptifine.value
		if (!optifine) return 'OptiFine'
		return `OptiFine ${optifine.type} ${optifine.patch}`
	}
	if (selectedLoader.value === 'liteloader') {
		return `LiteLoader ${selectedLoaderVersion.value ?? ''}`.trim()
	}
	const version = selectedLoaderVersion.value
	return `${selectedLoaderLabel.value}${version ? ` ${version}` : ''}`.trim()
})

/** OptiFine alone installs as a tweaker; with Forge stacked it installs as a mod. */
const optifineUsable = computed(() => {
	if (selectedLoader.value !== 'optifine') return true
	const optifine = selectedOptifine.value
	if (!optifine) return false
	if (stackedForge.value) return true
	return legacyVersion.value
})

/** Whether another loader can still be stacked onto the OptiFine build. */
const canStackForge = computed(
	() => selectedOptifine.value !== null && optifineHasForge(selectedOptifine.value),
)
const liteloaderUsable = computed(
	() => selectedLoader.value !== 'liteloader' || Boolean(selectedLiteLoader.value),
)
const canStart = computed(() => !installing.value && optifineUsable.value && liteloaderUsable.value)

watch(
	selectedLoaderVersion,
	(next, previous) => {
		const withoutCount = name.value.replace(/\s\(\d+\)$/, '')
		const previousSuffix = previous ? `-${previous}` : ''
		const base =
			previousSuffix && withoutCount.endsWith(previousSuffix)
				? withoutCount.slice(0, -previousSuffix.length)
				: withoutCount
		name.value = uniqueName(next ? `${base}-${next}` : base)
	},
	{ immediate: true },
)

function typeLabel() {
	if (versionKind.value === 'snapshot') return formatMessage(messages.typeSnapshot)
	if (versionKind.value === 'old') return formatMessage(messages.typeOld)
	if (versionKind.value === 'fools') return formatMessage(messages.typeFools)
	return formatMessage(messages.typeRelease)
}

function loaderKind(id: string): McVersionKind {
	return LOADERS.find((entry) => entry.id === id)?.kind ?? 'release'
}

async function openLoader(loader: string) {
	if (!(loaderCounts.value[loader] > 0)) return
	if (loader === 'forge' && selectedOptifine.value && !canStackForge.value) return
	const stacking = loader === 'forge' && selectedOptifine.value !== null
	const query = stacking
		? {
				forgeMatch: matchingForge.value ?? undefined,
				optifineVersion: selectedLoaderVersion.value ?? undefined,
			}
		: undefined
	await router.push({
		path: `/browse/instance/${encodeURIComponent(props.version)}/${loader}`,
		query,
	})
}

/** Drops the stacked Forge while keeping OptiFine selected. */
function clearStackedForge() {
	router.replace({
		path: `/browse/instance/${encodeURIComponent(props.version)}`,
		query: { loader: 'optifine', loaderVersion: selectedLoaderVersion.value ?? undefined },
	})
}

function clearLoader() {
	router.replace({ path: `/browse/instance/${encodeURIComponent(props.version)}` })
}

async function uploadIcon() {
	try {
		const selected = await open({
			multiple: false,
			filters: [{ name: 'Image', extensions: ['png', 'jpg', 'jpeg', 'webp'] }],
		})
		if (typeof selected !== 'string') return
		uploadedIcon.value = selected
		generatedIcon.value = null
		iconPreview.value = convertFileSrc(selected)
	} catch (error) {
		handleError(error as Error)
	}
}

async function randomizeIcon() {
	try {
		const generated = await iconEditor.value?.randomizeAndSave()
		if (!generated) return
		applyGeneratedIcon(generated.iconPath, generated.config)
	} catch (error) {
		handleError(error as Error)
	}
}

function applyGeneratedIcon(iconPath: string, config: InstanceIconConfig) {
	generatedIcon.value = { path: iconPath, config }
	uploadedIcon.value = null
	iconPreview.value = convertFileSrc(iconPath)
}

function customizeIcon() {
	iconEditor.value?.show()
}

function onIconSaved(iconPath: string, config: InstanceIconConfig) {
	applyGeneratedIcon(iconPath, config)
}

/** Caches the version icon as a real instance icon when the user picked none. */
async function ensureIcon(): Promise<{ path: string | null; config: InstanceIconConfig | null }> {
	if (uploadedIcon.value) return { path: null, config: null }
	if (generatedIcon.value) {
		return { path: generatedIcon.value.path, config: generatedIcon.value.config }
	}
	const config = versionIconConfig(versionKind.value)
	const path = await cache_generated_icon(
		config,
		await symbolBytes(versionIconSymbol(versionKind.value)),
	)
	applyGeneratedIcon(path, config)
	return { path, config }
}

const LITELOADER_TWEAKER = 'com.mumfrey.liteloader.launch.LiteLoaderTweaker'

/**
 * OptiFine build from the selected id (`HD_U_I6`), so installing never depends
 * on the loader list having finished loading.
 */
function optifineBuild(): OptifineVersion {
	const id = selectedLoaderVersion.value ?? ''
	const separator = id.lastIndexOf('_')
	const fromId =
		separator > 0
			? { type: id.slice(0, separator), patch: id.slice(separator + 1) }
			: { type: 'HD_U', patch: id }
	const known = selectedOptifine.value
	return {
		...(known ?? { filename: '' }),
		type: known?.type ?? fromId.type,
		patch: known?.patch ?? fromId.patch,
	}
}

async function startDownload() {
	if (!canStart.value) return
	installing.value = true
	try {
		await refreshExistingNames()
		const instanceName = uniqueName(name.value.trim() || props.version)
		name.value = instanceName
		const icon = await ensureIcon()

		if (selectedLoader.value === 'optifine') {
			const build = optifineBuild()
			const forgeVersion = stackedForge.value
			const instanceId = await createMinecraftInstance({
				name: instanceName,
				gameVersion: props.version,
				loader: forgeVersion ? 'forge' : 'vanilla',
				loaderVersion: forgeVersion,
				iconPath: icon.path,
				iconConfig: icon.config,
			})
			if (!instanceId) throw new Error('instance was not created')
			if (uploadedIcon.value) {
				await edit_icon(instanceId, uploadedIcon.value).catch(handleError)
			}
			if (forgeVersion) {
				await installOptifine(instanceId, props.version, build.type, build.patch)
			} else {
				await installOptifineStandalone(instanceId, props.version, build.type, build.patch)
			}
			await router.push(`/instance/${encodeURIComponent(instanceId)}`)
			return
		}

		if (selectedLoader.value === 'liteloader') {
			const version = selectedLoaderVersion.value
			if (!version) throw new Error('LiteLoader version missing from the selection')
			const instanceId = await createMinecraftInstance({
				name: instanceName,
				gameVersion: props.version,
				loader: 'vanilla',
				loaderVersion: null,
				iconPath: icon.path,
				iconConfig: icon.config,
			})
			if (!instanceId) throw new Error('instance was not created')
			if (uploadedIcon.value) {
				await edit_icon(instanceId, uploadedIcon.value).catch(handleError)
			}
			await installLiteLoader(instanceId, version, LITELOADER_TWEAKER)
			await router.push(`/instance/${encodeURIComponent(instanceId)}`)
			return
		}

		const instanceId = await createMinecraftInstance({
			name: instanceName,
			gameVersion: props.version,
			loader: selectedLoader.value ?? 'vanilla',
			loaderVersion: selectedLoader.value ? selectedLoaderVersion.value : null,
			iconPath: icon.path,
			iconConfig: icon.config,
		})
		if (instanceId && uploadedIcon.value) {
			await edit_icon(instanceId, uploadedIcon.value).catch(handleError)
		}
		if (instanceId) {
			await router.push(`/instance/${encodeURIComponent(instanceId)}`)
		}
	} catch (error) {
		handleError(error as Error)
	} finally {
		installing.value = false
	}
}

onMounted(async () => {
	breadcrumbManager.push({
		slot: 'browse-instance',
		id: () => props.version,
		label: () => `Minecraft ${props.version}`,
		to: () => '/browse/instance',
	})

	try {
		const versions = await fetchMcVersions()
		const match = versions.find((entry) => entry.id === props.version)
		if (match) {
			if (match.kind !== 'release') versionKind.value = match.kind as McVersionKind
			const stamp = new Date(match.released).getTime()
			versionDate.value = Number.isNaN(stamp) ? '' : new Date(stamp).toLocaleDateString()
		}
	} catch {
		// keep the default icon and empty date when the catalog cannot be read
	}

	await refreshExistingNames()
	name.value = uniqueName(name.value)

	try {
		const results = await Promise.all(
			LOADERS.map(
				async (loader) => [loader.id, await fetchLoaderVersions(loader.id, props.version)] as const,
			),
		)
		const versionsByLoader: Record<string, LoaderVersion[]> = {}
		for (const [loader, versions] of results) versionsByLoader[loader] = versions
		loaderVersions.value = versionsByLoader
	} finally {
		loadingLoaders.value = false
	}
})
</script>

<template>
	<div class="flex w-full flex-col gap-4 p-4 sm:gap-6 sm:p-6">
		<section
			class="flex max-w-[1200px] flex-wrap items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-4 sm:gap-5 sm:p-5"
		>
			<div class="flex items-start gap-3">
				<img
					v-if="iconPreview"
					:src="iconPreview"
					alt=""
					class="size-[88px] shrink-0 rounded-2xl object-cover"
				/>
				<McVersionIcon v-else :kind="versionKind" :size="88" />
				<div class="flex flex-col gap-2">
					<Button size="sm" @click="uploadIcon">
						<UploadIcon />
						{{ formatMessage(messages.upload) }}
					</Button>
					<Button size="sm" @click="randomizeIcon">
						<RotateClockwiseIcon />
						{{ formatMessage(messages.randomize) }}
					</Button>
					<Button size="sm" @click="customizeIcon">
						<PencilIcon />
						{{ formatMessage(messages.custom) }}
					</Button>
				</div>
			</div>

			<div class="flex min-w-[240px] flex-1 flex-col gap-3">
				<div class="flex flex-col gap-0.5">
					<h1 class="m-0 text-xl font-bold text-contrast sm:text-2xl">Minecraft {{ version }}</h1>
					<span class="text-sm text-secondary">
						{{ typeLabel() }}<template v-if="versionDate"> · {{ versionDate }}</template>
					</span>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<Input
						:model-value="name"
						type="text"
						:placeholder="formatMessage(messages.name)"
						wrapper-class="min-w-[200px] max-w-[36rem] flex-1"
						@update:model-value="name = String($event ?? '')"
					/>
					<Button
						type="colored"
						color="brand"
						size="lg"
						:disabled="!canStart"
						@click="startDownload"
					>
						{{ formatMessage(installing ? messages.starting : messages.start) }}
					</Button>
				</div>

				<div v-if="selectedLoader" class="flex flex-col gap-2">
					<div class="flex flex-wrap items-center gap-2">
						<span
							class="inline-flex items-center gap-2 rounded-xl border border-solid border-brand bg-brand-highlight px-3 py-1.5 text-sm font-semibold text-brand"
						>
							{{ selectionLabel }}
						</span>
						<IconButton :label="formatMessage(messages.useVanilla)" size="sm" @click="clearLoader">
							<XIcon />
						</IconButton>
						<template v-if="stackedForge">
							<span class="text-sm font-semibold text-primary">+</span>
							<span
								class="inline-flex items-center gap-2 rounded-xl border border-solid border-brand bg-brand-highlight px-3 py-1.5 text-sm font-semibold text-brand"
							>
								Forge {{ stackedForge }}
							</span>
							<IconButton
								:label="formatMessage(messages.removeForge)"
								size="sm"
								@click="clearStackedForge"
							>
								<XIcon />
							</IconButton>
						</template>
					</div>
					<p v-if="!optifineUsable" class="m-0 text-xs font-medium leading-4 text-secondary">
						{{ formatMessage(messages.optifineNeedsLegacy) }}
					</p>
					<p
						v-if="selectedLoader === 'optifine' && !canStackForge"
						class="m-0 text-xs font-medium leading-4 text-secondary"
					>
						{{ formatMessage(messages.optifineNeedsForge) }}
					</p>
					<p v-if="!liteloaderUsable" class="m-0 text-xs font-medium leading-4 text-secondary">
						{{ formatMessage(messages.liteloaderMissing) }}
					</p>
				</div>
			</div>
		</section>

		<section class="flex flex-col gap-3">
			<div class="flex flex-col gap-0.5">
				<h2 class="m-0 text-lg font-semibold text-contrast">
					{{ formatMessage(messages.loaders) }}
				</h2>
				<p class="m-0 text-sm text-secondary">{{ formatMessage(messages.loadersHint) }}</p>
			</div>

			<p v-if="loadingLoaders" class="m-0 text-sm text-secondary">
				{{ formatMessage(messages.loadingLoaders) }}
			</p>

			<div
				v-else
				class="grid grid-cols-[repeat(auto-fill,minmax(clamp(190px,15vw,260px),1fr))] gap-2 sm:gap-3"
			>
				<button
					v-for="loader in LOADERS"
					:key="loader.id"
					type="button"
					class="flex cursor-pointer touch-manipulation select-none items-center gap-3 rounded-2xl border border-solid p-3 text-left transition-[background-color,border-color,box-shadow,filter,transform] duration-150 ease-out focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow enabled:active:scale-[0.94] disabled:cursor-default disabled:opacity-60"
					:class="
						selectedLoader === loader.id
							? 'border-brand bg-brand-highlight'
							: 'border-surface-5 bg-surface-3 enabled:hover:bg-surface-4'
					"
					:disabled="!loaderCounts[loader.id]"
					@click="openLoader(loader.id)"
				>
					<McVersionIcon :kind="loaderKind(loader.id)" :size="40" />
					<span class="flex min-w-0 flex-col">
						<span class="text-sm font-semibold text-contrast">{{ loader.label }}</span>
						<span class="truncate text-xs text-secondary">
							{{
								loaderCounts[loader.id]
									? formatMessage(messages.available, { count: loaderCounts[loader.id] })
									: formatMessage(messages.unavailable)
							}}
						</span>
					</span>
				</button>
			</div>
		</section>

		<IconEditorModal
			ref="iconEditor"
			:config="generatedIcon?.config ?? null"
			@saved="onIconSaved"
		/>
	</div>
</template>
