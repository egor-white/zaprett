#!/usr/bin/env bash
set -euo pipefail
: "${MODULE_VERSION:?MODULE_VERSION is not set}"
: "${MODULE_VERSION_CODE:?MODULE_VERSION_CODE is not set}"

echo "Build zaprett binaries"
just -f rust/justfile build-android --release

echo "Make build dirs"
mkdir -p zaprett/system/bin
mkdir -p zaprett/zaprett/files/lists/include
mkdir -p zaprett-hosts/system/bin
mkdir -p zaprett-hosts/system/etc
mkdir -p zaprett-hosts/zaprett/files/lists/include
mkdir -p out lists

echo "Copy files to dirs"
cp rust/target/armv7-linux-androideabi/release/zaprett zaprett/system/bin/zaprett-armv7
cp rust/target/aarch64-linux-android/release/zaprett zaprett/system/bin/zaprett-aarch64
cp rust/target/x86_64-linux-android/release/zaprett zaprett/system/bin/zaprett-x86_64

echo "Copy shared libraries"
for arch in armeabi-v7a arm64-v8a x86_64; do
    case "$arch" in
        armeabi-v7a) target=armv7-linux-androideabi ;;
        arm64-v8a)   target=aarch64-linux-android ;;
        x86_64)      target=x86_64-linux-android ;;
    esac

    src_dir="rust/target/${target}/release"
    lib_dir="zaprett/system/lib/$arch"
    mkdir -p "$lib_dir"

    for lib in libnfqws.so libnfqws2.so; do
        found=$(find "$src_dir" -name "$lib" -type f | head -n 1)
        if [ -n "$found" ]; then
            cp "$found" "$lib_dir/"
            echo "Copied $lib for $arch"
        else
            echo "Warning: $lib not found for $arch in $src_dir"
        fi
    done
done

cp -a src/* zaprett/
cp -r zaprett/* zaprett-hosts/

echo "Download and copy actual lists"
wget https://raw.githubusercontent.com/CherretGit/zaprett-repo/refs/heads/main/files/lists/include/list-youtube.txt -O lists/list-youtube.txt
wget https://raw.githubusercontent.com/CherretGit/zaprett-repo/refs/heads/main/files/lists/include/list-discord.txt -O lists/list-discord.txt
cp lists/* zaprett/zaprett/files/lists/include/
cp lists/* zaprett-hosts/zaprett/files/lists/include/
cp hosts/hosts zaprett-hosts/system/etc

echo "Create module.prop"
cat > zaprett/module.prop <<EOF
id=zaprett
name=zaprett
version=$MODULE_VERSION
versionCode=$MODULE_VERSION_CODE
author=egor-white, Cherret
description=Ускорение CDN серверов Google. ТГК: https://t.me/zaprett_module
updateJson=https://raw.githubusercontent.com/egor-white/zaprett/refs/heads/main/update.json
EOF

cat > zaprett-hosts/module.prop <<EOF
id=zaprett
name=zaprett-hosts
version=$MODULE_VERSION
versionCode=$MODULE_VERSION_CODE
author=egor-white, Cherret
description=Ускорение CDN серверов Google. ТГК: https://t.me/zaprett_module
updateJson=https://raw.githubusercontent.com/egor-white/zaprett/refs/heads/main/update-hosts.json
EOF

echo "Create archives"
cd zaprett && zip -r ../zaprett.zip ./* && cd ..
cd zaprett-hosts && zip -r ../zaprett-hosts.zip ./* && cd ..
mv zaprett.zip out/
mv zaprett-hosts.zip out/

echo "Clean temp files"
rm -rf zaprett zaprett-hosts lists