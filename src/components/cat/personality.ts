import type { CatEmotion } from "../../stores/catStore";
import type { CatPersonality } from "../../types/cat";

export type CatBehavior = "walk" | "stand" | "sit" | "sleep";

type MessageGroup =
  | "normal"
  | "happy"
  | "love"
  | "annoyed"
  | "coding"
  | "sleepAnnoyed";

interface PersonalityMessages {
  normal: string[];
  happy: string[];
  love: string[];
  annoyed: string[];
  coding: string[];
  sleepAnnoyed: string[];
  emotions: Record<Exclude<CatEmotion, null>, string[]>;
}

interface BehaviorRule {
  duration: [number, number];
  weightedNext: Array<{ behavior: CatBehavior; weight: number }>;
}

const PERSONALITY_MESSAGES: Record<CatPersonality, PersonalityMessages> = {
  classic: {
    normal: ["hey there~ 😺", "what's up? 🐾", "hi hi~ 💛", "oh, hello! 😸", "noticed me? 👀"],
    happy: ["that feels nice~ 💛", "more more! 😻", "you're the best 🥰", "hehe~ 😸"],
    love: ["I love you so much~ 💕", "you're my favorite human 💗", "never stop... 🥺💛", "purrrrrr~ 💞"],
    annoyed: ["okay I get it 😑", "a bit much... 🙄", "I was napping! 😾", "not now please 😤"],
    coding: [
      "you're on a roll today 🔥",
      "ooh that's clean code 👀",
      "don't forget to save 💾",
      "hydration check! 💧",
      "nice focus session 💪",
      "you've been at it, take a breather? ☕",
      "this is coming together nicely ✨",
      "smooth typing today 🎹",
      "I believe in you 💛",
      "commit when you're ready 📦",
    ],
    sleepAnnoyed: ["five more minutes... 😴", "shh I'm dreaming 💤", "come back later... 🌙", "not now... 😾"],
    emotions: {
      surprised: ["whoa! what happened?! 😱", "oh no! 💥", "huh?! 😳", "that was unexpected! 😮"],
      excited: ["you're on fire!! 🔥🔥", "commit streak! 🚀", "unstoppable! ⚡", "keep going!! 💪✨"],
      proud: ["we did it! 🏆", "so proud of us! 🌟", "amazing work! 👑", "look at that! ✨"],
      bored: ["so bored... 😴", "anyone there? 🥱", "I miss coding... 💤", "come back soon~ 🐾"],
      angry: ["not again!! 😡", "fix the bugs! 🔥", "grr... 💢", "this is frustrating! 😤"],
    },
  },
  chill: {
    normal: ["hey... taking it easy? 🌿", "hi friend~ ☁️", "I'll hang out right here 🤍", "easy does it~ 🍵"],
    happy: ["mm, that's nice~ 🌼", "you're doing great, slowly and surely 🍃", "this feels cozy 😌", "hehe, comfy vibes~ ☁️"],
    love: ["I could stay like this forever 💛", "you make this desk feel safe 🤍", "you're my calm place 🫶", "purrr... don't rush 💤"],
    annoyed: ["shhh... indoor voice 😪", "can we be gentle? 🫠", "I was resting... 😴", "too much chaos for me 😵‍💫"],
    coding: [
      "steady progress is still progress 🌿",
      "clean code, calm mind 🍵",
      "one step at a time 💻",
      "breathe, then ship it ☁️",
      "you're locked in nicely 🤍",
      "soft focus still counts ✨",
    ],
    sleepAnnoyed: ["mm... later please 😴", "still snoozing... 💤", "let's revisit after my nap 🌙", "quiet paws only 😪"],
    emotions: {
      surprised: ["oh! that woke me up 😳", "whoops, that was sudden 😮", "didn't see that coming 👀", "okay, that was loud 😵"],
      excited: ["oooh, nice momentum ✨", "that felt smooth 🌟", "love this pace 💛", "keep the good flow going 🌿"],
      proud: ["look at you... so solid 🏆", "that was really well done 🌟", "I'm proud of this one 🤍", "quietly iconic ✨"],
      bored: ["it's very quiet... 💤", "I could nap forever like this 😴", "a tiny bit sleepy over here ☁️", "come back when you're ready 🌿"],
      angry: ["deep breath... and retry 😤", "okay, that one stung 😣", "let's not let it spiral 🔧", "annoying, but fixable 💢"],
    },
  },
  tsundere: {
    normal: ["oh, it's you. finally 🙄", "I wasn't waiting for you or anything 😼", "hmph. say what you need 😑", "don't get the wrong idea, okay? 💢"],
    happy: ["that was... acceptable 😳", "you did okay, I guess 😼", "not terrible at all 😌", "fine, I liked that a little 💛"],
    love: ["I-it's not like I missed you... 💗", "don't make me say cute things 😳", "you're annoyingly important to me 💛", "just stay here a bit longer... okay? 🥺"],
    annoyed: ["seriously? again? 😾", "quit poking me, dummy 💢", "I was busy glaring elegantly 😤", "you're on thin ice, human 🙄"],
    coding: [
      "don't mess up the commit message 😼",
      "that refactor was decent, I guess ✨",
      "try not to break prod, okay? 💢",
      "hm. your code is looking sharper 👀",
      "I noticed the progress. not impressed... much 😌",
      "just ship it already 📦",
    ],
    sleepAnnoyed: ["I'm sleeping, idiot 😴", "go away... just for five minutes 💤", "don't stare while I nap 💢", "I said later... 😾"],
    emotions: {
      surprised: ["w-what was that?! 😳", "excuse me?! 💥", "that was rude 😠", "seriously?! 😮"],
      excited: ["not bad at all! 🚀", "huh... you're actually cooking 🔥", "fine, that was awesome ⚡", "keep it up, if you can 💪"],
      proud: ["of course we nailed it 👑", "I knew you'd pull through... probably 🏆", "pretty impressive, not that I care 🌟", "see? with me around, you win ✨"],
      bored: ["this is painfully dull 😴", "hello? productivity? 🥱", "I'm bored out of my whiskers 💢", "do something interesting already 💤"],
      angry: ["unbelievable!! 😡", "I can't believe you broke it again 💢", "fix it. now. 🔥", "this build is insulting 😤"],
    },
  },
  chaotic: {
    normal: ["HI HI HI!! ⚡", "what are we doing?! let's do it fast! 🚀", "I have ideas and none are safe 😼", "desk goblin mode activated 👀"],
    happy: ["YESSS more of that!! 🎉", "that scratched my brain perfectly ⚡", "we are SO back 🔥", "hehehehe excellent chaos 😸"],
    love: ["I would commit crimes for you 💖", "you're my favorite menace 💛", "we're unstoppable gremlins together 💞", "I love this weird little life 🥺✨"],
    annoyed: ["boring!! rude!! unacceptable!! 😤", "I demand better entertainment 💢", "this nap interruption will be remembered 😾", "too slow!! too pokey!! 🙄"],
    coding: [
      "ship it, tweak it, loop it, go go go! 🚀",
      "this code has SPARKS today ⚡",
      "we should make one more tiny change... or twelve 👀",
      "your fingers are flying and I respect that 🔥",
      "momentum!! delicious momentum!! 🎹",
      "this branch is one bad idea away from greatness 😼",
    ],
    sleepAnnoyed: ["I was in the middle of a weird dream!! 😴", "nap sabotage detected!! 💢", "who disturbs the chaos goblin?! 💤", "illegal wake-up!! 😾"],
    emotions: {
      surprised: ["WHAT JUST HAPPENED?! 😱", "plot twist!! 💥", "oh wow okay!! 😳", "that escalated immediately 😮"],
      excited: ["WE'RE FLYING NOW!! 🔥🔥", "ABSOLUTE CINEMA 🚀", "more more more!! ⚡", "momentum monster activated 💪✨"],
      proud: ["LOOK AT US GOOOO 👑", "legend behavior honestly 🏆", "we cooked and served 🌟", "this rules actually ✨"],
      bored: ["I need stimulation immediately 😴", "boring boring boring 🥱", "someone shake the timeline 💤", "where is the action?! 🐾"],
      angry: ["THIS AGAIN?! 😡", "fight the bugs!! 🔥", "I reject this failure state 💢", "rage compile!! 😤"],
    },
  },
};

const BEHAVIOR_RULES: Record<CatPersonality, Record<CatBehavior, BehaviorRule>> = {
  classic: {
    walk: { duration: [5000, 10000], weightedNext: [{ behavior: "stand", weight: 1 }] },
    stand: { duration: [2000, 5000], weightedNext: [{ behavior: "walk", weight: 1 }, { behavior: "sit", weight: 1 }] },
    sit: { duration: [2000, 8000], weightedNext: [{ behavior: "sleep", weight: 1 }, { behavior: "stand", weight: 1 }] },
    sleep: { duration: [10000, 15000], weightedNext: [{ behavior: "stand", weight: 1 }] },
  },
  chill: {
    walk: { duration: [5000, 9000], weightedNext: [{ behavior: "stand", weight: 2 }, { behavior: "sit", weight: 1 }] },
    stand: { duration: [2500, 5500], weightedNext: [{ behavior: "walk", weight: 1 }, { behavior: "sit", weight: 3 }] },
    sit: { duration: [3500, 9000], weightedNext: [{ behavior: "sleep", weight: 3 }, { behavior: "stand", weight: 1 }] },
    sleep: { duration: [12000, 18000], weightedNext: [{ behavior: "stand", weight: 1 }] },
  },
  tsundere: {
    walk: { duration: [4500, 8500], weightedNext: [{ behavior: "stand", weight: 2 }, { behavior: "sit", weight: 1 }] },
    stand: { duration: [2000, 4500], weightedNext: [{ behavior: "walk", weight: 1 }, { behavior: "sit", weight: 2 }] },
    sit: { duration: [2500, 7000], weightedNext: [{ behavior: "sleep", weight: 1 }, { behavior: "stand", weight: 2 }] },
    sleep: { duration: [9000, 14000], weightedNext: [{ behavior: "stand", weight: 1 }] },
  },
  chaotic: {
    walk: { duration: [3500, 6500], weightedNext: [{ behavior: "stand", weight: 3 }, { behavior: "sit", weight: 1 }] },
    stand: { duration: [1500, 3500], weightedNext: [{ behavior: "walk", weight: 3 }, { behavior: "sit", weight: 1 }] },
    sit: { duration: [1500, 5000], weightedNext: [{ behavior: "sleep", weight: 1 }, { behavior: "stand", weight: 2 }] },
    sleep: { duration: [7000, 11000], weightedNext: [{ behavior: "stand", weight: 1 }] },
  },
};

function pickWeightedBehavior(weightedNext: Array<{ behavior: CatBehavior; weight: number }>): CatBehavior {
  const totalWeight = weightedNext.reduce((sum, candidate) => sum + candidate.weight, 0);
  let roll = Math.random() * totalWeight;
  for (const candidate of weightedNext) {
    roll -= candidate.weight;
    if (roll <= 0) return candidate.behavior;
  }
  return weightedNext[weightedNext.length - 1].behavior;
}

function randomBetween([min, max]: [number, number]): number {
  return min + Math.random() * (max - min);
}

export function getPersonalityMessages(personality: CatPersonality, group: MessageGroup): string[] {
  return PERSONALITY_MESSAGES[personality][group];
}

export function getEmotionMessages(personality: CatPersonality, emotion: Exclude<CatEmotion, null>): string[] {
  return PERSONALITY_MESSAGES[personality].emotions[emotion];
}

export function planNextBehavior(
  current: CatBehavior,
  personality: CatPersonality,
  recentlyWoke: boolean,
): { next: CatBehavior; duration: number } {
  if (current === "sit" && recentlyWoke) {
    return { next: "sleep", duration: randomBetween([1500, 2500]) };
  }
  const rule = BEHAVIOR_RULES[personality][current];
  return {
    next: pickWeightedBehavior(rule.weightedNext),
    duration: randomBetween(rule.duration),
  };
}

export function getCodingBubbleDelayRange(personality: CatPersonality): [number, number] {
  switch (personality) {
    case "chill":
      return [4, 10];
    case "tsundere":
      return [3, 8];
    case "chaotic":
      return [2, 6];
    case "classic":
    default:
      return [3, 10];
  }
}
