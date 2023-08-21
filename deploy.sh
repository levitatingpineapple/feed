rsync \
	--archive \
	--update \
	--delete \
	--exclude 'target' \
	--exclude '.git' \
	--exclude '.gitignore' \
	--exclude '.DS_Store' \
	--exclude 'deploy.sh' \
	. dendrite@n0g.rip:bot
