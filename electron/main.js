const { app, BrowserWindow, dialog } = require('electron');
const path = require('path');
const { spawn } = require('child_process');
const fs = require('fs');

let backendProcess;
let win;

function logFile(name, data) {
    const logPath = path.join(__dirname, 'logs', name);
    fs.appendFileSync(logPath, data + '\n');
}

function createWindow() {
    win = new BrowserWindow({
        width: 1200,
        height: 800,
        webPreferences: {
            nodeIntegration: false,
            contextIsolation: true,
        },
    });

    const frontendPath = app.isPackaged
        ? path.join(process.resourcesPath, 'frontend/.next/index.html')
        : path.join(__dirname, '../frontend/.next/index.html');

    if (!fs.existsSync(frontendPath)) {
        dialog.showErrorBox('Frontend Missing', `Cannot find frontend at ${frontendPath}`);
        app.quit();
        return;
    }

    win.loadFile(frontendPath);

    win.on('closed', () => {
        win = null;
        if (backendProcess) backendProcess.kill();
    });
}

app.whenReady().then(() => {
    // Determine backend path
    const backendPath = app.isPackaged
        ? path.join(process.resourcesPath, 'backend/target/release/nse-analyzer')
        : path.join(__dirname, '../backend/target/release/nse-analyzer');

    if (!fs.existsSync(backendPath)) {
        dialog.showErrorBox('Backend Missing', `Cannot find backend at ${backendPath}`);
        app.quit();
        return;
    }

    // Start backend
    backendProcess = spawn(backendPath, [], { stdio: ['ignore', 'pipe', 'pipe'] });

    backendProcess.stdout.on('data', (data) => logFile('backend.log', data.toString()));
    backendProcess.stderr.on('data', (data) => logFile('backend.log', data.toString()));

    backendProcess.on('exit', (code) => {
        logFile('backend.log', `Backend exited with code ${code}`);
        if (win) win.close();
    });

    createWindow();

    app.on('window-all-closed', () => {
        if (process.platform !== 'darwin') app.quit();
    });
});

app.on('before-quit', () => {
    if (backendProcess) backendProcess.kill();
});
