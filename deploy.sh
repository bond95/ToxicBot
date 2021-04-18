#!/bin/bash

ssh  "$DEPLOY_LOGIN@$DEPLOY_HOST"  <<'EOL'
	kill $(ps aux | grep 'toxic_bot' | awk '{print $2}')
	echo "Killed toxic_bot"
	exit
EOL

cd target/release

echo "Copying toxic bot to host..."
scp ./toxic_bot "$DEPLOY_LOGIN@$DEPLOY_HOST":~/

ssh  "$DEPLOY_LOGIN@$DEPLOY_HOST"  <<'EOL'
	echo "Starting toxic bot"
	nohup ./toxic_bot telegram > /dev/null &
	exit
EOL
