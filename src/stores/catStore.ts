import { create } from "zustand";
import type { CatColor, CatPersonality, CatProfile } from "../types/cat";

type CatState =
  | "idle"
  | "coding"
  | "celebrating"
  | "frustrated"
  | "sleeping"
  | "tired"
  | "interaction";

type CatMood = "happy" | "sad" | "sleeping" | "focused" | "excited";
export type CatEmotion = null | "surprised" | "excited" | "proud" | "bored" | "angry";

export interface SubCat {
  id: string;
  color: CatColor;
}

const ALL_COLORS: CatColor[] = ["white", "brown", "orange"];

export function getMaxCats(level: number, maxCompanions: number = 2): number {
  const levelMax = level < 10 ? 1 : level < 30 ? 2 : 3;
  // total cats = main + companions, capped by level
  return Math.min(1 + maxCompanions, levelMax);
}

interface CatStore {
  // State
  state: CatState;
  mood: CatMood;
  level: number;
  exp: number;
  expToNext: number;
  streakDays: number;
  catColor: CatColor;
  catPersonality: CatPersonality;

  // Activity
  activeIde: string | null;
  idleSeconds: number;

  // Today
  todayCodingMinutes: number;
  todayCommits: number;
  todayPomodoros: number;

  // Emotion
  emotion: CatEmotion;
  recentCommitCount: number;
  recentBuildFailCount: number;

  // Level up
  levelUp: number | null;

  // Pomodoro
  pomodoroActive: boolean;
  pomodoroPaused: boolean;
  pomodoroSeconds: number;
  pomodoroTotal: number;
  breakActive: boolean;
  breakSeconds: number;

  // Hats
  currentHat: string | null;
  unlockedHats: string[];

  // Sub cats
  subCats: SubCat[];

  // Actions
  setState: (state: string) => void;
  setLevel: (level: number, exp: number, expToNext: number) => void;
  setCatColor: (color: CatColor) => void;
  applyActiveProfile: (profile: CatProfile) => void;
  syncSubCats: (maxCompanions?: number) => void;
  setActiveIde: (ide: string | null) => void;
  setIdleSeconds: (seconds: number) => void;
  addCommit: () => void;
  addCodingMinute: () => void;
  addPomodoro: () => void;
  setEmotion: (emotion: CatEmotion) => void;
  clearEmotion: () => void;
  incrementCommitStreak: () => void;
  resetCommitStreak: () => void;
  incrementBuildFails: () => void;
  resetBuildFails: () => void;
  triggerLevelUp: (level: number) => void;
  clearLevelUp: () => void;
  startPomodoro: (totalSeconds: number) => void;
  pausePomodoro: () => void;
  resumePomodoro: () => void;
  stopPomodoro: () => void;
  tickPomodoro: () => void;
  startBreak: (totalSeconds: number) => void;
  stopBreak: () => void;
  tickBreak: () => void;
  setCurrentHat: (hat: string | null) => void;
  addUnlockedHat: (hat: string) => void;
}

const moodFromState = (state: CatState): CatMood => {
  const map: Record<CatState, CatMood> = {
    idle: "happy",
    coding: "focused",
    celebrating: "excited",
    frustrated: "sad",
    sleeping: "sleeping",
    tired: "sad",
    interaction: "happy",
  };
  return map[state] ?? "happy";
};

export const useCatStore = create<CatStore>((set) => ({
  // Initial state
  state: "idle",
  mood: "happy",
  level: 1,
  exp: 0,
  expToNext: 100,
  streakDays: 0,
  catColor: "brown",
  catPersonality: "classic",

  currentHat: null,
  unlockedHats: [],

  subCats: [],

  emotion: null,
  recentCommitCount: 0,
  recentBuildFailCount: 0,

  levelUp: null,

  activeIde: null,
  idleSeconds: 0,

  todayCodingMinutes: 0,
  todayCommits: 0,
  todayPomodoros: 0,

  pomodoroActive: false,
  pomodoroPaused: false,
  pomodoroSeconds: 0,
  pomodoroTotal: 0,
  breakActive: false,
  breakSeconds: 0,

  // Actions
  setState: (state) =>
    set({
      state: state as CatState,
      mood: moodFromState(state as CatState),
    }),

  setLevel: (level, exp, expToNext) =>
    set({ level, exp, expToNext }),

  setCatColor: (color) =>
    set({ catColor: color }),

  applyActiveProfile: (profile) =>
    set({
      catColor: profile.color,
      catPersonality: profile.personality,
    }),

  syncSubCats: (maxCompanions?: number) =>
    set((s) => {
      const companions = maxCompanions ?? 2;
      const maxSubs = getMaxCats(s.level, companions) - 1; // minus main cat
      if (maxSubs <= 0) return { subCats: [] };
      const available = ALL_COLORS.filter((c) => c !== s.catColor);
      const newSubs: SubCat[] = [];
      if (maxSubs >= 1) {
        const idx = Math.floor(Math.random() * available.length);
        newSubs.push({ id: "cat-sub-1", color: available[idx] });
      }
      if (maxSubs >= 2) {
        const remaining = available.filter((c) => c !== newSubs[0].color);
        newSubs.push({ id: "cat-sub-2", color: remaining[0] });
      }
      return { subCats: newSubs };
    }),

  setActiveIde: (ide) =>
    set({ activeIde: ide }),

  setIdleSeconds: (seconds) =>
    set({ idleSeconds: seconds }),

  addCommit: () =>
    set((s) => ({ todayCommits: s.todayCommits + 1 })),

  addCodingMinute: () =>
    set((s) => ({ todayCodingMinutes: s.todayCodingMinutes + 1 })),

  addPomodoro: () =>
    set((s) => ({ todayPomodoros: s.todayPomodoros + 1 })),

  setEmotion: (emotion) =>
    set({ emotion }),

  clearEmotion: () =>
    set({ emotion: null }),

  incrementCommitStreak: () =>
    set((s) => ({ recentCommitCount: s.recentCommitCount + 1 })),

  resetCommitStreak: () =>
    set({ recentCommitCount: 0 }),

  incrementBuildFails: () =>
    set((s) => ({ recentBuildFailCount: s.recentBuildFailCount + 1 })),

  resetBuildFails: () =>
    set({ recentBuildFailCount: 0 }),

  triggerLevelUp: (level) =>
    set({ levelUp: level }),

  clearLevelUp: () =>
    set({ levelUp: null }),

  startPomodoro: (totalSeconds) =>
    set({ pomodoroActive: true, pomodoroPaused: false, pomodoroSeconds: totalSeconds, pomodoroTotal: totalSeconds }),

  pausePomodoro: () =>
    set({ pomodoroPaused: true }),

  resumePomodoro: () =>
    set({ pomodoroPaused: false }),

  stopPomodoro: () =>
    set({ pomodoroActive: false, pomodoroPaused: false, pomodoroSeconds: 0, pomodoroTotal: 0 }),

  tickPomodoro: () =>
    set((s) => {
      if (!s.pomodoroActive || s.pomodoroPaused || s.pomodoroSeconds <= 0) return {};
      return { pomodoroSeconds: s.pomodoroSeconds - 1 };
    }),

  startBreak: (totalSeconds) =>
    set({ breakActive: true, breakSeconds: totalSeconds }),

  stopBreak: () =>
    set({ breakActive: false, breakSeconds: 0 }),

  tickBreak: () =>
    set((s) => {
      if (!s.breakActive || s.breakSeconds <= 0) return {};
      return { breakSeconds: s.breakSeconds - 1 };
    }),

  setCurrentHat: (hat) => set({ currentHat: hat }),
  addUnlockedHat: (hat) => set((s) => ({
    unlockedHats: s.unlockedHats.includes(hat) ? s.unlockedHats : [...s.unlockedHats, hat],
  })),
}));
