# GM Payout Accounting Services

This repository currently contains:
* Payment processing services
* Payout services
* Payout splitting (affiliates, vendors, platform) and accounting services
* Double-entry accounting between transactions and payouts.


<a name="table-of-contents"></a>
## Table of contents

<!--ts-->
   * [Table of contents](#table-of-contents)
   * [Basic Installation](#basic-installation)
      * [Enabling SSL](#enabling-ssl)
   * [Docker Build Instructions](#building-docker)
      * [Misc. Docker Commands](#docker-misc-commands)
   * [Directory Structure](#directory-structure)
<!--te-->

<a name="basic-installation"></a>
## Basic Installation

Run:
```bash
git clone https://github.com/peitalin/gm-payment-service
cd ./gm-payment-service
cargo build
```

<a name="enabling-ssl"></a>
Stripe frontends require HTTPS (ssl), meaning the connection to backed also needs SSL enabled.
This is required for local development.

### Enabling SSL
Run to make SSL certs, then move them:
```
mkcert -install;
mkcert localhost 127.0.0.1 0.0.0.0
mv localhost+2-key.pem keys/mkcert/localhost-key.pem
mv localhost+2.pem keys/mkcert/localhost.pem
```
* Required for Stripe requests
* These are fake self-signed certs just for development purposes
* You need a proper SSL cert (fileworks.net cert) to deploy

Once SSL is enabled you can run the payment service locally:
```bash
cargo run --bin payment
```


* [Back to Table of Contents](#table-of-contents)
---

<a name="building-docker"></a>
## Docker Build Instructions

Build the docker image with this command:
```bash
docker build -f ./Dockerfile -t gcr.io/fileworks/gm-payment-service:latest .
```

Run the docker image:
```bash
docker run -p 8898:8898 \
  -e JWT_DOMAIN="127.0.0.1" \
  gcr.io/fileworks/gm-payment-service
```
* **PS**: If using SSL locally, You need to set `-e JWT_DOMAIN="127.0.0.1"` for local development to get 'set-cookies' credentials to work.
* HttpOnly cookies will not be set if there is domain mismatch, it's automatically set.

Then push to Google Container Registry
```bash
docker push gcr.io/fileworks/gm-payment-service:latest
```


<a name="docker-misc-commands"></a>
### Misc. Docker Commands
To remove old containers and images, try:
```bash
### Remove images
docker rmi <image-id>
docker rm <container-id>
docker image prune

### Stop and remove all containers
docker stop $(docker ps -a -q)
docker rm $(docker ps -a -q)
```

* If there are dangling/detached containers, use `sudo kill $(lsof -t -i:8082)` which will kill the process hogging port 8082 (if it is runnning in the background).


* [Back to Table of Contents](#table-of-contents)
---

<a name="directory-structure"></a>

## Directory Structure

The top level directory is organized as follows:
1. Cargo.toml contains dependencies + commands
```
├── "Cargo.toml"
├── "Dockerfile"
├── "README.md"
├── "diesel.toml"  // set source for schema
├── "keys"
├── "migrations"
├── "src"
│   ├── "bin" // Where the main src code lives
│   ├── "db"
│   ├── "lib.rs"
│   └── "utils"
```
1. `src/bin/payment` is where the main src code resides.

2. The other folders in `src` are helper functions (`lib.rs`). Modules in the `src/bin/payment` folder can access `lib.rs` by `use gm::utils`.


### Services Directory Structure
The `src/bin/payment/main.rs` file is the entry point for the service, which you build and execute with `cargo run --bin payment`.

```bash
"src/bin/"
└── "payment"
    ├── "db/"      // Postgres and redis actors + message handlers
    ├── "main.rs"
    ├── "models/"  // structs and user models
    └── "rest/"    // handlers for each REST endpoint
```


1. If you want access to functions exposed in the top-level library (`./src/lib.rs`), you can `use gm::MODULENAME`

2. The `main.rs` file runs a server, which routes requests to "actors" that handle and process requests (e.g. db, graphql, login, etc). See: <https://actix.rs/>.

* 3a. Actors handles async event loops, and queues messages to be dispatched to db, websocket, and graphql services.

* 3b. Actors all implement the `Actor` trait (how and when the actor starts and stops etc), and `Handler<Message>` trait (how it handles specific `Messages`).




