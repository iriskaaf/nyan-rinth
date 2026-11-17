import type { App } from 'vue'
import type { Router } from 'vue-router'
/** NyanRinth does not submit errors or account details to upstream telemetry. */
export function setupErrorReporting(_app: App, _router: Router): void {}
