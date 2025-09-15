# WPP Client

WPPClient je klijentski dio fullstack aplikacije za real-time komunikaciju
pomoću WebSocketa (poruke i signalizacija) i WebRTC (video pozivi).
Izgrađen je pomoću [Dioxsus](https://dioxuslabs.com/) web frameworka radi
lakše poveznosti sa poslužiteljom (jedan od dependancy je wpp-server/shared
koji sadrži definicije strukture zahtjeva i odgovora).

## Build Requirements

Prije no što možeš build-ati cijeli projekt, moraš instalirati neke alate.
Treba ti [Nodejs i NPM](https://nodejs.org/en) i [Rust](https://www.rust-lang.org/) (verzija 1.85 ili veća).

## Building

Nodejs paketi

```shell
npm ci
```

Kompilacija CSS-a

```shell
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```

Instalacija framework cli alata

```shell
cargo install dioxus-cli
```

Adrese servera, potrebne pri kompilaciji, trebaju se izvesti u okruženje (env)

```shell
# bash
export BACKEND_URL="http://localhost:3030"
export BACKEND_URL_WS="ws://localhost:3030"
```

```fish
# fish
set -x BACKEND_URL "http://localhost:3030"
set -x BACKEND_URL_WS "ws://localhost:3030"
```

Pokreni server (po defaultu na portu 8080)

```shell
dx serve --release
```

Ili ga build-aj pa pokreni s [Nginx-om](https://nginx.org/) ili nekim drugim alatom

```shell
dx build --release
```
