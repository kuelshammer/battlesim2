import '@/styles/globals.scss'
import type { AppProps } from 'next/app'

// Critical: Ensure crypto is available globally for WASM before ANY components mount
if (typeof window !== 'undefined' && typeof window.crypto !== 'undefined') {
  const originalCrypto = window.crypto;

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
    if (typeof globalThis !== 'undefined' && typeof (globalThis as { crypto?: Crypto }).crypto === 'undefined') {
      (globalThis as { crypto?: Crypto }).crypto = cryptoWrapper;
    }
  } catch (e) {
    // Silently handle crypto polyfill errors in production
  }

  try {
    if (typeof self !== 'undefined' && self !== window && typeof self.crypto === 'undefined') {
      (self as { crypto: Crypto }).crypto = cryptoWrapper;
    }
  } catch (e) {
    // Silently handle crypto polyfill errors in production
  }
} else {
  }

export default function App({ Component, pageProps }: AppProps) {
  return <Component {...pageProps} />
}
