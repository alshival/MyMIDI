use std::process::Command;
use webbrowser;

pub fn launch_exe(path: &str) {
    match Command::new(path).spawn() {
        Ok(_) => println!("Launched successfully."),
        Err(e) => {
            println!("Failed to launch: {}", e);
            show_toast("Launch Failed", &format!("Failed to launch: {}", e));
        },
    }
}

pub fn open_url(url: &str) -> Result<(),webbrowser::ParseBrowserError> {
    // Open the URL using the user's default web browser
    webbrowser::open(url);
    Ok(())
}

pub fn show_toast(title: &str, message: &str) {
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


// Example usage:
// fn main() {
//     match launch_exe("C:\\Users\\samue\\AppData\\Local\\TIDAL\\TIDAL.exe") {
//         Ok(_) => println!("Launched successfully."),
//         Err(e) => println!("Failed to launch: {}", e),
//     }
// }
