---
description: Build Android APK only
agent: build
---

Build the Android APK (lib + gradle):

```
source scripts/android-env.sh && cargo ndk -t arm64-v8a -o app/android/app/src/main/jniLibs build --release --package tabletop && cd app/android && ./gradlew assembleDebug && cd ../..
```
