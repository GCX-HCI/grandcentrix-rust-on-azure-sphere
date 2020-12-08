#!/bin/bash

# extract debug info into separate file and strip
cp target/thumbv7em-none-eabihf/debug/rtapp target/thumbv7em-none-eabihf/debug/rtapp.debug
arm-none-eabi-strip --only-keep-debug target/thumbv7em-none-eabihf/debug/rtapp.debug
arm-none-eabi-strip target/thumbv7em-none-eabihf/debug/rtapp

mkdir -p target/approot/bin
cp target/thumbv7em-none-eabihf/debug/rtapp target/approot/bin/app
cp app_manifest.json target/approot

azsphere image-package pack-application --target-api-set 6 --input target/approot --destination-file target/rtapp.image --verbose
azsphere device sideload delete --component-id d3b80666-feaf-433a-b294-6a5846853b4a --verbose
azsphere device sideload deploy -p target/rtapp.image --verbose
