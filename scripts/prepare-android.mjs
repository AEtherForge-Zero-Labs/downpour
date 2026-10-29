// Reapply native integration after 'tauri android init' generates gen/android.
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
const android = join(root, 'src-tauri/gen/android');
const gradle = join(android, 'app/build.gradle.kts');
const manifest = join(android, 'app/src/main/AndroidManifest.xml');
if (!existsSync(gradle) || !existsSync(manifest)) {
  throw new Error('Run npm run android:init before npm run android:prepare.');
}
const dependency = 'implementation("dev.ffmpegkit-maintained:yt-dlp-android:2.0.2")';
let build = readFileSync(gradle, 'utf8');
if (!build.includes(dependency)) {
  build += '\n// Downpour embedded yt-dlp dependency\ndependencies {\n    ' + dependency + '\n}\n';
  writeFileSync(gradle, build);
}
let xml = readFileSync(manifest, 'utf8');
if (!xml.includes('android.permission.INTERNET')) {
  xml = xml.replace(/<application\b/, '<uses-permission android:name="android.permission.INTERNET" />\n    <application');
  writeFileSync(manifest, xml);
}
const dest = join(android, 'app/src/main/java/com/aetherforge/downpour/DownloaderPlugin.kt');
mkdirSync(dirname(dest), { recursive: true });
copyFileSync(join(root, 'src-tauri/android/DownloaderPlugin.kt'), dest);
console.log('Android native downloader installed into the generated Tauri project.');
