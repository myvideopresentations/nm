# nm - Network Monitoring tool for Web written in Rust and React

![intro](/server/static/intro.gif)

## Sources:
* Yuichi Inagaki [mone](https://github.com/gky360/mone) - Network monitoring tool written in Rust.
* Sergio Benitez [Rocket Web framework](https://github.com/SergioBenitez/Rocket/tree/master/examples/chat) - Chat Sample
* [Create React App](https://create-react-app.dev/)
* [Material-UI](https://material-ui.com/)
* [Hooks](https://reactjs.org/docs/hooks-intro.html)
* [Redux](https://redux.js.org/)
* [Reach Router](https://reach.tech/router/)

## Requirements

- Linux / OS X

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
