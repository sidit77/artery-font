
## Run the desktop version
```sh
cargo run
```

## Run the web version

```sh
cd platforms/wasm
npm install

# Builds the project and opens it in a new browser tab. Auto-reloads when the project changes.
npm start

#or

# Builds the project and places it into the `dist` folder.
npm run build
```

## Run the android version

```sh
cd platforms/android
cargo apk run
```