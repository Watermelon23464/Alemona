<script setup lang="ts">
import { ArrowLeftIcon } from '@modrinth/assets'
import { defineMessages, IconButton, useVIntl } from '@modrinth/ui'
import { computed } from 'vue'

import AddMinecraftAccountForm from '@/features/settings/privacy/add-minecraft-account-form.vue'

const emit = defineEmits<{ back: []; created: [] }>()

const { formatMessage } = useVIntl()

const messages = defineMessages({
	title: {
		id: 'app.settings.accounts.add-page-title',
		defaultMessage: 'Add Minecraft account',
	},
	back: { id: 'app.settings.accounts.back', defaultMessage: 'Back to accounts' },
})

const title = computed(() => formatMessage(messages.title))
</script>

<template>
	<div class="flex flex-col gap-4">
		<div class="flex items-center gap-2">
			<IconButton :label="formatMessage(messages.back)" @click="emit('back')">
				<ArrowLeftIcon />
			</IconButton>
			<h2 class="m-0 text-xl font-semibold text-contrast">{{ title }}</h2>
		</div>

		<AddMinecraftAccountForm @created="emit('created')" />
	</div>
</template>
