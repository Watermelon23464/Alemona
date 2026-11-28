<script setup lang="ts">
import { PlusIcon, SpinnerIcon } from '@modrinth/assets'
import {
	Admonition,
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { computed, ref } from 'vue'

import { notifyAccountsChanged } from '@/features/minecraft-accounts/sync'
import {
	add_offline_user,
	add_yggdrasil_user,
	login as microsoftLogin,
	register_microsoft_account,
	register_offline_account,
} from '@/helpers/auth'

const emit = defineEmits<{ created: [] }>()

const { formatMessage } = useVIntl()
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	pickerWarning: {
		id: 'app.settings.accounts.picker-warning',
		defaultMessage:
			'AL does not recommend third-party or offline accounts: Minecraft itself does not support them, problems may appear while playing, and third-party or offline accounts may not be able to join some servers.',
	},
	microsoftTitle: {
		id: 'app.settings.accounts.microsoft.title',
		defaultMessage: 'Microsoft account',
	},
	microsoftDescription: {
		id: 'app.settings.accounts.microsoft.description',
		defaultMessage:
			'Microsoft accounts have the best compatibility and medium security, and are the accounts you used to buy Minecraft. Microsoft accounts need an external authenticator, which AL opens for you automatically.',
	},
	microsoftButton: {
		id: 'app.settings.accounts.microsoft.button',
		defaultMessage: 'Sign in to Microsoft',
	},
	offlineTitle: { id: 'app.settings.accounts.offline.title', defaultMessage: 'Offline account' },
	offlineDescription: {
		id: 'app.settings.accounts.offline.description',
		defaultMessage:
			'Offline accounts are the safest accounts, but they cannot be shared and cannot use skins or capes. Use an offline account if you do not own Minecraft or if you run risky add-ons.',
	},
	offlineHint: {
		id: 'app.settings.accounts.offline.hint',
		defaultMessage:
			'Please do not use Chinese characters or special characters. Prefer letters, numbers and underscores.',
	},
	offlinePlaceholder: {
		id: 'app.settings.accounts.offline.placeholder',
		defaultMessage: 'For example: Steve_123',
	},
	offlineButton: {
		id: 'app.settings.accounts.offline.button',
		defaultMessage: 'Add offline account',
	},
	littleSkinTitle: {
		id: 'app.settings.accounts.littleskin.title',
		defaultMessage: 'LittleSkin account',
	},
	littleSkinDescription: {
		id: 'app.settings.accounts.littleskin.description',
		defaultMessage:
			'They register easily and offer the same features as a Mojang account. Their security is not high, and they still cannot join servers that require a Mojang account.',
	},
	littleSkinCredentials: {
		id: 'app.settings.accounts.littleskin.credentials',
		defaultMessage: 'LittleSkin account or email',
	},
	littleSkinPassword: {
		id: 'app.settings.accounts.littleskin.password',
		defaultMessage: 'LittleSkin password',
	},
	littleSkinButton: {
		id: 'app.settings.accounts.littleskin.button',
		defaultMessage: 'Sign in to LittleSkin',
	},
	littleSkinProfileHint: {
		id: 'app.settings.accounts.littleskin.profile-hint',
		defaultMessage:
			'LittleSkin requires a player profile: create your character on littleskin.cn before signing in here.',
	},
	yggdrasilTitle: {
		id: 'app.settings.accounts.yggdrasil.title',
		defaultMessage: 'Use another Yggdrasil login',
	},
	yggdrasilApi: {
		id: 'app.settings.accounts.yggdrasil.api',
		defaultMessage: 'Yggdrasil API address',
	},
	yggdrasilAccount: {
		id: 'app.settings.accounts.yggdrasil.account',
		defaultMessage: 'Account or email',
	},
	yggdrasilPassword: {
		id: 'app.settings.accounts.yggdrasil.password',
		defaultMessage: 'Password',
	},
	yggdrasilButton: {
		id: 'app.settings.accounts.yggdrasil.button',
		defaultMessage: 'Use Yggdrasil login',
	},
})

const busy = ref(false)
const signingIn = ref<'microsoft' | 'offline' | 'littleskin' | 'yggdrasil' | null>(null)

const offlineName = ref('')
const littleSkinAccount = ref('')
const littleSkinPassword = ref('')
const yggdrasilApi = ref('https://')
const yggdrasilAccount = ref('')
const yggdrasilPassword = ref('')

const littleSkinReady = computed(
	() => littleSkinAccount.value.trim().length > 0 && littleSkinPassword.value.length > 0,
)

async function addMicrosoft(): Promise<void> {
	if (busy.value) return
	busy.value = true
	signingIn.value = 'microsoft'
	try {
		const created = await microsoftLogin()
		const id = created?.profile?.id
		if (id) await register_microsoft_account(id)
		notifyAccountsChanged()
		emit('created')
	} catch (error) {
		console.error('[accounts] microsoft sign-in failed', error)
		handleError(error)
	} finally {
		busy.value = false
		signingIn.value = null
	}
}

async function addOffline(): Promise<void> {
	if (busy.value) return
	busy.value = true
	signingIn.value = 'offline'
	try {
		const created = await add_offline_user(offlineName.value.trim())
		const id = created?.profile?.id
		if (id) await register_offline_account(id)
		offlineName.value = ''
		notifyAccountsChanged()
		emit('created')
	} catch (error) {
		console.error('[accounts] offline sign-in failed', error)
		handleError(error)
	} finally {
		busy.value = false
		signingIn.value = null
	}
}

async function signInYggdrasil(
	apiRoot: string,
	username: string,
	password: string,
	kind: 'littleskin' | 'yggdrasil',
): Promise<void> {
	if (busy.value || !username || !password) return
	busy.value = true
	signingIn.value = kind
	try {
		console.log(`[accounts] signing in with ${kind} at ${apiRoot}`)
		await add_yggdrasil_user(apiRoot, username, password, kind)
		littleSkinAccount.value = ''
		littleSkinPassword.value = ''
		yggdrasilAccount.value = ''
		yggdrasilPassword.value = ''
		notifyAccountsChanged()
		emit('created')
	} catch (error) {
		console.error(`[accounts] ${kind} sign-in failed`, error)
		handleError(error)
	} finally {
		busy.value = false
		signingIn.value = null
	}
}

function signInLittleSkin(): void {
	void signInYggdrasil(
		'https://littleskin.cn/api/yggdrasil',
		littleSkinAccount.value.trim(),
		littleSkinPassword.value,
		'littleskin',
	)
}

function signInOtherYggdrasil(): void {
	void signInYggdrasil(
		yggdrasilApi.value.trim(),
		yggdrasilAccount.value.trim(),
		yggdrasilPassword.value,
		'yggdrasil',
	)
}
</script>

<template>
	<div class="flex flex-col gap-3">
		<Admonition type="warning">{{ formatMessage(messages.pickerWarning) }}</Admonition>

		<div class="flex flex-col gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.microsoftTitle) }}
			</h3>
			<p class="m-0 text-secondary">{{ formatMessage(messages.microsoftDescription) }}</p>
			<Button class="w-fit" :disabled="busy" @click="addMicrosoft">
				<SpinnerIcon v-if="signingIn === 'microsoft'" class="animate-spin" />
				<PlusIcon v-else />
				{{ formatMessage(messages.microsoftButton) }}
			</Button>
		</div>

		<div class="mt-5 flex flex-col gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.offlineTitle) }}
			</h3>
			<p class="m-0 text-secondary">{{ formatMessage(messages.offlineDescription) }}</p>
			<Input v-model="offlineName" :placeholder="formatMessage(messages.offlinePlaceholder)" />
			<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.offlineHint) }}</p>
			<Button class="w-fit" :disabled="busy" @click="addOffline">
				<SpinnerIcon v-if="signingIn === 'offline'" class="animate-spin" />
				<PlusIcon v-else />
				{{ formatMessage(messages.offlineButton) }}
			</Button>
		</div>

		<div class="mt-5 flex flex-col gap-2">
			<h3 class="m-0 text-base font-semibold text-contrast">
				{{ formatMessage(messages.littleSkinTitle) }}
			</h3>
			<p class="m-0 text-secondary">{{ formatMessage(messages.littleSkinDescription) }}</p>
			<label class="flex flex-col gap-1">
				<span class="text-sm font-medium text-secondary">
					{{ formatMessage(messages.littleSkinCredentials) }}
				</span>
				<Input
					v-model="littleSkinAccount"
					autocomplete="username"
					@keyup.enter="signInLittleSkin"
				/>
			</label>
			<label class="flex flex-col gap-1">
				<span class="text-sm font-medium text-secondary">
					{{ formatMessage(messages.littleSkinPassword) }}
				</span>
				<Input
					v-model="littleSkinPassword"
					type="password"
					autocomplete="current-password"
					@keyup.enter="signInLittleSkin"
				/>
			</label>
			<p class="m-0 text-xs text-secondary">{{ formatMessage(messages.littleSkinProfileHint) }}</p>
			<Button class="w-fit" :disabled="busy || !littleSkinReady" @click="signInLittleSkin">
				<SpinnerIcon v-if="signingIn === 'littleskin'" class="animate-spin" />
				<PlusIcon v-else />
				{{ formatMessage(messages.littleSkinButton) }}
			</Button>
		</div>

		<div class="mt-6 border-0 border-t border-solid border-divider pt-4">
			<h3 class="m-0 mb-3 text-base font-semibold text-contrast">
				{{ formatMessage(messages.yggdrasilTitle) }}
			</h3>
			<div class="flex flex-col gap-3">
				<label class="flex flex-col gap-1">
					<span class="text-sm font-medium text-secondary">
						{{ formatMessage(messages.yggdrasilApi) }}
					</span>
					<Input v-model="yggdrasilApi" placeholder="https://example.com/api/yggdrasil" />
				</label>
				<label class="flex flex-col gap-1">
					<span class="text-sm font-medium text-secondary">
						{{ formatMessage(messages.yggdrasilAccount) }}
					</span>
					<Input v-model="yggdrasilAccount" autocomplete="username" />
				</label>
				<label class="flex flex-col gap-1">
					<span class="text-sm font-medium text-secondary">
						{{ formatMessage(messages.yggdrasilPassword) }}
					</span>
					<Input
						v-model="yggdrasilPassword"
						type="password"
						autocomplete="current-password"
						@keyup.enter="signInOtherYggdrasil"
					/>
				</label>
				<Button class="w-fit" :disabled="busy" @click="signInOtherYggdrasil">
					<SpinnerIcon v-if="signingIn === 'yggdrasil'" class="animate-spin" />
					<PlusIcon v-else />
					{{ formatMessage(messages.yggdrasilButton) }}
				</Button>
			</div>
		</div>
	</div>
</template>
