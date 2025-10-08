import { inject } from 'vue'
import type { PostHog } from 'posthog-js'

export function usePostHogInstance() {
  const posthog = inject<PostHog>('posthog')
  
  if (!posthog) {
    throw new Error('PostHog is not initialized')
  }
  
  return posthog
}

