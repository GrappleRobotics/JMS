import App from 'App';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';

import { WebsocketManagerComponent } from 'support/ws-component';
import './App.scss';

// Small address bar on iOS 
console.log("Preload")
window.addEventListener("load",function() {
    setTimeout(function(){
        window.scrollTo(0, 1);
    }, 0);
})

const root = ReactDOM.createRoot(document.getElementById('root')!);

console.log("Prerender")
root.render(
  <BrowserRouter>
    <WebsocketManagerComponent>
        <App />
    </WebsocketManagerComponent>
  </BrowserRouter>
);
console.log("Postrender")