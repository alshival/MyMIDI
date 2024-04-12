use std::process::Command;
use crate::toast;
use webbrowser;

pub fn launch_exe(path: &str) {
    match Command::new(path).spawn() {
        Ok(_) => println!("Launched successfully."),
        Err(e) => {
            println!("Failed to launch: {}", e);
            toast::show_toast("Launch Failed", &format!("Failed to launch: {}", e));
        },
    }
}

pub fn open_url(url: &str) -> Result<(),webbrowser::ParseBrowserError> {
    // Open the URL using the user's default web browser
    webbrowser::open(url);
    Ok(())
}

// Example usage:
// fn main() {
//     match launch_exe("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe") {
//         Ok(_) => println!("Launched successfully."),
//         Err(e) => println!("Failed to launch: {}", e),
//     }
// }
