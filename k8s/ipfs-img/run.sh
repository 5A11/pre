#!/bin/sh

mkdir -p /var/spool/cron/crontabs
chmod +x /shutdown.sh
echo "0 0 * * * /./shutdown.sh" > ./cronjob.sh
chmod +x cronjob.sh
crontab cronjob.sh