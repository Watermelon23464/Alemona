<script setup lang="ts">
import { convertFileSrc } from '@tauri-apps/api/core'
import { PlayIcon, PlusIcon } from '@modrinth/assets'
import {
	Button,
	defineMessages,
	injectNotificationManager,
	TeleportOverflowMenu,
	useVIntl,
} from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, inject, onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'

import { remove } from '@/helpers/instance'
import { get_game_versions } from '@/helpers/tags'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

defineOptions({ name: 'InstancesPage' })

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()
const router = useRouter()
const showCreationModal = inject<() => void>('showCreationModal')

const messages = defineMessages({
	title: { id: 'app.instances.title', defaultMessage: 'Instances' },
	newInstance: { id: 'app.instances.new', defaultMessage: 'New instance' },
	empty: { id: 'app.instances.empty', defaultMessage: 'No instances yet' },
	play: { id: 'app.instances.play', defaultMessage: 'Play' },
	actions: { id: 'app.instances.actions', defaultMessage: 'Instance actions' },
	remove: { id: 'app.instances.remove', defaultMessage: 'Delete instance' },
	neverPlayed: { id: 'app.instances.never-played', defaultMessage: 'Never played' },
})

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => (instancesQuery.data.value ?? []) as any[])

const versionDates = ref<Record<string, string>>({})

onMounted(async () => {
	try {
		const versions = await get_game_versions()
		const map: Record<string, string> = {}
		for (const version of versions ?? []) {
			if (version?.id && version?.date_published) {
				map[version.id] = new Date(version.date_published).toLocaleDateString()
			}
		}
		versionDates.value = map
	} catch (error) {
		console.warn('Could not load game version release dates', error)
	}
})

function instanceIcon(instance: any) {
	if (instance?.icon_path) {
		return instance.icon_path.startsWith('http')
			? instance.icon_path
			: convertFileSrc(instance.icon_path)
	}
	return 'https://launcher-files.modrinth.com/assets/steve_head.png'
}

function releasedAt(instance: any) {
	return versionDates.value[instance?.game_version] ?? ''
}

function lastPlayedAt(instance: any) {
	const stamp = instance?.last_played ?? instance?.modified
	if (!stamp) return formatMessage(messages.neverPlayed)
	return new Date(Number(stamp) * 1000).toLocaleDateString()
}

function loaderLabel(instance: any) {
	const loader = instance?.loader ?? 'vanilla'
	const version = instance?.loader_version
	return version ? `${loader} ${version}` : loader
}

function openInstance(instance: any) {
	router.push(`/instance/${instance.id}`)
}

function play(instance: any) {
	router.push(`/instance/${instance.id}`)
}

async function deleteInstance(instance: any) {
	await remove(instance.id).catch(handleError)
	await instancesQuery.refetch()
}

function actions(instance: any) {
	return [
		{
			id: 'delete',
			label: formatMessage(messages.remove),
			color: 'red',
			action: () => deleteInstance(instance),
		},
	]
}

await instancesQuery.suspense().catch(handleError)
</script>

<template>
	<div class="flex flex-col gap-3 p-6">
		<div class="flex items-center justify-between">
			<h2 class="m-0 text-xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h2>
			<Button type="colored" color="brand" @click="showCreationModal?.()">
				<PlusIcon />
				{{ formatMessage(messages.newInstance) }}
			</Button>
		</div>

		<div v-if="instances.length === 0" class="rounded-xl bg-surface-3 p-6 text-secondary">
			{{ formatMessage(messages.empty) }}
		</div>

		<div v-else class="flex flex-col gap-2">
			<div
				v-for="instance in instances"
				:key="instance.id"
				class="flex cursor-pointer items-center gap-4 rounded-xl bg-surface-3 px-4 py-3 transition-[filter] hover:brightness-110"
				@click="openInstance(instance)"
			>
				<img :src="instanceIcon(instance)" alt="" class="size-11 shrink-0 rounded-lg object-cover" />
				<div class="flex min-w-0 flex-1 flex-col">
					<span class="truncate font-semibold text-contrast">{{ instance.name }}</span>
					<span class="truncate text-xs text-secondary">
						<template v-if="releasedAt(instance)">{{ releasedAt(instance) }} · </template>
						{{ lastPlayedAt(instance) }} · {{ loaderLabel(instance) }}
					</span>
				</div>
				<Button type="colored" color="brand" @click.stop="play(instance)">
					<PlayIcon />
					{{ formatMessage(messages.play) }}
				</Button>
				<TeleportOverflowMenu
					type="quiet"
					size="md"
					:label="formatMessage(messages.actions)"
					:options="actions(instance)"
					placement="right-start"
					:distance="4"
				>
					<span class="text-lg leading-none">⋯</span>
				</TeleportOverflowMenu>
			</div>
		</div>
	</div>
</template>
