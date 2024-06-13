import timeit
import os

import grpc

import memdatabase.v1.svc_pb2_grpc
import memdatabase.v1.set_pb2
import memdatabase.v1.del_pb2

def compose(f, g):
  return lambda t: g(f(t))

def key2env(alt=""):
  return lambda key: os.environ.get(key, alt)

addr = "localhost:50051"
callback = lambda: 0.0
key = b"py-client-bench"

byte_sz_i: int = compose(
  key2env(alt="1"),
  int,
)("ENV_BYTE_SZ")

def callback_new(stub, key=b"", val=b""):
  req = memdatabase.v1.set_pb2.SetRequest(
    key=key,
    value=dict(string_value=val),
  )
  return lambda: stub.Set(req)

with grpc.insecure_channel(addr) as chan:
  stub = memdatabase.v1.svc_pb2_grpc.MemoryDatabaseServiceStub(chan)
  dreq = memdatabase.v1.del_pb2.DelRequest(key=key)
  _dres = stub.Del(dreq)

  bulk = bytes(byte_sz_i).replace(b"\0", b"0")

  callback = callback_new(stub, key=key, val=bulk)
  t = timeit.Timer(stmt="callback()", globals=globals())
  bench = t.autorange()
  print(bench)
  pass
