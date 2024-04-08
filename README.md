# MyMIDI
Read about this project on [Substack](https://alshival.substack.com/p/a-coders-guide-to-midi-driven-hotkeys).

Code to use MIDI buttons as hot keys. Each button can be assigned a different task. This code is meant to serve as a sort of template, as much of it will be dependent on your preferred MIDI device and what tasks you wish to assign each button.
Originally, I had written this program in Python, but it was a resource hog, so it was ported over to Rust. It is light enough now to keep running in the background. MyMIDI was written for the Akai MPK Mini Play and must be adapted for your device.
<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240406_140035397.jpg">
<img src="https://github.com/alshival/MyMIDI/blob/main/media/IMG_20240405_1927445722.jpg">

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
