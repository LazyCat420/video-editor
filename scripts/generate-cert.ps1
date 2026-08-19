# ============================================================================
# LazyCat420 - Create Self-Signed Code Signing Certificate
# Run in PowerShell on Windows: .\scripts\generate-cert.ps1
# ============================================================================

$CertSubject = "CN=LazyCat420 Software"
$ExportPath = "scripts\LazyCat420_Root.cer"

Write-Host "Creating Code Signing Certificate for $CertSubject..." -ForegroundColor Cyan

$cert = New-SelfSignedCertificate `
    -Type CodeSigningCert `
    -Subject $CertSubject `
    -CertStoreLocation "Cert:\CurrentUser\My" `
    -KeyUsage DigitalSignature `
    -FriendlyName "LazyCat420 Developer Certificate" `
    -NotAfter (Get-Date).AddYears(10)

Write-Host "Certificate Created with Thumbprint: $($cert.Thumbprint)" -ForegroundColor Green

# Export public certificate for distribution to Grandma's PC
Export-Certificate -Cert $cert -FilePath $ExportPath | Out-Null

Write-Host "Exported public root certificate to: $ExportPath" -ForegroundColor Green
Write-Host ""
Write-Host "Instructions for Grandma's PC:" -ForegroundColor Yellow
Write-Host "1. Copy $ExportPath and scripts\trust-cert.bat to her computer."
Write-Host "2. Right-click trust-cert.bat -> Run as administrator."
Write-Host "3. All future builds signed with this certificate will run without SmartScreen/AV blocks!"
