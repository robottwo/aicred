// TypeScript interfaces for tagging and labeling system

export interface Tag {
  id: string;
  name: string;
  description?: string;
  color?: string;
  created_at: string;
}

export interface Label {
  id: string;
  name: string;
  description?: string;
  color?: string;
  created_at: string;
}

export interface TagAssignment {
  id: string;
  tag_id: string;
  instance_id: string;
  model_id?: string;
  assigned_at: string;
}

export interface LabelAssignment {
  id: string;
  label_id: string;
  instance_id: string;
  model_id?: string;
  assigned_at: string;
}

export interface AssignmentTarget {
  instance_id: string;
  model_id?: string;
}

export interface TagWithAssignments extends Tag {
  assignment_count: number;
  is_assigned: boolean;
}

export interface LabelWithAssignments extends Label {
  is_assigned: boolean;
  assignment_target?: AssignmentTarget;
}

export interface ProviderInstance {
  instance_id: string;
  app_name: string;
  config_path: string;
  discovered_at: string;
  keys: DiscoveredKey[];
  tags?: Tag[];
  labels?: Label[];
}

export interface DiscoveredKey {
  provider: string;
  source: string;
  value_type: string;
  confidence: number;
  redacted: string;
  hash: string;
}

export interface ScanResult {
  keys: DiscoveredKey[];
  config_instances: ProviderInstance[];
  home_dir: string;
  scanned_at: string;
  providers_scanned: string[];
}

export interface TagFormData {
  name: string;
  description?: string;
  color?: string;
}

export interface LabelFormData {
  name: string;
  description?: string;
  color?: string;
}

export interface AssignmentFormData {
  tag_id?: string;
  label_id?: string;
  instance_id: string;
  model_id?: string;
}