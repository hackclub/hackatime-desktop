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
  
  enableLogs: true
})

app.mount('#app')
