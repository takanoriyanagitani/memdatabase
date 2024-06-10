import timeit
import os

import grpc

import memdatabase.v1.svc_pb2_grpc
import memdatabase.v1.push_pb2
import memdatabase.v1.del_pb2

addr = "localhost:50051"
callback = lambda: 0.0
key = b"py-client-bench"

loop_cnt_s: str = os.environ.get("ENV_LOOP_CNT", "10")
loop_cnt_i: int = int(loop_cnt_s)

list_sz_s: str = os.environ.get("ENV_LIST_SIZE", "1")
list_sz_i: int = int(list_sz_s)

def callback_new(stub, key=b"", val=0.0, cnt=1):
  req = memdatabase.v1.push_pb2.PushRequest(
    key=key,
    value=dict(list_value=dict(
      values=[
        dict(number_value=val+i) for i in range(cnt)
      ],
    )),
    front=False,
  )
  return lambda: stub.Push(req)

with grpc.insecure_channel(addr) as chan:
  stub = memdatabase.v1.svc_pb2_grpc.MemoryDatabaseServiceStub(chan)
  dreq = memdatabase.v1.del_pb2.DelRequest(key=key)
  _dres = stub.Del(dreq)

  callback = callback_new(stub, key=key, val=42.0, cnt=list_sz_i)
  t = timeit.Timer(stmt="callback()", globals=globals())
  bench = t.autorange()
  print(bench)
  pass
