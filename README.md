# Alshival's MIDI Code
Code to use MIDI buttons as hot keys. Each button can be assigned a different task. This code is meant to serve as a sort of template, as much of it will be dependent on your prefered MIDI device and what tasks you wish to assign each button.
Originally, I had written this program in Python, but it was a resource hog, so I rewrote it in Rust. Works even better than the python version did.
<img src="https://github.com/alshival/MyMIDI/blob/main/IMG_20240405_192744572.jpg">
I have a dedicated knob for PC master volume, which is controlled via `winapi`. It requires a bit of setup, but this part shows how I use that specific knob to adjust the volume. My MIDI sends a value between 0 and 127, which I must standardize before setting the volume.

I am working on a Akai MPK Mini Play.
```
if message[0] == 176 && message[1] == 70 {
    let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0. 
    println!("Setting system volume to: {}", midi_volume * 100.0);
    match set_system_volume(midi_volume) {
        Ok(_) => println!("Volume set successfully"),
        Err(e) => println!("Failed to set volume: HRESULT {}", e),
    }
}
```
I have a dedicated profile button to swap between profiles. Here's the relevant code:
```
// Check if the MIDI message should trigger a profile change
if message[0] == 153 && message[1] == 43 {
    *profile = match *profile {
        Profile::Default => Profile::Genshin,
        Profile::Genshin => Profile::Sky,
        Profile::Sky => Profile::Default,
    };
    println!("Current profile: {:?}", *profile);
}
```
This allows me to assign each button a different task across different profiles. The `Genshin` profile is an example of how one would use their MIDI in a video game. My MIDI only has 25 keys, which isn't enough to cover the entire music scale available in Genshin Impact, and so I have a way of switching the music key layout.

Across all profiles, a few buttons are dedicated to music playback. One button opens Tidal, another is for play/pause, another to play the Previous Track, and another for Next Track:
```
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
```
