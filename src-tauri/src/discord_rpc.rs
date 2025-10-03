use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordRpcState {
    pub is_connected: bool,
    pub client_id: Option<String>,
    pub current_activity: Option<DiscordActivity>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DiscordActivity {
    pub project_name: String,
    pub language: Option<String>,
    pub editor: Option<String>,
    pub entity: Option<String>,
    pub start_time: Option<i64>,
}

pub struct DiscordRpcService {
    client: Option<DiscordIpcClient>,
    state: Arc<Mutex<DiscordRpcState>>,
}

impl DiscordRpcService {
    pub fn new() -> Self {
        Self {
            client: None,
            state: Arc::new(Mutex::new(DiscordRpcState {
                is_connected: false,
                client_id: None,
                current_activity: None,
            })),
        }
    }

    pub fn connect(&mut self, client_id: &str) -> Result<(), String> {
        // Close existing connection if any
        if self.client.is_some() {
            let _ = self.disconnect();
        }

        let mut client = DiscordIpcClient::new(client_id);

        client
            .connect()
            .map_err(|e| format!("Failed to connect to Discord: {}", e))?;

        self.client = Some(client);

        // Update state
        let mut state = self.state.lock().unwrap();
        state.is_connected = true;
        state.client_id = Some(client_id.to_string());

        Ok(())
    }

    pub fn disconnect(&mut self) -> Result<(), String> {
        if let Some(mut client) = self.client.take() {
            client
                .close()
                .map_err(|e| format!("Failed to disconnect from Discord: {}", e))?;
        }

        // Update state
        let mut state = self.state.lock().unwrap();
        state.is_connected = false;
        state.client_id = None;
        state.current_activity = None;

        Ok(())
    }

    pub fn set_activity(&mut self, activity: DiscordActivity) -> Result<(), String> {
        self.set_activity_internal(activity.clone())?;

        // Update state
        let mut state = self.state.lock().unwrap();
        state.current_activity = Some(activity);

        Ok(())
    }

    fn set_activity_internal(&mut self, activity: DiscordActivity) -> Result<(), String> {
        let client = self.client.as_mut().ok_or("Discord client not connected")?;

        // Build details string
        let mut details_parts = Vec::new();

        if let Some(language) = &activity.language {
            details_parts.push(format!("Language: {}", language));
        }

        if let Some(editor) = &activity.editor {
            details_parts.push(format!("Editor: {}", editor));
        }

        if let Some(entity) = &activity.entity {
            let filename = entity.split('/').last().unwrap_or(entity);
            details_parts.push(format!("File: {}", filename));
        }

        let details_string = if !details_parts.is_empty() {
            Some(details_parts.join(" • "))
        } else {
            None
        };

        // Create activity with all components
        let mut discord_activity = activity::Activity::new().state(&activity.project_name);

        if let Some(details) = &details_string {
            discord_activity = discord_activity.details(details);
        }

        // Set start time if provided
        if let Some(start_time) = activity.start_time {
            discord_activity =
                discord_activity.timestamps(activity::Timestamps::new().start(start_time));
        }

        // Add assets
        discord_activity = discord_activity.assets(
            activity::Assets::new()
                .large_image("kubetime")
                .large_text("KubeTime - Time Tracking")
                .small_image("coding")
                .small_text("Coding"),
        );

        client
            .set_activity(discord_activity)
            .map_err(|e| format!("Failed to set Discord activity: {}", e))?;

        Ok(())
    }

    pub fn clear_activity(&mut self) -> Result<(), String> {
        let client = self.client.as_mut().ok_or("Discord client not connected")?;

        client
            .clear_activity()
            .map_err(|e| format!("Failed to clear Discord activity: {}", e))?;

        // Update state
        let mut state = self.state.lock().unwrap();
        state.current_activity = None;

        Ok(())
    }

    pub fn get_state(&self) -> DiscordRpcState {
        self.state.lock().unwrap().clone()
    }

    pub fn is_connected(&self) -> bool {
        self.state.lock().unwrap().is_connected
    }

    pub fn update_activity_from_heartbeat(
        &mut self,
        heartbeat_data: &crate::HeartbeatData,
    ) -> Result<(), String> {
        let activity = DiscordActivity {
            project_name: heartbeat_data
                .project
                .clone()
                .unwrap_or_else(|| "Unknown Project".to_string()),
            language: heartbeat_data.language.clone(),
            editor: heartbeat_data.editor.clone(),
            entity: heartbeat_data.entity.clone(),
            start_time: Some(heartbeat_data.timestamp as i64),
        };

        self.set_activity(activity)
    }

    pub fn update_activity_from_session(
        &mut self,
        heartbeat_data: &crate::HeartbeatData,
        session_start_time: i64,
    ) -> Result<(), String> {
        let activity = DiscordActivity {
            project_name: heartbeat_data
                .project
                .clone()
                .unwrap_or_else(|| "Unknown Project".to_string()),
            language: heartbeat_data.language.clone(),
            editor: heartbeat_data.editor.clone(),
            entity: heartbeat_data.entity.clone(),
            start_time: Some(session_start_time),
        };

        self.set_activity(activity)
    }

    pub fn auto_connect(&mut self) -> Result<(), String> {
        const DEFAULT_CLIENT_ID: &str = "1423077619183779872";
        self.connect(DEFAULT_CLIENT_ID)
    }
}

impl Drop for DiscordRpcService {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}
