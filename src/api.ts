import { invoke } from "@tauri-apps/api/core";

interface ApiConfig {
  base_url: string;
}

interface AuthState {
  is_authenticated: boolean;
  access_token: string | null;
  user_info: Record<string, any> | null;
}

export class KubeTimeApi {
  private baseUrl: string = "http://localhost:3000";
  private accessToken: string | null = null;
  private latestPresenceCache: { data: any | null; fetchedAt: number } = { data: null, fetchedAt: 0 };

  async initialize() {
    try {
      const config: ApiConfig = await invoke("get_api_config");
      this.baseUrl = config.base_url;
      
      const authState: AuthState = await invoke("get_auth_state");
      this.accessToken = authState.access_token;
    } catch (error) {
      console.error("Failed to initialize API:", error);
    }
  }

  async getCurrentUser() {
    if (!this.accessToken) {
      throw new Error("Not authenticated");
    }

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/authenticated/me`, {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Failed to fetch current user:", error);
      throw error;
    }
  }

  async getStats() {
    if (!this.accessToken) {
      throw new Error("Not authenticated");
    }

    try {
      // Call the current user's dashboard stats endpoint
      const response = await fetch(`${this.baseUrl}/api/v1/authenticated/dashboard_stats`, {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Failed to fetch stats:", error);
      throw error;
    }
  }

  async getMyHeartbeats() {
    if (!this.accessToken) {
      throw new Error("Not authenticated");
    }

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/my/heartbeats`, {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Failed to fetch heartbeats:", error);
      throw error;
    }
  }

  async getUserDashboardStats(username: string) {
    if (!this.accessToken) {
      throw new Error("Not authenticated");
    }

    try {
      const response = await fetch(`${this.baseUrl}/api/v1/users/${username}/dashboard_stats`, {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      console.error("Failed to fetch user dashboard stats:", error);
      throw error;
    }
  }

  async getCurrentPresence() {
    if (!this.accessToken) {
      throw new Error("Not authenticated");
    }

    try {
      // Return cached result if fetched within last 60s
      const now = Date.now();
      if (now - this.latestPresenceCache.fetchedAt < 60_000 && this.latestPresenceCache.data !== null) {
        return this.latestPresenceCache.data;
      }
      
      const response = await fetch(`${this.baseUrl}/api/v1/presence/latest_heartbeat`, {
        headers: {
          'Authorization': `Bearer ${this.accessToken}`,
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const json = await response.json();
      this.latestPresenceCache = { data: json, fetchedAt: Date.now() };
      return json;
    } catch (error) {
      console.error("Failed to fetch current presence:", error);
      throw error;
    }
  }
}

export const api = new KubeTimeApi();
