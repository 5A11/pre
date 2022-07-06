#!/bin/sh

echo "[CRON] cleanup process starting at: $(date +'%Y/%m/%dTime-%H:%M:%S')">> /data/ipfs/customlogs.txt
rm -rf /data/ipfs/blocks/*/ 
rm -rf /data/ipfs/datastore/*
echo "[CRON] cleanup process completed at: $(date +'%Y/%m/%dTime-%H:%M:%S')">> /data/ipfs/customlogs.txt
echo "[CRON] IPFS shutting down at: $(date +'%Y/%m/%dTime-%H:%M:%S')">> /data/ipfs/customlogs.txt
ipfs shutdown