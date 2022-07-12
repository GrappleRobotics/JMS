import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import App from 'App';

import './App.scss';
import { WebsocketManagerComponent } from 'support/ws-component';

const root = ReactDOM.createRoot(document.getElementById('root')!);

root.render(
  <WebsocketManagerComponent>
    <BrowserRouter>
      <App />
    </BrowserRouter>
  </WebsocketManagerComponent>
);