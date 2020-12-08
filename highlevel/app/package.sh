#!/bin/bash

# extract debug info into separate file and strip
cp target/arm-v7-none-eabi/debug/sphere-app target/arm-v7-none-eabi/debug/sphere-app.debug
sysroots/6/tools/sysroots/x86_64-pokysdk-linux/usr/bin/arm-poky-linux-musleabi/arm-poky-linux-musleabi-strip --only-keep-debug target/arm-v7-none-eabi/debug/sphere-app.debug
sysroots/6/tools/sysroots/x86_64-pokysdk-linux/usr/bin/arm-poky-linux-musleabi/arm-poky-linux-musleabi-strip target/arm-v7-none-eabi/debug/sphere-app

mkdir -p target/approot/bin
mkdir -p target/approot/certs
cp target/arm-v7-none-eabi/debug/sphere-app target/approot/bin/app
cp app_manifest.json target/approot

azsphere image-package pack-application --input target/approot --destination-file target/sphere-app.image --verbose
azsphere device sideload delete --component-id 00f3df71-a397-4a5e-89cb-7dde6486888d --verbose
azsphere device sideload deploy -p target/sphere-app.image --verbose