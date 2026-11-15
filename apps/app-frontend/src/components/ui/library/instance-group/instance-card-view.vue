<script setup lang="ts">
import {
	Avatar,
	defineMessages,
	TagIcon,
	truncatedTooltip,
	useRelativeTime,
	useVIntl,
} from '@modrinth/ui'
import { computed, inject, type Ref, ref } from 'vue'

import { useAppSettings } from '@/composables/use-app-settings.ts'
import { useImageThumbnail } from '@/composables/use-image-thumbnail'
import { getInstanceIconUrl } from '@/helpers/instance'
import type { GameInstance } from '@/helpers/types'

const props = withDefaults(
	defineProps<{
		instance: GameInstance
		selected?: boolean
	}>(),
	{
		selected: false,
	},
)

const pageMode = inject<Ref<boolean>>('libraryPageMode', ref(false))
const { formatMessage } = useVIntl()
const relativeTime = useRelativeTime({ numeric: 'auto' })
const messages = defineMessages({
	neverPlayed: {
		id: 'app.library.instance.never-played',
		defaultMessage: 'Never played',
	},
})

const localIcon = computed(() => {
	const path = props.instance.icon_path
	return path && !/^https?:/.test(path) && !path.toLowerCase().endsWith('.svg') ? path : undefined
})
const appSettings = useAppSettings()
const compactMode = computed(() => appSettings.compactInstanceCards)
const thumbnail = useImageThumbnail(
	localIcon,
	() => (compactMode.value ? 96 : 384),
	() => String(props.instance.modified),
)
const iconSrc = computed(() =>
	localIcon.value ? thumbnail.value : getInstanceIconUrl(props.instance.icon_path),
)

const versionLabel = computed(() => `${props.instance.loader} ${props.instance.game_version}`)
const lastPlayedLabel = computed(() =>
	props.instance.last_played
		? relativeTime(props.instance.last_played)
		: formatMessage(messages.neverPlayed),
)

const nameRef = ref<HTMLElement | null>(null)
const versionRef = ref<HTMLElement | null>(null)
const lastPlayedRef = ref<HTMLElement | null>(null)
</script>

<template>
	<div
		class="relative flex w-full min-w-0 select-none overflow-clip border border-solid bg-surface-3 text-left transition-[background-color,border-color,box-shadow,filter]"
		:class="{
			'flex-row items-center justify-start gap-2.5 rounded-xl p-2.5': compactMode,
			'flex-col items-start justify-end gap-3 rounded-[20px] p-3': !compactMode && !pageMode,
			'flex-col items-start justify-end gap-2.5 rounded-2xl p-2.5 hover:bg-surface-4 hover:shadow-lg':
				!compactMode && pageMode,
			'[border-color:color-mix(in_srgb,var(--color-text-primary)_40%,transparent)] brightness-110':
				selected && (!pageMode || compactMode),
			'border-contrast/40 bg-surface-4 shadow-lg': selected && pageMode && !compactMode,
			'border-surface-4': !selected && !pageMode,
			'border-surface-4 hover:border-surface-5': !selected && pageMode,
		}"
	>
		<div
			class="relative flex shrink-0 items-center max-w-full overflow-clip"
			:class="
				compactMode
					? 'size-10 rounded-lg'
					: pageMode
						? 'aspect-square min-w-full rounded-xl'
						: 'aspect-square min-w-full rounded-2xl'
			"
		>
			<Avatar
				class="pointer-events-none outline-none"
				:class="compactMode ? '!rounded-lg' : pageMode ? '!rounded-xl' : '!rounded-2xl'"
				size="100%"
				:src="iconSrc"
				loading="lazy"
				:tint-by="instance.id"
				alt=""
				no-shadow
				pad-transparent-corners
			/>
			<slot name="loading" :compact="compactMode" />
			<div
				class="absolute z-[1] flex items-center justify-center"
				:class="compactMode ? 'inset-0' : 'bottom-1.5 right-1.5 size-12'"
			>
				<slot name="leading" :compact="compactMode" />
			</div>
		</div>
		<div
			class="flex min-w-0 w-full flex-col items-start justify-center px-0.5"
			:class="pageMode && !compactMode ? 'gap-0.5' : 'gap-1'"
			:style="{ paddingRight: compactMode ? '2.5rem' : undefined }"
		>
			<p
				ref="nameRef"
				v-tooltip="truncatedTooltip(nameRef, instance.name)"
				class="m-0 w-full truncate font-semibold text-contrast"
				:class="pageMode ? 'text-[0.95rem] leading-5' : 'text-base leading-5'"
			>
				{{ instance.name }}
			</p>
			<div
				v-if="pageMode"
				ref="versionRef"
				v-tooltip="truncatedTooltip(versionRef, versionLabel)"
				class="flex w-full min-w-0 items-center gap-1.5"
			>
				<TagIcon :tag="instance.loader" enforce-type="loader" class="size-4 shrink-0" />
				<span class="min-w-0 truncate text-[0.8rem] font-medium leading-[18px] text-primary">
					{{ instance.game_version }}
				</span>
			</div>
			<p
				v-else
				ref="versionRef"
				v-tooltip="truncatedTooltip(versionRef, versionLabel)"
				class="m-0 w-full truncate text-sm font-medium capitalize leading-[18px] text-primary"
			>
				{{ versionLabel }}
			</p>
			<p
				v-if="pageMode && !compactMode"
				ref="lastPlayedRef"
				v-tooltip="truncatedTooltip(lastPlayedRef, lastPlayedLabel)"
				class="m-0 w-full truncate text-xs font-medium leading-4 text-secondary"
			>
				{{ lastPlayedLabel }}
			</p>
		</div>
		<slot name="overlay" :compact="compactMode" />
	</div>
</template>
