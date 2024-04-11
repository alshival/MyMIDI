# MyMIDI
Read about this project on [Substack](https://alshival.substack.com/p/a-coders-guide-to-midi-driven-hotkeys).

Code to use MIDI buttons as hot keys. Each button can be assigned a different task or macro. Supports multiple profiles. This code is meant to serve as a template, as much of it will be dependent on your preferred MIDI device and what tasks you wish to assign each button.

<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240406_140035397.jpg">
<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240405_1927445722.jpg">

Originally, MyMIDI was written in Python, but it was a resource hog (12% CPU & 400MB Memory), so it was ported over to Rust (0% CPU & 1.5MB Memory).  It is light enough now to keep running in the background. 

<img src="https://github.com/alshival/MyMIDI/blob/main/media/Screenshot%202024-04-11%20172522.png">

MyMIDI was written for the Akai MPK Mini Play and must be adapted for your device. Out of the box, it is set up for handling multi-channel audio via SteelSeries Sonar, though code is in place if you wish to adjust volume using standard windows. You'll have to comment out the lines of code that handle Sonar and uncomment the Windows volume control. Currently, MyMIDI only supports SteelSeries Sonar on classic mode, meaning Sonar's Streamer Mode is not yet operational, though it is in the works.

<img src="https://github.com/alshival/MyMIDI/blob/main/media/demo.gif">

You can make the changes in `main.rs`. Look for these lines of code that handle the knobs:
```
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
```
You can dedicate buttons to your favorite macros or your favorite applications. You can also specify these at the profile level. MyMIDI includes a module `midi_commands::launch_exe` for quickly setting this up.
```
/*###############################################################################
Application Launch 
    Again, any button assignments defined within this main.rs function persist
    across profiles. I have Tidal launch across all profiles.
###############################################################################*/
if message[0] == 153 && message[1] == 39 {
    midi_commands::launch_exe("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe");
}
```
<img src="https://github.com/alshival/MyMIDI/blob/main/media/Screenshot%202024-04-08%2014365423.png">

# Setup

Navigate into `MyMIDI/my_midi` and run `cargo build --release`. I have it installed at `C:\MyMIDI`

```
cd C:\MyMIDI\my_midi\
cargo build --release
```

This creates the file `C:\MyMIDI\my_midi\target\release\my_midi.exe`. The script `run_silently.vbs` opens MyMIDI in the background. You can use this .vbs with Task Scheduler so that MyMIDI opens when you log on. You will have to point the script to the release file you just built, wherever you decide to install MyMIDI:
```
Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\MyMIDI\my_midi\target\release\my_midi.exe & pause", 0, True
Set WshShell = Nothing
```
<img src="https://github.com/alshival/MyMIDI/blob/main/media/Screenshot%202024-04-06%20194707.png">

Note that when Task Scheduler runs this .vbs script, it starts a separate process that runs `C:\MyMIDI\my_midi\target\release\my_midi.exe`, so ending the process through Task Scheduler will not kill `my_midi.exe`. You will have to end the process in Task Manager as well. If you have any problems with MyMIDI starting, check if there is a zombie process in Task Manager. It may be keeping the MIDI port open.
