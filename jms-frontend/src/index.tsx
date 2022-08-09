import App from 'App';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';

import { WebsocketManagerComponent } from 'support/ws-component';
import './App.scss';

const root = ReactDOM.createRoot(document.getElementById('root')!);

root.render(
  <BrowserRouter>
    <WebsocketManagerComponent>
        <App />
    </WebsocketManagerComponent>
  </BrowserRouter>
);