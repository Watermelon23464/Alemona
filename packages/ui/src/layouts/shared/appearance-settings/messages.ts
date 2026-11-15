import { defineMessages } from '#ui/composables'

export const appearanceSettingsMessages = defineMessages({
	colorThemeTitle: {
		id: 'settings.display.theme.title',
		defaultMessage: 'Color theme',
	},
	colorThemeDescription: {
		id: 'settings.display.theme.description',
		defaultMessage: 'Select your preferred color theme across Alemona Launcher.',
	},
	syncAcrossDevicesTitle: {
		id: 'settings.display.theme.sync-across-devices',
		defaultMessage: 'Sync theme across devices',
	},
	syncAcrossDevicesDescription: {
		id: 'settings.display.theme.sync-across-devices.description',
		defaultMessage:
			"Use this theme everywhere you're signed in. Turn this off to keep a separate theme on this device.",
	},
	syncAcrossDevicesSignedOutTooltip: {
		id: 'settings.display.theme.sync-across-devices.sign-in-tooltip',
		defaultMessage: 'Sign into Alemona Launcher to sync theme',
	},
	projectListLayoutsTitle: {
		id: 'settings.display.project-list-layouts.title',
		defaultMessage: 'Project list layouts',
	},
	projectListLayoutsDescription: {
		id: 'settings.display.project-list-layouts.description',
		defaultMessage: 'Select your preferred layout for each page that displays project lists.',
	},
	mod: {
		id: 'settings.display.project-list-layouts.mod',
		defaultMessage: 'Mods page',
	},
	plugin: {
		id: 'settings.display.project-list-layouts.plugin',
		defaultMessage: 'Plugins page',
	},
	datapack: {
		id: 'settings.display.project-list-layouts.datapack',
		defaultMessage: 'Data Packs page',
	},
	shader: {
		id: 'settings.display.project-list-layouts.shader',
		defaultMessage: 'Shaders page',
	},
	resourcepack: {
		id: 'settings.display.project-list-layouts.resourcepack',
		defaultMessage: 'Resource Packs page',
	},
	modpack: {
		id: 'settings.display.project-list-layouts.modpack',
		defaultMessage: 'Modpacks page',
	},
	server: {
		id: 'settings.display.project-list-layouts.server',
		defaultMessage: 'Servers page',
	},
	user: {
		id: 'settings.display.project-list-layouts.user',
		defaultMessage: 'User profile pages',
	},
	backgroundImagesTitle: {
		id: 'settings.display.background-images.title',
		defaultMessage: 'Background Images',
	},
	backgroundImagesDescription: {
		id: 'settings.display.background-images.description',
		defaultMessage: 'Select the background image to display on the homepage',
	},
	displayWelcomeTitle: {
		id: 'settings.display.background-images.display-welcome',
		defaultMessage: 'Display welcome message',
	},
	displayWelcomeDescription: {
		id: 'settings.display.background-images.display-welcome.description',
		defaultMessage: 'Show the welcome message and description above your library.',
	},
	welcomeTemplateReset: {
		id: 'settings.display.background-images.display-welcome.template.reset',
		defaultMessage: 'Reset to the default text',
	},
	latestNewsBackgroundTitle: {
		id: 'settings.display.background-images.latest-news',
		defaultMessage: 'Using the latest news background of Minecraft',
	},
	latestNewsBackgroundDescription: {
		id: 'settings.display.background-images.latest-news.description',
		defaultMessage: 'Use the newest Minecraft.net article image as the homepage background.',
	},
	customImageTitle: {
		id: 'settings.display.background-images.custom',
		defaultMessage: 'Use a custom image',
	},
	customImageDescription: {
		id: 'settings.display.background-images.custom.description',
		defaultMessage: 'Choose your own PNG image as the homepage background.',
	},
	newsMenuTitle: {
		id: 'settings.display.news-menu.title',
		defaultMessage: 'News Menu',
	},
	newsMenuDescription: {
		id: 'settings.display.news-menu.description',
		defaultMessage: 'Display Minecraft news on the right side of the homepage',
	},
	advancedRenderingTitle: {
		id: 'settings.display.sidebar.advanced-rendering.title',
		defaultMessage: 'Advanced rendering',
	},
	advancedRenderingDescription: {
		id: 'settings.display.sidebar.advanced-rendering.description',
		defaultMessage:
			'Enables advanced rendering such as blur effects that may cause performance issues without hardware-accelerated rendering.',
	},
	externalLinksNewTabTitle: {
		id: 'settings.display.sidebar.external-links-new-tab.title',
		defaultMessage: 'Open external links in new tab',
	},
	externalLinksNewTabDescription: {
		id: 'settings.display.sidebar.external-links-new-tab.description',
		defaultMessage:
			'Make links which go outside of Alemona Launcher open in a new tab. No matter this setting, links on the same domain and in Markdown descriptions will open in the same tab, and links on ads and edit pages will open in a new tab.',
	},
	rightAlignedFiltersSidebarTitle: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.title',
		defaultMessage: 'Right-aligned filters sidebar on search pages',
	},
	rightAlignedFiltersSidebarDescription: {
		id: 'settings.display.sidebar.right-aligned-filters-sidebar.description',
		defaultMessage: 'Aligns the filters sidebar to the right of the search results.',
	},
	leftAlignedContentSidebarTitle: {
		id: 'settings.display.sidebar.left-aligned-content-sidebar.title',
		defaultMessage: 'Left-aligned sidebar on content pages',
	},
	leftAlignedContentSidebarDescription: {
		id: 'settings.display.sidebar.left-aligned-content-sidebar.description',
		defaultMessage: "Aligns the sidebar to the left of the page's content.",
	},
	nativeDecorationsTitle: {
		id: 'app.appearance-settings.native-decorations.title',
		defaultMessage: 'System window frame',
	},
	nativeDecorationsDescription: {
		id: 'app.appearance-settings.native-decorations.description',
		defaultMessage:
			"Use your operating system's title bar and window controls. Requires an app restart.",
	},
})
