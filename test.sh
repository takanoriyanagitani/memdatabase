#!/bin/sh

which grpcurl | fgrep -q grpcurl || exec sh -c 'echo grpcurl missing.; exit 1'
which jaq | fgrep -q jaq || exec sh -c 'echo jaq missing.; exit 1'
which base64 | fgrep -q base64 || exec sh -c 'echo base64 missing.; exit 1'

protodir=memdatabase-proto
server=localhost:50051

varset() {

	jaq \
		-c \
		--arg key "$(echo -n helo | base64)" \
		-n '{ key: $key, value: "helo" }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Set

	jaq \
		-c \
		--arg key "$(echo -n HELO | base64)" \
		-n '{ key: $key, value: "helo" }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Set

	jaq \
		-c \
		--arg key "$(echo -n ZZZZ | base64)" \
		-n '{ key: $key, value: 3 }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Set

}

varget() {

	jaq \
		-c \
		--arg key "$(echo -n helo | base64)" \
		-n '{ key: $key }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Get

}

range() {
	jaq \
		-c \
		--arg lower "$(echo -n 0000 | base64)" \
		--arg upper "$(echo -n zzzz | base64)" \
		-n '{
      lower: { included: $lower },
      upper: { excluded: $upper },
    }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Range
}

varset
range
varget
