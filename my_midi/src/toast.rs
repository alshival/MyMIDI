
use std::process::Command;

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