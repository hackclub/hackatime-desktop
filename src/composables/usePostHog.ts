import posthog from 'posthog-js'

export function usePostHog() {
  posthog.init('phc_xwC3ygQBfstlwaJ3i1lRtz6bON6CR5lHFz7UhlSW6SZ', {
    api_host: 'https://at.leafd.dev',
    ui_host: 'https://at.leafd.dev',
    person_profiles: 'identified_only',
    session_recording: {
      
      
      maskAllInputs: true,
      maskInputOptions: {
        password: true, 
        email: true, 
      },
      
      maskTextSelector: '*',
      maskTextFn: (text, element) => {
        
        if (text.trim().length === 0) {
          return text
        }

        
        if (element?.dataset?.['record'] === 'true') {
          return text
        }

        
        const emailRegex = /(\S+)@(\S+\.\S+)/g
        if (emailRegex.test(text)) {
          return text.replace(emailRegex, (_match, g1, g2) => {
            return '*'.repeat(g1.length) + '@' + '*'.repeat(g2.length)
          })
        }

        
        const tokenRegex = /[a-zA-Z0-9_-]{20,}/g
        if (tokenRegex.test(text)) {
          return text.replace(tokenRegex, (match) => '*'.repeat(match.length))
        }

        return text
      },
      maskInputFn: (text, element) => {
        
        const passwordRelated = ['password', 'pwd', 'pass']
        const elementId = (element?.attributes?.['id' as any]?.value as string)?.toLowerCase() || ''
        const elementName = (element?.attributes?.['name' as any]?.value as string)?.toLowerCase() || ''
        
        if (
          passwordRelated.some(p => elementId.includes(p) || elementName.includes(p))
        ) {
          return '*'.repeat(text.length)
        }

        
        if (element?.attributes?.['type' as any]?.value === 'search') {
          return text
        }

        
        return '*'.repeat(text.length)
      },
    },
  })

  return { posthog }
}

