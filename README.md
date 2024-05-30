## Setup
Get libopenxr_loader.so from the Oculus OpenXR Mobile SDK and add it to `runtime_libs/arm64-v8a`
https://developer.oculus.com/downloads/package/oculus-openxr-mobile-sdk/
`runtime_libs/arm64-v8a/libopenxr_loader.so`

install `xbuild`. Note that the `--git` is
very important here.
```sh
cargo install --git https://github.com/rust-mobile/xbuild
```

```sh 
# List devices and copy device string "adb:***"
x devices

# Run on this device
x run --release --device adb:***
```