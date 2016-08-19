# Development

## Build

Start up influxdb in a container:

`$ docker run -d --name influxdb -p 8083:8083 -p 8086:8086 -v /tmp/influxdb:/var/lib/influxdb influxdb`

Build the Dockerfile for dev:

` $ docker build -t trubadur .`

Run it:

`$ docker run --rm -it --link influxdb -v $(pwd):/src rust`

Inside the container you can use `./scripts/sse.sh &` to generate dummy traffic.

Build with `cargo run` inside `./agent/`
