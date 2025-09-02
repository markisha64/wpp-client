# WPP Client

# Requirements

- nodejs and npm
- rust 1.85^

# Development

Install nodejs dependencies:

```shell
npm ci
```

Run the following command in the root of the project to start the tailwind CSS compiler:

```shell
npx tailwindcss -i ./input.css -o ./assets/tailwind.css --watch
```

Bind backend addresses
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

Run the following command in the root of the project to start the Dioxus dev server:

```shell
dx serve --hot-reload
```

- Open the browser to http://localhost:8080
