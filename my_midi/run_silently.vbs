Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\MyMIDI\my_midi\target\release\my_midi.exe & ", 0, True
Set WshShell = Nothing
