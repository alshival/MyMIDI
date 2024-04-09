use crate::midi_commands;
use std::env;

pub fn handle_message(message: &[u8]) {
    let username = env::var("USERNAME").unwrap_or_else(|_| String::from("default"));
    
    // Launch Terminal Shortcut
    if message[0] == 153 && message[1] == 40 {
        let path = "C:\\Program Files\\WindowsApps\\Microsoft.WindowsTerminalPreview_1.20.10822.0_x64__8wekyb3d8bbwe\\WindowsTerminal.exe";
        // Note that setting path like this returns an object of type &str, which is what we need for launch_exe
        midi_commands::launch_exe(path);
    }

    // Launch Visual Studio Code
    if message[0] == 153 && message[1] == 41 {
        let path = format!("C:\\Users\\{}\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe", username);
        // Note that setting path like this does NOT return an object of type &str. It returns a string, so we add &path when passing it to launch_exe
        midi_commands::launch_exe(&path);
    }

    // Launch SteelSeries GG
    if message[0] == 153 && message[1] == 42 {
        let path = "C:\\Program Files\\SteelSeries\\GG";
        // Note that setting path like this returns an object of type &str, which is what we need for launch_exe
        midi_commands::launch_exe(path);
    }

}
