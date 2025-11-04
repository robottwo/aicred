import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Label, LabelFormData, LabelWithAssignments } from '../types/tagging';

interface LabelManagementProps {
  onLabelSelect?: (label: Label) => void;
  selectedLabelId?: string;
}

export default function LabelManagement({ onLabelSelect, selectedLabelId }: LabelManagementProps) {
  const [labels, setLabels] = useState<LabelWithAssignments[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingLabel, setEditingLabel] = useState<Label | null>(null);
  const [formData, setFormData] = useState<LabelFormData>({
    name: '',
    description: '',
    color: '#17c964'
  });

  const predefinedColors = [
    '#17c964', '#6c8cff', '#f5a524', '#ff6b6b', '#9b6cff',
    '#00d4aa', '#ff8c42', '#ff6b9d', '#4ecdc4', '#45b7d1'
  ];

  useEffect(() => {
    loadLabels();
  }, []);

  const loadLabels = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      // Load labels and assignments
      const labelsResult = await invoke<string>('list_labels');
      const labelsData = JSON.parse(labelsResult);
      
      const assignmentsResult = await invoke<string>('list_label_assignments');
      const assignmentsData = JSON.parse(assignmentsResult);
      
      // Combine labels with assignment information
      const labelsWithAssignments: LabelWithAssignments[] = labelsData.map((label: Label) => {
        const assignment = assignmentsData.find(
          (assignment: any) => assignment.label_id === label.id
        );
        
        return {
          ...label,
          is_assigned: !!assignment,
          assignment_target: assignment ? {
            instance_id: assignment.instance_id,
            model_id: assignment.model_id
          } : undefined
        };
      });
      
      setLabels(labelsWithAssignments);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      setError('Label name is required');
      return;
    }

    try {
      setError(null);
      
      if (editingLabel) {
        // Update existing label
        await invoke('update_label', {
          name: editingLabel.name,
          color: formData.color,
          description: formData.description || null
        });
      } else {
        // Add new label
        await invoke('add_label', {
          name: formData.name,
          color: formData.color,
          description: formData.description || null
        });
      }
      
      await loadLabels();
      resetForm();
    } catch (err) {
      setError(err as string);
    }
  };

  const handleEdit = (label: Label) => {
    setEditingLabel(label);
    setFormData({
      name: label.name,
      description: label.description || '',
      color: label.color || '#17c964'
    });
    setShowAddForm(true);
  };

  const handleDelete = async (label: Label) => {
    const labelWithAssignments = labels.find(l => l.id === label.id);
    
    if (labelWithAssignments?.is_assigned) {
      setError(`Cannot delete label "${label.name}" because it is currently assigned. Please unassign it first.`);
      return;
    }

    if (!confirm(`Are you sure you want to delete label "${label.name}"?`)) {
      return;
    }

    try {
      setError(null);
      await invoke('remove_label', { 
        name: label.name,
        force: true 
      });
      await loadLabels();
    } catch (err) {
      setError(err as string);
    }
  };

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      color: '#17c964'
    });
    setEditingLabel(null);
    setShowAddForm(false);
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit'
    });
  };

  const getAssignmentStatus = (label: LabelWithAssignments) => {
    if (!label.is_assigned) {
      return <span className="status-badge unassigned">Unassigned</span>;
    }
    
    const target = label.assignment_target;
    if (target?.model_id) {
      return (
        <span className="status-badge assigned">
          Assigned to {target.instance_id}/{target.model_id}
        </span>
      );
    } else {
      return (
        <span className="status-badge assigned">
          Assigned to {target?.instance_id}
        </span>
      );
    }
  };

  if (isLoading) {
    return (
      <div className="label-management">
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Loading labels...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="label-management">
      <div className="section-header">
        <h2>Label Management</h2>
        <button 
          className="btn btn-primary"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          {showAddForm ? 'Cancel' : '+ Add Label'}
        </button>
      </div>

      {error && (
        <div className="error-message">
          <h4>Error</h4>
          <p>{error}</p>
        </div>
      )}

      {showAddForm && (
        <div className="form-container">
          <h3>{editingLabel ? 'Edit Label' : 'Add New Label'}</h3>
          <form onSubmit={handleSubmit} className="label-form">
            <div className="form-group">
              <label htmlFor="label-name">Name *</label>
              <input
                id="label-name"
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="Enter label name"
                disabled={!!editingLabel}
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="label-description">Description</label>
              <textarea
                id="label-description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Enter label description"
                rows={3}
              />
            </div>

            <div className="form-group">
              <label>Color</label>
              <div className="color-picker">
                {predefinedColors.map((color) => (
                  <button
                    key={color}
                    type="button"
                    className={`color-option ${formData.color === color ? 'selected' : ''}`}
                    style={{ backgroundColor: color }}
                    onClick={() => setFormData({ ...formData, color })}
                    title={color}
                  />
                ))}
                <input
                  type="color"
                  value={formData.color}
                  onChange={(e) => setFormData({ ...formData, color: e.target.value })}
                  className="custom-color-input"
                />
              </div>
            </div>

            <div className="form-actions">
              <button type="submit" className="btn btn-primary">
                {editingLabel ? 'Update Label' : 'Add Label'}
              </button>
              <button type="button" className="btn btn-secondary" onClick={resetForm}>
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="labels-list">
        {labels.length === 0 ? (
          <div className="empty-state">
            <p>No labels configured yet.</p>
            <p>Click "Add Label" to create your first label.</p>
          </div>
        ) : (
          <div className="labels-grid">
            {labels.map((label) => (
              <div
                key={label.id}
                className={`label-card ${selectedLabelId === label.id ? 'selected' : ''}`}
                onClick={() => onLabelSelect?.(label)}
              >
                <div className="label-header">
                  <div className="label-info">
                    <h4>{label.name}</h4>
                    <span className="label-id">{label.id}</span>
                  </div>
                  <div className="label-actions">
                    <button
                      className="btn-icon"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEdit(label);
                      }}
                      title="Edit label"
                    >
                      ✏️
                    </button>
                    <button
                      className={`btn-icon ${label.is_assigned ? 'disabled' : 'danger'}`}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(label);
                      }}
                      title={label.is_assigned ? 'Cannot delete assigned label' : 'Delete label'}
                      disabled={label.is_assigned}
                    >
                      🗑️
                    </button>
                  </div>
                </div>

                {label.description && (
                  <p className="label-description">{label.description}</p>
                )}

                <div className="label-status">
                  {getAssignmentStatus(label)}
                </div>

                <div className="label-footer">
                  <div className="label-color">
                    <div
                      className="color-indicator"
                      style={{ backgroundColor: label.color || '#17c964' }}
                    />
                    <span>{label.color || '#17c964'}</span>
                  </div>
                  <div className="label-meta">
                    <span className="created-date">
                      {formatDate(label.created_at)}
                    </span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}