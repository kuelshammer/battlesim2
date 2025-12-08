import '../../styles/globals.scss'
import type { AppProps } from 'next/app'

// Critical: Ensure crypto is available globally for WASM before ANY components mount
if (typeof window !== 'undefined' && typeof window.crypto !== 'undefined') {
  const originalCrypto = window.crypto;
  console.log('[Crypto Polyfill] window.crypto exists:', !!originalCrypto);
  console.log('[Crypto Polyfill] window.crypto.getRandomValues:', typeof originalCrypto.getRandomValues);

  // Create a wrapper object with getRandomValues properly bound
  // This is needed because wasm-bindgen expects a callable function
  const cryptoWrapper = {
    getRandomValues: originalCrypto.getRandomValues.bind(originalCrypto),
    // Copy other crypto methods if needed
    subtle: originalCrypto.subtle,
    randomUUID: originalCrypto.randomUUID?.bind(originalCrypto),
  };

  // Try to set on various global scopes (some may be read-only)
  try {
    if (typeof globalThis !== 'undefined' && typeof (globalThis as any).crypto === 'undefined') {
      (globalThis as any).crypto = cryptoWrapper;
      console.log('[Crypto Polyfill] Set globalThis.crypto with bound wrapper');
    }
  } catch (e) {
    console.log('[Crypto Polyfill] Could not set globalThis.crypto:', e);
  }

  try {
    if (typeof self !== 'undefined' && self !== window && typeof (self as any).crypto === 'undefined') {
      (self as any).crypto = cryptoWrapper;
      console.log('[Crypto Polyfill] Set self.crypto with bound wrapper');
    }
  } catch (e) {
    console.log('[Crypto Polyfill] Could not set self.crypto:', e);
  }
} else {
  }

export default function App({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />
}
