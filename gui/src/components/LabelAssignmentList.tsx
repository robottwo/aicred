import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { LabelAssignment, Label, ProviderInstance } from '../types/tagging';

interface LabelAssignmentWithDetails extends LabelAssignment {
  label?: Label;
  instance?: ProviderInstance;
  model_name?: string;
}

interface LabelAssignmentListProps {
  onAssignmentSelect?: (assignment: LabelAssignmentWithDetails) => void;
  selectedAssignmentId?: string;
  instanceId?: string;
  modelId?: string;
}

export default function LabelAssignmentList({ 
  onAssignmentSelect, 
  selectedAssignmentId,
  instanceId,
  modelId 
}: LabelAssignmentListProps) {
  const [assignments, setAssignments] = useState<LabelAssignmentWithDetails[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<'all' | 'instance' | 'model'>('all');

  useEffect(() => {
    loadAssignments();
  }, [instanceId, modelId]);

  const loadAssignments = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      // Load assignments
      const assignmentsResult = await invoke<string>('list_label_assignments');
      const assignmentsData = JSON.parse(assignmentsResult);
      
      // Load labels
      const labelsResult = await invoke<string>('list_labels');
      const labelsData = JSON.parse(labelsResult);
      
      // Load instances (this would need to be implemented in the backend)
      // For now, we'll create mock instance data
      const instancesData: ProviderInstance[] = [];
      
      // Combine assignments with label and instance details
      const assignmentsWithDetails: LabelAssignmentWithDetails[] = assignmentsData
        .filter((assignment: LabelAssignment) => {
          if (instanceId && assignment.instance_id !== instanceId) return false;
          if (modelId && assignment.model_id !== modelId) return false;
          return true;
        })
        .map((assignment: LabelAssignment) => {
          const label = labelsData.find((l: Label) => l.id === assignment.label_id);
          const instance = instancesData.find((i: ProviderInstance) => i.instance_id === assignment.instance_id);
          
          return {
            ...assignment,
            label,
            instance,
            model_name: assignment.model_id
          };
        });
      
      setAssignments(assignmentsWithDetails);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnassign = async (assignment: LabelAssignmentWithDetails) => {
    if (!confirm(`Are you sure you want to unassign label "${assignment.label?.name}" from ${assignment.model_id ? `model ${assignment.model_id} in ` : ''}instance ${assignment.instance_id}?`)) {
      return;
    }

    try {
      setError(null);
      await invoke('unassign_label', {
        label_name: assignment.label?.name,
        instance_id: assignment.instance_id,
        model_id: assignment.model_id
      });
      
      await loadAssignments();
    } catch (err) {
      setError(err as string);
    }
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

  const getTargetDescription = (assignment: LabelAssignmentWithDetails) => {
    if (assignment.model_id) {
      return `${assignment.instance_id}/${assignment.model_id}`;
    }
    return assignment.instance_id;
  };

  const filteredAssignments = assignments.filter(assignment => {
    if (filter === 'instance') return !assignment.model_id;
    if (filter === 'model') return !!assignment.model_id;
    return true;
  });

  if (isLoading) {
    return (
      <div className="label-assignment-list">
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Loading label assignments...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="label-assignment-list">
      <div className="section-header">
        <h2>Label Assignments</h2>
        <div className="filter-controls">
          <label htmlFor="label-assignment-filter">Filter:</label>
          <select
            id="label-assignment-filter"
            value={filter}
            onChange={(e) => setFilter(e.target.value as 'all' | 'instance' | 'model')}
          >
            <option value="all">All Assignments</option>
            <option value="instance">Instance Only</option>
            <option value="model">Instance/Model</option>
          </select>
        </div>
      </div>

      {error && (
        <div className="error-message">
          <h4>Error</h4>
          <p>{error}</p>
        </div>
      )}

      <div className="assignments-list">
        {filteredAssignments.length === 0 ? (
          <div className="empty-state">
            <p>No label assignments found.</p>
            {instanceId && (
              <p>No labels assigned to {modelId ? `model ${modelId} in ` : ''}instance {instanceId}.</p>
            )}
          </div>
        ) : (
          <div className="assignments-grid">
            {filteredAssignments.map((assignment) => (
              <div
                key={assignment.id}
                className={`assignment-card ${selectedAssignmentId === assignment.id ? 'selected' : ''}`}
                onClick={() => onAssignmentSelect?.(assignment)}
              >
                <div className="assignment-header">
                  <div className="label-cell">
                    <div
                      className="label-badge"
                      style={{ 
                        backgroundColor: assignment.label?.color || '#17c964',
                        color: '#fff'
                      }}
                    >
                      {assignment.label?.name || 'Unknown Label'}
                    </div>
                    {assignment.label?.description && (
                      <p className="label-description">
                        {assignment.label.description}
                      </p>
                    )}
                  </div>
                  <button
                    className="btn btn-danger btn-sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      handleUnassign(assignment);
                    }}
                    title="Unassign label"
                  >
                    Unassign
                  </button>
                </div>

                <div className="assignment-details">
                  <div className="target-info">
                    <span className="target-label">Target:</span>
                    <code className="target-id">{getTargetDescription(assignment)}</code>
                  </div>
                  
                  <div className="assignment-meta">
                    <span className={`assignment-type ${assignment.model_id ? 'model' : 'instance'}`}>
                      {assignment.model_id ? 'Model Assignment' : 'Instance Assignment'}
                    </span>
                    <span className="assigned-date">
                      {formatDate(assignment.assigned_at)}
                    </span>
                  </div>
                </div>

                <div className="assignment-footer">
                  <div className="label-color-info">
                    <div
                      className="color-indicator"
                      style={{ backgroundColor: assignment.label?.color || '#17c964' }}
                    />
                    <span>Color: {assignment.label?.color || '#17c964'}</span>
                  </div>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {assignments.length > 0 && (
        <div className="assignments-summary">
          <p>
            Showing {filteredAssignments.length} of {assignments.length} label assignments
            {instanceId && ` for ${modelId ? `model ${modelId} in ` : ''}instance ${instanceId}`}
          </p>
          <div className="assignment-stats">
            <span className="stat">
              Instance assignments: {assignments.filter(a => !a.model_id).length}
            </span>
            <span className="stat">
              Model assignments: {assignments.filter(a => a.model_id).length}
            </span>
          </div>
        </div>
      )}
    </div>
  );
}