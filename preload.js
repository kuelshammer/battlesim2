const { contextBridge, ipcRenderer } = require('electron');

// Polyfill for Web Crypto API is not needed in modern Electron as window.crypto is available
// const crypto = require('crypto');
// if (!global.crypto) { ... }

contextBridge.exposeInMainWorld('electronAPI', {
  loadWasm: async (wasmFileName) => {
    try {
      // In packaged app, files are relative to the HTML file
      const response = await fetch(wasmFileName);
      if (!response.ok) {
        throw new Error(`Failed to fetch WASM file: ${response.status}`);
      }
      const wasmBytes = await response.arrayBuffer();
      return wasmBytes;
    } catch (error) {
      console.error('Error loading WASM file in preload:', error);
      throw error;
    }
  },
});
