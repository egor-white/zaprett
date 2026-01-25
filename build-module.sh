#!/usr/bin/env bash
set -euo pipefail
: "${module_version:?module_version is not set}"
: "${module_version_code:?module_version_code is not set}"

echo "Build zaprett binaries"
just -f rust/justfile build-android --release

echo "Make build dirs"
mkdir -p zaprett/system/bin
mkdir -p zaprett/zaprett/bin
mkdir -p zaprett/zaprett/lists/include
mkdir -p zaprett-hosts/system/bin
mkdir -p zaprett-hosts/system/etc
mkdir -p zaprett-hosts/zaprett/bin
mkdir -p zaprett-hosts/zaprett/lists/include
mkdir -p out lists

echo "Copy files to dirs"
cp rust/target/armv7-linux-androideabi/release/zaprett zaprett/system/bin/zaprett-armv7
cp rust/target/aarch64-linux-android/release/zaprett zaprett/system/bin/zaprett-aarch64
cp rust/target/x86_64-linux-android/release/zaprett zaprett/system/bin/zaprett-x86_64
cp -a src/* zaprett/
cp -r zaprett/* zaprett-hosts/

echo "Download and copy actual lists"
wget https://raw.githubusercontent.com/CherretGit/zaprett-repo/refs/heads/main/lists/include/list-youtube.txt -O lists/list-youtube.txt
wget https://raw.githubusercontent.com/CherretGit/zaprett-repo/refs/heads/main/lists/include/list-discord.txt -O lists/list-discord.txt
cp lists/* zaprett/zaprett/lists/include/
cp lists/* zaprett-hosts/zaprett/lists/include/
cp hosts/hosts zaprett-hosts/system/etc

echo "Create module.prop"
cat > zaprett/module.prop <<EOF
id=zaprett
name=zaprett
version=$module_version
versionCode=$module_version_code
author=egor-white, Cherret
description=Ускорение CDN серверов Google. ТГК: https://t.me/zaprett_module
updateJson=https://raw.githubusercontent.com/egor-white/zaprett/refs/heads/main/update.json
EOF

cat > zaprett-hosts/module.prop <<EOF
id=zaprett
name=zaprett-hosts
version=$module_version
versionCode=$module_version_code
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
