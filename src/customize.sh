ui_print "                          _   _   "
ui_print "                         | | | |  "
ui_print "  ______ _ _ __  _ __ ___| |_| |_ "
ui_print " |_  / _' | '_ \| '__/ _ \ __| __|"
ui_print "  / / (_| | |_) | | |  __/ |_| |_ "
ui_print " /___\__,_| .__/|_|  \\___|\__|\__|"
ui_print "          | |                     "
ui_print "          |_|                     "
ui_print "(!) To download app, use Telegram channel"
ui_print "Module by: egor-white, Cherret"
ui_print "App by: egor-white, Cherret"
ui_print "####################"

ui_print "Moving old folder (if exists)"
PROP_FILE="/data/adb/modules/zaprett/module.prop"
if [ -f "$PROP_FILE" ]; then
    source "$PROP_FILE"

    if [ -n "$versionCode" ] && [ "$versionCode" -le 65 ]; then
        mv /sdcard/zaprett /sdcard/zaprett-old
    fi
fi

ui_print "Creating zaprett directory..."
mkdir /sdcard/zaprett

ui_print "Copying lists and binaries to /sdcard/zaprett..."
cp -r $MODPATH/zaprett/. /sdcard/zaprett/

ui_print "Copying files to /bin"
arch=$(uname -m)
case "$arch" in
    "x86_64")
        zaprett_bin="zaprett-x86_64"
        l_base="lib64"
        l_sub="x86_64"
        ;;
    "armv7l"|"arm"|"armv8l")
        zaprett_bin="zaprett-armv7"
        l_base="lib"
        l_sub="armeabi-v7a"
        ;;
    "aarch64")
        zaprett_bin="zaprett-aarch64"
        l_base="lib64"
        l_sub="arm64-v8a"
        ;;
    *)
        ui_print "Unknown arch: $arch"
        abort
        ;;
esac

mv "$MODPATH/system/bin/$zaprett_bin" "$MODPATH/system/bin/zaprett"
rm -f "$MODPATH/system/bin/zaprett-"*

if [ -d "$MODPATH/system/lib/$l_sub" ]; then
    mkdir -p "$MODPATH/system/lib_tmp"
    mv "$MODPATH/system/lib/$l_sub/"* "$MODPATH/system/lib_tmp/"
    rm -rf "$MODPATH/system/lib"
    mkdir -p "$MODPATH/system/$l_base"
    mv "$MODPATH/system/lib_tmp/"* "$MODPATH/system/$l_base/"
    rm -rf "$MODPATH/system/lib_tmp"
fi
mkdir $MODPATH/tmp

ui_print "Cleaning temp files..."
rm -rf $MODPATH/zaprett

ui_print "Installation done. Join us in Telegram: https://t.me/zaprett_module"
