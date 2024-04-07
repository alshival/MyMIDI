use std::{error::Error, thread, time::Duration};
#[macro_use] extern crate lazy_static;
use enigo::*;
use midir::{MidiInput, Ignore};
use std::sync::{Arc, Mutex};
use std::process::Command;
extern crate winapi;
use winapi::shared::winerror::*;
use winapi::um::combaseapi::*;
use winapi::um::endpointvolume::*;
use winapi::um::mmdeviceapi::*;
use std::ptr::null_mut;
mod profiles;
mod steelseries_sonar_api;
use std::fmt;

// Define IID_IMMDeviceEnumerator manually
// This is the interface ID for IMMDeviceEnumerator
#[allow(non_upper_case_globals)]
const IID_IMMDeviceEnumerator: winapi::shared::guiddef::GUID = winapi::shared::guiddef::GUID {
    Data1: 0xA95664D2,
    Data2: 0x9614,
    Data3: 0x4F35,
    Data4: [0xA7, 0x46, 0xDE, 0x8D, 0xB6, 0x36, 0x17, 0xE6],
};

// Define IID_IAudioEndpointVolume manually
// This is the interface ID for IAudioEndpointVolume
#[allow(non_upper_case_globals)]
const IID_IAudioEndpointVolume: winapi::shared::guiddef::GUID = winapi::shared::guiddef::GUID {
    Data1: 0x5CDF2C82,
    Data2: 0x841E,
    Data3: 0x4546,
    Data4: [0x97, 0x22, 0x0C, 0xF7, 0x40, 0x78, 0x22, 0x9A],
};


// Function to initialize COM library and get the audio endpoint volume interface
fn get_audio_endpoint_volume() -> Result<*mut IAudioEndpointVolume, HRESULT> {
    unsafe {
        CoInitializeEx(null_mut(), COINITBASE_MULTITHREADED);
        let mut device_enumerator: *mut IMMDeviceEnumerator = null_mut();
        let hr = CoCreateInstance(&CLSID_MMDeviceEnumerator, null_mut(), CLSCTX_ALL, &IID_IMMDeviceEnumerator, &mut device_enumerator as *mut _ as *mut _);
        if SUCCEEDED(hr) {
            let mut default_device: *mut IMMDevice = null_mut();
            (*device_enumerator).GetDefaultAudioEndpoint(eRender, eConsole, &mut default_device);
            let mut endpoint_volume: *mut IAudioEndpointVolume = null_mut();
            (*default_device).Activate(&IID_IAudioEndpointVolume, CLSCTX_ALL, null_mut(), &mut endpoint_volume as *mut _ as *mut _);
            (*device_enumerator).Release();
            (*default_device).Release();
            Ok(endpoint_volume)
        } else {
            Err(hr)
        }
    }
}

// Function to set the system volume
fn set_system_volume(volume: f32) -> Result<(), HRESULT> {
    let endpoint_volume = get_audio_endpoint_volume()?;
    if !endpoint_volume.is_null() {
        unsafe {
            // Convert MIDI volume (0-127) to a percentage (0.0-1.0), then to decibels if needed
            // Here we directly use the volume as a percentage
            let hr = (*endpoint_volume).SetMasterVolumeLevelScalar(volume, null_mut());
            (*endpoint_volume).Release();
            if SUCCEEDED(hr) {
                Ok(())
            } else {
                Err(hr)
            }
        }
    } else {
        Err(E_FAIL)
    }
}

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
fn show_toast(title: &str, message: &str) {
    let ps_script = format!(r#"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null
$template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)

$textNodes = $template.GetElementsByTagName("text")
$textNodes.Item(0).AppendChild($template.CreateTextNode("{title}")) > $null
$textNodes.Item(1).AppendChild($template.CreateTextNode("{message}")) > $null

$toast = [Windows.UI.Notifications.ToastNotification]::new($template)
$notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("MyMIDI")
$notifier.Show($toast)
"#, title = title, message = message);

    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(&ps_script)
        .output()
        .expect("Failed to execute process");
}

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
            // Volume Control
            
            if message[0] == 176 && message[1] == 70 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("master",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            if message[0] == 176 && message[1] == 71 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("game",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            if message[0] == 176 && message[1] == 72 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("chatRender",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            if message[0] == 176 && message[1] == 73 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("media",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            // Check if the MIDI message should trigger a profile change
            if message[0] == 153 && message[1] == 43 {
                *profile = match *profile {
                    Profile::Default => Profile::Genshin,
                    Profile::Genshin => Profile::Sky,
                    Profile::Sky => Profile::Default,
                };
                let profile_name = format!("{}", *profile); // Convert the profile to a string
                show_toast("Profile Changed", &format!("{} profile is now active.", profile_name));
                //println!("Current profile: {}", profile_name); // Use if needed for debugging
            }
            if message[0] == 153 && message[1] == 39 {
                // Launch TIDAL
                //println!("Launching Tidal...");
                let _ = Command::new("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe")
                    .spawn()
                    .expect("TIDAL launch failed");
            }

            // Go to the previous song on specific MIDI message
            if message[0] == 153 && message[1] == 36 {
                //n!("Playing previous song...");
                let mut enigo = Enigo::new();
                enigo.key_click(Key::MediaPrevTrack);
            }

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

            // Delegate to the appropriate profile's message handler
            match *profile {
                Profile::Default => profiles::default::handle_message(message),
                Profile::Genshin => profiles::genshin::handle_message(message),
                _ => {},
            }
        }, ())?;

        println!("Connected. Monitoring for disconnection...");

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