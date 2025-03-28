use std::collections::HashMap;
use std::sync::Mutex;
use crate::midi_commands;
use std::env;
use enigo::{
    Direction::{Click, Press, Release},
    Enigo, Key, Keyboard,Settings,
};

pub fn handle_message(enigo: &mut Enigo, button_states: &mut HashMap<u8, bool>, message: &[u8]) {
    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));

    // Launch Visual Studio Code
    if message[0] == 153 && message[1] == 40 {
        let path = format!("C:\\Users\\{}\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe", username);
        // Note that setting path like this does NOT return an object of type &str. It returns a string, so we add &path when passing it to launch_exe
        midi_commands::launch_exe(&path);
    }


    // Launch UpWork C:\Users\samue\AppData\Local\Programs\upwork
    if message[0] == 153 && message[1] == 41 {
        let path = format!("C:\\Users\\{}\\AppData\\Local\\Programs\\upwork\\Upwork.exe", username);
        // Note that setting path like this does NOT return an object of type &str. It returns a string, so we add &path when passing it to launch_exe
        midi_commands::launch_exe(&path);
    }

    // Launch SteelSeries GG
    if message[0] == 153 && message[1] == 42 {
        let path = "C:\\Program Files\\SteelSeries\\GG\\SteelSeriesGG.exe";
        // Note that setting path like this returns an object of type &str, which is what we need for launch_exe
        midi_commands::launch_exe(&path);
    }


    fn enter_hash(enigo: &mut Enigo) {
        enigo.key(Key::Shift, Press).unwrap();
        enigo.key(Key::Unicode('3'), Click).unwrap();
        enigo.key(Key::Shift, Release).unwrap();
    }
    // Code Header Line
    if message[0] == 144 && message[1] == 71 {
        enigo.key(Key::Alt, Press).unwrap();
        enigo.key(Key::Space, Click).unwrap();
        enigo.key(Key::Alt, Release).unwrap();
    }

    // Code Header Line
    if message[0] == 144 && message[1] == 72 {
        // Enter # 80 times to create a text header
        for _ in 0..80 { enter_hash(enigo); }
        enigo.key(Key::Return, Click).unwrap();
        enter_hash(enigo);
        enigo.key(Key::Return, Click).unwrap();
        for _ in 0..80 { enter_hash(enigo); }
        enigo.key(Key::Return, Click).unwrap();
    }


}
