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

/*###############################################################################
Music Layouts
    Genshin allows you to play music using the keyboard by clicking specific keys. 
    Here, we map the piano keys on our MIDI to letters on our keyboard by simulating a key press.
    Note that some video games use raw input, which ignore simulated key presses, 
    meaning this approach will not work. One example is Sky: Children of the Light.

    When we click a button using our MIDI, we obtain a signal of the 
    form [144,48,37]. The first two numbers help you distinguish the button
    being pressed. 
    144 is related to the channel the button is on. Many buttons will be on that channel. 
    48 is the specific note. This is what we map in the section below to the letter 'z' on our keyboard:
        (48,'z') 
    The last number, 37, is either a measure of velocity or pressure the button was pressed. 
    We don't really need it, but you might want to make sure it is nonzero in your message handling logic.

    Since the Akai MPK Mini Play is too small to use only the white keys to play music,
    which is what the scale should be, I incorporate two layouts. The 'Complete' layout includes
    all playable notes in Genshin by utilizing the black piano keys, even though Genshin plays in
    the scale of C which does not use black keys. The 'Highs' and 'Lows' use only white keys,
    i.e. standards C scale, but do not cover all playable notes.
    You can define the default music layout in this section.
###############################################################################*/
lazy_static! {
    // Layout 1: Complete
    static ref COMPLETE: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'z'), (49, 'x'), (50, 'c'), (51, 'v'), (52, 'b'), (53, 'n'), (54, 'm'),
        (55, 'a'), (56, 's'), (57, 'd'), (58, 'f'), (59, 'g'), (60, 'h'), (61, 'j'),
        (62, 'q'), (63, 'w'), (64, 'e'), (65, 'r'), (66, 't'), (67, 'y'), (68, 'u'),
    ]));
    // Layout 2: Lows
    static ref LOWS: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'z'), (50, 'x'), (52, 'c'), (53, 'v'), (55, 'b'), (57, 'n'), (59, 'm'),
        (60, 'a'), (62, 's'), (64, 'd'), (65, 'f'), (67, 'g'), (69, 'h'), (71, 'j'), (72, 'q'),
    ]));
    // Layout 2: Highs
    static ref HIGHS: Mutex<HashMap<u8, char>> = Mutex::new(HashMap::from([
        (48, 'a'), (50, 's'), (52, 'd'), (53, 'f'), (55, 'g'), (57, 'h'), (59, 'j'),
        (60, 'q'), (62, 'w'), (64, 'e'), (65, 'r'), (67, 't'), (69, 'y'), (71, 'u'),
    ]));

    // Set Default profile.
    static ref CURRENT_SCALE: Mutex<ScaleType> = Mutex::new(ScaleType::Complete);
}

/****************************************************************************** 
handle_message
    This is where you include all your logic for incoming messages for each button.
***************************************************************************** */
pub fn handle_message(message: &[u8]) {
    let mut enigo = Enigo::new(&Settings::default()).unwrap();
    // Used for relative paths in template code.
    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));

    /****************************************************************************** 
    This part is for switching between music layouts. If you only have one layout and don't need to switch,
    you can probably remove this.
    Determine the current scale and directly work with its lock guard
    ******************************************************************************/
    let scale_guard = match *CURRENT_SCALE.lock().unwrap() {
        ScaleType::Lows => LOWS.lock().unwrap(),
        ScaleType::Highs => HIGHS.lock().unwrap(),
        ScaleType::Complete => COMPLETE.lock().unwrap(),
    };
    /*###############################################################################
    Map Piano Keys
        When I press down on a piano key on my MIDI, I get a signal of the form [144,n,v], where v is nonzero.
        When I release the piano key, I get another signal. Something like [136,n,0].
        But I only care about when I press down on the button.
        So, I check if the incoming message starts with 144 and has a nonzero velocity here.
        Then n gets mapped to the letter using the music layout currently selected.
    ###############################################################################*/
    if message[0] == 144 && message[2] != 0 { //message[2] is velocity. Not really needed, though including it anyways.
        let note = message[1];
        match note { // Here, we match the note.
            _ => {
                if let Some(&key_char) = scale_guard.get(&note) { //If note is in our scale:
                    enigo.key(Key::Unicode(key_char),Click); //click the key_char defined in the scale.
                    //println!("Pressed key: {}", key_char);
                }
            }
        }
    }

    /*###############################################################################
    Button Assignment 
        Drum pad buttons on my MIDI send a signal of the form [153,n,v].
        So for an incomming message, I first check if the first number is 153.
        Then I check for the n for each specific button I wish to assign.
    ###############################################################################*/
    if message[0] == 153 { // Adjust channel checking as needed
        let note = message[1]; // MIDI note number
        
        /*###############################################################################
        Music Layout Switching
            If your MIDI is large enough to cover all notes, you may not need this button.
        ###############################################################################*/
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
        /*###############################################################################
        Button Assignment - HoyoLab
            Launch HoyoLab in a browser. For daily checkins and things.
        ###############################################################################*/
        if note == 41 {
            // Open HoyoLab in a browser - For Daily Check ins
            let url = "https://www.hoyolab.com/";
            crate::midi_commands::open_url(url);
        }
        /*###############################################################################
        Open Teyvat Map in a browser
        ###############################################################################*/
        if note == 42 {
            // Open Teyvat Map in a browser
            let url = "https://act.hoyolab.com/ys/app/interactive-map/index.html?bbs_presentation_style=no_header&utm_id=2&utm_medium=tool&utm_source=hoyolab&bbs_theme=dark&bbs_theme_device=1&lang=en-us#/map/2?shown_types=&center=2008.50,-1084.00&zoom=-3.00";
            crate::midi_commands::open_url(url);
        }
    }
}
