#!/system/bin/sh
while [ -z "$(getprop sys.boot_completed)" ]; do sleep 2; done
if [ -f "/data/adb/modules/zaprett/autostart" ]; then
  su -c "zaprett start"
  while true; do
      sleep 3600
	  su -c "zaprett restart"
  done
fi
