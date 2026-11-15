<script setup lang="ts">
import { defineMessages, NewModal, useVIntl } from '@modrinth/ui'
import { ref } from 'vue'

import AccountsCard from '@/features/minecraft-accounts/accounts-card.vue'

const { formatMessage } = useVIntl()
const messages = defineMessages({
	header: {
		id: 'app.minecraft-accounts.header',
		defaultMessage: 'Minecraft accounts',
	},
})

const modal = ref<InstanceType<typeof NewModal> | null>(null)

function show() {
	modal.value?.show()
}

defineExpose({ show })
</script>

<template>
	<NewModal ref="modal" :header="formatMessage(messages.header)" max-width="34rem">
		<Suspense>
			<AccountsCard />
		</Suspense>
	</NewModal>
</template>
