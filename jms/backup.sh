now=$(date "+%Y%m%d_%H:%M:%S")
mkdir -p /home/fta/jms_backups/$now
cp -r /home/fta/JMS/jms/event.kvdb /home/fta/jms_backups/$now
