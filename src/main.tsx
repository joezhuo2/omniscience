import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import App from './App';
import { applyAppearance, loadAppearance, syncAccent } from './lib/appearance';
import { initEmbed } from './lib/ipc';
import './index.css';
import './lib/glass';

initEmbed();
applyAppearance(loadAppearance());
void syncAccent();
window.addEventListener('focus', () => void syncAccent());

const root = document.getElementById('root');
if (!root) throw new Error('#root element missing from index.html');

createRoot(root).render(
  <StrictMode>
    <App />
  </StrictMode>,
);
