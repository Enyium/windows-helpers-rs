# The crate for each `windows` crate version must be able to be successfully built with the minimal and the full feature set.
Write-Host 'Building for `windows` v0.48...'
cargo build --features windows_v0_48
if (-not $?) { throw 'Failure' }
cargo build --features windows_v0_48_all
if (-not $?) { throw 'Failure' }

Write-Host 'Building for `windows` v0.52...'
cargo build --features windows_v0_52
if (-not $?) { throw 'Failure' }
cargo build --features windows_v0_52_all
if (-not $?) { throw 'Failure' }

# Test (same feature as on test modules).
Write-Host 'Testing...'
cargo test --features windows_latest_compatible_all
if (-not $?) { throw 'Failure' }
