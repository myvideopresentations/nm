import React from 'react';
import Helmet from 'react-helmet';
import ReactDOM from 'react-dom';
import PropTypes from 'prop-types';
import { Provider as ReduxProvider } from 'react-redux';
import initRedux from './redux/initRedux';
import { createHistory, LocationProvider }from "@reach/router";
import routes from './routes';
import apiClient from './helpers/apiClient';
import reportWebVitals from './reportWebVitals';

const client = apiClient();

const helpers = {
  client
}

const history = createHistory(window);

function AppWrapper(props) {
  const { children } = props;

  //const location = useLocation();
  const [redux] = React.useState(() =>
    initRedux({}, helpers),
  );

  let fonts = ['https://fonts.googleapis.com/css?family=Roboto:300,400,500,700&display=swap'];

  return (
    <React.Fragment>
      <Helmet>
        {fonts.map((font) => (
          <link rel="stylesheet" href={font} key={font} />
        ))}
      </Helmet>
      <ReduxProvider store={redux}>
        <LocationProvider history={history}>
          {children}
        </LocationProvider>
      </ReduxProvider>
    </React.Fragment>
  );
}

AppWrapper.propTypes = {
  children: PropTypes.node.isRequired
};

ReactDOM.render(
  <AppWrapper>
    {routes}
  </AppWrapper>
  ,
  document.getElementById('root')
);

// If you want to start measuring performance in your app, pass a function
// to log results (for example: reportWebVitals(console.log))
// or send to an analytics endpoint. Learn more: https://bit.ly/CRA-vitals
reportWebVitals();
