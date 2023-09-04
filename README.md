# rpot

Rpot is a very basic rpot honeypot that logs passwords and executed commands
# Usage:
## Running
```bash
cargo run
```
## Running in Docker
```bash
docker build . -t rpot
docker run -p 25575:25575 -d rpot
```
## Changing bind address or port
To change the bind address or port, set the `RPOT_BIND_ADDR` or `RPOT_BIND_PORT` enviroment variable
