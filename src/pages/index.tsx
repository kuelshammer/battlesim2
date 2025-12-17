import Head from 'next/head'
import React from 'react'
import Link from 'next/link'
import RGPD from '../components/utils/rgpd'
import Logo from '../components/utils/logo'
import Footer from '../components/utils/footer'
import Simulation from '../components/simulation/simulation'

export default function Home() {
  return (
    <>
      <Head>
        <title>Battle Sim</title>
        <meta name="description" content="Build balanced encounters!" />
        <link rel="icon" href="./ico.ico" />
      </Head>

      <main>
        <div style={{padding: "1rem", textAlign: "center", backgroundColor: "#f8f9fa", borderBottom: "1px solid #dee2e6"}}>
          <Link href="/simulation" style={{color: "#007bff", textDecoration: "none", fontWeight: "bold"}}>
            🚀 Try the Old Simulation Dashboard
          </Link>
        </div>
        <Simulation />
        <RGPD />
        <Logo />
        <Footer />
      </main>
    </>
  )
}
