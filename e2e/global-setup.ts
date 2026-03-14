import { execSync, spawn } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

async function globalSetup() {
  console.log('Starting Android emulator...');
  
  try {
    const devices = execSync('adb devices', { encoding: 'utf8' });
    if (devices.includes('emulator-')) {
      console.log('Emulator already running');
      return;
    }
  } catch {
  }
  
  const emulatorProcess = spawn(
    process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT || '/opt/android-sdk',
    ['-avd', 'Pixel_3a_API_34_extension_level_7_x86_64', '-no-audio', '-no-window', '-no-boot-anim'],
    { detached: true, stdio: 'ignore' }
  );
  
  const pidFile = path.join(process.cwd(), '.emulator.pid');
  fs.writeFileSync(pidFile, emulatorProcess.pid?.toString() || '');
  
  console.log('Waiting for emulator to boot...');
  let attempts = 0;
  const maxAttempts = 30;
  
  while (attempts < maxAttempts) {
    try {
      execSync('adb wait-for-device', { timeout: 5000 });
      const bootCompleted = execSync('adb shell getprop sys.boot_completed', { encoding: 'utf8' }).trim();
      if (bootCompleted === '1') {
        console.log('Emulator ready');
        return;
      }
    } catch {
    }
    await new Promise(r => setTimeout(r, 2000));
    attempts++;
  }
  
  throw new Error('Emulator failed to start within 60 seconds');
}

export default globalSetup;
