declare global {
  interface Window {
    electronAPI: {
      loadWasm: (wasmFileName: string) => Promise<Uint8Array>;
    };
  }
}

export {};
