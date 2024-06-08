#!/bin/sh

files(){
	ls ./memdatabase-proto/memdatabase/v1/*.proto
}

python3 \
	-m grpc_tools.protoc \
	-I ./memdatabase-proto \
	--python_out=. \
	--pyi_out=. \
	--grpc_python_out=. \
	$( files )
