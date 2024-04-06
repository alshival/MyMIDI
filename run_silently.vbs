Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\MyMIDI\target\release\MyMIDI.exe & pause", 0, True
Set WshShell = Nothing
