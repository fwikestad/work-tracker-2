import sharp from 'sharp';
import { execSync } from 'child_process';

const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 1024 1024">
  <!-- Background: dark rounded square -->
  <rect width="1024" height="1024" rx="220" fill="#1a1a2e"/>
  
  <!-- Outer circle rim -->
  <circle cx="512" cy="512" r="380" fill="none" stroke="#4ade80" stroke-width="28"/>
  
  <!-- Hour markers (12, 3, 6, 9) -->
  <line x1="512" y1="160" x2="512" y2="200" stroke="#4ade80" stroke-width="20" stroke-linecap="round"/>
  <line x1="864" y1="512" x2="824" y2="512" stroke="#4ade80" stroke-width="20" stroke-linecap="round"/>
  <line x1="512" y1="864" x2="512" y2="824" stroke="#4ade80" stroke-width="20" stroke-linecap="round"/>
  <line x1="160" y1="512" x2="200" y2="512" stroke="#4ade80" stroke-width="20" stroke-linecap="round"/>
  
  <!-- Minor hour markers (the other 8) -->
  <line x1="688" y1="191" x2="668" y2="225" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="833" y1="336" x2="799" y2="356" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="833" y1="688" x2="799" y2="668" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="688" y1="833" x2="668" y2="799" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="336" y1="833" x2="356" y2="799" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="191" y1="688" x2="225" y2="668" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="191" y1="336" x2="225" y2="356" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  <line x1="336" y1="191" x2="356" y2="225" stroke="#4ade80" stroke-width="10" stroke-linecap="round" opacity="0.6"/>
  
  <!-- Hour hand (pointing to ~10 o'clock) -->
  <line x1="512" y1="512" x2="348" y2="330" stroke="white" stroke-width="32" stroke-linecap="round"/>
  
  <!-- Minute hand (pointing to ~2 o'clock) -->
  <line x1="512" y1="512" x2="648" y2="230" stroke="white" stroke-width="22" stroke-linecap="round"/>
  
  <!-- Center dot -->
  <circle cx="512" cy="512" r="24" fill="#4ade80"/>
</svg>`;

console.log('Converting SVG to 1024x1024 PNG...');

// Convert SVG to 1024x1024 PNG
await sharp(Buffer.from(svg))
  .resize(1024, 1024)
  .png()
  .toFile('src-tauri/icons/app-source.png');

console.log('✓ Generated app-source.png');
console.log('Running Tauri icon generator...');

// Run tauri icon to generate all sizes
try {
  execSync('npx @tauri-apps/cli icon src-tauri/icons/app-source.png', { stdio: 'inherit' });
  console.log('✓ All icon sizes generated!');
} catch (e) {
  console.error('Failed to run Tauri icon generator:', e.message);
  process.exit(1);
}
