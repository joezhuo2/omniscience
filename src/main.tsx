import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { applyAppearance, loadAppearance } from './lib/appearance';
import { initEmbed } from './lib/ipc';
import './index.css';
import './lib/glass';

initEmbed();
applyAppearance(loadAppearance());

const root = document.getElementById('root');
if (!root) throw new Error('#root element missing from index.html');

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
