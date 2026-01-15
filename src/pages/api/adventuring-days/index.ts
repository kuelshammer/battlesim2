import type { NextApiRequest, NextApiResponse } from 'next'
import fs from 'fs'
import path from 'path'

const SAVES_DIR = path.join(process.cwd(), 'adventuring_days')

export default function handler(req: NextApiRequest, res: NextApiResponse) {
  if (req.method === 'GET') {
    try {
      if (!fs.existsSync(SAVES_DIR)) {
        fs.mkdirSync(SAVES_DIR)
      }
      const files = fs.readdirSync(SAVES_DIR)
      const saves = files
        .filter(file => file.endsWith('.json'))
        .map(file => {
          const filePath = path.join(SAVES_DIR, file)
          const stats = fs.statSync(filePath)
          try {
            const content = JSON.parse(fs.readFileSync(filePath, 'utf8'))
            return {
              name: content.name || file.replace('.json', ''),
              filename: file,
              updated: stats.mtime.getTime(),
              players: content.players,
              timeline: content.timeline || content.encounters || []
            }
          } catch {
            return null
          }
        })
        .filter(s => s !== null)
      
      res.status(200).json(saves)
    } catch {
      res.status(500).json({ error: 'Failed to load saves' })
    }
  } else if (req.method === 'POST') {
    try {
      const { name, players, timeline } = req.body
      if (!name) {
        return res.status(400).json({ error: 'Name is required' })
      }

      if (!fs.existsSync(SAVES_DIR)) {
        fs.mkdirSync(SAVES_DIR)
      }

      const filePath = path.join(SAVES_DIR, `${name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}.json`)
      const content = {
        name,
        players,
        timeline,
        updated: Date.now()
      }

      fs.writeFileSync(filePath, JSON.stringify(content, null, 2))
      res.status(200).json({ success: true })
    } catch {
      res.status(500).json({ error: 'Failed to save' })
    }
  } else {
    res.setHeader('Allow', ['GET', 'POST'])
    res.status(405).end(`Method ${req.method} Not Allowed`)
  }
}
