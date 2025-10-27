const { PNG } = require('pngjs');
const fs = require('fs');
const path = require('path');
const pngToIco = require('png-to-ico');

function createIcon(size, outputPath) {
  const png = new PNG({ width: size, height: size });
  
  // Fill with blue background (rounded rectangle approximation)
  const margin = Math.floor(size * 0.1);
  const radius = Math.floor(size * 0.1);
  
  for (let y = 0; y < size; y++) {
    for (let x = 0; x < size; x++) {
      const idx = (size * y + x) << 2;
      
      // Check if pixel is inside rounded rectangle
      let inShape = false;
      if (x >= margin && x < size - margin && y >= margin && y < size - margin) {
        // Check corners
        if ((x < margin + radius && y < margin + radius) ||
            (x >= size - margin - radius && y < margin + radius) ||
            (x < margin + radius && y >= size - margin - radius) ||
            (x >= size - margin - radius && y >= size - margin - radius)) {
          // Calculate distance from corner center
          let cx, cy;
          if (x < margin + radius && y < margin + radius) {
            cx = margin + radius; cy = margin + radius;
          } else if (x >= size - margin - radius && y < margin + radius) {
            cx = size - margin - radius; cy = margin + radius;
          } else if (x < margin + radius && y >= size - margin - radius) {
            cx = margin + radius; cy = size - margin - radius;
          } else {
            cx = size - margin - radius; cy = size - margin - radius;
          }
          const dist = Math.sqrt((x - cx) ** 2 + (y - cy) ** 2);
          inShape = dist <= radius;
        } else {
          inShape = true;
        }
      }
      
      if (inShape) {
        png.data[idx] = 74;      // R
        png.data[idx + 1] = 144; // G
        png.data[idx + 2] = 226; // B
        png.data[idx + 3] = 255; // A
      } else {
        png.data[idx] = 0;
        png.data[idx + 1] = 0;
        png.data[idx + 2] = 0;
        png.data[idx + 3] = 0;
      }
    }
  }
  
  // Draw a simple "K" shape in white
  const letterMargin = Math.floor(size * 0.3);
  const letterWidth = Math.floor(size * 0.08);
  
  for (let y = letterMargin; y < size - letterMargin; y++) {
    for (let x = letterMargin; x < size - letterMargin; x++) {
      const idx = (size * y + x) << 2;
      
      // Vertical line of K
      if (x >= letterMargin && x < letterMargin + letterWidth) {
        png.data[idx] = 255;
        png.data[idx + 1] = 255;
        png.data[idx + 2] = 255;
        png.data[idx + 3] = 255;
      }
      
      // Upper diagonal of K
      const midY = size / 2;
      const upperDiagY = y - letterMargin;
      const upperDiagX = x - (letterMargin + letterWidth);
      if (upperDiagY > 0 && upperDiagX > 0 && 
          Math.abs(upperDiagY - upperDiagX) < letterWidth && 
          y < midY) {
        png.data[idx] = 255;
        png.data[idx + 1] = 255;
        png.data[idx + 2] = 255;
        png.data[idx + 3] = 255;
      }
      
      // Lower diagonal of K
      const lowerDiagY = y - midY;
      const lowerDiagX = x - (letterMargin + letterWidth);
      if (lowerDiagY > 0 && lowerDiagX > 0 && 
          Math.abs(lowerDiagY - lowerDiagX) < letterWidth && 
          y >= midY) {
        png.data[idx] = 255;
        png.data[idx + 1] = 255;
        png.data[idx + 2] = 255;
        png.data[idx + 3] = 255;
      }
    }
  }
  
  png.pack().pipe(fs.createWriteStream(outputPath));
  console.log(`Created ${outputPath}`);
}

const iconsDir = path.join(__dirname, 'src-tauri', 'icons');

// Create icons in various sizes
createIcon(512, path.join(iconsDir, 'icon.png'));
createIcon(256, path.join(iconsDir, '256x256.png'));
createIcon(128, path.join(iconsDir, '128x128.png'));
createIcon(128, path.join(iconsDir, '128x128@2x.png'));
createIcon(32, path.join(iconsDir, '32x32.png'));

// Wait a bit for PNG files to be written, then create ICO
setTimeout(async () => {
  try {
    const icoBuffer = await pngToIco([
      path.join(iconsDir, '256x256.png'),
      path.join(iconsDir, '128x128.png'),
      path.join(iconsDir, '32x32.png')
    ]);
    fs.writeFileSync(path.join(iconsDir, 'icon.ico'), icoBuffer);
    console.log('Created icon.ico');
    console.log('All icons generated successfully!');
  } catch (err) {
    console.error('Error creating ICO file:', err);
    process.exit(1);
  }
}, 1000);