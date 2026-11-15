<script setup lang="ts">
import { computed } from 'vue'

import {
	backgroundStyle,
	symbolAsset,
	versionIconBackground,
	versionIconSymbol,
} from '@/composables/mc-version-icons'
import type { McVersionKind } from '@/composables/mc-versions'

const props = withDefaults(
	defineProps<{
		kind: McVersionKind
		size?: number
	}>(),
	{
		size: 48,
	},
)

const style = computed(() => backgroundStyle(versionIconBackground(props.kind)))
const symbol = computed(() => symbolAsset(versionIconSymbol(props.kind)))
const symbolSize = computed(() => Math.round(props.size * 0.62))
</script>

<template>
	<div
		class="flex shrink-0 items-center justify-center overflow-hidden"
		:style="[
			{
				width: `${size}px`,
				height: `${size}px`,
				borderRadius: `${Math.round(size * 0.24)}px`,
			},
			style,
		]"
	>
		<img
			v-if="symbol"
			:src="symbol"
			alt=""
			:style="{ width: `${symbolSize}px`, height: `${symbolSize}px` }"
			class="object-contain drop-shadow-[0_2px_4px_rgba(0,0,0,0.28)]"
		/>
	</div>
</template>
