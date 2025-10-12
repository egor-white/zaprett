ui_print "                          _   _   "
ui_print "                         | | | |  "
ui_print "  ______ _ _ __  _ __ ___| |_| |_ "
ui_print " |_  / _' | '_ \| '__/ _ \ __| __|"
ui_print "  / / (_| | |_) | | |  __/ |_| |_ "
ui_print " /___\__,_| .__/|_|  \\___|\__|\__|"
ui_print "          | |                     "
ui_print "          |_|                     "
ui_print "(!) To download app, use Telegram channel"
ui_print "Module by: egor-white, Cherret, Huananzhi X99"
ui_print "App by: egor-white, Cherret"
ui_print "####################"

ui_print "Unpacking archive..."
unzip -o "$ZIPFILE" -x 'META-INF/*' -d $MODPATH >&2

ui_print "Creating zaprett directory..."
mkdir /sdcard/zaprett; mkdir /sdcard/zaprett/lists; mkdir /sdcard/zaprett/bin; mkdir /sdcard/zaprett/strategies;

ui_print "Filling configuration file if not exist..."
if [ ! -f "/sdcard/zaprett/config" ]; then
    echo active_lists=/storage/emulated/0/zaprett/lists/list-youtube.txt >> /sdcard/zaprett/config
    echo active_exclude_lists= >> /sdcard/zaprett/config
    echo list_type=whitelist
    echo zaprettdir=/sdcard/zaprett >> /sdcard/zaprett/config
    echo strategy="" >> /sdcard/zaprett/config
fi

ui_print "Copying lists and binaries to /sdcard/zaprett..."
cp -r $MODPATH/system/etc/zaprett/. /sdcard/zaprett/

ui_print "Copying files to /bin"
arch=$(uname -m)
case "$arch" in
    "x86_64")
        nfqws="nfqws_x86_64"
        ;;
    "i386"|"i686")
        nfqws="nfqws_x86"
        ;;
    "armv7l"|"arm")
        nfqws="nfqws_arm32"
        ;;
    "aarch64"|"armv8l")
        nfqws="nfqws_arm64"
        ;;
    "mips")
        nfqws="nfqws_mips"
        ;;
    "mipsel")
        nfqws="nfqws_mipsel"
        ;;
    *)
        ui_print "Unknown arch: $arch"
        abort
        ;;
esac
mv $MODPATH/system/bin/$nfqws $MODPATH/system/bin/nfqws
rm $MODPATH/system/bin/nfqws_*
mkdir $MODPATH/tmp

ui_print "Setting permissions..."
chmod 777 /sdcard/zaprett; chmod 777 $MODPATH/service.sh

ui_print "Cleaning temp files..."
rm -rf $MODPATH/system/etc/zaprett

ui_print "Installation done. Telegram channel: https://t.me/zaprett_module"
