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

varpush() {

	jaq \
		-c \
		--arg key "$(echo -n queue0123 | base64)" \
		-n '{ key: $key, value: "wwww", front: true }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Push

}

varpopback() {

	jaq \
		-c \
		--arg key "$(echo -n queue0123 | base64)" \
		-n '{ key: $key, front: false }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Pop

}

varpopfront() {

	jaq \
		-c \
		--arg key "$(echo -n queue0123 | base64)" \
		-n '{ key: $key, front: true }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/Pop

}

qlen() {

	jaq \
		-c \
		--arg key "$(echo -n queue0123 | base64)" \
		-n '{ key: $key }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/QLen

}

dictset() {

	jaq \
		-c \
		--arg key "$(echo -n dict0123 | base64)" \
		--arg dkey "$(echo -n dkey0123 | base64)" \
		-n '{ key: $key, dkey: $dkey, value: 3776.0 }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/DSet

}

dictget() {

	jaq \
		-c \
		--arg key "$(echo -n dict0123 | base64)" \
		--arg dkey "$(echo -n dkey0123 | base64)" \
		-n '{ key: $key, dkey: $dkey }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/DGet

}

dictfind() {

	jaq \
		-c \
		--arg key "$(echo -n dict0123 | base64)" \
		--arg dkey "$(echo -n dkey0123 | base64)" \
		-n '{ key: $key, dkey: $dkey }' |
		grpcurl \
			-plaintext \
			-import-path "${protodir}" \
			-proto memdatabase/v1/svc.proto \
			-d @ \
			"${server}" \
			memdatabase.v1.MemoryDatabaseService/DHas

}

varset
range
varget
varpush
qlen
varpopfront
dictset
dictget
dictfind
