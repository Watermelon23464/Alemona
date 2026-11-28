<script setup lang="ts">
import { NavTabs, defineMessages, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'
import { useRoute } from 'vue-router'

const { formatMessage } = useVIntl()
const route = useRoute()

const messages = defineMessages({
	tabInstances: { id: 'app.downloads.tabs.instances', defaultMessage: 'Instances' },
	tabModpacks: { id: 'app.downloads.tabs.modpacks', defaultMessage: 'Modpacks' },
	tabMods: { id: 'app.downloads.tabs.mods', defaultMessage: 'Mods' },
	tabResourcePacks: { id: 'app.downloads.tabs.resource-packs', defaultMessage: 'Resource packs' },
	tabDataPacks: { id: 'app.downloads.tabs.data-packs', defaultMessage: 'Data packs' },
	tabShaders: { id: 'app.downloads.tabs.shaders', defaultMessage: 'Shaders' },
	tabServers: { id: 'app.downloads.tabs.servers', defaultMessage: 'Servers' },
})

/**
 * The project type switcher lives here instead of inside the individual pages so
 * it survives route changes and can animate its pill from tab to tab.
 */
const tabs = computed(() => {
	const query = new URLSearchParams()
	for (const key of ['i', 'ai', 'from', 'sid', 'wid']) {
		const value = route.query[key]
		if (typeof value === 'string' && value) query.set(key, value)
	}
	const suffix = query.toString() ? `?${query.toString()}` : ''

	return [
		{ label: formatMessage(messages.tabInstances), href: '/browse/instance' },
		{ label: formatMessage(messages.tabModpacks), href: `/browse/modpack${suffix}` },
		{ label: formatMessage(messages.tabMods), href: `/browse/mod${suffix}` },
		{ label: formatMessage(messages.tabResourcePacks), href: `/browse/resourcepack${suffix}` },
		{ label: formatMessage(messages.tabDataPacks), href: `/browse/datapack${suffix}` },
		{ label: formatMessage(messages.tabShaders), href: `/browse/shader${suffix}` },
		{ label: formatMessage(messages.tabServers), href: `/browse/server${suffix}` },
	]
})
</script>

<template>
	<div class="flex w-full min-w-0 flex-col">
		<div class="w-full px-4 pt-4 pb-1 sm:px-6 sm:pt-6">
			<NavTabs :links="tabs" replace class="w-fit" />
		</div>
		<RouterView />
	</div>
</template>
