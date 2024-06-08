#!/bin/sh

ENV_LOOP_CNT=1 python3 single.py
ENV_LOOP_CNT=16 python3 single.py
ENV_LOOP_CNT=128 python3 single.py
ENV_LOOP_CNT=1024 python3 single.py
ENV_LOOP_CNT=16384 python3 single.py
ENV_LOOP_CNT=131072 python3 single.py
