<script setup lang="ts">
import { SearchIcon } from '@modrinth/assets'
import {
	Combobox,
	type ComboboxOption,
	defineMessages,
	FilterPills,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { computed, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import McVersionIcon from '@/components/ui/mc-version-icon.vue'
import { fetchMcVersions, type McVersion, type McVersionFilter } from '@/composables/mc-versions'
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

const { formatMessage } = useVIntl()
const router = useRouter()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	tabInstances: { id: 'app.downloads.tabs.instances', defaultMessage: 'Instances' },
	tabModpacks: { id: 'app.downloads.tabs.modpacks', defaultMessage: 'Modpacks' },
	tabMods: { id: 'app.downloads.tabs.mods', defaultMessage: 'Mods' },
	tabResourcePacks: { id: 'app.downloads.tabs.resource-packs', defaultMessage: 'Resource packs' },
	tabDataPacks: { id: 'app.downloads.tabs.data-packs', defaultMessage: 'Data packs' },
	tabShaders: { id: 'app.downloads.tabs.shaders', defaultMessage: 'Shaders' },
	tabServers: { id: 'app.downloads.tabs.servers', defaultMessage: 'Servers' },
	title: { id: 'app.downloads.instances.title', defaultMessage: 'Instances' },
	search: {
		id: 'app.downloads.instances.search',
		defaultMessage: 'Search Minecraft versions...',
	},
	sortNewest: { id: 'app.downloads.instances.sort.newest', defaultMessage: 'Newest first' },
	sortOldest: { id: 'app.downloads.instances.sort.oldest', defaultMessage: 'Oldest first' },
	all: { id: 'app.downloads.filters.all', defaultMessage: 'All' },
	release: { id: 'app.downloads.filters.release', defaultMessage: 'Releases' },
	snapshot: { id: 'app.downloads.filters.snapshot', defaultMessage: 'Snapshots' },
	old: { id: 'app.downloads.filters.old', defaultMessage: 'Old versions' },
	fools: { id: 'app.downloads.filters.fools', defaultMessage: 'April Fools' },
	loading: { id: 'app.downloads.instances.loading', defaultMessage: 'Loading versions...' },
	empty: {
		id: 'app.downloads.instances.empty',
		defaultMessage: 'No versions match your filters.',
	},
})

const sortOptions = computed<ComboboxOption<'newest' | 'oldest'>[]>(() => [
	{ value: 'newest', label: formatMessage(messages.sortNewest) },
	{ value: 'oldest', label: formatMessage(messages.sortOldest) },
])

const filterOptions = computed(() => [
	{ id: 'release', label: formatMessage(messages.release) },
	{ id: 'snapshot', label: formatMessage(messages.snapshot) },
	{ id: 'old', label: formatMessage(messages.old) },
	{ id: 'fools', label: formatMessage(messages.fools) },
])

const FILTER_LABELS: Record<McVersionFilter, () => string> = {
	release: () => formatMessage(messages.release),
	snapshot: () => formatMessage(messages.snapshot),
	old: () => formatMessage(messages.old),
	fools: () => formatMessage(messages.fools),
}

const versions = ref<McVersion[]>([])
const loading = ref(true)
const query = ref('')
const sort = ref<'newest' | 'oldest'>('newest')
const filters = ref<string[]>(['release'])

const filtered = computed(() => {
	const term = query.value.trim().toLowerCase()
	const selected = new Set(filters.value)
	const list = versions.value.filter((version) => {
		if (selected.size > 0 && !selected.has(version.kind)) return false
		if (!term) return true
		return version.id.toLowerCase().includes(term)
	})
	return sort.value === 'newest' ? list : [...list].reverse()
})

function formatDate(version: McVersion) {
	const stamp = new Date(version.released).getTime()
	return Number.isNaN(stamp) ? '' : new Date(stamp).toLocaleDateString()
}

function openVersion(version: McVersion) {
	router.push(`/browse/instance/${encodeURIComponent(version.id)}`)
}

onMounted(async () => {
	breadcrumbManager.reset({
		slot: 'browse',
		id: () => 'download-instances',
		label: () => formatMessage(messages.title),
	})
	try {
		versions.value = await fetchMcVersions()
	} finally {
		loading.value = false
	}
})
</script>

<template>
	<div class="flex w-full flex-col gap-4 px-4 pb-6 pt-2 sm:gap-5 sm:px-6">
		<div class="flex flex-col gap-3">
			<Input
				v-model="query"
				:icon="SearchIcon"
				type="text"
				autocomplete="off"
				size="large"
				clearable
				wrapper-class="w-full"
				:placeholder="formatMessage(messages.search)"
			/>
			<div class="flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center">
				<Combobox v-model="sort" :options="sortOptions" class="!w-full sm:!w-[12rem]" />
				<FilterPills v-model="filters" :options="filterOptions">
					<template #all>{{ formatMessage(messages.all) }}</template>
				</FilterPills>
			</div>
		</div>

		<p v-if="loading" class="m-0 py-8 text-center text-sm text-secondary">
			{{ formatMessage(messages.loading) }}
		</p>
		<p v-else-if="filtered.length === 0" class="m-0 py-8 text-center text-sm text-secondary">
			{{ formatMessage(messages.empty) }}
		</p>

		<div
			v-else
			class="grid grid-cols-[repeat(auto-fill,minmax(clamp(160px,12vw,240px),1fr))] gap-2 sm:gap-3"
		>
			<button
				v-for="version in filtered"
				:key="`${version.id}:${version.type}`"
				type="button"
				class="group flex cursor-pointer touch-manipulation select-none items-center gap-2.5 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-2.5 text-left transition-[background-color,border-color,box-shadow,filter,transform] duration-150 ease-out hover:bg-surface-4 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow active:scale-[0.94] sm:gap-3 sm:p-3"
				@click="openVersion(version)"
			>
				<McVersionIcon :kind="version.kind" :size="40" />
				<span class="flex min-w-0 flex-col">
					<span class="truncate text-sm font-semibold text-contrast">{{ version.id }}</span>
					<span class="truncate text-xs text-secondary">
						{{ FILTER_LABELS[version.kind]() }} · {{ formatDate(version) }}
					</span>
				</span>
			</button>
		</div>
	</div>
</template>
