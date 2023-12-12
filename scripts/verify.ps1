cargo build --features windows_v0_48_all
if (-not $?) { throw 'Failure' }

cargo build --features windows_v0_52_all
if (-not $?) { throw 'Failure' }

cargo test --features windows_latest_compatible_all
if (-not $?) { throw 'Failure' }
