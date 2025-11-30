const { app, BrowserWindow } = require('electron')
const path = require('path')

function createWindow() {
  console.log('Creating window...')

  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: false,
      contextIsolation: true,
      // preload: path.join(__dirname, 'preload.js')
    }
  })

  // win.webContents.openDevTools()

  if (app.isPackaged) {
    console.log('Loading static file from out/index.html')
    win.loadFile(path.join(__dirname, 'out/index.html'))
  } else {
    console.log('Loading from localhost:3000')
    win.loadURL('http://localhost:3000')
      .catch(err => console.log('Error loading URL:', err))
  }
}

app.whenReady().then(() => {
  createWindow()

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow()
    }
  })
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  }
})
