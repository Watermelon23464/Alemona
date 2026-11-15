import { invoke } from '@tauri-apps/api/core'

export type McVersionKind =
	| 'release'
	| 'snapshot'
	| 'old'
	| 'fools'
	| 'fabric'
	| 'forge'
	| 'neoforge'
	| 'quilt'
	| 'optifine'
	| 'liteloader'

export type McVersionFilter = 'release' | 'snapshot' | 'old' | 'fools'

export interface McVersion {
	id: string
	type: string
	released: string
	kind: McVersionFilter
}

/** One OptiFine build of a Minecraft version, mirroring optifine.net rows. */
export interface OptifineVersion {
	patch: string
	type: string
	filename: string
	/** `Forge 47.2.18`, or `Forge N/A` when the build has no Forge support. */
	forge?: string | null
}

/** LiteLoader only has one build per supported Minecraft version. */
export interface LiteLoaderBuild {
	mcversion?: string | null
	version?: string | null
	build?: string | null
	file?: string | null
	tweakClass?: string | null
	libraries?: unknown[]
}

export interface LoaderVersion {
	id: string
	stable: boolean
	optifine?: OptifineVersion
	liteloader?: LiteLoaderBuild
	/** Forge version matching an OptiFine build, if it has one. */
	forge?: string | null
}

/** Snapshots that were released as April Fools jokes. */
const APRIL_FOOLS_IDS = new Set([
	'15w14a',
	'1.rv-pre1',
	'3d shareware v1.34',
	'20w14infinite',
	'22w13oneblockatatime',
	'23w13a_or_b',
	'24w14potato',
	'25w14craftmine',
	'26w14a',
])

function looksLikeAprilFools(id: string): boolean {
	const lowered = id.toLowerCase()
	return (
		APRIL_FOOLS_IDS.has(id) ||
		lowered.includes('potato') ||
		lowered.includes('oneblock') ||
		lowered.includes('or_b') ||
		lowered.includes('shareware') ||
		lowered.includes('infinite') ||
		lowered.includes('rv-pre') ||
		lowered.includes('craftmine') ||
		lowered.includes('love') ||
		lowered.includes('joke')
	)
}

export function classifyVersion(type: string, id: string): McVersionFilter {
	if (type === 'release') return 'release'
	if (type === 'old_beta' || type === 'old_alpha') return 'old'
	if (looksLikeAprilFools(id)) return 'fools'
	return 'snapshot'
}

function readString(source: Record<string, unknown>, ...keys: string[]): string {
	for (const key of keys) {
		const value = source[key]
		if (typeof value === 'string' && value) return value
	}
	return ''
}

/** Reads the Mojang version catalog as a flat, sorted list. */
export async function fetchMcVersions(): Promise<McVersion[]> {
	const manifest = (await invoke('plugin:metadata|metadata_get_game_versions')) as {
		versions?: Record<string, unknown>[]
	}
	const versions = Array.isArray(manifest?.versions) ? manifest.versions : []
	return versions
		.map((entry) => {
			const id = readString(entry, 'id')
			const type = readString(entry, 'ty', 'type', 'version_type') || 'release'
			return {
				id,
				type,
				released: readString(entry, 'releaseTime', 'release_time', 'time'),
				kind: classifyVersion(type, id),
			}
		})
		.filter((version) => version.id)
		.sort((a, b) => new Date(b.released).getTime() - new Date(a.released).getTime())
}

/** Loader versions that support a given Minecraft version. */
export async function fetchLoaderVersions(
	loader: string,
	gameVersion: string,
): Promise<LoaderVersion[]> {
	if (loader === 'optifine') {
		const versions = await fetchOptifineVersions(gameVersion)
		return versions.map((entry) => ({
			id: `${entry.type}_${entry.patch}`,
			stable: !/[a-z]/i.test(entry.patch),
			optifine: entry,
			forge: optifineForgeVersion(entry),
		}))
	}

	if (loader === 'liteloader') {
		const build = await fetchLiteLoaderBuild(gameVersion)
		if (!build) return []
		return [
			{
				id: build.version ?? build.build ?? 'SNAPSHOT',
				stable: true,
				liteloader: build,
			},
		]
	}

	try {
		const manifest = (await invoke('plugin:metadata|metadata_get_loader_versions', {
			loader: loader === 'neoforge' ? 'neo' : loader,
		})) as {
			gameVersions?: Record<string, unknown>[]
			versionGroups?: Record<string, unknown>[]
		}
		const entries = Array.isArray(manifest?.gameVersions) ? manifest.gameVersions : []
		// Mirrors the Modrinth App loader resolution:
		// - a placeholder entry holds the loader versions that work with every
		//   supported Minecraft version, but only when that game version is listed
		// - a `versionGroup` points at a shared list in `versionGroups`
		const ALL_GAME_VERSIONS = '${modrinth.gameVersion}'
		const placeholder = entries.find((entry) => readString(entry, 'id') === ALL_GAME_VERSIONS)
		if (placeholder) {
			if (!entries.some((entry) => readString(entry, 'id') === gameVersion)) return []
			return mapLoaders(placeholder.loaders)
		}

		const entry = entries.find((candidate) => readString(candidate, 'id') === gameVersion)
		if (!entry) return []

		const group = readString(entry, 'versionGroup')
		if (group) {
			const groups = Array.isArray(manifest?.versionGroups) ? manifest.versionGroups : []
			const versionGroup = groups.find((candidate) => readString(candidate, 'id') === group)
			return mapLoaders(versionGroup?.loaders)
		}

		return mapLoaders(entry.loaders)
	} catch {
		return []
	}
}

function mapLoaders(raw: unknown): LoaderVersion[] {
	if (!Array.isArray(raw)) return []
	return raw
		.map((entry) => {
			const value = (entry ?? {}) as Record<string, unknown>
			return {
				id: readString(value, 'id'),
				stable: value.stable === true,
			}
		})
		.filter((entry) => entry.id)
}

export interface CreateInstanceOptions {
	name: string
	gameVersion: string
	loader: string
	loaderVersion?: string | null
	iconPath?: string | null
	iconConfig?: unknown
}

/** Starts an instance install job and returns the new instance id. */
export async function createMinecraftInstance(
	options: CreateInstanceOptions,
): Promise<string | null> {
	const job = (await invoke('plugin:install|install_create_instance', {
		request: {
			name: options.name,
			gameVersion: options.gameVersion,
			loader: options.loader,
			loaderVersion: options.loaderVersion ?? null,
			iconPath: options.iconPath ?? null,
			iconConfig: options.iconConfig ?? null,
		},
	})) as { instance_id?: string; target?: { instance_id?: string } }
	return job?.instance_id ?? job?.target?.instance_id ?? null
}

/** OptiFine builds published for a Minecraft version, newest first. */
export async function fetchOptifineVersions(gameVersion: string): Promise<OptifineVersion[]> {
	try {
		const versions = (await invoke('plugin:metadata|metadata_get_optifine_versions', {
			mcVersion: gameVersion,
		})) as OptifineVersion[]
		return Array.isArray(versions) ? versions : []
	} catch {
		return []
	}
}

/** The single LiteLoader build for a Minecraft version, when it exists. */
export async function fetchLiteLoaderBuild(gameVersion: string): Promise<LiteLoaderBuild | null> {
	try {
		return (await invoke('plugin:metadata|metadata_get_liteloader_build', {
			mcVersion: gameVersion,
		})) as LiteLoaderBuild | null
	} catch {
		return null
	}
}

/**
 * Forge pairing of an OptiFine build, if it has one.
 *
 * Modern builds report a full version (`Forge 47.2.18`), legacy ones report a
 * build number (`Forge #2847`). The build number is kept as `#2847` and
 * resolved against the Forge version list later.
 */
export function optifineForgeVersion(entry: OptifineVersion): string | null {
	const forge = (entry.forge ?? '').trim()
	if (!forge || /n\/a/i.test(forge)) return null
	const dotted = forge.match(/(\d+\.\d+(?:\.\d+)*)/)
	if (dotted) return dotted[1]
	const build = forge.match(/#(\d+)/)
	return build ? `#${build[1]}` : null
}

/** Whether the build can be paired with Forge at all. */
export function optifineHasForge(entry: OptifineVersion): boolean {
	const forge = (entry.forge ?? '').trim()
	return Boolean(forge) && !/n\/a/i.test(forge)
}

/**
 * Picks the Forge loader version that matches an OptiFine build.
 * Legacy build numbers are matched against the last segment of the version.
 */
export function resolveForgeVersion(
	entry: OptifineVersion,
	forgeVersions: LoaderVersion[],
): string | null {
	const forge = optifineForgeVersion(entry)
	if (!forge) return null
	if (!forge.startsWith('#')) {
		return forgeVersions.some((candidate) => candidate.id === forge)
			? forge
			: (forgeVersions[0]?.id ?? forge)
	}
	const build = forge.slice(1)
	return (
		forgeVersions.find((candidate) => candidate.id.endsWith(`.${build}`))?.id ??
		forgeVersions[0]?.id ??
		null
	)
}

/** Installs OptiFine as a standalone launchwrapper tweaker profile. */
export async function installOptifineStandalone(
	instanceId: string,
	gameVersion: string,
	kind: string,
	patch: string,
): Promise<void> {
	await invoke('plugin:install|install_optifine_standalone', {
		instanceId,
		mcVersion: gameVersion,
		kind,
		patch,
	})
}

/** Installs LiteLoader as a launchwrapper tweaker profile. */
export async function installLiteLoader(
	instanceId: string,
	version: string,
	tweaker: string,
): Promise<void> {
	await invoke('plugin:install|install_liteloader', {
		instanceId,
		version,
		tweaker,
	})
}

/** Downloads an OptiFine build into an existing instance's mods folder. */
export async function installOptifine(
	instanceId: string,
	gameVersion: string,
	kind: string,
	patch: string,
): Promise<string> {
	return (await invoke('plugin:install|install_optifine', {
		instanceId,
		mcVersion: gameVersion,
		kind,
		patch,
	})) as string
}
