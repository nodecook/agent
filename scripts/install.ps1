$ErrorActionPreference = "Stop"

$DownloadBaseUrl = if ($env:NODECOOK_AGENT_DOWNLOAD_BASE_URL) { $env:NODECOOK_AGENT_DOWNLOAD_BASE_URL.TrimEnd("/") } else { "https://dl.nodecook.com" }
$InstallDir = if ($env:NODECOOK_AGENT_INSTALL_DIR) { $env:NODECOOK_AGENT_INSTALL_DIR } else { Join-Path $env:ProgramFiles "NodeCookAgent" }
$ServiceName = "nodecook-agent"
$Target = "x86_64-pc-windows-msvc"
$Asset = "nodecook-agent-$Target.zip"

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
  throw "Please run this script as Administrator."
}

$Url = "$DownloadBaseUrl/$Asset"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("nodecook-agent-" + [System.Guid]::NewGuid())
$ZipPath = Join-Path $TempDir $Asset
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Write-Host "Downloading $Url"
Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $TempDir -Force
Copy-Item -Path (Join-Path $TempDir "nodecook-agent-$Target\nodecook-agent.exe") -Destination (Join-Path $InstallDir "nodecook-agent.exe") -Force

$EnvValues = @()
foreach ($Name in "NCA_DEBUG", "NCA_V4_ONLY", "NCA_V6_ONLY", "NCA_V4_SERVER", "NCA_V6_SERVER", "NCA_V4_NODE_ID", "NCA_V6_NODE_ID", "NCA_TITLE", "NCA_LINK") {
  $Value = [Environment]::GetEnvironmentVariable($Name, "Process")
  if ($Value) {
    $EnvValues += "$Name=$Value"
  }
}

if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
  Stop-Service -Name $ServiceName -ErrorAction SilentlyContinue
  sc.exe delete $ServiceName | Out-Null
}

$Binary = Join-Path $InstallDir "nodecook-agent.exe"
sc.exe create $ServiceName binPath= "`"$Binary`"" start= auto DisplayName= "NodeCook Agent" | Out-Null
sc.exe description $ServiceName "NodeCook Agent" | Out-Null
if ($EnvValues.Count -gt 0) {
  New-ItemProperty -Path "HKLM:\SYSTEM\CurrentControlSet\Services\$ServiceName" -Name Environment -PropertyType MultiString -Value $EnvValues -Force | Out-Null
}
Start-Service -Name $ServiceName

Remove-Item -Recurse -Force $TempDir
Write-Host "NodeCook Agent installed."
