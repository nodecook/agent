$ErrorActionPreference = "Stop"

$InstallDir = if ($env:NODECOOK_AGENT_INSTALL_DIR) { $env:NODECOOK_AGENT_INSTALL_DIR } else { Join-Path $env:ProgramFiles "NodeCookAgent" }
$ServiceName = "nodecook-agent"

if (-not ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
  throw "Please run this script as Administrator."
}

if (Get-Service -Name $ServiceName -ErrorAction SilentlyContinue) {
  Stop-Service -Name $ServiceName -ErrorAction SilentlyContinue
  sc.exe delete $ServiceName | Out-Null
}

Remove-Item -Recurse -Force $InstallDir -ErrorAction SilentlyContinue

Write-Host "NodeCook Agent uninstalled."
