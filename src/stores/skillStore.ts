import { create } from "zustand";
import type { ParsedSkill } from "../types/skill";

interface SkillState {
  skills: ParsedSkill[];
  activeSkillPath: string | null;
  setSkills: (skills: ParsedSkill[]) => void;
  setActiveSkill: (path: string | null) => void;
}

export const useSkillStore = create<SkillState>((set) => ({
  skills: [],
  activeSkillPath: null,
  setSkills: (skills) => set({ skills }),
  setActiveSkill: (path) => set({ activeSkillPath: path }),
}));
