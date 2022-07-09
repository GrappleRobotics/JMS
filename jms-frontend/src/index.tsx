import ReactDOM from 'react-dom';
import { BrowserRouter } from 'react-router-dom';
import App from 'App';
import JmsWebsocket from 'support/ws';

import './App.scss';

let ws = new JmsWebsocket();
ws.connect();
// @ts-ignore
window['ws'] = ws;

ReactDOM.render(
  // @ts-ignore
  <BrowserRouter>
    <App ws={ws} />
  </BrowserRouter>, 
  document.getElementById("root")
);