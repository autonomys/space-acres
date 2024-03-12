@echo off
:UseChoice
%SystemRoot%\System32\choice.exe /C YN /N /M "Delele configuration and logs [Y/N]?"
if not errorlevel 1 goto UseChoice
if errorlevel 2 exit /B
for /F "delims=" %%I in ('dir "%HOMEDRIVE%\Users\*.*" /AD /B 2^>nul') do rd /Q /S "%HOMEDRIVE%\Users\%%I\AppData\Local\space-acres" 2>nul
