rsync \
        --archive \
        --update \
        --delete \
        --exclude 'target' \
        --exclude '.git' \
        --exclude '.gitignore' \
        ~/repo/social dendrite@n0g.rip:
