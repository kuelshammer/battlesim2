import Head from 'next/head'
import React from 'react'
import dynamic from 'next/dynamic'
import RGPD from '../components/utils/rgpd'
import Logo from '../components/utils/logo'
import Footer from '../components/utils/footer'

const Simulation = dynamic(() => import('../components/simulation/simulation').catch(err => {
  console.error("Failed to load Simulation component:", err);
  return () => <div style={{color: "#ef4444", padding: "2rem", textAlign: "center"}}>Error loading simulation engine: {err.message}</div>;
}), {
  ssr: false,
  loading: () => <div style={{textAlign: "center", padding: "2rem", animation: "pulse 2s cubic-bezier(0.4, 0, 0.6, 1) infinite"}}>Loading Simulation Engine...</div>
})

export default function Home() {
  return (
    <>
      <Head>
        <title>Battle Sim</title>
        <meta name="description" content="Build balanced encounters!" />
        <link rel="icon" href="./ico.ico" />
      </Head>

      <main>
        <Simulation />
        <RGPD />
        <Logo />
        <Footer />
      </main>
    </>
  )
}
