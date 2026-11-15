import {
	backgroundOptions,
	symbolOptions,
} from '@/components/ui/instance_settings/icon-editor-modal/editor-catalog'
import type { McVersionKind } from '@/composables/mc-versions'
import type { IconBackground, InstanceIconConfig } from '@/helpers/types'

/** Reuses the Modrinth App instance icon backgrounds and symbols. */
const VERSION_ICONS: Record<McVersionKind, { background: string; symbol: string }> = {
	release: { background: 'yellow', symbol: 'grass_block' },
	snapshot: { background: 'purple', symbol: 'command_block' },
	old: { background: 'light_gray', symbol: 'furnace' },
	fools: { background: 'orange', symbol: 'tnt' },
	fabric: { background: 'yellow', symbol: 'fabric' },
	forge: { background: 'yellow', symbol: 'forge' },
	neoforge: { background: 'yellow', symbol: 'neoforge' },
	quilt: { background: 'yellow', symbol: 'quilt' },
	optifine: { background: 'rose', symbol: 'enchanting_table' },
	liteloader: { background: 'blue', symbol: 'bookshelf' },
}

export function versionIconBackground(kind: McVersionKind): IconBackground | null {
	const id = VERSION_ICONS[kind].background
	return backgroundOptions.find((entry) => entry.id === id)?.background ?? null
}

export function versionIconSymbol(kind: McVersionKind): string {
	return VERSION_ICONS[kind].symbol
}

/** The icon config the launcher uses for generated instance icons. */
export function versionIconConfig(kind: McVersionKind): InstanceIconConfig {
	const background = versionIconBackground(kind)
	if (!background) throw new Error(`Unknown icon background for ${kind}`)
	return { background, symbol: versionIconSymbol(kind) }
}

export function backgroundStyle(background: IconBackground | null): Record<string, string> {
	if (!background) return {}
	if (background.type === 'color') return { backgroundColor: background.value }
	return {
		backgroundImage: `linear-gradient(to bottom, ${background.top_color}, ${background.bottom_color})`,
	}
}

export function symbolAsset(symbolId: string): string {
	return symbolOptions.find((entry) => entry.id === symbolId)?.asset ?? ''
}

/** Loads the PNG bytes of a symbol so they can be cached as an instance icon. */
export async function symbolBytes(symbolId: string): Promise<number[]> {
	const asset = symbolAsset(symbolId)
	if (!asset) throw new Error(`Unknown icon symbol ${symbolId}`)
	const response = await fetch(asset)
	if (!response.ok) throw new Error('Failed to load the icon symbol.')
	return Array.from(new Uint8Array(await response.arrayBuffer()))
}
