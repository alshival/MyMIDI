use std::collections::HashMap;
use std::{error::Error, thread, time::Duration};
#[macro_use] extern crate lazy_static;
use std::sync::{Arc, Mutex};
use enigo::{
    Direction::{Click,Press,Release},
    Enigo, Key, Keyboard, Settings,
};
use std::fmt;
use std::env;
use midir::{MidiInput, Ignore};
mod profiles;
mod steelseries_sonar_api;
mod midi_commands;

/*###############################################################################
Profile Delegation 
Adding a new profile requires a few steps.
    1. Create `src/profiles/profile_name.rs` with your logic to handle messages.
    2. Add `pub mod profile_name;` to `src/profiles/mod.rs` so you can import it
        into main.rs
    3. Add to enum list in this next section.
    4. Include it in the profile change button within the main() function.
    5. Map the profile name to the handle_message function within the main() function.
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

fn main() -> Result<(), Box<dyn Error>> {
    loop {
        /*###############################################################################
        Set default profile here.
            Currently, the default profile is called Default
        ###############################################################################*/
        let current_profile = Arc::new(Mutex::new(Profile::Default));

        // Used for relative paths in template code.
        let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));
     
        //let mut enigo = Enigo::new(&Settings::default()).unwrap();
        /*******************************************************************************
        SteelSeries Audio setup
            If you are not using SteelSeries, you can comment this part out.
            I didn't want to run this each time I turn a knob, so I setup Sonar here.
        *******************************************************************************/    
        use steelseries_sonar_api::Sonar;
        let mut sonar = Sonar::new(false,None)?;
        
        // button_states is used to sustain key presses. For example, in some rhythm games,
        // you hold down a button for long notes. To accomplish this, we need to understand
        // how our MIDI handles note-on/note-off buttons.
        // My midi uses [144,x,y] for piano key note ons and [128,x,y] for piano key note offs.
        // Since I am using the drum pad buttons for launching apps and things, I don't need those
        // to sustain. 
        let button_states: Arc<Mutex<HashMap<u8, bool>>> = Arc::new(Mutex::new(HashMap::new()));

        /*******************************************************************************
        MIDI input reading
        *******************************************************************************/
        let mut midi_in = MidiInput::new("midi_reader_input")?;
        midi_in.ignore(Ignore::None);
        let ports = midi_in.ports();
        let mut enigo = Enigo::new(&Settings::default()).unwrap();

        // This checks if there is a device open at startup. If there is none, the app waits.
        if ports.is_empty() {
            println!("No MIDI input ports available. Waiting for connection...");
            thread::sleep(Duration::from_secs(5));
            continue; // Skip the rest of the loop and check again
        }

        let in_port = &ports[0];
        // Let user know MyMIDI is listening for input
        midi_commands::show_toast("MyMIDI", &format!("Listening on {}",midi_in.port_name(in_port)?));
        println!("Listening on {}", midi_in.port_name(in_port)?);
        
        let button_states_clone = Arc::clone(&button_states);
        // Clone the profile Arc for use in the closure
        let profile_for_closure = current_profile.clone();
        
        let mut connection = midi_in.connect(in_port, "midi_reader_input", move |_stamp, message, _| {
            println!("Received MIDI message: {:?}", message);
            let mut profile = profile_for_closure.lock().unwrap(); // Lock the mutex and get the profile
            /*###############################################################################
            Profile Change Button 
                Dedicate a button to changing profiles
            ###############################################################################*/
            if message[0] == 153 && message[1] == 43 {
                // Cycle through the profiles: Default -> Genshin -> Sky (dummy profile) -> Default
                *profile = match *profile {
                    Profile::Default => Profile::Genshin,
                    Profile::Genshin => Profile::Sky,
                    Profile::Sky => Profile::Default,
                };
                let profile_name = format!("{}", *profile); // Convert the profile to a string
                midi_commands::show_toast("Profile Changed", &format!("{} profile is now active.", profile_name));
                //println!("Current profile: {}", profile_name); // Use if needed for debugging
            }
            /*###############################################################################
            Cross-profile Button Assignment
                Again, button assignments defined within this main.rs function persist
                across profiles. I have three buttons assigned as media keys for 
                Previous Track, Play/Pause, and Next Track
            ###############################################################################*/
            // Go to the previous song on specific MIDI message
            if message[0] == 153 && message[1] == 36 {
                //n!("Playing previous song...");
                enigo.key(Key::MediaPrevTrack,Click);
            }

            // Play/Pause the music
            if message[0] == 153 && message[1] == 37 {
                // Simulate play/pause key press
                //println!("Toggling play/pause...");
                enigo.key(Key::MediaPlayPause,Click);
            }

            // Go to the next song on specific MIDI message
            if message[0] == 153 && message[1] == 38 {
                //println!("Playing next song...");
                enigo.key(Key::MediaNextTrack,Click);
            }
            // Launch music player
            if message[0] == 153 && message[1] == 39 {
                let path = format!(r"C:\\Users\\{}\\AppData\\Local\\TIDAL\\TIDAL.exe",username);
                // Note that setting path like this does NOT return an object of type &str. It returns a string, so we add &path when passing it to launch_exe
                // If we had instead done this:
                // let path = "your/path/here"
                // without using format, you wouldn't need to add an & before passing it to launch_exe
                midi_commands::launch_exe(&path);
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
                Profile::Genshin => profiles::genshin::handle_message(&mut enigo, &mut button_states_clone.lock().unwrap(), message),
                _ => {},
            }
        }, ())?;

        println!("Connected. Monitoring for disconnection...");
        /*******************************************************************************
        Scheduled Tasks
            This section includes code for routine maintenance. For example, there is code
            that monitors whether the MIDI is disconnected, and if it is, it closes the port
            and the app waits for another MIDI device to connect.
        *******************************************************************************/
        // Monitor connection in a block to allow for connection closure
        {
            let midi_in_check = MidiInput::new("midi_checker")?;
            let mut is_connected = true;
            while is_connected {
                thread::sleep(Duration::from_secs(1)); // Adjust based on desired responsiveness

                if midi_in_check.ports().is_empty() || !midi_in_check.ports().contains(in_port) {
                    println!("MIDI device disconnected.");
                    midi_commands::show_toast("MyMIDI", "MIDI disconnected. Standing by.");
                    is_connected = false; // Exit the monitoring loop
                }
            }
        } // `midi_in_check` goes out of scope and is dropped here

        // Properly close the connection before attempting to reconnect
        connection.close();
        println!("Attempting to reconnect...");
    }
}