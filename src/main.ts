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
  sendDefaultPii: true,
  integrations: [
    Sentry.browserTracingIntegration()
  ],
  
  tracesSampleRate: 1.0, 
  
  tracePropagationTargets: ["localhost", /^https:\/\/yourserver\.io\/api/],
  
  enableLogs: true
})

app.mount('#app')
