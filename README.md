# MyMIDI
Read about this project on [Substack](https://alshival.substack.com/p/a-coders-guide-to-midi-driven-hotkeys).

Code to use MIDI buttons as hot keys. Each button can be assigned a different task. This code is meant to serve as a sort of template, as much of it will be dependent on your preferred MIDI device and what tasks you wish to assign each button.
Originally, I had written this program in Python, but it was a resource hog, so it was ported over to Rust. It is light enough now to keep running in the background. MyMIDI was written for the Akai MPK Mini Play and must be adapted for your device.
<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240406_140035397.jpg">
<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240405_1927445722.jpg">

Out of the box, it is set up for handling multi-channel audio via SteelSeries Sonar, though code is in place if you wish to adjust volume using standard windows. You'll have to comment out the lines of code that handle Sonar and uncomment the Windows volume control. Currently, MyMIDI only supports SteelSeries Sonar on classic mode, meaning Sonar's Streamer Mode is not yet operational, though it is in the works.

<img src="https://github.com/alshival/MyMIDI/blob/main/media/demo.gif">

You can make the changes in `main.rs`. Look for these lines of code that handle the knobs:
```
// Master Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this:
            if message[0] == 176 && message[1] == 70 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("master",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            // Game Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this:
            if message[0] == 176 && message[1] == 71 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("game",midi_volume);
                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            // Chat Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust or delete this:
            if message[0] == 176 && message[1] == 72 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("chatRender",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }

            // Media Channel Volume
            // If you don't want to use SteelSeries Sonar, you'll have to adjust this:
            if message[0] == 176 && message[1] == 73 {
                let midi_volume = message[2] as f32 / 127.0; // Convert MIDI volume to a float in range 0.0 to 1.0
                sonar.set_volume_for_channel("media",midi_volume);

                // If you don't want to use Steelseries Sonar, then comment out the above and uncomment this next part.

                // match set_system_volume(midi_volume){
                //     Ok(_) => {},
                //     Err(_e) => {},
                // };
            }
```
<img src="https://github.com/alshival/MyMIDI/blob/main/media/Screenshot%202024-04-08%2014365423.png">

# Setup

Navigate into `MyMIDI/my_midi` and run `cargo build --release`. I have it installed at `C:\MyMIDI`

```
cd C:\MyMIDI\my_midi\
cargo build --release
```

The script `run_silently.vbs` opens MyMIDI in the background. You can use this .vbs with Task Scheduler so that MyMIDI opens when you log on. You will have to point the script to the release file you just built, wherever you decide to install MyMIDI:
```
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\MyMIDI\my_midi\target\release\my_midi.exe & pause", 0, True
Set WshShell = Nothing
```
<img src="https://github.com/alshival/MyMIDI/blob/main/media/Screenshot%202024-04-06%20194707.png">
