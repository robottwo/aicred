import { useState } from 'react';

interface ScanOptionsProps {
  onScan: (options: any) => void;
  isScanning: boolean;
}

export default function ScanOptions({ onScan, isScanning }: ScanOptionsProps) {
  const [homeDir, setHomeDir] = useState('');
  const [includeFullValues, setIncludeFullValues] = useState(false);
  const [maxFileSize, setMaxFileSize] = useState(1048576);
  const [selectedProviders, setSelectedProviders] = useState<string[]>([]);

  const handleScan = () => {
    onScan({
      home_dir: homeDir || undefined,
      include_full_values: includeFullValues,
      max_file_size: maxFileSize,
      only_providers: selectedProviders.length > 0 ? selectedProviders : undefined,
    });
  };

  return (
    <div className="scan-options">
      <h3>Scan Options</h3>
      
      <div className="option-group">
        <label>Home Directory (optional)</label>
        <input
          type="text"
          value={homeDir}
          onChange={(e) => setHomeDir(e.target.value)}
          placeholder="Leave empty for default"
        />
      </div>

      <div className="option-group">
        <label>Max File Size (bytes)</label>
        <input
          type="number"
          value={maxFileSize}
          onChange={(e) => setMaxFileSize(Number(e.target.value))}
        />
      </div>

      <div className="option-group">
        <label>
          <input
            type="checkbox"
            checked={includeFullValues}
            onChange={(e) => setIncludeFullValues(e.target.checked)}
          />
          Include Full Values (⚠️ Dangerous)
        </label>
      </div>

      <button
        className="scan-button"
        onClick={handleScan}
        disabled={isScanning}
      >
        {isScanning ? 'Scanning...' : 'Start Scan'}
      </button>
    </div>
  );
}