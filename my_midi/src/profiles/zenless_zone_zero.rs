// zenless_zone_zero.rs
use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard,Settings,
};
use crate::midi_commands;
/*###############################################################################
zenless_zone_zero Layout
The zenless_zone_zero layout isn't the most complicated layout. It's most important 
feature is that it allows you to take screenshots quickly. 
I have set up Xbox Game Bar to take screenshots with 'Alt+P', which I emulate
in this profile. The drum pad buttons are similar to the Genshin Impact layout.
One button allows me to open up Hoyo's social website quickly.
###############################################################################*/
pub fn handle_message(enigo: &mut Enigo, button_states: &mut HashMap<u8, bool>, message: &[u8]) {
    let note = message[1];
    if message[0] == 153{
        if note == 41 {
            //Open Hololab in a browser - For Daily Check ins
            let url: &str = "https://www.hoyolab.com/";
            crate::midi_commands::open_url(url);
        }

        if note == 42 {
            //Open Wiki in a browser
            let url: &str = "https://zenless-zone-zero.fandom.com/wiki/Zenless_Zone_Zero";
            crate::midi_commands::open_url(url);
        }
    }

    if message[0] == 144 {
        //Assign last button on Midi to take a screenshot 
        if message[1] == 72 {
            //Take a screenshot
            enigo.key(Key::Alt,Press).unwrap();
            enigo.key(Key::Unicode('p'),Click).unwrap();
            enigo.key(Key::Alt,Release).unwrap();
            print!("Screenshot taken.");
        }
    }
}