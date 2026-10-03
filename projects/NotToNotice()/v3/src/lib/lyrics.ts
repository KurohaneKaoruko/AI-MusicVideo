// ---------------------------------------------------------------------------
// lyrics.ts - bilingual lyric table (timings from the published LRC; the zh
// pairing follows the official translation placement).
// ---------------------------------------------------------------------------

export type Lyric = { t: number; en: string; zh: string };

export const LYRICS: Lyric[] = [
  { t: 10.89, en: "Like Blue Fairy", zh: "如若蔚蓝色的妖姬啊" },
  { t: 13.61, en: "void Execute The Mission", zh: "推诿这无上的职责" },
  { t: 17.21, en: "the meaning of my existence", zh: "我存在的意义为何？" },
  { t: 23.84, en: "If EVE Become Human", zh: "于新夜化作人形" },
  { t: 30.66, en: "else suppressing my emotion", zh: "寒冰平息了" },
  { t: 33.18, en: "Create Soul For EVE", zh: "我于白夜躁动的创造之灵" },
  { t: 37.89, en: "I don't know why I can cry", zh: "我不知道，我为什么会哭泣" },
  { t: 44.36, en: "I don't know why I wish you happiness", zh: "我不知为何，要庇佑你幸福" },
  { t: 51.34, en: "I don't know why I can cry", zh: "我无从知晓啊，我为何要哭泣" },
  { t: 58.14, en: "Pretend Not To Notice", zh: "伪装作，毫不在意的样子" },
  { t: 87.62, en: "what do I dream of...?", zh: "我所向往之物为何？" },
  { t: 90.76, en: "void Achieve My Dream", zh: "缺乏逐梦的理想" },
  { t: 94.07, en: "until I get the fruit in the Eden", zh: "直到我取下，伊甸的禁果" },
  { t: 100.96, en: "while Be A Doll", zh: "哪怕做一具人偶也好" },
  { t: 104.47, en: "Focus On The Mission", zh: "坚守职责" },
  { t: 107.68, en: "cause you light my way of life", zh: "因为你照亮了我永生的路" },
  { t: 111.07, en: "Fight For You", zh: "誓死为你而战" },
  { t: 115.03, en: "I don't know why I can cry", zh: "我不知为何，我能够哭泣" },
  { t: 121.55, en: "I don't know why I want to be with you", zh: "我不知道为什么，我想伴你左右" },
  { t: 128.56, en: "I don't know why I can cry for you", zh: "我情不自禁地为你而恸哭" },
  { t: 135.73, en: "Pretend Not To Notice", zh: "却伪装作，毫不在意的模样" },
];

export type LyricState = {
  line: Lyric | null;
  age: number;       // seconds since line start
  hold: number;      // seconds the line stays
  p: number;         // 0..1 through the line
  idx: number;
};

export const lyricAt = (t: number): LyricState => {
  let idx = -1;
  for (let i = 0; i < LYRICS.length; i++) {
    if (t >= LYRICS[i].t) idx = i;
  }
  if (idx < 0) return { line: null, age: 0, hold: 0, p: 0, idx: -1 };
  const line = LYRICS[idx];
  const next = LYRICS[idx + 1];
  const end = next ? Math.min(next.t, line.t + 7.2) : Math.min(line.t + 7.2, 182.5);
  const hold = end - line.t;
  const age = t - line.t;
  return { line, age, hold, p: Math.min(1, age / (hold * 0.82)), idx };
};
