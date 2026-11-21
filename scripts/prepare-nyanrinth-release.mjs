import { createHash, createPublicKey, verify } from 'node:crypto'
import { execFileSync } from 'node:child_process'
import { existsSync, mkdirSync, readFileSync, writeFileSync, copyFileSync } from 'node:fs'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..')
const version = JSON.parse(
	readFileSync(join(root, 'apps/app-frontend/package.json'), 'utf8'),
).version
const repository = 'iriskaaf/nyan-rinth'
const tag = `v${version}`
const target = process.env.CARGO_TARGET_DIR ?? join(root, 'target')
const installerName = `NyanRinth_${version}_x64-setup.exe`
const executableName = `NyanRinth_${version}_x64.exe`
const installer = join(target, 'release/bundle/nsis', installerName)
const signatureFile = `${installer}.sig`
const executable = join(target, 'release/NyanRinth.exe')
const output = join(root, 'release', version)

function git(...args) {
	return execFileSync('git', args, { cwd: root, encoding: 'utf8' }).trim()
}

if (!/^\d+\.\d+\.\d+(?:-(?:alpha|beta))?$/.test(version)) {
	throw new Error(`Unsupported release version ${version}`)
}
for (const file of [installer, signatureFile, executable]) {
	if (!existsSync(file)) throw new Error(`Missing signed release artifact ${file}`)
}

const signature = readFileSync(signatureFile, 'utf8').trim()
const signatureLines = Buffer.from(signature, 'base64').toString('utf8').split('\n')
if (!signatureLines[0]?.startsWith('untrusted comment:') || !signatureLines[1]) {
	throw new Error('The updater signature is invalid')
}
const config = JSON.parse(readFileSync(join(root, 'apps/app/tauri.conf.json'), 'utf8'))
const publicKeyLines = Buffer.from(config.plugins.updater.pubkey, 'base64')
	.toString('utf8')
	.split('\n')
const publicKey = Buffer.from(publicKeyLines[1], 'base64')
const signedData = Buffer.from(signatureLines[1], 'base64')
if (
	publicKey.length !== 42 ||
	signedData.length !== 74 ||
	!publicKey.subarray(2, 10).equals(signedData.subarray(2, 10)) ||
	!signatureLines[2]?.startsWith('trusted comment: ')
) {
	throw new Error('The updater signature does not match the configured public key')
}
const algorithm = signedData.subarray(0, 2).toString('ascii')
if (algorithm !== 'ED' && algorithm !== 'Ed') throw new Error('Unsupported signature algorithm')
const data = readFileSync(installer)
const message = algorithm === 'ED' ? createHash('blake2b512').update(data).digest() : data
const verifier = createPublicKey({
	key: Buffer.concat([Buffer.from('302a300506032b6570032100', 'hex'), publicKey.subarray(10)]),
	format: 'der',
	type: 'spki',
})
const fileSignature = signedData.subarray(10)
const trustedComment = Buffer.from(signatureLines[2].slice('trusted comment: '.length))
if (
	!verify(null, message, verifier, fileSignature) ||
	!verify(
		null,
		Buffer.concat([fileSignature, trustedComment]),
		verifier,
		Buffer.from(signatureLines[3], 'base64'),
	)
) {
	throw new Error('Installer signature verification failed')
}
const changelog = readFileSync(join(root, 'CHANGELOG.md'), 'utf8')
const heading = `## ${version} `
const sectionStart = changelog.indexOf(heading)
if (sectionStart < 0) throw new Error(`Missing changelog entry for ${version}`)
const nextSection = changelog.indexOf('\n## ', sectionStart + heading.length)
const notes = changelog.slice(sectionStart, nextSection < 0 ? undefined : nextSection).trim()

mkdirSync(output, { recursive: true })
copyFileSync(installer, join(output, installerName))
copyFileSync(executable, join(output, executableName))
copyFileSync(signatureFile, join(output, `${installerName}.sig`))
const manifest = {
	version,
	notes,
	pub_date: new Date().toISOString(),
	platforms: {
		'windows-x86_64': {
			url: `https://github.com/${repository}/releases/download/${tag}/${installerName}`,
			signature,
		},
	},
}
writeFileSync(join(output, 'latest.json'), `${JSON.stringify(manifest, null, '\t')}\n`)
writeFileSync(join(output, 'release-notes.md'), `${notes}\n`)
const artifacts = [installerName, executableName, `${installerName}.sig`, 'latest.json']
const checksums = artifacts.map((name) => {
	const digest = createHash('sha256')
		.update(readFileSync(join(output, name)))
		.digest('hex')
	return `${digest}  ${name}`
})
writeFileSync(join(output, 'SHA256SUMS.txt'), `${checksums.join('\n')}\n`)

if (process.argv.includes('--publish')) {
	if (git('status', '--porcelain')) throw new Error('Commit the release before publishing')
	if (git('rev-parse', `${tag}^{commit}`) !== git('rev-parse', 'HEAD')) {
		throw new Error('The release tag must point to the current commit')
	}
	const gh = process.env.GH_PATH ?? 'gh'
	const prerelease = version.endsWith('-alpha')
	const files = [...artifacts, 'SHA256SUMS.txt'].map((name) => join(output, name))
	execFileSync(
		gh,
		[
			'release',
			'create',
			tag,
			...files,
			'--repo',
			repository,
			'--verify-tag',
			'--draft',
			'--title',
			version,
			'--notes-file',
			join(output, 'release-notes.md'),
		],
		{ cwd: root, stdio: 'inherit' },
	)
	execFileSync(
		gh,
		[
			'release',
			'edit',
			tag,
			'--repo',
			repository,
			'--draft=false',
			`--prerelease=${prerelease}`,
			`--latest=${!prerelease}`,
		],
		{ cwd: root, stdio: 'inherit' },
	)
}

console.log(`Prepared signed NyanRinth ${version} in ${output}`)
