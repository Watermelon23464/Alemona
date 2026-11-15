<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { computed, onMounted } from 'vue'

import LibrarySection from '@/components/ui/library/index.vue'
import { instanceListQueryOptions } from '@/pages/instance/query-options'
import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

defineOptions({ name: 'LibraryPage' })

const { formatMessage } = useVIntl()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	title: { id: 'app.library.title', defaultMessage: 'Library' },
})

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])

onMounted(() => {
	breadcrumbManager.reset({
		slot: 'library',
		id: () => 'library',
		label: () => formatMessage(messages.title),
	})
})
</script>

<template>
	<div class="flex w-full flex-col p-6">
		<LibrarySection :instances="instances" page-mode />
	</div>
</template>
