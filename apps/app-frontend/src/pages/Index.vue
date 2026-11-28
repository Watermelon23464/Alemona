<script setup lang="ts">
import { defineMessages, useVIntl } from '@modrinth/ui'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import LibrarySection from '@/components/ui/library/index.vue'
import { useAppSettings } from '@/composables/use-app-settings.ts'
import { homeBackgroundImage } from '@/composables/use-home-background.ts'
import {
	formatPlaytime,
	substituteWelcomeVars,
	welcomeBlocks,
	welcomeTemplate,
} from '@/composables/use-welcome-template.ts'
import { get_default_user, users } from '@/helpers/auth'
import { get_recent_worlds } from '@/helpers/worlds'
import { instanceListQueryOptions } from '@/pages/instance/query-options'

defineOptions({ name: 'HomePage' })

const { formatMessage } = useVIntl()

const messages = defineMessages({
	news: { id: 'app.home.news', defaultMessage: 'Recent news' },
	newsEmpty: { id: 'app.home.news.empty', defaultMessage: 'No news right now.' },
	expandMore: { id: 'app.home.news.expand-more', defaultMessage: 'Expand More' },
	collapse: { id: 'app.home.news.collapse', defaultMessage: 'Collapse' },
})

type MinecraftNewsItem = { title: string; date: string; image: string | null; link: string }

const CONTENT_BASE = 'https://launchercontent.mojang.com'

function normaliseImage(url: string | null | undefined): string | null {
	if (!url) return null
	return url.startsWith('http') ? url : CONTENT_BASE + url
}

type JsonRecord = Record<string, unknown>

function asRecord(value: unknown): JsonRecord {
	return typeof value === 'object' && value !== null ? (value as JsonRecord) : {}
}

function asString(value: unknown): string | undefined {
	return typeof value === 'string' && value ? value : undefined
}

async function fetchMinecraftNews(): Promise<MinecraftNewsItem[]> {
	const endpoints = [`${CONTENT_BASE}/v2/news.json`]

	for (const endpoint of endpoints) {
		try {
			const response = await fetch(endpoint)
			if (!response.ok) continue
			const payload = asRecord(await response.json())
			const rawEntries = payload.entries ?? payload.articles ?? payload
			const entries = Array.isArray(rawEntries) ? rawEntries : []
			const items = entries
				.map((entry) => {
					const item = asRecord(entry)
					const id = asString(item.id)
					return {
						title: asString(item.title) ?? '',
						date: asString(item.date) ?? asString(item.published) ?? '',
						image: normaliseImage(
							asString(asRecord(item.newsPageImage).url) ??
								asString(asRecord(item.playPageImage).url) ??
								asString(asRecord(item.image).url) ??
								asString(item.imageURL),
						),
						link:
							asString(item.link) ??
							asString(item.readMoreLink) ??
							(id ? `https://www.minecraft.net/en-us/article/${id}` : ''),
					}
				})
				.filter((entry) => entry.title && entry.link)
			if (items.length) {
				return items.sort((a, b) => new Date(b.date).getTime() - new Date(a.date).getTime())
			}
		} catch {
			continue
		}
	}
	return []
}

const instancesQuery = useQuery(instanceListQueryOptions())
const instances = computed(() => instancesQuery.data.value ?? [])

const newsQuery = useQuery({
	queryKey: ['minecraft-net-news'],
	queryFn: fetchMinecraftNews,
	staleTime: 60 * 60 * 1000,
	retry: false,
})
const news = computed(() => (newsQuery.data.value ?? []) as MinecraftNewsItem[])

const playerName = ref<string | null>(null)
const launcherVersion = ref<string | null>(null)
const lastWorldName = ref<string | null>(null)
const now = ref(new Date())
let clock: ReturnType<typeof setInterval> | null = null

const lastInstance = computed(() => {
	const played = instances.value.filter((instance) => instance?.last_played)
	if (!played.length) return null
	return played.reduce((latest, instance) =>
		new Date(instance.last_played).getTime() > new Date(latest.last_played).getTime()
			? instance
			: latest,
	)
})

const totalPlaytime = computed(() =>
	instances.value.reduce(
		(total, instance) =>
			total + (instance?.submitted_time_played ?? 0) + (instance?.recent_time_played ?? 0),
		0,
	),
)

const welcomeVars = computed(() => ({
	minecraft_name: playerName.value ?? '',
	launcher_version: launcherVersion.value ?? '',
	last_play_name: lastInstance.value?.name ?? '',
	last_play_version: lastInstance.value?.game_version ?? '',
	last_play_time: lastInstance.value?.last_played
		? new Date(lastInstance.value.last_played).toLocaleString()
		: '',
	game_time: formatPlaytime(totalPlaytime.value),
	last_world: lastWorldName.value ?? '',
	time: now.value.toLocaleTimeString(),
	date: now.value.toLocaleDateString(),
}))

const welcomeRender = computed(() =>
	welcomeBlocks(welcomeTemplate.value).map((block) => {
		const text = substituteWelcomeVars(block.text, welcomeVars.value)
		if (block.kind === 'heading') {
			return { tag: 'h1' as const, class: 'm-0 text-2xl font-bold', text }
		}
		if (block.kind === 'subtitle') {
			return {
				tag: 'p' as const,
				class: 'm-0 max-w-md text-sm leading-relaxed text-secondary',
				text,
			}
		}
		return { tag: 'p' as const, class: 'm-0 max-w-md text-sm leading-relaxed', text }
	}),
)

// At most three articles are visible at a time; "Expand more" pages through
// the full feed instead of growing the column.
const NEWS_PAGE_SIZE = 3
const newsExpanded = ref(false)
const newsBodyRef = ref<HTMLElement | null>(null)
const newsBodyHeight = ref(0)

function measureNewsBody(): void {
	if (newsExpanded.value || !newsBodyRef.value) return
	const height = newsBodyRef.value.scrollHeight
	if (height > 0) newsBodyHeight.value = height
}

const visibleNews = computed(() =>
	newsExpanded.value ? news.value : news.value.slice(0, NEWS_PAGE_SIZE),
)

watch(
	() => [newsExpanded.value, visibleNews.value.length] as const,
	async () => {
		await nextTick()
		measureNewsBody()
	},
	{ immediate: true },
)

onMounted(() => {
	measureNewsBody()
	window.addEventListener('resize', measureNewsBody)
})

onBeforeUnmount(() => {
	window.removeEventListener('resize', measureNewsBody)
})

function toggleNewsExpanded() {
	newsExpanded.value = !newsExpanded.value
}

onMounted(async () => {
	try {
		const defaultId = await get_default_user()
		const accounts = (await users()) as Array<{
			profile?: { id?: string; name?: string }
		}>
		const account = accounts?.find((entry) => entry?.profile?.id === defaultId) ?? accounts?.[0]
		playerName.value = account?.profile?.name ?? null
	} catch {
		playerName.value = null
	}

	try {
		launcherVersion.value = await getVersion()
	} catch {
		launcherVersion.value = null
	}

	try {
		const worlds = await get_recent_worlds(1)
		lastWorldName.value = worlds?.[0]?.name ?? null
	} catch {
		lastWorldName.value = null
	}

	clock = setInterval(() => {
		now.value = new Date()
	}, 1000)
})

onBeforeUnmount(() => {
	if (clock) clearInterval(clock)
	clock = null
})

const appSettings = useAppSettings()

const homeBackgroundUrl = computed(() => {
	if (appSettings.getFeatureFlag('home_background_custom')) return homeBackgroundImage.value
	if (appSettings.getFeatureFlag('home_background_latest_news')) {
		return news.value[0]?.image ?? null
	}
	return null
})

await instancesQuery.suspense().catch(() => {})
</script>

<template>
	<div class="relative flex min-h-0 flex-col">
		<div
			v-if="homeBackgroundUrl"
			class="pointer-events-none absolute inset-x-0 top-0 h-[340px] z-0"
		>
			<img :src="homeBackgroundUrl" alt="" class="size-full object-cover" />
			<div class="absolute inset-0 bg-gradient-to-b from-black/30 via-black/10 to-transparent" />
			<div
				class="absolute inset-x-0 bottom-0 h-24 bg-gradient-to-b from-transparent to-[var(--surface-1)]"
			/>
		</div>

		<div class="relative z-10 flex min-h-0 gap-4 p-6">
			<div class="flex min-w-0 flex-[2] flex-col gap-6">
				<div
					v-if="appSettings.getFeatureFlag('home_display_welcome')"
					class="flex flex-col items-start gap-2 pl-[10%] pt-6 pb-10 text-contrast"
				>
					<component
						:is="block.tag"
						v-for="(block, index) in welcomeRender"
						:key="index"
						:class="[block.class, 'whitespace-pre-line']"
					>
						{{ block.text }}
					</component>
				</div>

				<Suspense>
					<LibrarySection :instances="instances" />
				</Suspense>
			</div>

			<aside
				v-if="appSettings.getFeatureFlag('home_news_menu')"
				class="flex w-[21rem] shrink-0 flex-col gap-4 self-start p-1 xl:w-[23rem]"
			>
				<h2 class="m-0 text-base font-medium text-primary">
					{{ formatMessage(messages.news) }}
				</h2>

				<div
					ref="newsBodyRef"
					class="flex min-h-0 flex-col gap-4"
					:class="newsExpanded ? 'overflow-y-auto pr-1' : ''"
					:style="newsBodyHeight ? { height: `${newsBodyHeight}px` } : undefined"
				>
					<p v-if="news.length === 0" class="m-0 text-sm text-secondary">
						{{ formatMessage(messages.newsEmpty) }}
					</p>

					<a
						v-for="item in visibleNews"
						:key="item.link"
						:href="item.link"
						target="_blank"
						rel="noreferrer"
						class="group flex flex-col gap-4 transition-all ease-in-out hover:brightness-125"
					>
						<img
							v-if="item.image"
							:src="item.image"
							alt=""
							class="w-full shrink-0 rounded-xl border border-solid border-button-border"
							@load="measureNewsBody"
						/>
						<div class="flex flex-col gap-2">
							<h3 class="m-0 text-base leading-tight text-primary group-hover:underline">
								{{ item.title }}
							</h3>
							<div class="text-sm text-secondary">{{ item.date }}</div>
						</div>
					</a>
				</div>

				<button
					v-if="news.length > NEWS_PAGE_SIZE"
					class="m-0 w-fit border-0 bg-transparent p-0 text-sm font-medium text-brand hover:underline"
					@click="toggleNewsExpanded"
				>
					{{ formatMessage(newsExpanded ? messages.collapse : messages.expandMore) }}
				</button>
			</aside>
		</div>
	</div>
</template>
