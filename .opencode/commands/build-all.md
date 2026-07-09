---
description: Full build (Linux + Android APK + deploy phone)
agent: build
---

Run the full build script:

```
./scripts/build.sh
```

This builds Linux release, Android APK via cargo-ndk + gradle, and installs on phone via ADB.
