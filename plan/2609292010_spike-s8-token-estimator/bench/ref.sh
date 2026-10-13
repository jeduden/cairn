#!/bin/sh
# Runs a command on emulated reference hardware (SRS §7: 2 vCPU, 4 GiB
# RAM) as spike S2 did: a cgroup v1 pair pinning two CPUs and capping
# memory, page cache included, at 4 GiB. Create the pair once, as root:
#   mkdir /sys/fs/cgroup/cpuset/s8ref /sys/fs/cgroup/memory/s8ref
#   echo 0-1 > /sys/fs/cgroup/cpuset/s8ref/cpuset.cpus
#   cat /sys/fs/cgroup/cpuset/cpuset.mems > /sys/fs/cgroup/cpuset/s8ref/cpuset.mems
#   echo 4G > /sys/fs/cgroup/memory/s8ref/memory.limit_in_bytes
echo $$ > /sys/fs/cgroup/cpuset/s8ref/tasks
echo $$ > /sys/fs/cgroup/memory/s8ref/tasks
exec "$@"
