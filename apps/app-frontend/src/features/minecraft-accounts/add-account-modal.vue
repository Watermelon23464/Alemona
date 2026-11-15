<script setup lang="ts">
import { Button, defineMessages, injectNotificationManager, Input, NewModal, useVIntl } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { add_offline_user } from '@/helpers/auth'

const emit = defineEmits<{
	microsoft: []
	created: [account: { profile: { id: string } }]
}>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const modal = ref<InstanceType<typeof NewModal> | null>(null)
const step = ref<'choose' | 'offline'>('choose')
const username = ref('')
const creating = ref(false)

const trimmed = computed(() => username.value.trim())
const valid = computed(() => /^[A-Za-z0-9_]{3,16}$/.test(trimmed.value))

const messages = defineMessages({
	chooseTitle: {
		id: 'minecraft-account.add.choose-title',
		defaultMessage: 'Choose account type',
	},
	microsoft: {
		id: 'minecraft-account.add.microsoft',
		defaultMessage: 'Microsoft account',
	},
	offline: {
		id: 'minecraft-account.add.offline',
		defaultMessage: 'Offline account',
	},
	offlineTitle: {
		id: 'minecraft-account.add.offline-title',
		defaultMessage: 'Create offline account',
	},
	nameLabel: {
		id: 'minecraft-account.add.name-label',
		defaultMessage: 'Account name',
	},
	namePlaceholder: {
		id: 'minecraft-account.add.name-placeholder',
		defaultMessage: 'For example: Steve_123',
	},
	nameHint: {
		id: 'minecraft-account.add.name-hint',
		defaultMessage:
			'Please do not use Chinese characters or special characters. Prefer letters, numbers and underscores.',
	},
	nameInvalid: {
		id: 'minecraft-account.add.name-invalid',
		defaultMessage: 'Use 3-16 letters, numbers or underscores.',
	},
	confirm: {
		id: 'minecraft-account.add.confirm',
		defaultMessage: 'Create',
	},
})

function show() {
	step.value = 'choose'
	username.value = ''
	creating.value = false
	modal.value?.show()
}

function hide() {
	modal.value?.hide()
}

function chooseMicrosoft() {
	hide()
	emit('microsoft')
}

async function createOffline() {
	if (!valid.value || creating.value) return
	creating.value = true
	try {
		const account = await add_offline_user(trimmed.value)
		emit('created', account)
		hide()
	} catch (error) {
		handleError(error)
	} finally {
		creating.value = false
	}
}

defineExpose({ show })
</script>

<template>
	<NewModal
		ref="modal"
		:header="
			formatMessage(step === 'choose' ? messages.chooseTitle : messages.offlineTitle)
		"
		max-width="26rem"
	>
		<div v-if="step === 'choose'" class="flex flex-col gap-2">
			<Button class="w-full" @click="chooseMicrosoft">
				{{ formatMessage(messages.microsoft) }}
			</Button>
			<Button class="w-full" @click="step = 'offline'">
				{{ formatMessage(messages.offline) }}
			</Button>
		</div>
		<div v-else class="flex flex-col gap-3">
			<label class="font-semibold text-contrast" for="offline-account-name">
				{{ formatMessage(messages.nameLabel) }}
			</label>
			<Input
				id="offline-account-name"
				v-model="username"
				:placeholder="formatMessage(messages.namePlaceholder)"
				class="w-full"
				@keyup.enter="createOffline"
			/>
			<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.nameHint) }}</p>
			<p v-if="trimmed.length > 0 && !valid" class="m-0 text-xs text-red">
				{{ formatMessage(messages.nameInvalid) }}
			</p>
			<div class="mt-2 flex justify-end gap-2">
				<Button :disabled="!valid || creating" @click="createOffline">
					{{ formatMessage(messages.confirm) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>
