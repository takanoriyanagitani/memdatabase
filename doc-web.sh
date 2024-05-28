#!/bin/sh

addr=127.0.0.1
port=52880

python3 \
	-m http.server \
	--bind "${addr}" \
	--directory target \
	${port}
