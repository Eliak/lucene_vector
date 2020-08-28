##### build library
```shell script
cd ./rust
cargo +nightly build --release
```

##### run jmh
```shell script
./gradlew --stop
./gradlew clean jmh --info
```