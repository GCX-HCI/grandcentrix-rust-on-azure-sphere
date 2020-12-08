# Needs this

- recent Rust environment (`rustup override set nightly`)  (e.g. 1.45.0-nightly (99cb9ccb9 2020-05-11))
- MUSL GCC (on OSX `brew install FiloSottile/musl-cross/musl-cross --without-x86_64  --with-arm-hf`)
- before compiling `source setenv.sh`  
- link sysroots to a shared folder


# Build & Sideload

`cargo xbuild`

`./package.sh`

