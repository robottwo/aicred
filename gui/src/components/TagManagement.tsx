import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Tag, TagFormData, TagWithAssignments } from '../types/tagging';

interface TagManagementProps {
  onTagSelect?: (tag: Tag) => void;
  selectedTagId?: string;
}

export default function TagManagement({ onTagSelect, selectedTagId }: TagManagementProps) {
  const [tags, setTags] = useState<TagWithAssignments[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showAddForm, setShowAddForm] = useState(false);
  const [editingTag, setEditingTag] = useState<Tag | null>(null);
  const [formData, setFormData] = useState<TagFormData>({
    name: '',
    description: '',
    color: '#6c8cff'
  });

  const predefinedColors = [
    '#6c8cff', '#17c964', '#f5a524', '#ff6b6b', '#9b6cff',
    '#00d4aa', '#ff8c42', '#ff6b9d', '#4ecdc4', '#45b7d1'
  ];

  useEffect(() => {
    loadTags();
  }, []);

  const loadTags = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      // Load tags and assignments
      const tagsResult = await invoke<string>('list_tags');
      const tagsData = JSON.parse(tagsResult);
      
      const assignmentsResult = await invoke<string>('list_tag_assignments');
      const assignmentsData = JSON.parse(assignmentsResult);
      
      // Combine tags with assignment information
      const tagsWithAssignments: TagWithAssignments[] = tagsData.map((tag: Tag) => {
        const assignmentCount = assignmentsData.filter(
          (assignment: any) => assignment.tag_id === tag.id
        ).length;
        
        return {
          ...tag,
          assignment_count: assignmentCount,
          is_assigned: assignmentCount > 0
        };
      });
      
      setTags(tagsWithAssignments);
    } catch (err) {
      setError(err as string);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!formData.name.trim()) {
      setError('Tag name is required');
      return;
    }

    try {
      setError(null);
      
      if (editingTag) {
        // Update existing tag
        await invoke('update_tag', {
          name: editingTag.name,
          color: formData.color,
          description: formData.description || null
        });
      } else {
        // Add new tag
        await invoke('add_tag', {
          name: formData.name,
          color: formData.color,
          description: formData.description || null
        });
      }
      
      await loadTags();
      resetForm();
    } catch (err) {
      setError(err as string);
    }
  };

  const handleEdit = (tag: Tag) => {
    setEditingTag(tag);
    setFormData({
      name: tag.name,
      description: tag.description || '',
      color: tag.color || '#6c8cff'
    });
    setShowAddForm(true);
  };

  const handleDelete = async (tag: Tag) => {
    if (!confirm(`Are you sure you want to delete tag "${tag.name}"?`)) {
      return;
    }

    try {
      setError(null);
      await invoke('remove_tag', { 
        name: tag.name,
        force: true 
      });
      await loadTags();
    } catch (err) {
      setError(err as string);
    }
  };

  const resetForm = () => {
    setFormData({
      name: '',
      description: '',
      color: '#6c8cff'
    });
    setEditingTag(null);
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

  if (isLoading) {
    return (
      <div className="tag-management">
        <div className="loading-spinner">
          <div className="spinner"></div>
          <p>Loading tags...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="tag-management">
      <div className="section-header">
        <h2>Tag Management</h2>
        <button 
          className="btn btn-primary"
          onClick={() => setShowAddForm(!showAddForm)}
        >
          {showAddForm ? 'Cancel' : '+ Add Tag'}
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
          <h3>{editingTag ? 'Edit Tag' : 'Add New Tag'}</h3>
          <form onSubmit={handleSubmit} className="tag-form">
            <div className="form-group">
              <label htmlFor="tag-name">Name *</label>
              <input
                id="tag-name"
                type="text"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                placeholder="Enter tag name"
                disabled={!!editingTag}
                required
              />
            </div>

            <div className="form-group">
              <label htmlFor="tag-description">Description</label>
              <textarea
                id="tag-description"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                placeholder="Enter tag description"
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
                {editingTag ? 'Update Tag' : 'Add Tag'}
              </button>
              <button type="button" className="btn btn-secondary" onClick={resetForm}>
                Cancel
              </button>
            </div>
          </form>
        </div>
      )}

      <div className="tags-list">
        {tags.length === 0 ? (
          <div className="empty-state">
            <p>No tags configured yet.</p>
            <p>Click "Add Tag" to create your first tag.</p>
          </div>
        ) : (
          <div className="tags-grid">
            {tags.map((tag) => (
              <div
                key={tag.id}
                className={`tag-card ${selectedTagId === tag.id ? 'selected' : ''}`}
                onClick={() => onTagSelect?.(tag)}
              >
                <div className="tag-header">
                  <div className="tag-info">
                    <h4>{tag.name}</h4>
                    <span className="tag-id">{tag.id}</span>
                  </div>
                  <div className="tag-actions">
                    <button
                      className="btn-icon"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleEdit(tag);
                      }}
                      title="Edit tag"
                    >
                      ✏️
                    </button>
                    <button
                      className="btn-icon danger"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleDelete(tag);
                      }}
                      title="Delete tag"
                    >
                      🗑️
                    </button>
                  </div>
                </div>

                {tag.description && (
                  <p className="tag-description">{tag.description}</p>
                )}

                <div className="tag-footer">
                  <div className="tag-color">
                    <div
                      className="color-indicator"
                      style={{ backgroundColor: tag.color || '#6c8cff' }}
                    />
                    <span>{tag.color || '#6c8cff'}</span>
                  </div>
                  <div className="tag-meta">
                    <span className="assignment-count">
                      {tag.assignment_count} assignment{tag.assignment_count !== 1 ? 's' : ''}
                    </span>
                    <span className="created-date">
                      {formatDate(tag.created_at)}
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