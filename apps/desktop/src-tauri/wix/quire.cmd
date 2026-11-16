@echo off
rem The quire command on Windows (TD-20): the console helper next to quire.exe answers
rem --help, --version and argument errors here and starts the app for the rest.
"%~dp0..\quire-cli.exe" %*
exit /b %ERRORLEVEL%
