import '@/styles/globals.scss'
import type { AppProps } from 'next/app'
import { Crimson_Pro, Cinzel_Decorative } from 'next/font/google'

const crimsonPro = Crimson_Pro({
  subsets: ['latin'],
  weight: ['400', '600', '700'],
  variable: '--font-crimson-pro',
})

const cinzelDecorative = Cinzel_Decorative({
  subsets: ['latin'],
  weight: ['400', '700'],
  variable: '--font-cinzel-decorative',
})

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
  } catch {
    // Silently handle crypto polyfill errors in production
  }

  try {
    if (typeof self !== 'undefined' && self !== window && typeof self.crypto === 'undefined') {
      (self as { crypto: Crypto }).crypto = cryptoWrapper;
    }
  } catch {
    // Silently handle crypto polyfill errors in production
  }
}

export default function App({ Component, pageProps }: AppProps) {
  return (
    <main className={`${crimsonPro.variable} ${cinzelDecorative.variable} font-serif`}>
      <Component {...pageProps} />
    </main>
  )
}
