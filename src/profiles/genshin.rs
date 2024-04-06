use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use enigo::*;
use std::process::Command;

fn show_toast(title: &str, message: &str) {
    let ps_script = format!(r#"
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] > $null
$template = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02)

$textNodes = $template.GetElementsByTagName("text")
$textNodes.Item(0).AppendChild($template.CreateTextNode("{title}")) > $null
$textNodes.Item(1).AppendChild($template.CreateTextNode("{message}")) > $null

$toast = [Windows.UI.Notifications.ToastNotification]::new($template)
$notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("MyMIDI")
$notifier.Show($toast)
"#, title = title, message = message);

    Command::new("powershell")
        .arg("-NoProfile")
        .arg("-ExecutionPolicy")
        .arg("Bypass")
        .arg("-Command")
        .arg(&ps_script)
        .output()
        .expect("Failed to execute process");
}

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
    let mut enigo = Enigo::new();

    // Determine the current scale and directly work with its lock guard
    let scale_guard = match *CURRENT_SCALE.lock().unwrap() {
        ScaleType::Lows => LOWS.lock().unwrap(),
        ScaleType::Highs => HIGHS.lock().unwrap(),
        ScaleType::Complete => COMPLETE.lock().unwrap(),
    };

    if message[0] >= 144 && message[0] <= 159 && message[2] != 0 { // Adjust channel checking as needed
        let note = message[1]; // MIDI note number

        // Handling special notes for changing scales or other actions
        match note {
            40 => {
                let mut scale = CURRENT_SCALE.lock().unwrap();
                *scale = ScaleType::Complete;
                show_toast("Layout Change","Switched to complete scale");
                //println!("Switched to complete scale");
            },
            41 => {
                let mut scale = CURRENT_SCALE.lock().unwrap();
                *scale = match *scale {
                    ScaleType::Lows => {
                        //println!("Toggled to Highs");
                        show_toast("Layout Change","Toggled to Highs");
                        ScaleType::Highs
                    },
                    ScaleType::Highs | ScaleType::Complete => {
                        //println!("Toggled to Lows");
                        show_toast("Layout Change","Toggled to Lows");
                        ScaleType::Lows
                    },
                };
            },
            42 => {
                // Simulate pressing Alt + S
                enigo.key_down(Key::Alt);
                enigo.key_click(Key::Layout('s'));
                enigo.key_up(Key::Alt);
                //println!("Simulated Alt+S press");
            },
            _ => {
                if let Some(&key_char) = scale_guard.get(&note) {
                    enigo.key_click(Key::Layout(key_char));
                    //println!("Pressed key: {}", key_char);
                }
            }
        }
    }
}
