Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\MyMIDI\my_midi\target\release\my_midi.exe & pause", 0, True
Set WshShell = Nothing
