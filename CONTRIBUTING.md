# Contributing to amibot

## Reporting bugs

If you find a bug, you can contribute by opening an issue on the
[GitHub](https://www.github.com/blonteractor/discord-amibot/issues). You can
choose to work on it yourself if you want, but rest assured I will get to it
when I have the time. If you want any help or have any questions you can reach
me on discord.

## Creating pull requests

If you are working on an issue or would like to address bugs/add functionality, it is recommended you write your commit messages in the [conventional commit spec](https://www.conventionalcommits.org/en/v1.0.0/) before creating pull requests.

## Seting up a local developement envoirement

### Prerequisites

- [Rust](https://www.rustlang.org) 1.17 or later.
- [Protoc]() 23.4 or later
- protoc-gen-grcp-go

### Getting the source

You can download the source by running
`git clone https://www.github.com/blonteractor/discord-amibot`. You will need
git installed, obviously. After downloading the source, run `git submodule init`
and `git submodule update` to get the dependencies.

### Even more dependencies

Download an [intermidiate certificate](https://letsencrypt.org/certificates/#intermediate-certificates) from Let's Encrypt, rename it to
`lets-encrypt.pem` and keep it in a folder named `tls/` in the root of the
project. You also need to set up a few envoirement variables in the `.env` file.
You should find a `example.env`, you use it for refrence. 

    - DISCORD_TOKEN: Make an app on the discord developer portal, go the OAuth tab, copy the token/client secret, thats your discord token.
    - DATABASE_URL: The url to your mongodb server, I recommend running one on docker, see docker docs for more info on how to set up mongodb container.
    - AMIZONE_API_URL: The url to the go-amizone backend, set it to https://fly.amizone.dev if you are not sure.
    - DEV_SERVER_ID: The ID of the server you are gonna test the bot on, this is needed so command registration doesnt take long while testing.
    - DEV_ID: Your discord user ID, right click your profile pic and click `Copy ID.`
    - PRIVATE_KEY: An encryption key that will be used to encrypt and decrypt while fetching credentials from the database (should be an AES 256 bit key encoded to Base64).

**Note:** You may use the following script to generate an encryption key; just paste the output into the `.env` file.

```rust
use aes_gcm::{aed::OsRng, Aes256Gcm, KeyInit};
use base64::Engine;

fn main() {
    let key = Aes256Gcm::generate_key(&mut OsRng);

    let base64_engine = base64::engine::general_purpose::STANDARD;

    let secret_key = base64_engine.encode(key);

    println!("private key: {}", secret_key);
}
```

### Project structure

At this point, your project structure should look a little something like so

```text
├── amizone
│  ├── proto
│  │  ├── googleapis
│  │  ├── grpc-gateway
│  │  │  └── protoc-gen-openapiv2
│  │  │     └── options
│  │  └── v1
│  └── src
│     └── api
│        └── user
├── bot
│  └── src
│     └── commands
│        ├── authentication
│        └── mac
└── tls
```

The bot is mainly divided into two parts:

- The `amizone` crate contains code to interact with the `go-amizone` backend,
  this includes a layer of abstraction over said backend so the discord specific
  code doesnt have to bother with gRPC.
- The `bot` crate contains code to interact with the discord API, like commands
  and callbacks.
