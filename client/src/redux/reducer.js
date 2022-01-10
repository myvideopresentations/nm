import notifs from './modules/notifs';
import traffic from './modules/traffic';
import pages from './modules/pages';
import notificationsReducer from './modules/notificationsReducer';

export default function createReducers() {
  return {
    online: (v = true) => v,
    notifs,
    pages,
    traffic,
    notificationsReducer
  };
}
