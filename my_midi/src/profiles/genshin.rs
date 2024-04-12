use std::collections::HashMap;
use std::sync::Mutex;
use std::env;
use enigo::{
    Direction::Click,
    Enigo, Key, Keyboard,Settings,
};
use crate::toast;
use crate::midi_commands;

// Define an enum to represent the current scale state
enum ScaleType {
    Lows,
    Highs,
    Complete,
}

lazy_static! {
    static ref LOWS: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'z'), (50, 'x'), (52, 'c'), (53, 'v'), (55, 'b'), (57, 'n'), (59, 'm'),
        (60, 'a'), (62, 's'), (64, 'd'), (65, 'f'), (67, 'g'), (69, 'h'), (71, 'j'), (72, 'q'),
    ]));
    static ref HIGHS: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'a'), (50, 's'), (52, 'd'), (53, 'f'), (55, 'g'), (57, 'h'), (59, 'j'),
        (60, 'q'), (62, 'w'), (64, 'e'), (65, 'r'), (67, 't'), (69, 'y'), (71, 'u'),
    ]));
    static ref COMPLETE: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'z'), (49, 'x'), (50, 'c'), (51, 'v'), (52, 'b'), (53, 'n'), (54, 'm'),
        (55, 'a'), (56, 's'), (57, 'd'), (58, 'f'), (59, 'g'), (60, 'h'), (61, 'j'),
        (62, 'q'), (63, 'w'), (64, 'e'), (65, 'r'), (66, 't'), (67, 'y'), (68, 'u'),
    ]));
    static ref CURRENT_SCALE: Mutex<ScaleType> = Mutex::new(ScaleType::Complete);
}

pub fn handle_message(message: &[u8]) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    
    // Used for relative paths in template code.
    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));

    // Determine the current scale and directly work with its lock guard
    let scale_guard = match *CURRENT_SCALE.lock().unwrap() {
        ScaleType::Lows => LOWS.lock().unwrap(),
        ScaleType::Highs => HIGHS.lock().unwrap(),
        ScaleType::Complete => COMPLETE.lock().unwrap(),
    };

    if message[0] == 144 && message[2] != 0 { //message[2] is velocity. Not really needed, though including it anyways.
        let note = message[1];
        match note {
            _ => {
                if let Some(&key_char) = scale_guard.get(&note) {
                    enigo.key(Key::Unicode(key_char),Click);
                    //println!("Pressed key: {}", key_char);
                }
            }
        }
    }

    if message[0] == 153 { // Adjust channel checking as needed
        let note = message[1]; // MIDI note number

        if note == 40 {
            let mut scale = CURRENT_SCALE.lock().unwrap();
            *scale = match *scale {
                ScaleType::Complete => {
                    //println!("Toggled to Complete Layout");
                    toast::show_toast("Music Layout Change", "Toggled to Layout 2: Lows");
                    ScaleType::Lows
                },
                ScaleType::Lows => {
                    //println!("Toggled to Highs");
                    toast::show_toast("Music Layout Change","Toggled to Layout 2: Highs");
                    ScaleType::Highs
                },
                ScaleType::Highs => {
                    //println!("Toggled to Lows");
                    toast::show_toast("Music Layout Change","Toggled to Layout 1: Complete");
                    ScaleType::Complete
                },
            };
        }

        if note == 41 {
            // Open HoyoLab in a browser - For Daily Check ins
            let url = "https://www.hoyolab.com/";
            crate::midi_commands::open_url(url);
        }

        if note == 42 {
            // Open Teyvat Map in a browser
            let url = "https://act.hoyolab.com/ys/app/interactive-map/index.html?bbs_presentation_style=no_header&utm_id=2&utm_medium=tool&utm_source=hoyolab&bbs_theme=dark&bbs_theme_device=1&lang=en-us#/map/2?shown_types=&center=2008.50,-1084.00&zoom=-3.00";
            crate::midi_commands::open_url(url);
        }
    }
}
