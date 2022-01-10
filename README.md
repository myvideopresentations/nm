# nm - Network interfaces monitor tool

## Development mode

Start server
```
cd server
cargo run
```

Start client in debug mode

```
cd client
yarn
yarn start
```

## Build client and start server
```
cd client
yarn build
````
Copy contents of the client's `build` directory into server's `static` folder

```
cd server
cargo run
```
