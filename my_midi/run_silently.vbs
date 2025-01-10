Set WshShell = CreateObject("WScript.Shell")
WshShell.Run "C:\Users\samue\Documents\Github\MyMIDI\my_midi\target\release\my_midi.exe & ", 0, True
Set WshShell = Nothing