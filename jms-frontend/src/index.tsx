import React from 'react';
import ReactDOM from 'react-dom';
import { BrowserRouter } from 'react-router-dom';
import App from 'App';

import './App.scss';

ReactDOM.render(
  // @ts-ignore
  <BrowserRouter>
    <App />
  </BrowserRouter>, 
  document.getElementById("root")
);