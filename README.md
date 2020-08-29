tokio-graceful-restart-experiment
=================================

How to do an experiment:

1. Get and build my [fork](https://github.com/hnakamur/systemfd) of [mitsuhiko/systemfd](https://github.com/mitsuhiko/systemfd).

```
git clone https://github.com/hnakamur/systemfd
cd systemfd
git switch change_to_graceful_restarter
cargo build
cd ..
```

2. Clone this repository and build `http-server` package.

```
git clone https://github.com/hnakamur/tokio-graceful-restart-experiment
cd tokio-graceful-restart-experiment
cargo build -p http-server
```

3. Run the http-server.

```
../systemfd/target/debug/systemfd --no-pid -s http::8080 -- ./target/debug/http-server
```

4. Run `curl` loop in another terminal.

```
while :; do curl http://localhost:8080/foo/1/index.html; done
```

5. In another terminal, Modify source of `http-server` and build it.

```
sed -i 's/Hello/Hi/' http-server/src/main.rs
cargo build -p http-server
```

6. Find the PID of `systemfd` and send `USR2` to it.

```
ps auxww | grep [s]ystemfd
kill -USR2 _PID_of_systemfd_found_above_here_
```
