@echo off
setlocal

for %%I in ("%~dp0..") do set "SEMAURI_HOME=%%~fI"
set "RUBY_BIN=%SEMAURI_HOME%\runtime\ruby\bin\ruby.exe"
set "APP_ENTRYPOINT=%SEMAURI_HOME%\app\bin\semauri"

if not exist "%RUBY_BIN%" (
  echo Semauri installation is incomplete: private Ruby runtime not found at %RUBY_BIN% 1>&2
  exit /b 70
)

if not exist "%APP_ENTRYPOINT%" (
  echo Semauri installation is incomplete: compiler entrypoint not found at %APP_ENTRYPOINT% 1>&2
  exit /b 70
)

set "SEMAURI_HOME=%SEMAURI_HOME%"
"%RUBY_BIN%" "%APP_ENTRYPOINT%" %*
exit /b %ERRORLEVEL%
