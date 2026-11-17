import cssText from '@modrinth/assets/styles/ace.css?raw'
import ace from 'ace-builds'
import cssWorkerUrl from 'ace-builds/src-noconflict/worker-css?url'
import htmlWorkerUrl from 'ace-builds/src-noconflict/worker-html?url'
import javascriptWorkerUrl from 'ace-builds/src-noconflict/worker-javascript?url'
import jsonWorkerUrl from 'ace-builds/src-noconflict/worker-json?url'
import phpWorkerUrl from 'ace-builds/src-noconflict/worker-php?url'
import xmlWorkerUrl from 'ace-builds/src-noconflict/worker-xml?url'
import yamlWorkerUrl from 'ace-builds/src-noconflict/worker-yaml?url'

ace.config.setModuleLoader('ace/mode/text', () => import('ace-builds/src-noconflict/mode-text'))
ace.config.setModuleLoader('ace/mode/json', () => import('ace-builds/src-noconflict/mode-json'))
ace.config.setModuleLoader('ace/mode/toml', () => import('ace-builds/src-noconflict/mode-toml'))
ace.config.setModuleLoader('ace/mode/sh', () => import('ace-builds/src-noconflict/mode-sh'))
ace.config.setModuleLoader(
	'ace/mode/batchfile',
	() => import('ace-builds/src-noconflict/mode-batchfile'),
)
ace.config.setModuleLoader(
	'ace/mode/powershell',
	() => import('ace-builds/src-noconflict/mode-powershell'),
)
ace.config.setModuleLoader('ace/mode/yaml', () => import('ace-builds/src-noconflict/mode-yaml'))
ace.config.setModuleLoader(
	'ace/mode/javascript',
	() => import('ace-builds/src-noconflict/mode-javascript'),
)
ace.config.setModuleLoader(
	'ace/mode/typescript',
	() => import('ace-builds/src-noconflict/mode-typescript'),
)
ace.config.setModuleLoader('ace/mode/python', () => import('ace-builds/src-noconflict/mode-python'))
ace.config.setModuleLoader('ace/mode/ruby', () => import('ace-builds/src-noconflict/mode-ruby'))
ace.config.setModuleLoader('ace/mode/php', () => import('ace-builds/src-noconflict/mode-php'))
ace.config.setModuleLoader('ace/mode/html', () => import('ace-builds/src-noconflict/mode-html'))
ace.config.setModuleLoader('ace/mode/css', () => import('ace-builds/src-noconflict/mode-css'))
ace.config.setModuleLoader('ace/mode/java', () => import('ace-builds/src-noconflict/mode-java'))
ace.config.setModuleLoader('ace/mode/c_cpp', () => import('ace-builds/src-noconflict/mode-c_cpp'))
ace.config.setModuleLoader('ace/mode/rust', () => import('ace-builds/src-noconflict/mode-rust'))
ace.config.setModuleLoader('ace/mode/golang', () => import('ace-builds/src-noconflict/mode-golang'))
ace.config.setModuleLoader(
	'ace/mode/markdown',
	() => import('ace-builds/src-noconflict/mode-markdown'),
)
ace.config.setModuleLoader(
	'ace/mode/properties',
	() => import('ace-builds/src-noconflict/mode-properties'),
)
ace.config.setModuleLoader('ace/mode/ini', () => import('ace-builds/src-noconflict/mode-ini'))
ace.config.setModuleLoader(
	'ace/ext/searchbox',
	() => import('ace-builds/src-noconflict/ext-searchbox'),
)

ace.config.setModuleUrl('ace/mode/css_worker', cssWorkerUrl)
ace.config.setModuleUrl('ace/mode/html_worker', htmlWorkerUrl)
ace.config.setModuleUrl('ace/mode/javascript_worker', javascriptWorkerUrl)
ace.config.setModuleUrl('ace/mode/json_worker', jsonWorkerUrl)
ace.config.setModuleUrl('ace/mode/php_worker', phpWorkerUrl)
ace.config.setModuleUrl('ace/mode/xml_worker', xmlWorkerUrl)
ace.config.setModuleUrl('ace/mode/yaml_worker', yamlWorkerUrl)

ace['define'](
	'ace/theme/modrinth',
	['require', 'exports', 'module', 'ace/lib/dom'],
	function (require, exports, _module) {
		exports.isDark = false
		exports.cssClass = 'ace-modrinth'
		exports.cssText = cssText

		const dom = require('ace/lib/dom')
		dom.importCssString(exports.cssText, exports.cssClass, false)
	},
)
