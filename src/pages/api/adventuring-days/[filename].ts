import type { NextApiRequest, NextApiResponse } from 'next'
import fs from 'fs'
import path from 'path'

const SAVES_DIR = path.join(process.cwd(), 'adventuring_days')

export default function handler(req: NextApiRequest, res: NextApiResponse) {
  const { filename } = req.query
  if (!filename || typeof filename !== 'string') {
    return res.status(400).json({ error: 'Filename is required' })
  }

  const filePath = path.join(SAVES_DIR, filename.endsWith('.json') ? filename : `${filename}.json`)

  if (req.method === 'DELETE') {
    try {
      if (fs.existsSync(filePath)) {
        fs.unlinkSync(filePath)
        res.status(200).json({ success: true })
      } else {
        res.status(404).json({ error: 'File not found' })
      }
    } catch (error) {
      res.status(500).json({ error: 'Failed to delete' })
    }
  } else if (req.method === 'GET') {
    try {
      if (fs.existsSync(filePath)) {
        const content = JSON.parse(fs.readFileSync(filePath, 'utf8'))
        res.status(200).json(content)
      } else {
        res.status(404).json({ error: 'File not found' })
      }
    } catch (error) {
      res.status(500).json({ error: 'Failed to load' })
    }
  } else {
    res.setHeader('Allow', ['GET', 'DELETE'])
    res.status(405).end(`Method ${req.method} Not Allowed`)
  }
}
