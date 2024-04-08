use std::{error::Error, thread, time::Duration};
#[macro_use] extern crate lazy_static;
use enigo::*;
use midir::{MidiInput, Ignore};
use std::sync::{Arc, Mutex};
use rusqlite::{Connection, Result as SqlResult};
use std::path::Path;
use std::fmt;
// Custom mods
mod profiles;
mod steelseries_sonar_api;
mod windows_volume_control;
mod toast;
mod midi_commands;


/*###############################################################################
Profile Delegation
Adding a new profile requires a few steps.
    1. Create `src/profiles/profile_name.rs` with your logic to handle messages.
    2. Add `pub mod profile_name;` to `src/profiles/mod.rs` so you can import it
        into main.rs
    3. Add to enum list in this next section.
    4. Map the profile name to the handle_message function within main.rs.
       (see near end of main.rs script)
###############################################################################*/
#[derive(Debug, Clone, Copy)]
enum Profile {
    Default,
    Genshin,
    Sky,
}
impl fmt::Display for Profile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Profile::Default => "Default",
            Profile::Genshin => "Genshin",
            Profile::Sky => "Sky",
        })
    }
}

/*###############################################################################
Currently, SteelSeries Sonar Streamer Mode is not functional, but it is a work in progress. 
This is the code to extract the mode from the SteelSeries database.
If you aren't using SteelSeries Sonar for multi-channel audio, you can remove this safely
as well as any SteelSeries code in the main() function.
###############################################################################*/
fn fetch_streamer_mode() -> SqlResult<bool> {
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

/*###############################################################################
main.rs
    Any button assignments made within main.rs will persist across profiles.
###############################################################################*/
fn main() -> Result<(), Box<dyn Error>> {
    loop {
        // Recreate the MidiInput object each loop iteration to avoid ownership issues
        let mut midi_in = MidiInput::new("midi_reader_input")?;
        midi_in.ignore(Ignore::None);
        use steelseries_sonar_api::Sonar;
        let mut sonar = Sonar::new(false,None)?;

        let current_profile = Arc::new(Mutex::new(Profile::Default));

        let ports = midi_in.ports();
        if ports.is_empty() {
            println!("No MIDI input ports available. Waiting for connection...");
            thread::sleep(Duration::from_secs(5));
            continue; // Skip the rest of the loop and check again
        }

        let in_port = &ports[0]; // Assuming we want the first available port
        println!("Listening on {}", midi_in.port_name(in_port)?);
        // Clone the Arc for use in the closure
        let profile_for_closure = current_profile.clone();

        // The `connect` method consumes `midi_in`, so it's not available after this call
        let mut connection = midi_in.connect(in_port, "midi_reader", move |_, message, _| {
            let mut profile = profile_for_closure.lock().unwrap(); // Lock the mutex and get the profile
            // Handle MIDI messages here
            println!("Received MIDI message: {:?}", message);

            /*###############################################################################
            Profile Change Button
                Dedicate a button to changing profiles
            ###############################################################################*/
            // Check if the MIDI message should trigger a profile change
            if message[0] == 153 && message[1] == 43 {
                *profile = match *profile {
                    Profile::Default => Profile::Genshin,
                    Profile::Genshin => Profile::Sky,
                    Profile::Sky => Profile::Default,
                };
                let profile_name = format!("{}", *profile); // Convert the profile to a string
                toast::show_toast("Profile Changed", &format!("{} profile is now active.", profile_name));
                //println!("Current profile: {}", profile_name); // Use if needed for debugging
            }

            /*###############################################################################
            Volume Control
                If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this.
                Note that you can define volume control at the profile level by including it 
                in the profile's `handle_message` function. This would allow you to reassign knobs
                across different profiles. 
            ###############################################################################*/
            if message[0] == 176 && message[1] == 70 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                
                // Using Sonar:
                sonar.set_volume_for_channel("master",midi_volume);

                // Using Windows:
                //windows_volume_control::set_system_volume(midi_volume);
            }

            // Game Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this:
            if message[0] == 176 && message[1] == 71 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("game",midi_volume);

                // Using Windows:
                //windows_volume_control::set_system_volume(midi_volume);
            }

            // Chat Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this:
            if message[0] == 176 && message[1] == 72 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("chatRender",midi_volume);

                // Using Windows:
                //windows_volume_control::set_system_volume(midi_volume);
            }

            // Media Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust this:
            if message[0] == 176 && message[1] == 73 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("media",midi_volume);

                // Using Windows:
                //windows_volume_control::set_system_volume(midi_volume);
            }
            /*###############################################################################
            Media Keys
                Again, anything button assignments defined within this main.rs function persist
                across profiles. I have three buttons assigned as media keys for 
                Previous Track, Play/Pause, and Next Track
            ###############################################################################*/
            // Go to the previous song on specific MIDI message
            if message[0] == 153 && message[1] == 36 {
                //n!("Playing previous song...");
                let mut enigo = Enigo::new();
                enigo.key_click(Key::MediaPrevTrack);
            }

            // Play/Pause the music
            if message[0] == 153 && message[1] == 37 {
                // Simulate play/pause key press
                //println!("Toggling play/pause...");
                let mut enigo = Enigo::new();
                enigo.key_click(Key::MediaPlayPause);
            }

            // Go to the next song on specific MIDI message
            if message[0] == 153 && message[1] == 38 {
                //println!("Playing next song...");
                let mut enigo = Enigo::new();
                enigo.key_click(Key::MediaNextTrack);
            }
            /*###############################################################################
            Application Launch 
                Again, anything button assignments defined within this main.rs function persist
                across profiles. I have Tidal launch across all profiles.
            ###############################################################################*/
            if message[0] == 153 && message[1] == 39 {
                midi_commands::launch_exe("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe");
            }


            /*###############################################################################
            Profile Delegation
                Adding a new profile requires a few steps.
                    1. Create `src/profiles/profile_name.rs` with your logic to handle messages.
                    2. Add `pub mod profile_name;` to `src/profiles/mod.rs` so you can import it
                    into main.rs
                    3. Add to enum list near the top of main.rs.
                    4. Map the profile name to the handle_message function in this next section.
            ###############################################################################*/
            // Delegate to the appropriate profile's message handler
            match *profile {
                Profile::Default => profiles::default::handle_message(message),
                Profile::Genshin => profiles::genshin::handle_message(message),
                _ => {},
            }
        }, ())?;

        println!("Connected. Monitoring for disconnection...");

        /*###############################################################################
        Scheduled Tasks
            This section includes code for routine maintenance. For example, there is code
            that monitors whether the MIDI is disconnected, and if it is, it closes the port
            and the app waits for another MIDI device to connect.
        ###############################################################################*/
        // Monitor connection in a block to allow for connection closure
        {
            let midi_in_check = MidiInput::new("midi_checker")?;
            let mut is_connected = true;
            while is_connected {
                thread::sleep(Duration::from_secs(1)); // Adjust based on desired responsiveness

                if midi_in_check.ports().is_empty() || !midi_in_check.ports().contains(in_port) {
                    println!("MIDI device disconnected.");
                    is_connected = false; // Exit the monitoring loop
                }
            }
        } // `midi_in_check` goes out of scope and is dropped here

        // Properly close the connection before attempting to reconnect
        connection.close();
        println!("Attempting to reconnect...");
    }
}