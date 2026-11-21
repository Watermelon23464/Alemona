<script setup lang="ts">
import {
	Button,
	defineMessages,
	injectNotificationManager,
	Input,
	NewModal,
	useVIntl,
} from '@modrinth/ui'
import { invoke } from '@tauri-apps/api/core'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'

import { injectBreadcrumbManager } from '@/shared/breadcrumbs'

defineOptions({ name: 'TerracottaPage' })

interface TerracottaInstall {
	installed: boolean
	dir: string
	executable: string | null
	version: string | null
}

interface RoomProfile {
	machine_id?: string
	name?: string
	kind?: string
}

interface RoomState {
	state?: string
	index?: number
	room?: string
	url?: string
	difficulty?: string
	type?: number
	profiles?: RoomProfile[]
}

const { formatMessage } = useVIntl()
const { handleError, addNotification } = injectNotificationManager()
const breadcrumbManager = injectBreadcrumbManager()

const messages = defineMessages({
	title: { id: 'app.terracotta.title', defaultMessage: 'Terracotta Networking' },
	hostCard: { id: 'app.terracotta.host.card', defaultMessage: 'I want to host' },
	hostCardBody: {
		id: 'app.terracotta.host.card.body',
		defaultMessage:
			'Open your LAN game to the internet with Terracotta, then invite your friends into your world.',
	},
	guestCard: { id: 'app.terracotta.guest.card', defaultMessage: 'I want to join' },
	guestCardBody: {
		id: 'app.terracotta.guest.card.body',
		defaultMessage: "Join a friend's world with the ID they gave you and play together!",
	},
	hostModalTitle: { id: 'app.terracotta.host.modal.title', defaultMessage: 'Create a room' },
	hostWaiting: {
		id: 'app.terracotta.host.waiting',
		defaultMessage: 'You need to open a Minecraft world and allow it on the local network.',
	},
	hostReadyHint: {
		id: 'app.terracotta.host.ready',
		defaultMessage:
			'Copy your invitation code and send it to your friends. They can join with HMCL, PCL CE, Alemona or any launcher with Terracotta support through the "I want to join" feature.',
	},
	copyInvite: { id: 'app.terracotta.copy.invite', defaultMessage: 'Copy invitation code' },
	copied: { id: 'app.terracotta.copied', defaultMessage: 'Invitation code copied' },
	closeRoom: { id: 'app.terracotta.close.room', defaultMessage: 'Close room' },
	close: { id: 'app.terracotta.close', defaultMessage: 'Close' },
	players: { id: 'app.terracotta.players', defaultMessage: 'Players in the room' },
	playerName: { id: 'app.terracotta.player.name', defaultMessage: 'Player' },
	playerRole: { id: 'app.terracotta.player.role', defaultMessage: 'Role' },
	roleHost: { id: 'app.terracotta.role.host', defaultMessage: 'Host' },
	roleGuest: { id: 'app.terracotta.role.guest', defaultMessage: 'Guest' },
	roleLocal: { id: 'app.terracotta.role.local', defaultMessage: 'This computer' },
	joinModalTitle: { id: 'app.terracotta.join.modal.title', defaultMessage: 'Join a room' },
	joinDescription: {
		id: 'app.terracotta.join.description',
		defaultMessage:
			'Enter the invitation code you received in the format X/XXXX-XXXX-XXXX-XXXX, then join through Minecraft.',
	},
	joinPlaceholder: {
		id: 'app.terracotta.join.placeholder',
		defaultMessage: 'X/XXXX-XXXX-XXXX-XXXX',
	},
	joinAction: { id: 'app.terracotta.join.action', defaultMessage: 'Join' },
	successTitle: { id: 'app.terracotta.success.title', defaultMessage: 'Success!' },
	successBody: {
		id: 'app.terracotta.success.body',
		defaultMessage: 'You joined the room network, now join the room through Minecraft.',
	},
	download: { id: 'app.terracotta.download', defaultMessage: 'Download' },
	downloading: { id: 'app.terracotta.downloading', defaultMessage: 'Downloading...' },
	introTitle: {
		id: 'app.terracotta.intro.title',
		defaultMessage: 'Download the Terracotta Networking files',
	},
	introBody: {
		id: 'app.terracotta.intro.body',
		defaultMessage:
			'Terracotta is a third-party program! Report problems through its own channels; Alemona does not guarantee that it works.',
	},
	introAuto: {
		id: 'app.terracotta.intro.auto',
		defaultMessage:
			'If you click download, Alemona creates the needed files and runs them every time the multiplayer feature is enabled. Download speed varies, so you can also get them from the',
	},
	officialSite: { id: 'app.terracotta.official-site', defaultMessage: 'official website download' },
	introDisclaim: {
		id: 'app.terracotta.intro.disclaim',
		defaultMessage: 'If the multiplayer feature breaks, URLA and Alemona accept no responsibility.',
	},
	cancel: { id: 'app.terracotta.cancel', defaultMessage: 'Not now' },
	warningTitle: { id: 'app.terracotta.warning.title', defaultMessage: 'This is your last warning' },
	warningBody: {
		id: 'app.terracotta.warning.body',
		defaultMessage:
			'If you close the room, every player inside will be kicked out and the room shuts down for good!',
	},
	warningConfirm: {
		id: 'app.terracotta.warning.confirm',
		defaultMessage: 'I know what I am doing',
	},
})

const install = ref<TerracottaInstall | null>(null)
const downloading = ref(false)
const showIntro = ref(false)
const busy = ref(false)
const port = ref(0)
const roomState = ref<RoomState>({ state: 'waiting', index: 0 })
const roomInput = ref('')
const playerName = ref(localStorage.getItem('terracotta-player-name') ?? '')
const hostModal = ref(false)
const joinModal = ref(false)
const successModal = ref(false)
const closeRoomModal = ref(false)
let pollTimer: ReturnType<typeof setInterval> | null = null
let lastIndex = 0
let hostModalRemembered = false

const installed = computed(() => install.value?.installed === true)
const stateName = computed(() => roomState.value.state ?? 'waiting')
const roomCode = computed(() => roomState.value.room ?? '')
const profiles = computed(() => roomState.value.profiles ?? [])
const roomReady = computed(() => stateName.value === 'host-ok' && Boolean(roomCode.value))

function roleLabel(profile: RoomProfile) {
	if (profile.kind === 'HOST') return formatMessage(messages.roleHost)
	if (profile.kind === 'LOCAL') return formatMessage(messages.roleLocal)
	return formatMessage(messages.roleGuest)
}

async function refresh() {
	try {
		install.value = await invoke<TerracottaInstall>('plugin:terracotta|terracotta_get_install')
		if (!install.value.installed) {
			showIntro.value = true
			return
		}
		await start()
		if (hostModalRemembered) hostModal.value = true
	} catch (error) {
		handleError(error as Error)
	}
}

async function start() {
	try {
		port.value = await invoke<number>('plugin:terracotta|terracotta_launch')
		startPolling()
	} catch (error) {
		handleError(error as Error)
	}
}

async function poll() {
	if (!installed.value) return
	try {
		const next = await invoke<RoomState>('plugin:terracotta|terracotta_state')
		const index = next.index ?? 0
		if (index && index < lastIndex) return
		lastIndex = index
		roomState.value = next
	} catch {
		// Terracotta is not up yet
	}
}

function startPolling() {
	if (pollTimer) return
	void poll()
	pollTimer = setInterval(() => void poll(), 700)
}

function stopPolling() {
	if (pollTimer) clearInterval(pollTimer)
	pollTimer = null
}

async function download() {
	if (downloading.value) return
	downloading.value = true
	showIntro.value = false
	try {
		install.value = await invoke<TerracottaInstall>('plugin:terracotta|terracotta_install')
		await start()
	} catch (error) {
		handleError(error as Error)
	} finally {
		downloading.value = false
	}
}

async function openHostModal() {
	if (!installed.value) {
		showIntro.value = true
		return
	}
	hostModal.value = true
	hostModalRemembered = true
	if (stateName.value !== 'host-ok') await createRoom()
}

function closeHostModal() {
	hostModal.value = false
	hostModalRemembered = false
}

async function createRoom() {
	if (busy.value || stateName.value === 'host-ok') return
	busy.value = true
	try {
		localStorage.setItem('terracotta-player-name', playerName.value)
		await invoke('plugin:terracotta|terracotta_host', { player: playerName.value || null })
		startPolling()
	} catch (error) {
		handleError(error as Error)
	} finally {
		busy.value = false
	}
}

function askCloseRoom() {
	closeRoomModal.value = true
}

async function confirmCloseRoom() {
	closeRoomModal.value = false
	await closeRoom()
}

async function closeRoom() {
	if (busy.value) return
	busy.value = true
	try {
		await invoke('plugin:terracotta|terracotta_reset')
		lastIndex = 0
		roomState.value = { state: 'waiting', index: 0 }
		closeHostModal()
	} catch (error) {
		handleError(error as Error)
	} finally {
		busy.value = false
	}
}

function openJoinModal() {
	if (!installed.value) {
		showIntro.value = true
		return
	}
	joinModal.value = true
}

async function joinRoom() {
	const code = roomInput.value.trim()
	if (!code || busy.value) return
	busy.value = true
	try {
		localStorage.setItem('terracotta-player-name', playerName.value)
		await invoke('plugin:terracotta|terracotta_join', {
			room: code,
			player: playerName.value || null,
		})
		joinModal.value = false
		successModal.value = true
		startPolling()
	} catch (error) {
		handleError(error as Error)
	} finally {
		busy.value = false
	}
}

async function copyInvite() {
	try {
		await navigator.clipboard.writeText(roomCode.value)
		addNotification({ title: formatMessage(messages.copied) })
	} catch {
		// clipboard may be unavailable, the code stays selectable
	}
}

const hostModalRef = ref<{ show?: () => void } | null>(null)
const joinModalRef = ref<{ show?: () => void } | null>(null)
const successModalRef = ref<{ show?: () => void } | null>(null)
const introModalRef = ref<{ show?: () => void } | null>(null)

watch(hostModal, async (open) => {
	if (!open) return
	await nextTick()
	hostModalRef.value?.show?.()
})

watch(joinModal, async (open) => {
	if (!open) return
	await nextTick()
	joinModalRef.value?.show?.()
})

watch(successModal, async (open) => {
	if (!open) return
	await nextTick()
	successModalRef.value?.show?.()
})

watch(showIntro, async (open) => {
	if (!open) return
	await nextTick()
	introModalRef.value?.show?.()
})

watch(closeRoomModal, async (open) => {
	if (!open) return
	await nextTick()
	closeRoomModalRef.value?.show?.()
})

onMounted(async () => {
	breadcrumbManager.reset({
		slot: 'terracotta',
		id: () => 'terracotta',
		label: () => formatMessage(messages.title),
	})
	await refresh()
})

onUnmounted(stopPolling)
</script>

<template>
	<div class="flex w-full flex-col gap-5 p-4 sm:p-6">
		<h1 class="m-0 text-2xl font-bold text-contrast">{{ formatMessage(messages.title) }}</h1>

		<div class="grid gap-4 lg:grid-cols-2">
			<button
				type="button"
				class="flex cursor-pointer items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5 text-left transition-[background-color,border-color,transform] duration-150 ease-out hover:bg-surface-4 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow active:scale-[0.99]"
				@click="openHostModal"
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					class="size-8 shrink-0 text-contrast"
				>
					<circle cx="12" cy="8" r="3.6" />
					<path d="M4.6 20c.6-3.6 3.7-5.6 7.4-5.6s6.8 2 7.4 5.6" />
				</svg>
				<span class="flex min-w-0 flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.hostCard) }}
					</span>
					<span class="text-sm leading-5 text-secondary">
						{{ formatMessage(messages.hostCardBody) }}
					</span>
				</span>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					class="ml-auto size-5 shrink-0 text-secondary"
				>
					<path d="m9 5 7 7-7 7" />
				</svg>
			</button>

			<button
				type="button"
				class="flex cursor-pointer items-start gap-4 rounded-2xl border border-solid border-surface-5 bg-surface-3 p-5 text-left transition-[background-color,border-color,transform] duration-150 ease-out hover:bg-surface-4 focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-brand-shadow active:scale-[0.99]"
				@click="openJoinModal"
			>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="1.8"
					class="size-8 shrink-0 text-contrast"
				>
					<circle cx="12" cy="12" r="9" />
					<path d="M12 8v8M8 12h8" />
				</svg>
				<span class="flex min-w-0 flex-col gap-1">
					<span class="text-lg font-semibold text-contrast">
						{{ formatMessage(messages.guestCard) }}
					</span>
					<span class="text-sm leading-5 text-secondary">
						{{ formatMessage(messages.guestCardBody) }}
					</span>
				</span>
				<svg
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					class="ml-auto size-5 shrink-0 text-secondary"
				>
					<path d="m9 5 7 7-7 7" />
				</svg>
			</button>
		</div>

		<button
			v-if="!installed"
			type="button"
			class="flex w-fit cursor-pointer items-center gap-2 rounded-xl bg-brand-highlight px-3 py-2 text-sm font-semibold text-brand hover:underline"
			@click="showIntro = true"
		>
			{{ formatMessage(messages.download) }}
		</button>
	</div>

	<NewModal
		ref="hostModalRef"
		v-if="hostModal"
		:header="formatMessage(messages.hostModalTitle)"
		max-width="640px"
		:on-hide="closeHostModal"
	>
		<div class="flex flex-col gap-4">
			<template v-if="roomReady">
				<span class="select-all break-all font-mono text-3xl font-bold text-contrast">
					{{ roomCode }}
				</span>
				<Button class="w-fit" @click="copyInvite">
					{{ formatMessage(messages.copyInvite) }}
				</Button>
				<p class="m-0 text-sm leading-6 text-primary">
					{{ formatMessage(messages.hostReadyHint) }}
				</p>
				<div class="flex flex-col gap-2">
					<span class="text-xs font-medium uppercase tracking-wide text-secondary">
						{{ formatMessage(messages.players) }}
					</span>
					<table class="w-full border-collapse text-sm">
						<thead>
							<tr class="text-left text-xs uppercase tracking-wide text-secondary">
								<th class="border-b border-surface-5 py-1.5 pr-2 font-medium">
									{{ formatMessage(messages.playerName) }}
								</th>
								<th class="border-b border-surface-5 py-1.5 font-medium">
									{{ formatMessage(messages.playerRole) }}
								</th>
							</tr>
						</thead>
						<tbody>
							<tr v-for="profile in profiles" :key="profile.machine_id">
								<td class="border-b border-surface-5 py-1.5 pr-2 text-contrast">
									{{ profile.name || profile.kind }}
								</td>
								<td class="border-b border-surface-5 py-1.5 text-secondary">
									{{ roleLabel(profile) }}
								</td>
							</tr>
						</tbody>
					</table>
				</div>
			</template>
			<p v-else class="m-0 text-sm leading-6 text-primary">
				{{ formatMessage(messages.hostWaiting) }}
			</p>
		</div>
		<template #actions>
			<div class="ml-auto flex w-full items-center justify-end gap-2">
				<Button type="colored" color="red" :disabled="busy" @click="askCloseRoom">
					{{ formatMessage(messages.closeRoom) }}
				</Button>
			</div>
		</template>
	</NewModal>

	<NewModal
		ref="joinModalRef"
		v-if="joinModal"
		:header="formatMessage(messages.joinModalTitle)"
		max-width="560px"
		:on-hide="() => (joinModal = false)"
	>
		<div class="flex flex-col gap-4">
			<p class="m-0 text-sm leading-6 text-primary">
				{{ formatMessage(messages.joinDescription) }}
			</p>
			<Input
				v-model="roomInput"
				type="text"
				:placeholder="formatMessage(messages.joinPlaceholder)"
				wrapper-class="w-full"
			/>
		</div>
		<template #actions>
			<div class="ml-auto flex w-full items-center justify-end gap-2">
				<Button @click="joinModal = false">{{ formatMessage(messages.close) }}</Button>
				<Button
					type="colored"
					color="brand"
					:disabled="busy || !roomInput.trim()"
					@click="joinRoom"
				>
					{{ formatMessage(messages.joinAction) }}
				</Button>
			</div>
		</template>
	</NewModal>

	<NewModal
		ref="successModalRef"
		v-if="successModal"
		:header="formatMessage(messages.successTitle)"
		max-width="460px"
		:on-hide="() => (successModal = false)"
	>
		<p class="m-0 text-sm leading-6 text-primary">
			{{ formatMessage(messages.successBody) }}
		</p>
		<template #actions>
			<div class="ml-auto flex w-full items-center justify-end gap-2">
				<Button type="colored" color="brand" @click="successModal = false">
					{{ formatMessage(messages.close) }}
				</Button>
			</div>
		</template>
	</NewModal>

	<NewModal
		ref="closeRoomModalRef"
		v-if="closeRoomModal"
		:header="formatMessage(messages.warningTitle)"
		max-width="520px"
		:fade="'danger'"
		:on-hide="() => (closeRoomModal = false)"
	>
		<p class="m-0 text-sm leading-6 text-primary">
			{{ formatMessage(messages.warningBody) }}
		</p>
		<template #actions>
			<div class="ml-auto flex w-full items-center justify-end gap-2">
				<Button type="colored" color="red" :disabled="busy" @click="confirmCloseRoom">
					{{ formatMessage(messages.warningConfirm) }}
				</Button>
			</div>
		</template>
	</NewModal>

	<NewModal
		ref="introModalRef"
		v-if="showIntro"
		:header="formatMessage(messages.introTitle)"
		max-width="820px"
		:on-hide="() => (showIntro = false)"
	>
		<div class="flex flex-col gap-4">
			<p class="m-0 text-sm leading-6 text-primary">{{ formatMessage(messages.introBody) }}</p>
			<p class="m-0 text-sm leading-6 text-primary">
				{{ formatMessage(messages.introAuto)
				}}<!--
			--><a
					class="text-brand hover:underline"
					href="https://burningtnt.github.io/Terracotta/"
					target="_blank"
					rel="noreferrer"
					>{{ formatMessage(messages.officialSite) }}</a
				>。
			</p>
			<p class="m-0 text-sm leading-6 text-secondary">
				{{ formatMessage(messages.introDisclaim) }}
			</p>
		</div>
		<template #actions>
			<div class="ml-auto flex w-full items-center justify-end gap-2">
				<Button :disabled="downloading" @click="showIntro = false">
					{{ formatMessage(messages.cancel) }}
				</Button>
				<Button type="colored" color="brand" :disabled="downloading" @click="download">
					{{ downloading ? formatMessage(messages.downloading) : formatMessage(messages.download) }}
				</Button>
			</div>
		</template>
	</NewModal>
</template>
