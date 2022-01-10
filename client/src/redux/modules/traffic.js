const LOAD = 'redux-example/traffic/LOAD';
const LOAD_SUCCESS = 'redux-example/traffic/LOAD_SUCCESS';
const LOAD_FAIL = 'redux-example/traffic/LOAD_FAIL';
const UPDATEMESSAGE = 'redux-example/traffic/updateMessage';
const CHARTSIZE = 30;
const CHARTSTEP = 2;

function getXAxisValue(index) {
  return ((CHARTSIZE - index)* CHARTSTEP ).toString();
}

const initialState = {
  message: "Default Message",
  loaded: false,
  interfaces: []
};

export default function reducer(state = initialState, action = {}) {
  switch (action.type) {
    case LOAD: {
      return {
        ...state,
        loading: true
      };
    }
    case LOAD_SUCCESS: {
      const { result } = action;
      const { version, name } = result;
      return {
        ...state,
        loading: false,
        loaded: true,
        version,
        name
      };
    }
    case LOAD_FAIL: {
      const { error } = action;
      return {
        ...state,
        version: 'File not found',
        loading: false,
        loaded: false,
        error
      };
    }
    case UPDATEMESSAGE: {
      var {info, stats} = action;
      var { interfaces } = state;
      var hash = interfaces.reduce((acc, value) => {
        acc[value.name] = value;
        return acc;
      }, {});
      var newInterfaces = info.map(({ name }, index) => {
        var current = hash[name];
        if(!current) {
          const indexes = new Array(CHARTSIZE - 1).fill(0)
          const rx = indexes.map((_, index) => { return {x: getXAxisValue(index), y: 0};});
          const tx = indexes.map((_, index) => { return {x: getXAxisValue(index), y: 0};});
          current = {
            name,
            rx,
            tx
          }
        } else {
          const indexes = new Array(CHARTSIZE - 1).fill(0)
          const rx = indexes.map((_, index) => { return {x: getXAxisValue(index), y: current.rx[index + 1].y};});
          const tx = indexes.map((_, index) => { return {x: getXAxisValue(index), y: current.tx[index + 1].y};});
          current = {
            name,
            rx,
            tx
          }          
        }
        current.rx.push({x: getXAxisValue(CHARTSIZE-1), y: stats[index].rx});
        current.tx.push({x: getXAxisValue(CHARTSIZE-1), y: stats[index].tx});
        return { name,
          rx: current.rx,
          tx: current.tx
        }
      })
      return {
        ...state,
        interfaces: newInterfaces
      };
    }
    default:
      return state;
  }
}

export function isLoaded(globalState) {
  return globalState.traffic && globalState.traffic.loaded;
}

export function load() {
  return {
    types: [LOAD, LOAD_SUCCESS, LOAD_FAIL],
    promise: ({ client }) => client.get(`/info`)
  };
}

export function updateMessage(message) {
  return {
    type: UPDATEMESSAGE,
    ...message
  };
}
