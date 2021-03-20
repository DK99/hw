We only modified the desktop client. All other clients (e.g. android, iOS, ...) will not work.

Build Instructions:

```
cmake -DCMAKE_BUILD_TYPE="Release" -DNOSERVER=1 -DNOVIDEOREC=1 -DNOPNG=1 .
make install
./bin/hedgewars
```