#!/system/bin/sh
while [ -z "$(getprop sys.boot_completed)" ]; do sleep 2; done
sleep 1
source /storage/emulated/0/zaprett/config
if [ "$autorestart" = "true" ]; then
  su -c "zaprett start"
  while true; do
      sleep 3600
	  su -c "zaprett restart"
  done 
fi
