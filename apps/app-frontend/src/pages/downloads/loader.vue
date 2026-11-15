<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import { defineMessages, Input, useVIntl } from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import McVersionIcon from '@/components/ui/mc-version-icon.vue'
import {
	fetchLoaderVersions,
	type LoaderVersion,
	type McVersionKind,
} from '@/composables/mc-versions'
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

const props = defineProps<{ version: string; loader: string }>()
const router = useRouter()
const { formatMessage } = useVIntl()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	title: { id: 'app.downloads.loader.title', defaultMessage: '{loader} versions' },
	subtitle: {
		id: 'app.downloads.loader.subtitle',
		defaultMessage: 'Minecraft {version} · {count} versions',
	},
	search: {
		id: 'app.downloads.loader.search',
		defaultMessage: 'Search {loader} versions...',
	},
	empty: {
		id: 'app.downloads.loader.empty',
		defaultMessage: 'No {loader} versions are available for Minecraft {version}.',
	},
	loading: { id: 'app.downloads.loader.loading', defaultMessage: 'Loading versions...' },
	stable: { id: 'app.downloads.loader.stable', defaultMessage: 'Stable' },
	forgeCompat: {
		id: 'app.downloads.loader.forge-compat',
		defaultMessage: 'Forge {version}',
	},
	optifineVanillaOnly: {
		id: 'app.downloads.loader.optifine-vanilla-only',
		defaultMessage: 'Optimized only for vanilla, cannot be combined with Forge',
	},
	liteloaderNote: {
		id: 'app.downloads.loader.liteloader-note',
		defaultMessage: 'Installs LiteLoader into a new 1.12.2 instance',
	},
	forgeMatchTitle: {
		id: 'app.downloads.loader.forge-match.title',
		defaultMessage: 'Works with the selected OptiFine',
	},
	forgeMatchHint: {
		id: 'app.downloads.loader.forge-match.hint',
		defaultMessage: 'Recommended pairing',
	},
	showOther: {
		id: 'app.downloads.loader.show-other',
		defaultMessage: 'Show {count} other Forge versions',
	},
	hideOther: {
		id: 'app.downloads.loader.hide-other',
		defaultMessage: 'Hide {count} other Forge versions',
	},
})

const LOADER_META: Record<string, { label: string; kind: McVersionKind }> = {
	fabric: { label: 'Fabric', kind: 'fabric' },
	forge: { label: 'Forge', kind: 'forge' },
	neoforge: { label: 'NeoForge', kind: 'neoforge' },
	quilt: { label: 'Quilt', kind: 'quilt' },
	optifine: { label: 'OptiFine', kind: 'optifine' },
	liteloader: { label: 'LiteLoader', kind: 'liteloader' },
}

/** OptiFine rows are labelled with the build channel and Forge pairing. */
function rowTitle(entry: LoaderVersion) {
	if (entry.optifine) return `OptiFine ${entry.optifine.type} ${entry.optifine.patch}`
	return entry.id
}

/**
 * OptiFine is installed as a Forge mod, so builds without a Forge pairing can
 * only be used on vanilla and are not selectable here.
 */
function isSelectable(_entry: LoaderVersion) {
	return true
}

function rowDisabledReason(_entry: LoaderVersion) {
	return undefined
}

function rowHint(entry: LoaderVersion) {
	if (entry.liteloader) return formatMessage(messages.liteloaderNote)
	const forge = entry.optifine?.forge?.trim()
	if (forge && !/n\/a/i.test(forge)) return forge
	if (entry.optifine) return formatMessage(messages.optifineVanillaOnly)
	return null
}

const versions = ref<LoaderVersion[]>([])
const loading = ref(true)
const query = ref('')
const showOtherVersions = ref(false)

const route = useRoute()

/**
 * When Forge is stacked on top of an OptiFine build, the matching Forge version
 * is the only one that works with it; everything else gets folded away but can
 * still be picked.
 */
const forgeMatch = computed(() => {
	const match = route.query.forgeMatch
	return typeof match === 'string' && match ? match : null
})

const meta = computed(() => LOADER_META[props.loader] ?? { label: props.loader, kind: 'release' })

const matched = computed(() => {
	if (!forgeMatch.value) return versions.value
	return versions.value.filter((entry) => entry.id === forgeMatch.value)
})

const unmatched = computed(() => {
	if (!forgeMatch.value) return []
	return versions.value.filter((entry) => entry.id !== forgeMatch.value)
})

const filtered = computed(() => {
	const term = query.value.trim().toLowerCase()
	if (!term) return versions.value
	return versions.value.filter((entry) => entry.id.toLowerCase().includes(term))
})

const matchedFiltered = computed(() =>
	filtered.value.filter((entry) => matched.value.includes(entry)),
)
const unmatchedFiltered = computed(() =>
	unmatched.value.filter((entry) => filtered.value.includes(entry)),
)

/** Selecting a version returns to the version page with the choice in the query. */
function choose(version: LoaderVersion, stacked = false) {
	if (stacked) {
		router.replace({
			path: `/browse/instance/${encodeURIComponent(props.version)}`,
			query: { loader: 'optifine', loaderVersion: route.query.optifineVersion, forge: version.id },
		})
		return
	}
	router.replace({
		path: `/browse/instance/${encodeURIComponent(props.version)}`,
		query: { loader: props.loader, loaderVersion: version.id },
	})
}

onMounted(async () => {
	breadcrumbManager.push({
		slot: 'browse-instance-loader',
		id: () => props.loader,
		label: () => formatMessage(messages.title, { loader: meta.value.label }),
		to: () => `/browse/instance/${encodeURIComponent(props.version)}`,
	})
	try {
		versions.value = await fetchLoaderVersions(props.loader, props.version)
	} finally {
		loading.value = false
	}
})
</script>

<template>
	<div class="flex w-full flex-col gap-4 p-4 sm:gap-5 sm:p-6">
		<div class="flex items-center gap-3 sm:gap-4">
			<McVersionIcon :kind="meta.kind" :size="48" />
			<div class="flex min-w-0 flex-col gap-0.5">
				<h1 class="m-0 truncate text-xl font-bold text-contrast sm:text-2xl">
					{{ formatMessage(messages.title, { loader: meta.label }) }}
				</h1>
				<span class="truncate text-sm text-secondary">
					{{
						formatMessage(messages.subtitle, {
							version: props.version,
							count: versions.length,
						})
					}}
				</span>
			</div>
		</div>

		<Input
			v-model="query"
			:icon="SearchIcon"
			type="text"
			autocomplete="off"
			size="large"
			clearable
			wrapper-class="w-full"
			:placeholder="formatMessage(messages.search, { loader: meta.label })"
		/>

		<p v-if="loading" class="m-0 py-8 text-center text-sm text-secondary">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="filtered.length === 0" class="m-0 py-8 text-center text-sm text-secondary">
			{{ formatMessage(messages.empty, { loader: meta.label, version: props.version }) }}
		</p>

		<template v-if="!loading && filtered.length > 0">
			<div
				v-if="forgeMatch"
				class="flex flex-col gap-2 rounded-2xl border border-solid border-brand bg-brand-highlight/40 p-3"
			>
				<span class="text-xs font-semibold uppercase tracking-wide text-brand">
					{{ formatMessage(messages.forgeMatchTitle) }}
				</span>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(clamp(160px,10vw,220px),1fr))] gap-2">
					<button
						v-for="entry in matchedFiltered"
						:key="entry.id"
						type="button"
						class="flex w-full cursor-pointer touch-manipulation select-none flex-col items-start gap-0.5 rounded-2xl border border-solid border-brand/40 bg-surface-3 p-3 text-left transition-[background-color,border-color,box-shadow,filter,transform] duration-150 ease-out hover:bg-surface-4 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow active:scale-[0.94]"
						@click="choose(entry, true)"
					>
						<span class="w-full truncate text-sm font-semibold text-contrast">
							{{ rowTitle(entry) }}
						</span>
						<span class="w-full truncate text-xs font-medium text-secondary">
							{{ formatMessage(messages.forgeMatchHint) }}
						</span>
					</button>
				</div>
			</div>

			<button
				v-if="forgeMatch && unmatchedFiltered.length > 0"
				type="button"
				class="flex w-fit cursor-pointer items-center gap-2 rounded-xl px-2 py-1 text-sm font-medium text-secondary transition-colors hover:text-contrast"
				@click="showOtherVersions = !showOtherVersions"
			>
				{{
					formatMessage(showOtherVersions ? messages.hideOther : messages.showOther, {
						count: unmatchedFiltered.length,
					})
				}}
			</button>

			<div
				v-if="!forgeMatch || showOtherVersions"
				class="grid grid-cols-[repeat(auto-fill,minmax(clamp(160px,10vw,220px),1fr))] gap-2"
			>
				<button
					v-for="entry in forgeMatch ? unmatchedFiltered : filtered"
					:key="entry.id"
					type="button"
					class="flex w-full cursor-pointer touch-manipulation select-none flex-col items-start gap-0.5 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-3 text-left transition-[background-color,border-color,box-shadow,filter,transform] duration-150 ease-out enabled:hover:bg-surface-4 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow enabled:active:scale-[0.94] disabled:cursor-not-allowed disabled:opacity-50"
					:title="rowDisabledReason(entry)"
					:disabled="!isSelectable(entry)"
					@click="choose(entry, Boolean(forgeMatch))"
				>
					<span class="w-full truncate text-sm font-semibold text-contrast">
						{{ rowTitle(entry) }}
					</span>
					<span v-if="rowHint(entry)" class="w-full truncate text-xs font-medium text-secondary">
						{{ rowHint(entry) }}
					</span>
					<span
						v-else-if="entry.stable && !entry.optifine"
						class="text-xs font-medium text-secondary"
					>
						{{ formatMessage(messages.stable) }}
					</span>
				</button>
			</div>
		</template>
	</div>
</template>
