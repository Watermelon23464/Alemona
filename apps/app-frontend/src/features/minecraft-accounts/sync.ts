import { ref } from 'vue'

/**
 * Bumped whenever Minecraft accounts change, so every place that lists them
 * (the settings page, the account card, the sidebar) refreshes together.
 */
export const accountsRevision = ref(0)

export function notifyAccountsChanged(): void {
	accountsRevision.value += 1
}
