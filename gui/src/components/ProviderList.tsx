import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Tag, Label, ProviderInstance } from '../types/tagging';
import AssignmentModal from './AssignmentModal';

interface ProviderListProps {
  onInstanceSelect?: (instance: ProviderInstance) => void;
  selectedInstanceId?: string;
  scanResult?: {
    config_instances: ProviderInstance[];
  };
}

export default function ProviderList({ onInstanceSelect, selectedInstanceId, scanResult }: ProviderListProps) {
  const [providers, setProviders] = useState<string[]>([]);
  const [scanners, setScanners] = useState<string[]>([]);
  const [tags, setTags] = useState<Tag[]>([]);
  const [labels, setLabels] = useState<Label[]>([]);
  const [tagAssignments, setTagAssignments] = useState<any[]>([]);
  const [labelAssignments, setLabelAssignments] = useState<any[]>([]);
  const [instances, setInstances] = useState<ProviderInstance[]>([]);
  const [selectedTagIds, setSelectedTagIds] = useState<string[]>([]);
  const [selectedLabelIds, setSelectedLabelIds] = useState<string[]>([]);
  const [showAssignmentModal, setShowAssignmentModal] = useState(false);
  const [assignmentModalData, setAssignmentModalData] = useState<{
    type: 'tag' | 'label';
    instanceId?: string;
    modelId?: string;
  }>({ type: 'tag' });

  useEffect(() => {
    loadData();
  }, []);

  useEffect(() => {
    if (scanResult?.config_instances) {
      setInstances(scanResult.config_instances);
    }
  }, [scanResult]);

  const loadData = async () => {
    try {
      // Load providers and scanners
      const providersData = await invoke<string[]>('get_providers');
      const scannersData = await invoke<string[]>('get_scanners');
      setProviders(providersData);
      setScanners(scannersData);

      // Load tags and labels
      const tagsResult = await invoke<string>('list_tags');
      const tagsData = JSON.parse(tagsResult);
      setTags(tagsData);

      const labelsResult = await invoke<string>('list_labels');
      const labelsData = JSON.parse(labelsResult);
      setLabels(labelsData);

      // Load assignments
      const tagAssignmentsResult = await invoke<string>('list_tag_assignments');
      const tagAssignmentsData = JSON.parse(tagAssignmentsResult);
      setTagAssignments(tagAssignmentsData);

      const labelAssignmentsResult = await invoke<string>('list_label_assignments');
      const labelAssignmentsData = JSON.parse(labelAssignmentsResult);
      setLabelAssignments(labelAssignmentsData);

    } catch (err) {
      console.error('Error loading data:', err);
    }
  };

  const getInstanceTags = (instanceId: string, modelId?: string) => {
    const instanceTagIds = tagAssignments
      .filter((assignment: any) => 
        assignment.instance_id === instanceId &&
        (!modelId || !assignment.model_id || assignment.model_id === modelId)
      )
      .map((assignment: any) => assignment.tag_id);
    
    return tags.filter(tag => instanceTagIds.includes(tag.id));
  };

  const getInstanceLabels = (instanceId: string, modelId?: string) => {
    const instanceLabelIds = labelAssignments
      .filter((assignment: any) => 
        assignment.instance_id === instanceId &&
        (!modelId || !assignment.model_id || assignment.model_id === modelId)
      )
      .map((assignment: any) => assignment.label_id);
    
    return labels.filter(label => instanceLabelIds.includes(label.id));
  };

  const filteredInstances = instances.filter(instance => {
    // Filter by selected tags
    if (selectedTagIds.length > 0) {
      const instanceTags = getInstanceTags(instance.instance_id);
      const hasSelectedTags = selectedTagIds.some(tagId => 
        instanceTags.some(tag => tag.id === tagId)
      );
      if (!hasSelectedTags) return false;
    }

    // Filter by selected labels
    if (selectedLabelIds.length > 0) {
      const instanceLabels = getInstanceLabels(instance.instance_id);
      const hasSelectedLabels = selectedLabelIds.some(labelId => 
        instanceLabels.some(label => label.id === labelId)
      );
      if (!hasSelectedLabels) return false;
    }

    return true;
  });

  const handleTagFilter = (tagId: string) => {
    setSelectedTagIds(prev => 
      prev.includes(tagId) 
        ? prev.filter(id => id !== tagId)
        : [...prev, tagId]
    );
  };

  const handleLabelFilter = (labelId: string) => {
    setSelectedLabelIds(prev => 
      prev.includes(labelId) 
        ? prev.filter(id => id !== labelId)
        : [...prev, labelId]
    );
  };

  const handleAssignTag = (instanceId: string, modelId?: string) => {
    setAssignmentModalData({ type: 'tag', instanceId, modelId });
    setShowAssignmentModal(true);
  };

  const handleAssignLabel = (instanceId: string, modelId?: string) => {
    setAssignmentModalData({ type: 'label', instanceId, modelId });
    setShowAssignmentModal(true);
  };

  const handleAssignmentComplete = () => {
    loadData(); // Reload data to reflect changes
  };

  const clearFilters = () => {
    setSelectedTagIds([]);
    setSelectedLabelIds([]);
  };

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

      {instances.length > 0 && (
        <div className="provider-section">
          <div className="section-header">
            <h3>Instances ({filteredInstances.length})</h3>
            {(selectedTagIds.length > 0 || selectedLabelIds.length > 0) && (
              <button className="btn btn-sm btn-secondary" onClick={clearFilters}>
                Clear Filters
              </button>
            )}
          </div>

          {/* Tag Filters */}
          {tags.length > 0 && (
            <div className="filter-section">
              <h4>Filter by Tags</h4>
              <div className="tag-filters">
                {tags.map(tag => (
                  <button
                    key={tag.id}
                    className={`tag-filter ${selectedTagIds.includes(tag.id) ? 'active' : ''}`}
                    onClick={() => handleTagFilter(tag.id)}
                    style={{
                      backgroundColor: selectedTagIds.includes(tag.id) ? tag.color : 'transparent',
                      borderColor: tag.color,
                      color: selectedTagIds.includes(tag.id) ? '#fff' : tag.color
                    }}
                  >
                    {tag.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Label Filters */}
          {labels.length > 0 && (
            <div className="filter-section">
              <h4>Filter by Labels</h4>
              <div className="label-filters">
                {labels.map(label => (
                  <button
                    key={label.id}
                    className={`label-filter ${selectedLabelIds.includes(label.id) ? 'active' : ''}`}
                    onClick={() => handleLabelFilter(label.id)}
                    style={{
                      backgroundColor: selectedLabelIds.includes(label.id) ? label.color : 'transparent',
                      borderColor: label.color,
                      color: selectedLabelIds.includes(label.id) ? '#fff' : label.color
                    }}
                  >
                    {label.name}
                  </button>
                ))}
              </div>
            </div>
          )}

          {/* Instances List */}
          <div className="instances-list">
            {filteredInstances.map(instance => {
              const instanceTags = getInstanceTags(instance.instance_id);
              const instanceLabels = getInstanceLabels(instance.instance_id);

              return (
                <div
                  key={instance.instance_id}
                  className={`instance-item ${selectedInstanceId === instance.instance_id ? 'selected' : ''}`}
                  onClick={() => onInstanceSelect?.(instance)}
                >
                  <div className="instance-header">
                    <h4>{instance.app_name}</h4>
                    <code className="instance-id">{instance.instance_id}</code>
                  </div>

                  <div className="instance-path">
                    <small>{instance.config_path}</small>
                  </div>

                  {/* Tags */}
                  {instanceTags.length > 0 && (
                    <div className="instance-tags">
                      <span className="section-label">Tags:</span>
                      <div className="tag-badges">
                        {instanceTags.map(tag => (
                          <span
                            key={tag.id}
                            className="tag-badge"
                            style={{ backgroundColor: tag.color || '#6c8cff' }}
                            title={tag.description || ''}
                          >
                            {tag.name}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Labels */}
                  {instanceLabels.length > 0 && (
                    <div className="instance-labels">
                      <span className="section-label">Labels:</span>
                      <div className="label-badges">
                        {instanceLabels.map(label => (
                          <span
                            key={label.id}
                            className="label-badge"
                            style={{ backgroundColor: label.color || '#17c964' }}
                            title={label.description || ''}
                          >
                            {label.name}
                          </span>
                        ))}
                      </div>
                    </div>
                  )}

                  {/* Assignment Controls */}
                  <div className="assignment-controls">
                    <button
                      className="btn btn-sm btn-primary"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleAssignTag(instance.instance_id);
                      }}
                      title="Assign tag"
                    >
                      + Tag
                    </button>
                    <button
                      className="btn btn-sm btn-success"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleAssignLabel(instance.instance_id);
                      }}
                      title="Assign label"
                      disabled={labels.length === 0 || labels.every(label => 
                        instanceLabels.some(il => il.id === label.id)
                      )}
                    >
                      + Label
                    </button>
                  </div>

                  <div className="instance-meta">
                    <span className="key-count">
                      {instance.keys.length} key{instance.keys.length !== 1 ? 's' : ''}
                    </span>
                    <span className="discovered-date">
                      {new Date(instance.discovered_at).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              );
            })}
          </div>
        </div>
      )}

      {/* Assignment Modal */}
      <AssignmentModal
        isOpen={showAssignmentModal}
        onClose={() => setShowAssignmentModal(false)}
        onAssignmentComplete={handleAssignmentComplete}
        assignmentType={assignmentModalData.type}
        targetInstanceId={assignmentModalData.instanceId}
        targetModelId={assignmentModalData.modelId}
      />
    </div>
  );
}