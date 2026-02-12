export interface ParsedSkill {
  file_path: string;
  metadata: SkillMetadata;
  prompt_template: string;
}

export interface SkillMetadata {
  name: string;
  description: string;
  enabled: boolean;
  inputs: SkillInput[];
  outputs: SkillOutput[];
  tags: string[];
}

export interface SkillInput {
  name: string;
  label: string;
  input_type: string;
  required: boolean;
  default?: unknown;
  options?: string[];
  placeholder?: string;
}

export interface SkillOutput {
  name: string;
  label: string;
  output_type: string;
}
