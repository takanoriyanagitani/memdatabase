#!/bin/sh

single(){
  ENV_LOOP_CNT=1 python3 single.py
  ENV_LOOP_CNT=16 python3 single.py
  ENV_LOOP_CNT=128 python3 single.py
  ENV_LOOP_CNT=1024 python3 single.py
  ENV_LOOP_CNT=16384 python3 single.py
  ENV_LOOP_CNT=131072 python3 single.py
}

multi(){
  ENV_LIST_SIZE=1 python3 multi.py
  ENV_LIST_SIZE=16 python3 multi.py
  ENV_LIST_SIZE=128 python3 multi.py
  ENV_LIST_SIZE=1024 python3 multi.py
  ENV_LIST_SIZE=16384 python3 multi.py
  ENV_LIST_SIZE=131072 python3 multi.py
}

multi
