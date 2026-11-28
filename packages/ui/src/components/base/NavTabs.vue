<template>
	<div
		v-if="filteredLinks.length > 1"
		:class="pageNav ? '-mx-6 -mt-2 mb-1 overflow-x-auto px-6 py-2' : 'contents'"
		v-bind="pageNav ? $attrs : {}"
	>
		<nav
			ref="scrollContainer"
			class="relative flex w-fit rounded-full bg-bg-raised p-1 text-xs sm:text-sm font-bold"
			:class="{
				'card-shadow border border-solid border-surface-4': mode === 'navigation',
				'tab-color-delayed': colorChangeDelayed,
			}"
			v-bind="pageNav ? {} : $attrs"
		>
			<template v-if="mode === 'navigation'">
				<RouterLink
					v-for="(link, index) in filteredLinks"
					v-show="link.shown ?? true"
					:key="link.href"
					ref="tabLinkElements"
					:replace="replace"
					:to="query ? (link.href ? `?${query}=${link.href}` : '?') : link.href"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@mouseenter="link.onHover?.()"
					@focus="link.onHover?.()"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="tab-color hidden sm:block size-5"
						:class="getIconClasses(index)"
					/>
					<span class="tab-color text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</RouterLink>
			</template>

			<template v-else>
				<div
					v-for="(link, index) in filteredLinks"
					v-show="link.shown ?? true"
					:key="link.href"
					ref="tabLinkElements"
					class="button-animation z-[1] flex flex-row items-center gap-2 px-4 py-2 hover:cursor-pointer focus:rounded-full"
					:class="getSSRFallbackClasses(index)"
					@click="emit('tabClick', index, link)"
				>
					<component
						:is="link.icon"
						v-if="link.icon"
						class="tab-color size-5"
						:class="getIconClasses(index)"
					/>
					<span class="tab-color text-nowrap" :class="getLabelClasses(index)">
						{{ link.label }}
					</span>
				</div>
			</template>

			<div
				v-if="sliderReady && currentActiveIndex !== -1"
				class="pointer-events-none absolute rounded-full"
				:class="[
					subpageSelected ? 'bg-button-bg' : 'bg-button-bgSelected',
					{ 'navtabs-transition': transitionsEnabled },
				]"
				:style="sliderStyle"
				aria-hidden="true"
			/>
		</nav>
	</div>
</template>

<script setup lang="ts">
import type { Component } from 'vue'
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

defineOptions({ inheritAttrs: false })

interface Tab {
	label: string
	href: string
	shown?: boolean
	icon?: Component
	subpages?: string[]
	onHover?: () => void
}

const props = withDefaults(
	defineProps<{
		replace?: boolean
		links: Tab[]
		query?: string
		mode?: 'navigation' | 'local'
		activeIndex?: number
		pageNav?: boolean
	}>(),
	{
		mode: 'navigation',
		query: undefined,
		activeIndex: undefined,
		pageNav: false,
	},
)

const emit = defineEmits<{
	tabClick: [index: number, tab: Tab]
}>()

// DOM refs
const scrollContainer = ref<HTMLElement | null>(null)
const tabLinkElements = ref<HTMLElement[]>()

const sliderLeft = ref(0)
const sliderTop = ref(0)
const sliderRight = ref(0)
const sliderBottom = ref(0)

const currentActiveIndex = ref(-1)
const subpageSelected = ref(false)

const sliderReady = ref(false)
const transitionsEnabled = ref(false)
const colorChangeDelayed = ref(false)

const sliderDelays = ref({ left: '0ms', top: '0ms', right: '0ms', bottom: '0ms' })

const filteredLinks = computed(() => props.links.filter((link) => link.shown ?? true))

const sliderStyle = computed(() => ({
	left: `${sliderLeft.value}px`,
	top: `${sliderTop.value}px`,
	right: `${sliderRight.value}px`,
	bottom: `${sliderBottom.value}px`,
}))

const leftDelay = computed(() => sliderDelays.value.left)
const rightDelay = computed(() => sliderDelays.value.right)
const topDelay = computed(() => sliderDelays.value.top)
const bottomDelay = computed(() => sliderDelays.value.bottom)

const isActiveAndNotSubpage = computed(
	() => (index: number) => currentActiveIndex.value === index && !subpageSelected.value,
)

function getSSRFallbackClasses(index: number) {
	if (sliderReady.value) return {}
	if (currentActiveIndex.value !== index) return {}

	return {
		'rounded-full': true,
		'bg-button-bgSelected': !subpageSelected.value,
		'bg-button-bg': subpageSelected.value,
	}
}

function getIconClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-secondary': !isActiveAndNotSubpage.value(index),
	}
}

function getLabelClasses(index: number) {
	return {
		'text-button-textSelected': isActiveAndNotSubpage.value(index),
		'text-contrast': !isActiveAndNotSubpage.value(index),
	}
}

function computeActiveIndex(): { index: number; isSubpage: boolean } {
	if (props.mode === 'local' && props.activeIndex !== undefined) {
		return {
			index: Math.min(props.activeIndex, filteredLinks.value.length - 1),
			isSubpage: false,
		}
	}

	for (let i = filteredLinks.value.length - 1; i >= 0; i--) {
		const link = filteredLinks.value[i]
		const decodedPath = decodeURIComponent(route.path)
		const decodedHref = decodeURIComponent(link.href.split('?')[0])

		if (props.query) {
			const queryValue = route.query[props.query]
			if (queryValue === link.href || (!queryValue && !link.href)) {
				return { index: i, isSubpage: false }
			}
			continue
		}

		if (decodedPath === decodedHref) {
			const hrefQuery = link.href.split('?')[1]
			if (!hrefQuery) {
				return { index: i, isSubpage: false }
			}

			// Links may carry context parameters (for example the instance an
			// install should go to), which have to match as well.
			const expected = new URLSearchParams(hrefQuery)
			const matchesContext = [...expected.entries()].every(
				([key, value]) => String(route.query[key] ?? '') === value,
			)
			if (matchesContext) {
				return { index: i, isSubpage: false }
			}
			continue
		}

		const isSubpageMatch =
			(decodedPath.startsWith(decodedHref) &&
				(decodedPath.length === decodedHref.length || decodedPath[decodedHref.length] === '/')) ||
			link.subpages?.some((subpage) => decodedPath.includes(subpage))

		if (isSubpageMatch) {
			return { index: i, isSubpage: true }
		}
	}

	return { index: -1, isSubpage: false }
}

function getTabElement(index: number): HTMLElement | null {
	if (index === -1) return null

	const container = scrollContainer.value as HTMLElement | undefined
	if (!container) return null

	const tabs = container.querySelectorAll('.button-animation')
	const element = tabs[index] as HTMLElement | undefined

	if (!element) return null

	return element
}

function getSliderPosition(index = currentActiveIndex.value) {
	const el = getTabElement(index)
	const parent = scrollContainer.value
	if (!el || !parent) return null

	return {
		left: el.offsetLeft,
		top: el.offsetTop,
		right: parent.clientWidth - el.offsetLeft - el.offsetWidth,
		bottom: parent.clientHeight - el.offsetTop - el.offsetHeight,
	}
}

function getCollapsedPosition(index: number) {
	const el = getTabElement(index)
	const parent = scrollContainer.value
	if (!el || !parent) return null

	const centre = el.offsetLeft + el.offsetWidth / 2
	return {
		left: centre,
		top: el.offsetTop,
		right: parent.clientWidth - centre,
		bottom: parent.clientHeight - el.offsetTop - el.offsetHeight,
	}
}

function applySliderPosition(newPosition: {
	left: number
	top: number
	right: number
	bottom: number
}) {
	sliderLeft.value = newPosition.left
	sliderTop.value = newPosition.top
	sliderRight.value = newPosition.right
	sliderBottom.value = newPosition.bottom
}

function animateSliderTo(newPosition: {
	left: number
	top: number
	right: number
	bottom: number
}) {
	const STAGGER_DELAY = '60ms'

	sliderDelays.value = {
		left: newPosition.left < sliderLeft.value ? '0ms' : STAGGER_DELAY,
		right: newPosition.left < sliderLeft.value ? STAGGER_DELAY : '0ms',
		top: newPosition.top < sliderTop.value ? '0ms' : STAGGER_DELAY,
		bottom: newPosition.top < sliderTop.value ? STAGGER_DELAY : '0ms',
	}

	applySliderPosition(newPosition)
}

function positionSlider(animate = true) {
	const newPosition = getSliderPosition()
	if (!newPosition) {
		return
	}

	if (!sliderReady.value) {
		applySliderPosition(newPosition)
		sliderReady.value = true
		nextTick(() => {
			requestAnimationFrame(() => {
				transitionsEnabled.value = true
			})
		})
		return
	}

	if (!animate) {
		transitionsEnabled.value = false
		applySliderPosition(newPosition)
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
		})
		return
	}

	animateSliderTo(newPosition)
}

function resetSliderPosition() {
	if (!sliderReady.value || currentActiveIndex.value === -1) {
		return
	}
	positionSlider(false)
}

async function updateActiveTab() {
	await nextTick()
	const { index, isSubpage } = computeActiveIndex()
	colorChangeDelayed.value = sliderReady.value && index !== currentActiveIndex.value
	currentActiveIndex.value = index
	subpageSelected.value = isSubpage
	rememberActiveIndex()

	if (index !== -1) {
		positionSlider()
	}
}

let lastActiveIndex = 0

const initialActive = computeActiveIndex()
currentActiveIndex.value = initialActive.index
subpageSelected.value = initialActive.isSubpage

let resizeObserver: ResizeObserver | undefined

function rememberActiveIndex(): void {
	if (currentActiveIndex.value >= 0) lastActiveIndex = currentActiveIndex.value
}

function animateIn(): void {
	const target = currentActiveIndex.value
	if (target === -1) {
		sliderReady.value = false
		return
	}

	const parent = scrollContainer.value
	if (!parent) {
		requestAnimationFrame(animateIn)
		return
	}

	const originIndex = Math.min(lastActiveIndex, filteredLinks.value.length - 1)
	const origin = originIndex !== target ? getSliderPosition(originIndex) : null

	if (origin) {
		applySliderPosition(origin)
		sliderReady.value = true
		requestAnimationFrame(() => {
			transitionsEnabled.value = true
			requestAnimationFrame(() => {
				positionSlider()
				rememberActiveIndex()
			})
		})
		return
	}

	// Same tab as before: grow the pill out of the centre of its tab.
	const collapsed = getCollapsedPosition(target)
	if (!collapsed) {
		requestAnimationFrame(animateIn)
		return
	}
	applySliderPosition(collapsed)
	sliderReady.value = true
	requestAnimationFrame(() => {
		transitionsEnabled.value = true
		requestAnimationFrame(() => {
			positionSlider()
			rememberActiveIndex()
		})
	})
}

onMounted(() => {
	nextTick(() => requestAnimationFrame(animateIn))

	resizeObserver = new ResizeObserver(resetSliderPosition)
	if (scrollContainer.value) {
		resizeObserver.observe(scrollContainer.value)
	}
})

onUnmounted(() => {
	resizeObserver?.disconnect()
	rememberActiveIndex()
})

watch(
	() => [route.path, route.query],
	() => {
		if (props.mode === 'navigation') {
			updateActiveTab()
		}
	},
)

watch(
	() => props.activeIndex,
	() => {
		if (props.mode === 'local') {
			updateActiveTab()
		}
	},
)

watch(
	() => props.links,
	async () => {
		await nextTick()
		updateActiveTab()
	},
	{ deep: true },
)
</script>

<style scoped>
.navtabs-transition {
	transition:
		left 260ms cubic-bezier(0.34, 1.2, 0.64, 1) v-bind(leftDelay),
		right 260ms cubic-bezier(0.34, 1.2, 0.64, 1) v-bind(rightDelay),
		opacity 200ms ease,
		top 260ms cubic-bezier(0.34, 1.2, 0.64, 1) v-bind(topDelay),
		bottom 260ms cubic-bezier(0.34, 1.2, 0.64, 1) v-bind(bottomDelay);
}

.tab-color {
	transition: color 100ms cubic-bezier(0.4, 0, 0.2, 1);
}

.tab-color-delayed .tab-color {
	transition-delay: 100ms;
}
</style>
