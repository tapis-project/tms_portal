#!/bin/bash
#set -xv

docker run -it -v `pwd`/client:`pwd`/client -w `pwd`/client  node:24 /bin/bash -c 'npm install -g pnpm && pnpm install && pnpm run build'