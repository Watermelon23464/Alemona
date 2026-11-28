<script setup lang="ts">
import {
	ArrowLeftIcon,
	PlusIcon,
	RadioButtonCheckedIcon,
	RadioButtonIcon,
	SpinnerIcon,
	TrashIcon,
} from '@modrinth/assets'
import {
	Admonition,
	Avatar,
	Button,
	defineMessages,
	IconButton,
	injectNotificationManager,
	Input,
	useVIntl,
} from '@modrinth/ui'
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRouter } from 'vue-router'

import { useAppEvent } from '@/composables/use-app-event'
import { accountsRevision, notifyAccountsChanged } from '@/features/minecraft-accounts/sync'
import {
	account_metas,
	add_offline_user,
	add_yggdrasil_user,
	get_default_user,
	login as microsoftLogin,
	register_microsoft_account,
	register_offline_account,
	remove_user,
	set_default_user,
	users,
} from '@/helpers/auth'

interface MinecraftCredential {
	profile: {
		id: string
		name: string
	}
}

interface AccountMeta {
	uuid: string
	kind: 'microsoft' | 'offline' | 'littleskin' | 'yggdrasil'
	api_root?: string
	usage: number
	last_used?: number
}

const { formatMessage } = useVIntl()
const router = useRouter()

function goAdd(): void {
	void router.push('/settings/minecraft-accounts/add')
}
const { handleError } = injectNotificationManager()

const messages = defineMessages({
	title: { id: 'app.settings.accounts.title', defaultMessage: 'Minecraft accounts' },
	addAccount: { id: 'app.settings.accounts.add', defaultMessage: 'Add account' },
	warning: {
		id: 'app.settings.accounts.warning',
		defaultMessage:
			'Microsoft Minecraft accounts are not completely safe. Some people may be able to steal your Microsoft account through unusual means. Prefer offline accounts when running mods or modpacks that are not licensed by Modrinth or CurseForge.',
	},
	description: {
		id: 'app.settings.accounts.description',
		defaultMessage:
			'Choose the account to use the next time you launch Minecraft. Use a Microsoft account unless you have a specific reason not to.',
	},
	search: {
		id: 'app.settings.accounts.search',
		defaultMessage: 'Search Minecraft accounts',
	},
	playAccounts: { id: 'app.settings.accounts.play-accounts', defaultMessage: 'Play accounts' },
	noAccounts: {
		id: 'app.settings.accounts.no-accounts',
		defaultMessage: 'No Minecraft accounts yet. Add one to start playing.',
	},
	noResults: { id: 'app.settings.accounts.no-results', defaultMessage: 'No matching accounts.' },
	selected: { id: 'app.settings.accounts.selected', defaultMessage: 'In use' },
	usage: { id: 'app.settings.accounts.usage', defaultMessage: 'Usage' },
	neverUsed: { id: 'app.settings.accounts.never-used', defaultMessage: 'Not played yet' },
	remove: { id: 'app.settings.accounts.remove', defaultMessage: 'Remove account' },
	kindMicrosoft: {
		id: 'app.settings.accounts.kind.microsoft',
		defaultMessage: 'Microsoft account',
	},
	kindOffline: { id: 'app.settings.accounts.kind.offline', defaultMessage: 'Offline account' },
	kindLittleSkin: {
		id: 'app.settings.accounts.kind.littleskin',
		defaultMessage: 'LittleSkin account',
	},
	kindYggdrasil: {
		id: 'app.settings.accounts.kind.yggdrasil',
		defaultMessage: 'Yggdrasil account',
	},
	pickerTitle: { id: 'app.settings.accounts.picker-title', defaultMessage: 'Choose account type' },
	pickerBack: { id: 'app.settings.accounts.back', defaultMessage: 'Back to accounts' },
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

const view = ref<'list' | 'add'>('list')
const accounts = ref<MinecraftCredential[]>([])
const metas = ref<AccountMeta[]>([])
const defaultUser = ref<string | undefined>()
const search = ref('')
const loading = ref(true)
const busy = ref(false)
const signingIn = ref<'microsoft' | 'offline' | 'littleskin' | 'yggdrasil' | null>(null)

const offlineName = ref('')
const littleSkinAccount = ref('')
const littleSkinPassword = ref('')
const yggdrasilApi = ref('https://')
const yggdrasilAccount = ref('')
const yggdrasilPassword = ref('')

const avatarIndex = ref<Record<string, number>>({})
const avatarFailed = ref<Record<string, boolean>>({})

const totalUsage = computed(() => metas.value.reduce((total, meta) => total + (meta.usage ?? 0), 0))

async function refresh(): Promise<void> {
	const [userList, metaList, selected] = await Promise.all([
		users().catch((error) => {
			handleError(error)
			return []
		}),
		account_metas().catch(() => []),
		get_default_user().catch(() => undefined),
	])
	const next = Array.isArray(userList) ? [...userList] : []
	const changed =
		next.length !== accounts.value.length ||
		next.some(
			(account, index) =>
				account.profile.id !== accounts.value[index]?.profile.id ||
				account.profile.name !== accounts.value[index]?.profile.name,
		)
	if (changed) {
		accounts.value = next
		avatarIndex.value = {}
		avatarFailed.value = {}
	}
	metas.value = Array.isArray(metaList) ? [...metaList] : []
	defaultUser.value = selected
	loading.value = false
}

void refresh()

const refreshTimer = setInterval(() => void refresh(), 4000)

watch(accountsRevision, () => void refresh())

useAppEvent('process', () => void refresh())
document.addEventListener('visibilitychange', () => void refresh())
watch(view, (value) => {
	if (value === 'list') void refresh()
})

function metaFor(id: string): AccountMeta | undefined {
	return metas.value.find((meta) => meta.uuid === id)
}

function isLittleSkinAccount(id: string): boolean {
	const meta = metaFor(id)
	if (!meta) return false
	if (meta.kind === 'littleskin' || (meta.kind as string) === 'little_skin') return true
	return meta.kind === 'yggdrasil' && (meta.api_root ?? '').toLowerCase().includes('littleskin')
}

function kindLabel(id: string): string {
	const raw = (metaFor(id)?.kind ?? 'microsoft') as string
	const kind = raw === 'little_skin' ? 'littleskin' : raw
	if (isLittleSkinAccount(id)) return formatMessage(messages.kindLittleSkin)
	switch (kind) {
		case 'offline':
			return formatMessage(messages.kindOffline)
		case 'yggdrasil':
			return formatMessage(messages.kindYggdrasil)
		default:
			return formatMessage(messages.kindMicrosoft)
	}
}

function usageCount(id: string): number {
	return metaFor(id)?.usage ?? 0
}

function usageShare(id: string): number {
	const total = totalUsage.value
	if (!total) return 0
	return Math.round((usageCount(id) / total) * 100)
}

function usageLabel(id: string): string {
	return usageCount(id) ? `${usageShare(id)}%` : formatMessage(messages.neverUsed)
}

function avatarCandidates(account: MinecraftCredential): string[] {
	const id = account.profile.id
	const name = encodeURIComponent(account.profile.name)
	const candidates: string[] = []
	if (isLittleSkinAccount(id)) {
		candidates.push(`https://littleskin.cn/avatar/player/${name}?size=64`)
	}
	candidates.push(
		`https://minotar.net/helm/${id}/64`,
		`https://mc-heads.net/avatar/${id}/64`,
		`https://crafatar.com/avatars/${id}?overlay&size=64`,
	)
	return candidates
}

function avatarSrc(account: MinecraftCredential): string {
	const candidates = avatarCandidates(account)
	const index = Math.min(avatarIndex.value[account.profile.id] ?? 0, candidates.length - 1)
	return candidates[index] ?? ''
}

function onAvatarError(account: MinecraftCredential): void {
	const id = account.profile.id
	const candidates = avatarCandidates(account)
	const next = (avatarIndex.value[id] ?? 0) + 1
	if (next >= candidates.length) avatarFailed.value = { ...avatarFailed.value, [id]: true }
	else avatarIndex.value = { ...avatarIndex.value, [id]: next }
}

function initialColor(name: string): string {
	let hash = 0
	for (const character of name) hash = (hash * 31 + character.charCodeAt(0)) % 360
	return `hsl(${hash} 60% 45%)`
}

function compareNames(a: MinecraftCredential, b: MinecraftCredential): number {
	const left = a.profile.name.toUpperCase()
	const right = b.profile.name.toUpperCase()
	if (left !== right) return left < right ? -1 : 1
	return a.profile.id < b.profile.id ? -1 : 1
}

const filtered = computed(() => {
	const query = search.value.trim().toLowerCase()
	const sorted = [...accounts.value].sort(compareNames)
	if (!query) return sorted
	return sorted.filter(
		(account) =>
			account.profile.name.toLowerCase().includes(query) ||
			kindLabel(account.profile.id).toLowerCase().includes(query),
	)
})

async function select(id: string): Promise<void> {
	if (defaultUser.value === id) return
	try {
		await set_default_user(id)
		notifyAccountsChanged()
		await refresh()
	} catch (error) {
		handleError(error)
	}
}

async function remove(id: string): Promise<void> {
	try {
		await remove_user(id)
		notifyAccountsChanged()
		await refresh()
	} catch (error) {
		handleError(error)
	}
}

async function addMicrosoft(): Promise<void> {
	if (busy.value) return
	busy.value = true
	signingIn.value = 'microsoft'
	try {
		const created = await microsoftLogin()
		const id = created?.profile?.id
		if (id) await register_microsoft_account(id)
		await refresh()
		view.value = 'list'
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
		await refresh()
		view.value = 'list'
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
		console.log(`[accounts] ${kind} sign-in succeeded`)
		littleSkinAccount.value = ''
		littleSkinPassword.value = ''
		yggdrasilAccount.value = ''
		yggdrasilPassword.value = ''
		await refresh()
		view.value = 'list'
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

const littleSkinReady = computed(
	() => littleSkinAccount.value.trim().length > 0 && littleSkinPassword.value.length > 0,
)

defineExpose({ refresh })

onBeforeUnmount(() => clearInterval(refreshTimer))
</script>

<template>
	<div class="flex flex-col">
		<Transition name="pane" mode="out-in">
			<div v-if="view === 'list'" key="list" class="flex flex-col gap-3">
				<div class="flex items-center gap-2">
					<h2 class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.title) }}
					</h2>
					<IconButton class="ml-auto" :label="formatMessage(messages.addAccount)" @click="goAdd()">
						<PlusIcon />
					</IconButton>
				</div>

				<Admonition type="warning">{{ formatMessage(messages.warning) }}</Admonition>

				<p class="m-0 text-secondary">{{ formatMessage(messages.description) }}</p>

				<Input
					v-model="search"
					:placeholder="formatMessage(messages.search)"
					:aria-label="formatMessage(messages.search)"
				/>

				<h3 class="m-0 mt-1 text-base font-semibold text-contrast">
					{{ formatMessage(messages.playAccounts) }}
				</h3>

				<div
					v-if="!loading"
					class="overflow-clip rounded-xl border border-solid border-surface-5 bg-button-bg"
				>
					<p v-if="filtered.length === 0" class="m-0 px-3 py-4 text-secondary">
						{{ formatMessage(accounts.length === 0 ? messages.noAccounts : messages.noResults) }}
					</p>

					<div
						v-for="account in filtered"
						:key="account.profile.id"
						class="group flex items-center gap-1 border-0 border-b border-solid border-surface-5 last:border-b-0"
						:class="
							defaultUser === account.profile.id
								? 'bg-[color-mix(in_srgb,var(--color-purple-400)_22%,transparent)]'
								: ''
						"
					>
						<button
							type="button"
							class="button-base flex min-w-0 flex-grow cursor-pointer items-center gap-2 overflow-clip border-0 bg-transparent p-2 text-left"
							@click="select(account.profile.id)"
						>
							<RadioButtonCheckedIcon
								v-if="defaultUser === account.profile.id"
								class="size-5 shrink-0 text-brand"
							/>
							<RadioButtonIcon v-else class="size-5 shrink-0 text-secondary" />

							<span
								v-if="avatarFailed[account.profile.id]"
								class="flex size-6 shrink-0 items-center justify-center rounded-md text-xs font-bold text-white"
								:style="{ background: initialColor(account.profile.name) }"
							>
								{{ account.profile.name.slice(0, 1).toUpperCase() }}
							</span>
							<Avatar
								v-else
								:src="avatarSrc(account)"
								size="24px"
								class="shrink-0 rounded-md"
								@error="onAvatarError(account)"
							/>

							<span class="flex min-w-0 flex-grow flex-col">
								<span
									class="truncate"
									:class="
										defaultUser === account.profile.id
											? 'font-semibold text-contrast'
											: 'text-primary'
									"
								>
									{{ account.profile.name }}
								</span>
								<span class="truncate text-xs text-secondary">
									{{ kindLabel(account.profile.id) }}
								</span>
							</span>

							<span class="flex w-24 shrink-0 flex-col items-end gap-1 pr-1">
								<span class="text-xs text-secondary">{{ usageLabel(account.profile.id) }}</span>
								<span class="h-1 w-full overflow-hidden rounded-full bg-surface-5">
									<span
										class="block h-full rounded-full bg-brand transition-all duration-300"
										:style="{ width: `${usageShare(account.profile.id)}%` }"
									/>
								</span>
							</span>
						</button>

						<IconButton
							v-tooltip="formatMessage(messages.remove)"
							:label="formatMessage(messages.remove)"
							class="mr-1 shrink-0 opacity-0 transition-opacity group-hover:opacity-100 focus-visible:opacity-100"
							@click.stop="remove(account.profile.id)"
						>
							<TrashIcon />
						</IconButton>
					</div>

					<button
						type="button"
						class="button-base flex w-full cursor-pointer items-center justify-center gap-2 border-0 bg-transparent p-2.5 text-sm font-semibold text-secondary transition-colors hover:bg-surface-4 hover:text-contrast"
						@click="goAdd()"
					>
						<PlusIcon class="size-4" />
						{{ formatMessage(messages.addAccount) }}
					</button>
				</div>
			</div>

			<div v-else key="add" class="flex flex-col gap-3">
				<div class="flex items-center gap-2">
					<IconButton :label="formatMessage(messages.pickerBack)" @click="view = 'list'">
						<ArrowLeftIcon />
					</IconButton>
					<h2 class="m-0 text-xl font-semibold text-contrast">
						{{ formatMessage(messages.pickerTitle) }}
					</h2>
				</div>

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
					<p class="m-0 text-xs text-secondary">
						{{ formatMessage(messages.littleSkinProfileHint) }}
					</p>
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
		</Transition>
	</div>
</template>

<style scoped>
.pane-enter-active,
.pane-leave-active {
	transition:
		opacity 220ms cubic-bezier(0.4, 0, 0.2, 1),
		transform 220ms cubic-bezier(0.4, 0, 0.2, 1);
}

.pane-enter-from {
	opacity: 0;
	transform: translateX(14px) scale(0.995);
}

.pane-leave-to {
	opacity: 0;
	transform: translateX(-10px);
}
</style>
