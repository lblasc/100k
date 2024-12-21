## 100k challenge

```
$ nix build
$ ulimit -Sn 20048
$ ./result/bin/stoeura &.
$ wrk -t28 -c5000 -d30s http://127.0.0.1:3000
Running 30s test @ http://127.0.0.1:3000
  28 threads and 5000 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency    41.67ms   17.10ms   1.06s    97.59%
    Req/Sec     4.34k   512.76     4.96k    96.48%
  3617874 requests in 30.10s, 503.74MB read
Requests/sec: 120193.90
Transfer/sec:     16.74MB
```
