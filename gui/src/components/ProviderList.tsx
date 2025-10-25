import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

export default function ProviderList() {
  const [providers, setProviders] = useState<string[]>([]);
  const [scanners, setScanners] = useState<string[]>([]);

  useEffect(() => {
    invoke<string[]>('get_providers').then(setProviders);
    invoke<string[]>('get_scanners').then(setScanners);
  }, []);

  return (
    <div className="provider-list">
      <div className="provider-section">
        <h3>Providers</h3>
        <ul>
          {providers.map(p => (
            <li key={p}>{p}</li>
          ))}
        </ul>
      </div>

      <div className="provider-section">
        <h3>Scanners</h3>
        <ul>
          {scanners.map(s => (
            <li key={s}>{s}</li>
          ))}
        </ul>
      </div>
    </div>
  );
}