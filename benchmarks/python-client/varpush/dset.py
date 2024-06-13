import timeit
import os

import grpc

import memdatabase.v1.svc_pb2_grpc
import memdatabase.v1.dset_pb2
import memdatabase.v1.del_pb2

def compose(f, g):
  return lambda t: g(f(t))

def key2env(alt=""):
  return lambda key: os.environ.get(key, alt)

addr = "localhost:50051"
callback = lambda: 0.0
key = b"py-client-bench"

keys_cnt: int = compose(
  key2env(alt="1"),
  int,
)("ENV_DKEY_CNT")

def dset_many(stub, key=b"", val=0.0, keymax=1):
  for i in range(keymax):
    skey: str = f"{i}"
    bkey: bytes = bytes(skey, "utf8")
    req = memdatabase.v1.dset_pb2.DSetRequest(
      key=key,
      dkey=bkey,
      value=dict(number_value=val),
    )
    stub.DSet(req)
  pass

def callback_new(stub, key=b"", val=0.0, keymax=1):
  return lambda: dset_many(stub, key, val, keymax)

with grpc.insecure_channel(addr) as chan:
  stub = memdatabase.v1.svc_pb2_grpc.MemoryDatabaseServiceStub(chan)
  dreq = memdatabase.v1.del_pb2.DelRequest(key=key)
  _dres = stub.Del(dreq)

  callback = callback_new(stub, key=key, val=42.0, keymax=keys_cnt)
  t = timeit.Timer(stmt="callback()", globals=globals())
  bench = t.autorange()
  print(bench)
  pass
