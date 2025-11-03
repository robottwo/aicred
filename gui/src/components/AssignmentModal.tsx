import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Tag, Label, ProviderInstance, AssignmentFormData } from '../types/tagging';

interface AssignmentModalProps {
  isOpen: boolean;
  onClose: () => void;
  onAssignmentComplete: () => void;
  assignmentType: 'tag' | 'label';
  targetInstanceId?: string;
  targetModelId?: string;
}

export default function AssignmentModal({
  isOpen,
  onClose,
  onAssignmentComplete,
  assignmentType,
  targetInstanceId,
  targetModelId
}: AssignmentModalProps) {
  const [tags, setTags] = useState<Tag[]>([]);
  const [labels, setLabels] = useState<Label[]>([]);
  const [labelAssignments, setLabelAssignments] = useState<any[]>([]);
  const [instances, setInstances] = useState<ProviderInstance[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [formData, setFormData] = useState<AssignmentFormData>({
    tag_id: undefined,
    label_id: undefined,
    instance_id: targetInstanceId || '',
    model_id: targetModelId
  });

  useEffect(() => {
    if (isOpen) {
      loadData();
    }
  }, [isOpen]);

  useEffect(() => {
    // Reset form when target changes
    setFormData({
      tag_id: undefined,
      label_id: undefined,
      instance_id: targetInstanceId || '',
      model_id: targetModelId
    });
  }, [targetInstanceId, targetModelId]);

  const loadData = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Load tags
      const tagsResult = await invoke<string>('list_tags');
      const tagsData = JSON.parse(tagsResult);
      setTags(tagsData);

      // Load labels
      const labelsResult = await invoke<string>('list_labels');
      const labelsData = JSON.parse(labelsResult);
      setLabels(labelsData);

      // Load label assignments to check which labels are already assigned
      const labelAssignmentsResult = await invoke<string>('list_label_assignments');
      const labelAssignmentsData = JSON.parse(labelAssignmentsResult);
      setLabelAssignments(labelAssignmentsData);

      // Load instances (this would need to be implemented in the backend)
      // For now, we'll create mock instance data
      const mockInstances: ProviderInstance[] = [
        {
          instance_id: 'claude-desktop-1',
          app_name: 'Claude Desktop',
          config_path: '~/.config/claude/claude_desktop_config.json',
          discovered_at: new Date().toISOString(),
          keys: []
        },
        {
          instance_id: 'roo-code-1',
          app_name: 'Roo Code',
          config_path: '~/.vscode/extensions/rooveterinary.roo-code-*/package.json',
          discovered_at: new Date().toISOString(),
          keys: []
        }
      ];
      setInstances(mockInstances);

    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.instance_id) {
      setError('Instance ID is required');
      return;
    }

    if (assignmentType === 'tag' && !formData.tag_id) {
      setError('Please select a tag');
      return;
    }

    if (assignmentType === 'label' && !formData.label_id) {
      setError('Please select a label');
      return;
    }

    try {
      setError(null);
      setIsLoading(true);

      if (assignmentType === 'tag') {
        const tag = tags.find(t => t.id === formData.tag_id);
        await invoke('assign_tag', {
          tag_name: tag?.name,
          instance_id: formData.instance_id,
          model_id: formData.model_id
        });
      } else {
        const label = labels.find(l => l.id === formData.label_id);
        await invoke('assign_label', {
          label_name: label?.name,
          instance_id: formData.instance_id,
          model_id: formData.model_id
        });
      }

      onAssignmentComplete();
      onClose();
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    setError(null);
    setFormData({
      tag_id: undefined,
      label_id: undefined,
      instance_id: targetInstanceId || '',
      model_id: targetModelId
    });
    onClose();
  };

  const getAvailableItems = () => {
    if (assignmentType === 'tag') {
      return tags;
    } else {
      // Filter out labels that are already assigned
      const assignedLabelIds = labelAssignments.map((assignment: any) => assignment.label_id);
      return labels.filter(label => !assignedLabelIds.includes(label.id));
    }
  };

  const getSelectedItem = () => {
    if (assignmentType === 'tag') {
      return tags.find(t => t.id === formData.tag_id);
    } else {
      return labels.find(l => l.id === formData.label_id);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="modal-overlay" onClick={handleClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>
            Assign {assignmentType === 'tag' ? 'Tag' : 'Label'}
            {targetModelId && ` to ${targetModelId}`}
            {targetInstanceId && ` in ${targetInstanceId}`}
          </h2>
          <button className="modal-close" onClick={handleClose}>
            ×
          </button>
        </div>

        {error && (
          <div className="error-message">
            <h4>Error</h4>
            <p>{error}</p>
          </div>
        )}

        <form onSubmit={handleSubmit} className="assignment-form">
          <div className="form-group">
            <label htmlFor="assignment-item">
              {assignmentType === 'tag' ? 'Tag' : 'Label'} *
            </label>
            <select
              id="assignment-item"
              value={assignmentType === 'tag' ? formData.tag_id : formData.label_id}
              onChange={(e) => {
                if (assignmentType === 'tag') {
                  setFormData({ ...formData, tag_id: e.target.value });
                } else {
                  setFormData({ ...formData, label_id: e.target.value });
                }
              }}
              required
            >
              <option value="">
                Select {assignmentType === 'tag' ? 'a tag' : 'a label'}
              </option>
              {getAvailableItems().map((item) => (
                <option key={item.id} value={item.id}>
                  {item.name}
                  {item.description && ` - ${item.description}`}
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="instance-id">Instance ID *</label>
            <select
              id="instance-id"
              value={formData.instance_id}
              onChange={(e) => setFormData({ ...formData, instance_id: e.target.value })}
              required
            >
              <option value="">Select an instance</option>
              {instances.map((instance) => (
                <option key={instance.instance_id} value={instance.instance_id}>
                  {instance.instance_id} ({instance.app_name})
                </option>
              ))}
            </select>
          </div>

          <div className="form-group">
            <label htmlFor="model-id">Model ID (optional)</label>
            <input
              id="model-id"
              type="text"
              value={formData.model_id || ''}
              onChange={(e) => setFormData({ ...formData, model_id: e.target.value || undefined })}
              placeholder="Enter model ID if applicable"
            />
          </div>

          {getSelectedItem() && (
            <div className="selected-item-preview">
              <h4>Selected {assignmentType === 'tag' ? 'Tag' : 'Label'}</h4>
              <div className="item-preview">
                <div
                  className="preview-badge"
                  style={{
                    backgroundColor: getSelectedItem()?.color || (assignmentType === 'tag' ? '#6c8cff' : '#17c964'),
                    color: '#fff'
                  }}
                >
                  {getSelectedItem()?.name}
                </div>
                {getSelectedItem()?.description && (
                  <p className="item-description">{getSelectedItem()?.description}</p>
                )}
                <div className="item-meta">
                  <span>Color: {getSelectedItem()?.color}</span>
                  <span>ID: {getSelectedItem()?.id}</span>
                </div>
              </div>
            </div>
          )}

          <div className="form-actions">
            <button
              type="button"
              className="btn btn-secondary"
              onClick={handleClose}
              disabled={isLoading}
            >
              Cancel
            </button>
            <button
              type="submit"
              className="btn btn-primary"
              disabled={isLoading || !formData.instance_id || 
                (assignmentType === 'tag' ? !formData.tag_id : !formData.label_id)}
            >
              {isLoading ? 'Assigning...' : `Assign ${assignmentType === 'tag' ? 'Tag' : 'Label'}`}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}