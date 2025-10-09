import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import * as Sentry from '@sentry/vue'
import { usePostHog } from './composables/usePostHog'

const app = createApp(App)


const { posthog } = usePostHog()


app.provide('posthog', posthog)

Sentry.init({
  app,
  dsn: "https://d67e5cceba1b80139ca09c806efc616a@o4509680631087104.ingest.us.sentry.io/4510156240060417",
  
  release: __SENTRY_RELEASE__,
  environment: __SENTRY_ENVIRONMENT__,
  
  sendDefaultPii: true,
  debug: __SENTRY_ENVIRONMENT__ === 'development',
  maxBreadcrumbs: 100,
  attachStacktrace: true,
  
  sampleRate: 1.0,
  
  integrations: [
    Sentry.browserTracingIntegration(),
    Sentry.replayIntegration({
      maskAllText: false,
      blockAllMedia: false,
    }),
  ],
  
  tracesSampleRate: __SENTRY_ENVIRONMENT__ === 'production' ? 0.1 : 1.0,
  
  profilesSampleRate: __SENTRY_ENVIRONMENT__ === 'production' ? 0.1 : 1.0,
  
  enableLogs: true,
  
  beforeSend(event, hint) {
    const error = hint.originalException as Error | undefined
    const errorMessage = (error as any)?.message || error?.toString() || ''
    
    if (
      errorMessage.includes('callbackId') ||
      errorMessage.includes('IPC custom protocol failed') ||
      errorMessage.includes('Tauri will now use the postMessage interface') ||
      errorMessage.includes('Load failed') && errorMessage.includes('localhost') ||
      errorMessage.includes('ipc://localhost') ||
      errorMessage.includes('project.editors.some') ||
      errorMessage.includes('project.total_heartbeats.toLocaleString') ||
      errorMessage.includes('el.__vnode') ||
      errorMessage.includes('patchElement') && errorMessage.includes('null') ||
      event.exception?.values?.some(value => 
        value.value?.includes('callbackId') || 
        value.value?.includes('[callbackId, data]') ||
        value.value?.includes('project.editors.some') ||
        value.value?.includes('project.total_heartbeats') ||
        value.value?.includes('el.__vnode') ||
        (value.value?.includes('patchElement') && value.value?.includes('null'))
      )
    ) {
      console.log('[SENTRY] Filtered out known benign error:', errorMessage)
      return null
    }
    
    return event
  },
  
  ignoreErrors: [
    'IPC custom protocol failed',
    'callbackId',
    'Load failed',
    'ipc://localhost',
    'undefined is not an object (evaluating \'[callbackId, data]\')',
    'undefined is not an object (evaluating \'project.editors.some\')',
    'undefined is not an object (evaluating \'project.total_heartbeats.toLocaleString\')',
    'null is not an object (evaluating \'el.__vnode = n2\')',
    'null is not an object (evaluating \'el.__vnode\')',
    /IPC custom protocol/,
    /callbackId/,
    /project\.editors\.some/,
    /project\.total_heartbeats/,
    /el\.__vnode/,
    /patchElement/,
  ]
})

app.mount('#app')
