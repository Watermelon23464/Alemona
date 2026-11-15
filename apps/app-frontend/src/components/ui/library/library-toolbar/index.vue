<script setup lang="ts">
import { PlusIcon, SearchIcon, SquarePlusIcon } from '@modrinth/assets'
import { Button, defineMessages, Input, TagItem, useVIntl } from '@modrinth/ui'
import { computed, inject, type Ref, ref } from 'vue'

import FilterMenu from '@/components/ui/library/library-toolbar/filter-menu.vue'
import NewGroupModal from '@/components/ui/library/library-toolbar/new-group-modal.vue'
import SortMenu from '@/components/ui/library/library-toolbar/sort-menu.vue'
import { useLibrary } from '@/components/ui/library/use-library'

const { search, selectedLibraryInstances, openNewGroupModal, instanceGroups } = useLibrary()
const showCreationModal = inject<() => void>('showCreationModal')
const pageMode = inject<Ref<boolean>>('libraryPageMode', ref(false))
const { formatMessage } = useVIntl()
const messages = defineMessages({
	search: { id: 'app.library.search.placeholder', defaultMessage: 'Search' },
	newGroup: { id: 'app.library.group.new', defaultMessage: 'New group' },
	newInstance: { id: 'app.library.instance.new', defaultMessage: 'New instance' },
	instanceCount: {
		id: 'app.library.toolbar.instance-count',
		defaultMessage: '{count} instances',
	},
})
const selectedInstanceIds = computed(
	() =>
		new Set([...selectedLibraryInstances.value.values()].map((selection) => selection.instanceId)),
)
const instanceCount = computed(() =>
	instanceGroups.value.reduce((total, group) => total + group.instances.length, 0),
)

function openNewGroup() {
	openNewGroupModal(selectedInstanceIds.value)
}
</script>

<template>
	<div v-if="pageMode" class="flex flex-col gap-3">
		<div class="flex flex-wrap items-center gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="text"
				size="large"
				:placeholder="formatMessage(messages.search)"
				clearable
				wrapper-class="min-w-[12rem] flex-1"
			/>
			<Button class="shrink-0" @click="openNewGroup">
				<SquarePlusIcon />
				{{ formatMessage(messages.newGroup) }}
			</Button>
			<Button class="shrink-0" type="colored" color="brand" @click="showCreationModal?.()">
				<PlusIcon />
				{{ formatMessage(messages.newInstance) }}
			</Button>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<SortMenu />
			<FilterMenu />
			<TagItem class="ml-auto shrink-0 border-surface-4 bg-surface-2">
				{{ formatMessage(messages.instanceCount, { count: instanceCount }) }}
			</TagItem>
		</div>
	</div>
	<div v-else class="flex flex-col gap-2">
		<div class="flex flex-wrap gap-2">
			<Input
				v-model="search"
				:icon="SearchIcon"
				type="text"
				:placeholder="formatMessage(messages.search)"
				clearable
				wrapper-class="min-w-[16rem] flex-1"
			/>
			<Button @click="openNewGroup">
				<SquarePlusIcon />
				{{ formatMessage(messages.newGroup) }}
			</Button>
			<Button type="colored" color="brand" @click="showCreationModal?.()">
				<PlusIcon />
				{{ formatMessage(messages.newInstance) }}
			</Button>
		</div>
		<div class="flex flex-wrap items-center gap-2">
			<SortMenu />
			<div class="mx-2 h-6 w-px bg-surface-5" />
			<FilterMenu />
		</div>
	</div>
	<NewGroupModal />
</template>
