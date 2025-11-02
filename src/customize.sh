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

ui_print "Unpacking archive..."
unzip -o "$ZIPFILE" -x 'META-INF/*' -d $MODPATH >&2

ui_print "Creating zaprett directory..."
mkdir /sdcard/zaprett; mkdir /sdcard/zaprett/lists; mkdir /sdcard/zaprett/bin; mkdir /sdcard/zaprett/strategies;

ui_print "Filling configuration file if not exist..."
if [ ! -f "/sdcard/zaprett/config.json" ]; then
    cat > /sdcard/zaprett/config.json << EOL
    {
      "active_lists": ["/sdcard/zaprett/lists/include/list-youtube.txt", "/sdcard/zaprett/lists/include/list-youtube.txt"],
      "active_ipsets": [],
      "active_exclude_lists": [],
      "active_exclude_ipsets": [],
      "list_type": "whitelist",
      "strategy": "",
      "app_list": "whitelist",
      "whitelist": [],
      "blacklist": []
    }
    EOL
fi

ui_print "Copying lists and binaries to /sdcard/zaprett..."
cp -r $MODPATH/system/etc/zaprett/. /sdcard/zaprett/

ui_print "Copying files to /bin"
arch=$(uname -m)
case "$arch" in
    "x86_64")
        zaprett_bin="zaprett-x86_64"
        ;;
    "armv7l"|"arm")
        zaprett_bin="zaprett-armv7"
        ;;
    "aarch64"|"armv8l")
        zaprett_bin="zaprett-aarch64"
        ;;
    *)
        ui_print "Unknown arch: $arch"
        abort
        ;;
esac
mv $MODPATH/system/bin/$zaprett_bin $MODPATH/system/bin/zaprett
rm $MODPATH/system/bin/zaprett-*
mkdir $MODPATH/tmp

ui_print "Setting permissions..."
chmod 777 /sdcard/zaprett; chmod 777 $MODPATH/service.sh

ui_print "Cleaning temp files..."
rm -rf $MODPATH/system/etc/zaprett

ui_print "Installation done. Join us in Telegram: https://t.me/zaprett_module"
