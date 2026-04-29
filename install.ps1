$ErrorActionPreference = "Stop"

$Repo = "VectorSophie/agentgauntlet"
$Artifact = "agentgauntlet-windows-x86_64.exe"

$Release = (Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest").tag_name
$Url = "https://github.com/$Repo/releases/download/$Release/$Artifact"

$InstallDir = "$env:LOCALAPPDATA\agentgauntlet"
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$Dest = "$InstallDir\agentgauntlet.exe"
Write-Host "Installing agentgauntlet $Release -> $Dest"
Invoke-WebRequest -Uri $Url -OutFile $Dest

# Add to user PATH if not already there
$CurrentPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("PATH", "$CurrentPath;$InstallDir", "User")
    Write-Host "Added $InstallDir to PATH (restart terminal to take effect)"
}

Write-Host ""
Write-Host "✅ Done! Run: agentgauntlet scan"
