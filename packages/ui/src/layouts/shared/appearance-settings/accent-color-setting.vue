<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

import { Button, IconButton } from '#ui/components/base/buttons'
import Checkbox from '#ui/components/base/Checkbox.vue'
import NewModal from '#ui/components/modal/NewModal.vue'
import { defineMessages, useVIntl } from '#ui/composables'

import AppearanceSettingRow from './components/AppearanceSettingRow.vue'
import {
	autoDeriveAccent,
	hexToRgb,
	hslToHex,
	primaryAccent,
	resetPrimaryAccent,
	resetSecondaryAccent,
	rgbToHex,
	rgbToHsl,
	secondaryAccent,
	setAutoDeriveAccent,
	setPrimaryAccent,
	setSecondaryAccent,
} from '#ui/composables/accent-color'

const props = withDefaults(defineProps<{ kind?: 'primary' | 'secondary' }>(), {
	kind: 'primary',
})

const { formatMessage } = useVIntl()

const messages = defineMessages({
	primaryTitle: { id: 'settings.display.accent.title', defaultMessage: 'Accent colour' },
	primaryDescription: {
		id: 'settings.display.accent.description',
		defaultMessage:
			'The accent colour is the secondary colour used across Alemona Launcher wherever a choice needs to stand out.',
	},
	secondaryTitle: {
		id: 'settings.display.accent.secondary-title',
		defaultMessage: 'Secondary accent colour',
	},
	secondaryDescription: {
		id: 'settings.display.accent.secondary-description',
		defaultMessage:
			'The secondary accent colour is used for highlight surfaces such as selected menu backgrounds and soft badges.',
	},
	autoDerive: {
		id: 'settings.display.accent.auto',
		defaultMessage: 'Automatically calculate the other colour',
	},
	autoDeriveDescription: {
		id: 'settings.display.accent.auto-description',
		defaultMessage:
			'When enabled, changing one accent colour recalculates the other the same way the default purple pair is built.',
	},
	pick: { id: 'settings.display.accent.pick', defaultMessage: 'Pick a colour' },
	pickerTitle: { id: 'settings.display.accent.picker-title', defaultMessage: 'Colour picker' },
	reset: { id: 'settings.display.accent.reset', defaultMessage: 'Reset to the default purple' },
	confirm: { id: 'settings.display.accent.confirm', defaultMessage: 'Done' },
	red: { id: 'settings.display.accent.red', defaultMessage: 'Red' },
	green: { id: 'settings.display.accent.green', defaultMessage: 'Green' },
	blue: { id: 'settings.display.accent.blue', defaultMessage: 'Blue' },
	hex: { id: 'settings.display.accent.hex', defaultMessage: 'Hex' },
	alpha: { id: 'settings.display.accent.alpha', defaultMessage: 'Alpha' },
})

const pickerOpen = ref(false)
const pickerModal = ref<{ show?: () => void; hide?: () => void } | null>(null)

const color = computed(() =>
	props.kind === 'primary' ? primaryAccent.value : secondaryAccent.value,
)

const hsv = ref(toHsv(color.value))
watch(color, (next) => {
	if (hexColor.value.toLowerCase() !== next.toLowerCase()) hsv.value = toHsv(next)
})

const hexColor = computed(() => hsvToHex(hsv.value.h, hsv.value.s, hsv.value.v))
const rgb = computed(() => hexToRgb(hexColor.value))

function toHsv(hex: string) {
	const [r, g, b] = hexToRgb(hex)
	const max = Math.max(r, g, b)
	const min = Math.min(r, g, b)
	const delta = max - min
	let h = 0
	if (delta !== 0) {
		h =
			max === r
				? 60 * (((g - b) / delta) % 6)
				: max === g
					? 60 * ((b - r) / delta + 2)
					: 60 * ((r - g) / delta + 4)
	}
	return { h: (h + 360) % 360, s: max === 0 ? 0 : delta / max, v: max / 255 }
}

function hsvToHex(h: number, s: number, v: number): string {
	const chroma = v * s
	const secondary = chroma * (1 - Math.abs(((h / 60) % 2) - 1))
	const m = v - chroma
	const [r, g, b] =
		h < 60
			? [chroma, secondary, 0]
			: h < 120
				? [secondary, chroma, 0]
				: h < 180
					? [0, chroma, secondary]
					: h < 240
						? [0, secondary, chroma]
						: h < 300
							? [secondary, 0, chroma]
							: [chroma, 0, secondary]
	return rgbToHex((r + m) * 255, (g + m) * 255, (b + m) * 255)
}

function commit(hex: string) {
	if (props.kind === 'primary') setPrimaryAccent(hex)
	else setSecondaryAccent(hex)
}

function pickFromSquare(event: PointerEvent) {
	const target = event.currentTarget as HTMLElement
	const rect = target.getBoundingClientRect()
	hsv.value = {
		...hsv.value,
		s: Math.min(1, Math.max(0, (event.clientX - rect.left) / rect.width)),
		v: 1 - Math.min(1, Math.max(0, (event.clientY - rect.top) / rect.height)),
	}
	commit(hexColor.value)
}

function onSquarePointerDown(event: PointerEvent) {
	;(event.currentTarget as HTMLElement).setPointerCapture?.(event.pointerId)
	pickFromSquare(event)
}

function onSquarePointerMove(event: PointerEvent) {
	if (event.buttons !== 1) return
	pickFromSquare(event)
}

function onHueInput(event: Event) {
	hsv.value = { ...hsv.value, h: Number((event.target as HTMLInputElement).value) }
	commit(hexColor.value)
}

function syncFromHex(hex: string) {
	const normalised = (hex.startsWith('#') ? hex : `#${hex}`).toLowerCase()
	if (!/^#[0-9a-f]{6}$/.test(normalised)) return
	hsv.value = toHsv(normalised)
	commit(normalised)
}

function setChannel(channel: 'r' | 'g' | 'b', event: Event) {
	const [r, g, b] = rgb.value
	const next = Math.max(0, Math.min(255, Number((event.target as HTMLInputElement).value) || 0))
	syncFromHex(
		rgbToHex(channel === 'r' ? next : r, channel === 'g' ? next : g, channel === 'b' ? next : b),
	)
}

async function openPicker() {
	hsv.value = toHsv(color.value)
	pickerOpen.value = true
	await nextTick()
	pickerModal.value?.show?.()
}

function closePicker() {
	pickerModal.value?.hide?.()
}

function reset() {
	if (props.kind === 'primary') resetPrimaryAccent()
	else resetSecondaryAccent()
}

const title = computed(() =>
	formatMessage(props.kind === 'primary' ? messages.primaryTitle : messages.secondaryTitle),
)
const description = computed(() =>
	formatMessage(
		props.kind === 'primary' ? messages.primaryDescription : messages.secondaryDescription,
	),
)
</script>

<template>
	<div class="flex flex-col gap-2 pt-2">
		<div class="flex flex-col gap-1">
			<h3 :id="`accent-${kind}-label`" class="m-0 text-lg font-semibold text-contrast">
				{{ title }}
			</h3>
			<p class="m-0 text-secondary">{{ description }}</p>
		</div>

		<div class="mt-2 flex flex-col gap-1">
			<div class="flex w-full items-center gap-3">
				<input
					:id="`accent-${kind}`"
					:aria-labelledby="`accent-${kind}-label`"
					type="range"
					min="0"
					max="359"
					step="1"
					:value="hsv.h"
					class="accent-slider min-w-0 flex-1"
					@input="onHueInput"
				/>
				<IconButton :label="formatMessage(messages.pick)" size="sm" @click="openPicker">
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
						class="size-4"
					>
						<path d="M12 21a9 9 0 1 1 0-18c4.97 0 9 3.58 9 8 0 2.21-1.79 4-4 4h-2a2 2 0 0 0 0 4z" />
						<circle cx="8" cy="10" r="1" fill="currentColor" stroke="none" />
						<circle cx="12" cy="7.5" r="1" fill="currentColor" stroke="none" />
						<circle cx="16" cy="10" r="1" fill="currentColor" stroke="none" />
					</svg>
				</IconButton>
				<IconButton :label="formatMessage(messages.reset)" size="sm" @click="reset">
					<svg
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="1.8"
						class="size-4"
					>
						<path d="M4 10a8 8 0 1 1 2.3 6" />
						<path d="M4 5v5h5" />
					</svg>
				</IconButton>
			</div>
			<span class="flex items-center gap-1.5 font-mono text-xs text-secondary">
				<span
					class="size-2.5 rounded-sm border border-solid border-surface-5"
					:style="{ background: color }"
				/>
				{{ hexColor.toUpperCase() }}
			</span>
		</div>

		<AppearanceSettingRow
			v-if="kind === 'secondary'"
			class="mt-4"
			control-id="accent-auto-derive"
			:heading-level="3"
			:title="formatMessage(messages.autoDerive)"
			:description="formatMessage(messages.autoDeriveDescription)"
		>
			<template #default="{ labelledBy }">
				<Checkbox
					id="accent-auto-derive"
					:aria-labelledby="labelledBy"
					:model-value="autoDeriveAccent"
					@update:model-value="setAutoDeriveAccent"
				/>
			</template>
		</AppearanceSettingRow>
	</div>

	<NewModal
		v-if="pickerOpen"
		ref="pickerModal"
		:header="formatMessage(messages.pickerTitle)"
		max-width="520px"
		:on-hide="() => undefined"
		:on-after-hide="() => (pickerOpen = false)"
	>
		<div class="flex flex-col gap-4">
			<div class="flex gap-3">
				<div
					class="relative h-48 flex-1 cursor-crosshair touch-none rounded-lg border border-solid border-surface-5"
					:style="{
						backgroundColor: `hsl(${hsv.h} 100% 50%)`,
						backgroundImage:
							'linear-gradient(to top, #000, transparent), linear-gradient(to right, #fff, transparent)',
					}"
					@pointerdown="onSquarePointerDown"
					@pointermove="onSquarePointerMove"
				>
					<span
						class="pointer-events-none absolute size-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-solid border-white shadow"
						:style="{
							left: `${hsv.s * 100}%`,
							top: `${(1 - hsv.v) * 100}%`,
							background: hexColor,
						}"
					/>
				</div>
				<input
					type="range"
					min="0"
					max="359"
					:value="hsv.h"
					class="hue-slider"
					@input="onHueInput"
				/>
			</div>

			<div class="grid grid-cols-5 gap-2">
				<label class="flex flex-col gap-1 text-xs font-medium text-secondary">
					{{ formatMessage(messages.red) }}
					<input
						type="number"
						min="0"
						max="255"
						class="w-full rounded-md border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm text-contrast"
						:value="rgb[0]"
						@input="setChannel('r', $event)"
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs font-medium text-secondary">
					{{ formatMessage(messages.green) }}
					<input
						type="number"
						min="0"
						max="255"
						class="w-full rounded-md border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm text-contrast"
						:value="rgb[1]"
						@input="setChannel('g', $event)"
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs font-medium text-secondary">
					{{ formatMessage(messages.blue) }}
					<input
						type="number"
						min="0"
						max="255"
						class="w-full rounded-md border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm text-contrast"
						:value="rgb[2]"
						@input="setChannel('b', $event)"
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs font-medium text-secondary">
					{{ formatMessage(messages.hex) }}
					<input
						type="text"
						class="w-full rounded-md border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm text-contrast"
						:value="hexColor"
						@input="syncFromHex(($event.target as HTMLInputElement).value)"
					/>
				</label>
				<label class="flex flex-col gap-1 text-xs font-medium text-secondary">
					{{ formatMessage(messages.alpha) }}
					<input
						type="number"
						min="0"
						max="100"
						value="100"
						class="w-full rounded-md border border-solid border-surface-5 bg-surface-4 px-2 py-1 text-sm text-contrast"
					/>
				</label>
			</div>

			<div class="flex justify-end">
				<Button type="colored" color="brand" @click="closePicker">
					{{ formatMessage(messages.confirm) }}
				</Button>
			</div>
		</div>
	</NewModal>
</template>

<style scoped>
.accent-slider {
	height: 1.25rem;
	appearance: none;
	border-radius: 0.5rem;
	border: 1px solid var(--surface-5);
	background: linear-gradient(to right, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
	cursor: pointer;
	transition: transform 120ms ease-out;
	margin: 0;
}

.accent-slider:active {
	transform: scale(0.99);
}

.accent-slider::-webkit-slider-thumb {
	appearance: none;
	width: 0.75rem;
	height: 1.5rem;
	border-radius: 0.25rem;
	border: 2px solid #fff;
	background:
		linear-gradient(
			to right,
			transparent calc(50% - 1px),
			rgb(0 0 0 / 55%) calc(50% - 1px),
			rgb(0 0 0 / 55%) calc(50% + 1px),
			transparent calc(50% + 1px)
		),
		var(--color-brand);
	box-shadow: 0 1px 3px rgb(0 0 0 / 40%);
	transition: transform 120ms ease-out;
}

.accent-slider:hover::-webkit-slider-thumb {
	transform: scale(1.08);
}

.hue-slider {
	writing-mode: vertical-lr;
	direction: rtl;
	width: 1.5rem;
	height: 12rem;
	appearance: none;
	border-radius: 0.5rem;
	border: 1px solid var(--surface-5);
	background: linear-gradient(to top, #f00, #ff0, #0f0, #0ff, #00f, #f0f, #f00);
	cursor: pointer;
}

.hue-slider::-webkit-slider-thumb {
	appearance: none;
	width: 1.25rem;
	height: 0.75rem;
	border-radius: 0.25rem;
	border: 2px solid #fff;
	background: transparent;
}
</style>
