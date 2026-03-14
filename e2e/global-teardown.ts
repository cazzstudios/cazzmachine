import { execSync } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

async function globalTeardown() {
  console.log('Stopping Android emulator...');
  
  try {
    execSync('adb emu kill');
    console.log('Emulator stopped');
  } catch {
    console.log('Emulator was not running or could not be stopped');
  }
  
  const pidFile = path.join(process.cwd(), '.emulator.pid');
  if (fs.existsSync(pidFile)) {
    fs.unlinkSync(pidFile);
  }
}

export default globalTeardown;
