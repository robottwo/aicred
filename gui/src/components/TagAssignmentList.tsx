import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { TagAssignment, Tag, ProviderInstance } from '../types/tagging';

interface TagAssignmentWithDetails extends TagAssignment {
  tag?: Tag;
  instance?: ProviderInstance;
  model_name?: string;
}

interface TagAssignmentListProps {
  onAssignmentSelect?: (assignment: TagAssignmentWithDetails) => void;
  selectedAssignmentId?: string;
  instanceId?: string;
  modelId?: string;
}

export default function TagAssignmentList({ 
  onAssignmentSelect, 
  selectedAssignmentId,
  instanceId,
  modelId 
}: TagAssignmentListProps) {
  const [assignments, setAssignments] = useState<TagAssignmentWithDetails[]>([]);
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
      const assignmentsResult = await invoke<string>('list_tag_assignments');
      const assignmentsData = JSON.parse(assignmentsResult);
      
      // Load tags
      const tagsResult = await invoke<string>('list_tags');
      const tagsData = JSON.parse(tagsResult);
      
      // Load instances (this would need to be implemented in the backend)
      // For now, we'll create mock instance data
      const instancesData: ProviderInstance[] = [];
      
      // Combine assignments with tag and instance details
      const assignmentsWithDetails: TagAssignmentWithDetails[] = assignmentsData
        .filter((assignment: TagAssignment) => {
          if (instanceId && assignment.instance_id !== instanceId) return false;
          if (modelId && assignment.model_id !== modelId) return false;
          return true;
        })
        .map((assignment: TagAssignment) => {
          const tag = tagsData.find((t: Tag) => t.id === assignment.tag_id);
          const instance = instancesData.find((i: ProviderInstance) => i.instance_id === assignment.instance_id);
          
          return {
            ...assignment,
            tag,
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

  const handleUnassign = async (assignment: TagAssignmentWithDetails) => {
    if (!confirm(`Are you sure you want to unassign tag "${assignment.tag?.name}" from ${assignment.model_id ? `model ${assignment.model_id} in ` : ''}instance ${assignment.instance_id}?`)) {
      return;
    }

    try {
      setError(null);
      await invoke('unassign_tag', {
        tag_name: assignment.tag?.name,
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

  const getTargetDescription = (assignment: TagAssignmentWithDetails) => {
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
      <div className="tag-assignment-list">
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Loading tag assignments...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="tag-assignment-list">
      <div className="section-header">
        <h2>Tag Assignments</h2>
        <div className="filter-controls">
          <label htmlFor="assignment-filter">Filter:</label>
          <select
            id="assignment-filter"
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
            <p>No tag assignments found.</p>
            {instanceId && (
              <p>No tags assigned to {modelId ? `model ${modelId} in ` : ''}instance {instanceId}.</p>
            )}
          </div>
        ) : (
          <div className="assignments-table-container">
            <table className="assignments-table">
              <thead>
                <tr>
                  <th>Tag</th>
                  <th>Target</th>
                  <th>Type</th>
                  <th>Assigned Date</th>
                  <th>Actions</th>
                </tr>
              </thead>
              <tbody>
                {filteredAssignments.map((assignment) => (
                  <tr
                    key={assignment.id}
                    className={selectedAssignmentId === assignment.id ? 'selected' : ''}
                    onClick={() => onAssignmentSelect?.(assignment)}
                  >
                    <td>
                      <div className="tag-cell">
                        <div
                          className="tag-badge"
                          style={{ 
                            backgroundColor: assignment.tag?.color || '#6c8cff',
                            color: '#fff'
                          }}
                        >
                          {assignment.tag?.name || 'Unknown Tag'}
                        </div>
                        {assignment.tag?.description && (
                          <span className="tag-description">
                            {assignment.tag.description}
                          </span>
                        )}
                      </div>
                    </td>
                    <td>
                      <div className="target-cell">
                        <code className="target-id">{getTargetDescription(assignment)}</code>
                      </div>
                    </td>
                    <td>
                      <span className={`assignment-type ${assignment.model_id ? 'model' : 'instance'}`}>
                        {assignment.model_id ? 'Model' : 'Instance'}
                      </span>
                    </td>
                    <td>
                      <span className="assigned-date">
                        {formatDate(assignment.assigned_at)}
                      </span>
                    </td>
                    <td>
                      <button
                        className="btn btn-danger btn-sm"
                        onClick={(e) => {
                          e.stopPropagation();
                          handleUnassign(assignment);
                        }}
                        title="Unassign tag"
                      >
                        Unassign
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {assignments.length > 0 && (
        <div className="assignments-summary">
          <p>
            Showing {filteredAssignments.length} of {assignments.length} tag assignments
            {instanceId && ` for ${modelId ? `model ${modelId} in ` : ''}instance ${instanceId}`}
          </p>
        </div>
      )}
    </div>
  );
}