# MyMIDI
Read about this project on [Substack](https://alshival.substack.com/p/a-coders-guide-to-midi-driven-hotkeys).

Code to use MIDI buttons as hot keys. Each button can be assigned a different task. This code is meant to serve as a sort of template, as much of it will be dependent on your prefered MIDI device and what tasks you wish to assign each button.
Originally, I had written this program in Python, but it was a resource hog, so it as ported over to Rust.
<img src="https://github.com/alshival/MyMIDI/blob/main/IMG_20240405_1927445722.jpg">
I have a dedicated knob for PC master volume, which is controlled via `winapi`. It requires a bit of setup, but this part shows how I use that specific knob to adjust the volume. My MIDI sends a value between 0 and 127, which I must standardize before setting the volume.

MyMIDI was written for the Akai MPK Mini Play.
