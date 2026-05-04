export default {
	ignoreExportsUsedInFile: {
		interface: true,
		type: true,
		class: true,
		function: true
	},
	ignoreIssues: {
		'src/lib/stores/*.ts': ['exports'],
		'src/lib/api/playback.ts': ['exports']
	}
};
