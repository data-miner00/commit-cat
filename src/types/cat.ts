export type CatColor = "white" | "brown" | "orange";
export type CatPersonality = "classic" | "chill" | "tsundere" | "chaotic";

export interface CatProfile {
  id: string;
  name: string;
  color: CatColor;
  personality: CatPersonality;
}

export interface CatProfilesResponse {
  profiles: CatProfile[];
  activeProfileId: string;
}

export interface ActiveCatProfileResponse {
  activeProfile: CatProfile;
}

export const CAT_COLORS: CatColor[] = ["brown", "orange", "white"];

export const CAT_PERSONALITY_OPTIONS: { id: CatPersonality; label: string; description: string }[] = [
  { id: "classic", label: "Classic", description: "Balanced reactions and the original CommitCat vibe." },
  { id: "chill", label: "Chill", description: "Sleepier movement and softer, reassuring lines." },
  { id: "tsundere", label: "Tsundere", description: "Sharper tone, less mushy affection, still secretly supportive." },
  { id: "chaotic", label: "Chaotic", description: "Restless pacing, louder reactions, and more playful energy." },
];
