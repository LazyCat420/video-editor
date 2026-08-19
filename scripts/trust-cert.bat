@echo off
:: ============================================================================
:: LazyCat420 Video Editor - One-Time Certificate Trust Script
:: Run this ONCE on Grandma's computer (Right-click -> Run as Administrator)
:: ============================================================================

net session >nul 2>&1
if %errorLevel% neq 0 (
    echo [ERROR] This script requires Administrator privileges.
    echo Please right-click "trust-cert.bat" and select "Run as administrator".
    echo.
    pause
    exit /b 1
)

echo [INFO] Installing Developer Certificate into Trusted Root Certification Authorities...
certutil -addstore -f "Root" "%~dp0LazyCat420_Root.cer"

if %errorLevel% equ 0 (
    echo.
    echo ============================================================================
    echo [SUCCESS] Certificate installed successfully!
    echo All Video Editor updates signed by LazyCat420 are now trusted on this PC.
    echo ============================================================================
) else (
    echo.
    echo [ERROR] Failed to install certificate. Error code: %errorLevel%
)

echo.
pause
