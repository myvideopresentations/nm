const LOAD = 'redux-example/pages/LOAD';
const LOAD_SUCCESS = 'redux-example/pages/LOAD_SUCCESS';
const LOAD_FAIL = 'redux-example/pages/LOAD_FAIL';

const initialState = {
  loaded: false,
  pages: [
    { pathname: '/', title: 'Traffic' },
  ]
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
      const { info } = result;
      return {
        ...state,
        loading: false,
        loaded: true,
        version: info.version
      };
    }
    case LOAD_FAIL: {
      const { error } = action;
      return {
        ...state,
        loading: false,
        loaded: false,
        error
      };
    }
    default:
      return state;
  }
}

export function isLoaded(globalState) {
  return globalState.pages && globalState.pages.loaded;
}

export function load() {
  return {
    types: [LOAD, LOAD_SUCCESS, LOAD_FAIL],
    promise: ({ client }) => Promise.all([
      client.get('/info')
    ]).then(results => ({
      info: results[0]
    }))
  };
}

