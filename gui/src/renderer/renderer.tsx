import './index.css';
import React from 'react';
import { createRoot } from 'react-dom/client';
import App, { router } from './App';
import { RouterProvider } from '@tanstack/react-router';


const container = document.getElementById('root');

if (container) {
  const root = createRoot(container);
  root.render(
    <React.StrictMode>
      <RouterProvider router={router} />
    </React.StrictMode >
  )
}

