import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';
import ScanResults from './components/ScanResults';
import ScanOptions from './components/ScanOptions';
import ProviderList from './components/ProviderList';

interface ScanResult {
  keys: DiscoveredKey[];
  config_instances: ConfigInstance[];
  home_dir: string;
  scanned_at: string;
  providers_scanned: string[];
}

interface DiscoveredKey {
  provider: string;
  source: string;
  value_type: string;
  confidence: number;
  redacted: string;
  hash: string;
}

interface ConfigInstance {
  instance_id: string;
  app_name: string;
  config_path: string;
  discovered_at: string;
  keys: DiscoveredKey[];
}

interface ScanOptionsType {
  home_dir?: string;
  include_full_values: boolean;
  max_file_size: number;
  only_providers?: string[];
  exclude_providers?: string[];
}

function App() {
  const [scanResult, setScanResult] = useState<ScanResult | null>(null);
  const [isScanning, setIsScanning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [version, setVersion] = useState<string>('');

  useState(() => {
    invoke<string>('get_version').then(setVersion);
  });

  const handleScan = async (options: ScanOptionsType) => {
    setIsScanning(true);
    setError(null);
    
    try {
      const result = await invoke<string>('perform_scan', { options });
      const parsed = JSON.parse(result);
      setScanResult(parsed);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsScanning(false);
    }
  };

  return (
    <div className="app">
      <header className="app-header">
        <h1>🔍 AICred</h1>
        <span className="version">v{version}</span>
      </header>

      <div className="app-content">
        <aside className="sidebar">
          <ProviderList />
          <ScanOptions onScan={handleScan} isScanning={isScanning} />
        </aside>

        <main className="main-content">
          {error && (
            <div className="error-message">
              <h3>Error</h3>
              <p>{error}</p>
            </div>
          )}

          {isScanning && (
            <div className="scanning-message">
              <div className="spinner"></div>
              <p>Scanning for credentials...</p>
            </div>
          )}

          {scanResult && !isScanning && (
            <ScanResults result={scanResult} />
          )}

          {!scanResult && !isScanning && !error && (
            <div className="welcome-message">
              <h2>Welcome to AICred</h2>
              <p>Configure your scan options and click "Start Scan" to begin.</p>
            </div>
          )}
        </main>
      </div>
    </div>
  );
}

export default App;