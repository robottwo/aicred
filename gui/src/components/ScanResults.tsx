interface ScanResultsProps {
  result: {
    keys: any[];
    config_instances: any[];
    home_dir: string;
    scanned_at: string;
    providers_scanned: string[];
  };
}

export default function ScanResults({ result }: ScanResultsProps) {
  return (
    <div className="scan-results">
      <div className="results-header">
        <h2>Scan Results</h2>
        <div className="results-meta">
          <span>Scanned: {new Date(result.scanned_at).toLocaleString()}</span>
          <span>Home: {result.home_dir}</span>
        </div>
      </div>

      <div className="results-summary">
        <div className="summary-card">
          <h3>{result.keys.length}</h3>
          <p>Keys Found</p>
        </div>
        <div className="summary-card">
          <h3>{result.config_instances.length}</h3>
          <p>Config Instances</p>
        </div>
        <div className="summary-card">
          <h3>{result.providers_scanned.length}</h3>
          <p>Providers Scanned</p>
        </div>
      </div>

      {result.keys.length > 0 && (
        <div className="keys-section">
          <h3>Discovered Keys</h3>
          <table className="keys-table">
            <thead>
              <tr>
                <th>Provider</th>
                <th>Source</th>
                <th>Type</th>
                <th>Confidence</th>
                <th>Value</th>
              </tr>
            </thead>
            <tbody>
              {result.keys.map((key, idx) => (
                <tr key={idx}>
                  <td>{key.provider}</td>
                  <td className="source-cell" title={key.source}>
                    {String(key.source || "").split("/").pop()}
                  </td>
                  <td>{key.value_type}</td>
                  <td>
                    <span
                      className={`confidence confidence-${
                        key.confidence > 0.9
                          ? "high"
                          : key.confidence > 0.7
                          ? "medium"
                          : "low"
                      }`}
                    >
                      {(key.confidence * 100).toFixed(0)}%
                    </span>
                  </td>
                  <td className="redacted-value">{key.redacted}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}

      {result.config_instances.length > 0 && (
        <div className="instances-section">
          <h3>Config Instances</h3>
          <div className="instances-grid">
            {result.config_instances.map((instance: any, idx: number) => (
              <div key={idx} className="instance-card">
                <h4>{instance.app_name}</h4>
                <p className="instance-path">{instance.config_path}</p>
                <span className="instance-keys">
                  {(instance.keys || []).length} keys
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}