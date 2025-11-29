const { contextBridge, ipcRenderer } = require('electron');

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
