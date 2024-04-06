#[macro_use] extern crate lazy_static;
use enigo::*;
use midir::{MidiInput, Ignore};
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::process::Command;
extern crate winapi;
use winapi::shared::winerror::*;
use winapi::um::combaseapi::*;
use winapi::um::endpointvolume::*;
use winapi::um::mmdeviceapi::*;
use winapi::shared::guiddef::*;
use std::ptr::null_mut;
mod profiles;

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

fn main() -> Result<(), Box<dyn Error>> {
    let mut midi_in = MidiInput::new("midi_reader_input")?;
    midi_in.ignore(Ignore::None);

    let ports = midi_in.ports();
    let in_port = ports.get(0).ok_or("No MIDI input ports available.")?;

    println!("Listening on {}", midi_in.port_name(in_port)?);

    let current_profile = Arc::new(Mutex::new(Profile::Default));

    let profile_for_closure = current_profile.clone();
    let _conn_in = midi_in.connect(in_port, "midi_reader", move |_, message, _| {
        let mut profile = profile_for_closure.lock().unwrap();
        println!("{:?}", message);
        // Volume Control
        if message[0] == 176 && message[1] == 70 {
            let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
            println!("Setting system volume to: {}", midi_volume * 100.0);
            match set_system_volume(midi_volume) {
                Ok(_) => println!("Volume set successfully"),
                Err(e) => println!("Failed to set volume: HRESULT {}", e),
            }
        }
        // Check if the MIDI message should trigger a profile change
        if message[0] == 153 && message[1] == 43 {
            *profile = match *profile {
                Profile::Default => Profile::Genshin,
                Profile::Genshin => Profile::Sky,
                Profile::Sky => Profile::Default,
            };
            println!("Current profile: {:?}", *profile);
        }
        if message[0] == 153 && message[1] == 39 {
            // Launch TIDAL
            println!("Launching Tidal...");
            let _ = Command::new("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe")
                .spawn()
                .expect("TIDAL launch failed");
        }

        // Go to the previous song on specific MIDI message
        if message[0] == 153 && message[1] == 36 {
            println!("Playing previous song...");
            let mut enigo = Enigo::new();
            enigo.key_click(Key::MediaPrevTrack);
        }

        if message[0] == 153 && message[1] == 37 {
            // Simulate play/pause key press
            println!("Toggling play/pause...");
            let mut enigo = Enigo::new();
            enigo.key_click(Key::MediaPlayPause);
        }

        // Go to the next song on specific MIDI message
        if message[0] == 153 && message[1] == 38 {
            println!("Playing next song...");
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

    // Infinite loop to keep the program running
    loop {
        // You may want to introduce a sleep here to avoid excessive CPU usage
        //std::thread::sleep(std::time::Duration::from(1));
    }

    // Ok(())
}
