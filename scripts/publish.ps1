# Note: You can use `cargo publish` flags like `--dry-run` on this script.

$answer = Read-Host "Really publish now? (y/N)"
if ($answer -cne 'y') {
    Write-Host 'Aborting.'
    exit
}

& "$PSScriptRoot\verify.ps1"
if (-not $?) { throw 'Failure' }

cargo publish --features windows_latest_compatible_all @args
