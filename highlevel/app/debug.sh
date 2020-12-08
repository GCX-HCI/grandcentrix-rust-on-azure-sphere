#!/bin/sh 
azsphere device app stop --component-id 00f3df71-a397-4a5e-89cb-7dde6486888d
azsphere device app start --debug-mode --component-id 00f3df71-a397-4a5e-89cb-7dde6486888d

gnome-terminal -- /bin/sh -c 'sleep 4 ; telnet 192.168.35.2 2342'

sysroots/7+Beta2010/tools/sysroots/x86_64-pokysdk-linux/usr/bin/arm-poky-linux-musleabi/arm-poky-linux-musleabi-gdb -s target/arm-v7-none-eabi/debug/sphere-app.debug -e target/arm-v7-none-eabi/debug/sphere-app
