import { Router } from "@reach/router"
import {
  App,
  NotFound,
  Traffic
} from './containers';

const routes =   
<Router>
  <App path="/">
    <Traffic path="/"/>
    <NotFound default />
  </App>
</Router>

export default routes;
