import { invoke } from '@tauri-apps/api/core'

interface ApiConfig {
  base_url: string
}

interface AuthState {
  is_authenticated: boolean
  access_token: string | null
  user_info: Record<string, any> | null
}

export class KubeTimeApi {
  private baseUrl: string = 'https://hackatime.hackclub.com'
  private accessToken: string | null = null
  private latestPresenceCache: { data: any | null, fetchedAt: number } = { data: null, fetchedAt: 0 }

  async initialize () {
    try {
      const config: ApiConfig = await invoke('get_api_config')
      if (config.base_url && config.base_url.trim()) {
        this.baseUrl = config.base_url
      }

      const authState: AuthState = await invoke('get_auth_state')
      this.accessToken = authState.access_token
    } catch (error) {
      console.error('Failed to initialize API:', error)
      if (!this.baseUrl || !this.baseUrl.trim()) {
        this.baseUrl = 'https://hackatime.hackclub.com'
      }
    }
  }

  async getCurrentUser () {
    if (!this.accessToken) {
      throw new Error('Not authenticated')
    }

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/authenticated/me`, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      return await response.json()
    } catch (error) {
      console.error('Failed to fetch current user:', error)
      throw error
    }
  }

  async getHours (startDate?: string, endDate?: string) {
    if (!this.accessToken) {
      throw new Error('Not authenticated')
    }

    try {
      const params = new URLSearchParams()
      if (startDate) params.append('start_date', startDate)
      if (endDate) params.append('end_date', endDate)

      const url = `${this.baseUrl}/api/v1/authenticated/hours${params.toString() ? `?${params.toString()}` : ''}`

      const response = await fetch(url, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      return await response.json()
    } catch (error) {
      console.error('Failed to fetch hours:', error)
      throw error
    }
  }

  async getWeeklyHours () {
    const endDate = new Date()
    const startDate = new Date()
    startDate.setDate(endDate.getDate() - 7)

    const formatDate = (date: Date) => date.toISOString().split('T')[0]

    return await this.getHours(formatDate(startDate), formatDate(endDate))
  }

  async getStreak () {
    if (!this.accessToken) {
      throw new Error('Not authenticated')
    }

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/authenticated/streak`, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      return await response.json()
    } catch (error) {
      console.error('Failed to fetch streak:', error)
      throw error
    }
  }

  async getStats () {
    if (!this.accessToken) {
      throw new Error('Not authenticated')
    }

    try {
      const [hoursData, streakData] = await Promise.all([
        this.getWeeklyHours(),
        this.getStreak()
      ])

      return {
        ...hoursData,
        current_streak: streakData.current_streak,
        longest_streak: streakData.longest_streak
      }
    } catch (error) {
      console.error('Failed to fetch stats:', error)
      throw error
    }
  }

  async getCurrentPresence () {
    if (!this.accessToken) {
      throw new Error('Not authenticated')
    }

    try {
      
      const now = Date.now()
      if (now - this.latestPresenceCache.fetchedAt < 60_000 && this.latestPresenceCache.data !== null) {
        return this.latestPresenceCache.data
      }

      const response = await fetch(`${this.baseUrl}/api/v1/authenticated/heartbeats/latest`, {
        headers: {
          Authorization: `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json'
        }
      })

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`)
      }

      const json = await response.json()
      this.latestPresenceCache = { data: json, fetchedAt: Date.now() }
      return json
    } catch (error) {
      console.error('Failed to fetch current presence:', error)
      throw error
    }
  }
}

export const api = new KubeTimeApi()
