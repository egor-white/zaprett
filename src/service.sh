#!/system/bin/sh
while [ -z "$(getprop sys.boot_completed)" ]; do sleep 2; done
su -c "zaprett start"
while true; do
    sleep 3600
    if [ -f "/data/adb/modules/zaprett/autostart" ]; then
        su -c "zaprett restart"
    fi
done
