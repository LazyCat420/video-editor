# ============================================================================
# LazyCat420 - Sign Executable with Local Code Signing Certificate
# Usage: .\scripts\sign-exe.ps1 -ExePath "target\x86_64-pc-windows-gnullvm\release\video-editor.exe"
# ============================================================================
param (
    [string]$ExePath = "target\x86_64-pc-windows-gnullvm\release\video-editor.exe"
)

if (-not (Test-Path $ExePath)) {
    Write-Error "Executable not found at: $ExePath"
    exit 1
}

$cert = Get-ChildItem Cert:\CurrentUser\My -CodeSigningCert | Where-Object { $_.Subject -like "*LazyCat420*" } | Select-Object -First 1

if (-not $cert) {
    Write-Error "No LazyCat420 code signing certificate found in Cert:\CurrentUser\My. Run generate-cert.ps1 first!"
    exit 1
}

Write-Host "Signing $ExePath with certificate: $($cert.Subject)..." -ForegroundColor Cyan
Set-AuthenticodeSignature -FilePath $ExePath -Certificate $cert -HashAlgorithm SHA256 -TimestampServer "http://timestamp.digicert.com"

Write-Host "Successfully signed: $ExePath" -ForegroundColor Green
