use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use crate::midi_commands; 
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
struct CommonAppData {
    #[serde(rename = "ggEncryptedAddress")]
    gg_encrypted_address: String, 
}

// Structs for setting the volume
#[derive(Debug, Serialize, Deserialize)]
struct SubAppsResponse {
    #[serde(rename = "subApps")]
    sub_apps: SubApps,
}

#[derive(Debug, Serialize, Deserialize)]
struct SubApps {
    sonar: SonarApp,
}

#[derive(Debug, Serialize, Deserialize)]
struct SonarApp {
    metadata: SonarMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
struct SonarMetadata {
    #[serde(rename = "webServerAddress")]
    web_server_address: String,
}

#[derive(Debug)]
pub struct Sonar {
    streamer_mode: bool,
    volume_path: String,
    base_url: String,
    web_server_address: String,
}

/*###############################################################################
Currently, SteelSeries Sonar Streamer Mode is not functional, but it is a work in progress. 
This is the code to extract the mode from the SteelSeries database.
If you aren't using SteelSeries Sonar for multi-channel audio, you can remove this safely
as well as any SteelSeries code in the main() function.
###############################################################################*/
pub fn fetch_streamer_mode() -> SqlResult<bool> {
    let db_path = Path::new("C:\\ProgramData\\SteelSeries\\GG\\apps\\sonar\\db\\database.db");
    let conn = Connection::open(db_path)?;

    let mut stmt = conn.prepare("SELECT value FROM key_value WHERE key = 'MODE'")?;
    let mut mode_iter = stmt.query_map([], |row| row.get::<_, String>(0))?;

    if let Some(mode) = mode_iter.next() {
        // If there's a result, compare it (case-insensitively) to "streamer"
        Ok(mode?.to_lowercase() == "stream")
    } else {
        // If there's no result, default to false
        Ok(false)
    }
}

impl Sonar {
    pub fn new(streamer_mode: bool, app_data_path: Option<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let mut sonar = Sonar {
            streamer_mode,
            volume_path: if streamer_mode { "/volumeSettings/streamer/monitoring".to_string() } else { "/volumeSettings/classic".to_string() },
            base_url: String::new(),
            web_server_address: String::new(),
        };

        if streamer_mode {
            sonar.volume_path = "/volumeSettings/streamer/monitoring".to_string();
        }

        sonar.load_base_url(app_data_path)?;
        Ok(sonar)
    }

    pub fn load_base_url(&mut self, app_data_path: Option<PathBuf>) -> Result<(), Box<dyn std::error::Error>> {
        let path = app_data_path.unwrap_or_else(|| {
            PathBuf::from(env::var("ProgramData").unwrap_or_default())
                .join("SteelSeries")
                .join("SteelSeries Engine 3")
                .join("coreProps.json")
        });

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let common_app_data: CommonAppData = serde_json::from_str(&contents)?;

        self.base_url = format!("https://{}", common_app_data.gg_encrypted_address);
        Ok(())
    }

    pub fn update_web_server_address(&mut self, body: &str) -> Result<(), Box<dyn std::error::Error>> {
        let parsed: SubAppsResponse = serde_json::from_str(body)?;
        self.web_server_address = parsed.sub_apps.sonar.metadata.web_server_address;
        Ok(())
    }

    pub fn set_volume(&self, channel: &str, volume: f32) -> Result<(), Box<dyn std::error::Error>> {
        if !["master", "game", "chatRender", "media", "aux", "chatCapture"].contains(&channel) {
            return Err("ChannelNotFoundError".into());
        }

        if volume < 0.0 || volume > 1.0 {
            return Err("InvalidVolumeError".into());
        }

        let client = Client::builder().danger_accept_invalid_certs(true).build()?;
        let url = format!("{}{}/{}/Volume/{}", self.web_server_address, self.volume_path, channel, volume);
        //let url = format!("{}/{}{}/{}/Volume", self.web_server_address, self.volume_path, channel, volume);
        let response = client.put(&url).send()?;

        if response.status() != reqwest::StatusCode::OK {
            return Err("ServerNotAccessibleError".into());
        }        

        Ok(())
    }

    pub fn set_volume_for_channel(&mut self, channel: &str, volume: f32) {
        // First, ensure the web server address is up-to-date
        if let Err(e) = self.update_web_server_address_from_sub_apps() {
            midi_commands::show_toast("Volume Control Failed", &format!("Failed to update web server address: {}", e));
            return;
        }
        
        if !["master", "game", "chatRender", "media", "aux", "chatCapture"].contains(&channel) {
            midi_commands::show_toast("Volume Control Failed", "Channel not found");
            return;
        }
    
        if volume < 0.0 || volume > 1.0 {
            midi_commands::show_toast("Volume Control Failed", "Invalid volume");
            return;
        }
    
        let client = match Client::builder().danger_accept_invalid_certs(true).build() {
            Ok(client) => client,
            Err(e) => {
                midi_commands::show_toast("Volume Control Failed", &format!("Failed to build client: {}", e));
                return;
            },
        };
    
        let url = format!("{}{}/{}/Volume/{}", self.web_server_address, self.volume_path, channel, volume);
        let response = match client.put(&url).send() {
            Ok(response) => response,
            Err(e) => {
                midi_commands::show_toast("Volume Control Failed", &format!("Failed to send request: {}", e));
                return;
            },
        };
    
        if response.status() != reqwest::StatusCode::OK {
            midi_commands::show_toast("Volume Control Failed", "Server not accessible");
        }
    }
    
    
    /// Fetches the /subApps response and updates the web server address.
    fn update_web_server_address_from_sub_apps(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let client = Client::builder().danger_accept_invalid_certs(true).build()?;
        let sub_apps_url = format!("{}{}", self.base_url, "/subApps");
        let response_body = client.get(&sub_apps_url).send()?.text()?;
        
        // Assuming the SubAppsResponse and related structs are correctly defined to match the JSON response structure
        let parsed: SubAppsResponse = serde_json::from_str(&response_body)?;
        self.web_server_address = parsed.sub_apps.sonar.metadata.web_server_address;
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sonar = Sonar::new(false, None)?;

    // Assuming `danger_accept_invalid_certs(true)` is for local development only.
    let client = Client::builder()
        .danger_accept_invalid_certs(true)
        .build()?;

    // Fetch and parse the /subApps response to get the web server address.
    let sub_apps_url = format!("{}{}", sonar.base_url, "/subApps");
    let response_body = client.get(&sub_apps_url).send()?.text()?;
    println!("full response body: {}",response_body);
    sonar.update_web_server_address(&response_body)?;

    // Now, set the volume for a specific channel to 50%
    // The channel name should be one of the ones you've defined: ["master", "game", "chatRender", "media", "aux", "chatCapture"]
    // Volume is a float, so 50% is represented as 0.5
    sonar.set_volume("master", 0.5)?;

    println!("Volume set successfully.");

    Ok(())
}

