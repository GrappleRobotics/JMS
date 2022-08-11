import App from 'App';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { resource_id_lock } from 'support/resourceid';

import { WebsocketManagerComponent } from 'support/ws-component';
import './App.scss';

// Ensure only the current loaded page has a resource id attached - duplicates should be cleared
resource_id_lock();

const root = ReactDOM.createRoot(document.getElementById('root')!);

root.render(
  <BrowserRouter>
    <WebsocketManagerComponent>
        <App />
    </WebsocketManagerComponent>
  </BrowserRouter>
);