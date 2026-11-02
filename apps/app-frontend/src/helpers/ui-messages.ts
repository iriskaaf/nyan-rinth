import { defineMessages } from '@modrinth/ui'

export const nyanMessages = defineMessages({
	installContent: { id: 'nyanrinth.interface.installContent', defaultMessage: 'Install content' },
	fromFile: { id: 'nyanrinth.interface.fromFile', defaultMessage: 'Add from file' },
	moreOptions: { id: 'nyanrinth.interface.moreOptions', defaultMessage: 'More options' },
	name: { id: 'nyanrinth.interface.name', defaultMessage: 'Name' },
	supports: { id: 'nyanrinth.interface.supports', defaultMessage: 'Supports' },
	changeModpack: {
		id: 'nyanrinth.interface.changeModpack',
		defaultMessage: 'Change modpack version',
	},
	switchVersion: { id: 'nyanrinth.interface.switchVersion', defaultMessage: 'Switch version' },
	install: { id: 'nyanrinth.interface.install', defaultMessage: 'Install' },
	detect: { id: 'nyanrinth.interface.detect', defaultMessage: 'Detect' },
	browse: { id: 'nyanrinth.interface.browse', defaultMessage: 'Browse' },
	noPendingFriends: {
		id: 'nyanrinth.interface.noPendingFriends',
		defaultMessage: 'You have no pending friend requests :C',
	},
	reset: { id: 'nyanrinth.interface.reset', defaultMessage: 'Reset to default' },
	inviteFriends: { id: 'nyanrinth.interface.inviteFriends', defaultMessage: 'Invite friends' },
	all: { id: 'nyanrinth.interface.all', defaultMessage: 'All' },
	installNamed: { id: 'nyanrinth.interface.installNamed', defaultMessage: 'Install {name}' },
	installVersion: {
		id: 'nyanrinth.interface.installVersion',
		defaultMessage: 'Installing <code>{version}</code> from Modrinth',
	},
	searchUsers: { id: 'nyanrinth.interface.searchUsers', defaultMessage: 'Search {count} users...' },
	userAvatar: { id: 'nyanrinth.interface.userAvatar', defaultMessage: "{name}'s avatar" },
	whatHappened: {
		id: 'nyanrinth.interface.whatHappened',
		defaultMessage: 'What we think happened',
	},
	howToFix: { id: 'nyanrinth.interface.howToFix', defaultMessage: 'How to fix it' },
	unknownError: { id: 'nyanrinth.interface.unknownError', defaultMessage: 'Unknown error' },
	signInFailed: { id: 'nyanrinth.interface.signInFailed', defaultMessage: 'Sign in Failed' },
	contactSupport: { id: 'nyanrinth.interface.contactSupport', defaultMessage: 'Contact support' },
	signInAgain: { id: 'nyanrinth.interface.signInAgain', defaultMessage: 'Sign in again' },
	debugInfo: { id: 'nyanrinth.interface.debugInfo', defaultMessage: 'Debug information' },
	copyDebug: { id: 'nyanrinth.interface.copyDebug', defaultMessage: 'Copy debug info' },
	unknownErrorBody: {
		id: 'nyanrinth.interface.unknownErrorBody',
		defaultMessage:
			'We don’t recognize this error and can’t recommend specific steps to resolve it.',
	},
	microsoftWarning: {
		id: 'nyanrinth.interface.microsoftWarning',
		defaultMessage:
			"We couldn't sign you into your Microsoft account. This may be due to account restrictions or regional limitations.",
	},
	microsoftHelp: {
		id: 'nyanrinth.interface.microsoftHelp',
		defaultMessage:
			'Try visiting <login>Minecraft Login</login> and signing in, as it may prompt you with the necessary steps. You can also contact support and we can look into it further.',
	},
	noErrorMessage: { id: 'nyanrinth.interface.noErrorMessage', defaultMessage: 'No error message.' },
	errorOccurred: { id: 'nyanrinth.interface.errorOccurred', defaultMessage: 'An error occurred' },
	minecraftSignInError: {
		id: 'nyanrinth.interface.minecraftSignInError',
		defaultMessage: 'Unable to sign in to Minecraft',
	},
	moveDirectoryError: {
		id: 'nyanrinth.interface.moveDirectoryError',
		defaultMessage: 'Could not change app directory',
	},
	noLoader: { id: 'nyanrinth.interface.noLoader', defaultMessage: 'No loader selected' },
	authLibraryRequired: {
		id: 'nyanrinth.interface.authLibraryRequired',
		defaultMessage: 'Authentication library required',
	},
	initializationError: {
		id: 'nyanrinth.interface.initializationError',
		defaultMessage: 'Error initializing NyanRinth App',
	},
	networkIssues: { id: 'nyanrinth.interface.networkIssues', defaultMessage: 'Network issues' },
	tryOtherMicrosoft: {
		id: 'nyanrinth.interface.tryOtherMicrosoft',
		defaultMessage: 'Try another Microsoft account',
	},
	checkMicrosoft: {
		id: 'nyanrinth.interface.checkMicrosoft',
		defaultMessage:
			"Double check you've signed in with the right account. You may own Minecraft on a different Microsoft account.",
	},
	tryOtherAccount: {
		id: 'nyanrinth.interface.tryOtherAccount',
		defaultMessage: 'Try another account',
	},
	newGameAccount: {
		id: 'nyanrinth.interface.newGameAccount',
		defaultMessage: 'Using PC Game Pass, coming from Bedrock, or just bought the game?',
	},
	trySigningInAgain: {
		id: 'nyanrinth.interface.trySigningInAgain',
		defaultMessage: 'Try signing in again',
	},
	changePermissions: {
		id: 'nyanrinth.interface.changePermissions',
		defaultMessage: 'Change directory permissions',
	},
	permissionsBody: {
		id: 'nyanrinth.interface.permissionsBody',
		defaultMessage:
			'It looks like the NyanRinth App is unable to write to the directory you selected. Please adjust the permissions of the directory and try again or cancel the directory change.',
	},
	notEnoughSpace: { id: 'nyanrinth.interface.notEnoughSpace', defaultMessage: 'Not enough space' },
	spaceBody: {
		id: 'nyanrinth.interface.spaceBody',
		defaultMessage:
			'It looks like there is not enough space on the disk containing the directory you selected. Please free up some space and try again or cancel the directory change.',
	},
	moveBody: {
		id: 'nyanrinth.interface.moveBody',
		defaultMessage:
			'The NyanRinth App is unable to migrate to the new directory you selected. Please contact support for help or cancel the directory change.',
	},
	retryMove: { id: 'nyanrinth.interface.retryMove', defaultMessage: 'Retry directory change' },
	cancelMove: { id: 'nyanrinth.interface.cancelMove', defaultMessage: 'Cancel directory change' },
	initBody: {
		id: 'nyanrinth.interface.initBody',
		defaultMessage:
			'NyanRinth App failed to load correctly. This may be because of a corrupted file, or because the app is missing crucial files.',
	},
	initOptions: {
		id: 'nyanrinth.interface.initOptions',
		defaultMessage: 'You may be able to fix it through one of the following ways:',
	},
	restartApp: {
		id: 'nyanrinth.interface.restartApp',
		defaultMessage: 'Ensuring you are connected to the internet, then try restarting the app.',
	},
	redownloadApp: {
		id: 'nyanrinth.interface.redownloadApp',
		defaultMessage: 'Redownloading the app.',
	},
	loaderBody: {
		id: 'nyanrinth.interface.loaderBody',
		defaultMessage: 'The NyanRinth App failed to find the loader version for this instance.',
	},
	repairBody: {
		id: 'nyanrinth.interface.repairBody',
		defaultMessage:
			'To resolve this, you need to repair the instance. Click the button below to do so.',
	},
	repairInstance: { id: 'nyanrinth.interface.repairInstance', defaultMessage: 'Repair instance' },
	authLibraryBody: {
		id: 'nyanrinth.interface.authLibraryBody',
		defaultMessage:
			'Choose and install a version under Settings → NyanRinth → Authentication libraries, then launch the instance again.',
	},
	getSupport: { id: 'nyanrinth.interface.getSupport', defaultMessage: 'Get support' },
	close: { id: 'nyanrinth.interface.close', defaultMessage: 'Close' },
	networkBody: {
		id: 'nyanrinth.interface.networkBody',
		defaultMessage:
			'NyanRinth could not connect to Microsoft servers. Check your connection and try again. If the problem persists, see <support>the support article</support>.',
	},
	hostsBody: {
		id: 'nyanrinth.interface.hostsBody',
		defaultMessage:
			'Microsoft, Xbox, or Minecraft services rejected the connection. The services may be blocked in your hosts file. See <support>the support article</support>.',
	},
	officialLauncherBody: {
		id: 'nyanrinth.interface.officialLauncherBody',
		defaultMessage:
			'First sign in with <launcher>the official Minecraft Launcher</launcher>. Then return here and sign in again.',
	},
	supportBody: {
		id: 'nyanrinth.interface.supportBody',
		defaultMessage:
			'If you need help, visit <support>the support page</support>. Include the diagnostic information below when describing the problem.',
	},
})
