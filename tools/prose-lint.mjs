#!/usr/bin/env node

/**
 * prose-lint.mjs — Deterministic quality gate for scene prose.
 *
 * Runs regex-based checks against DeepSeek draft output. No LLM judgment.
 * Returns structured JSON with findings grouped by severity.
 *
 * Usage:
 *   node tools/prose-lint.mjs <file>
 *   node tools/prose-lint.mjs --stdin
 *   node tools/prose-lint.mjs --test
 */

import fs from "node:fs/promises";
import process from "node:process";

// ─── Banned phrases (Critical severity) ──────────────────────────────────────

const BANNED_PHRASES = [
  { pattern: /none of this was conscious/gi, message: "Narrator analyzing body/transformation" },
  { pattern: /your body is making (calculations|them)/gi, message: "Narrator analyzing body" },
  { pattern: /you used to do this/gi, message: "Banned transformation commentary" },
  { pattern: /you used to (\w+) this/gi, message: "Banned transformation commentary" },
  { pattern: /you know what he'?s (doing|thinking)/gi, message: "Banned omniscient claim" },
  { pattern: /you recognize the calculation/gi, message: "Banned transformation commentary" },
  { pattern: /without you deciding/gi, message: "Narrator analyzing body" },
  { pattern: /the armor went up/gi, message: "Narrator analyzing body" },
  { pattern: /which is what you came here for/gi, message: "Narrator explaining motivation" },
  { pattern: /because your hands need something to do/gi, message: "Narrator explaining motivation" },
  { pattern: /your body is making them/gi, message: "Narrator analyzing body" },
  { pattern: /you'?re just watching it work/gi, message: "Narrator analyzing body" },
  { pattern: /none of this is conscious/gi, message: "Narrator analyzing body/transformation" },
  // Recurring interiority tells caught in the looping-adult review (2026-06):
  // naming an internal faculty then reporting it suppressed, or the body as a
  // deliberating agent. Show the body; never narrate the mental process.
  { pattern: /the part of you that (takes notes|takes stock|might object|could object|keeps a hand|usually keeps)/gi, message: "Naming an internal faculty then reporting it suppressed (interiority)" },
  { pattern: /the (?:part|version) of you that (?:takes notes|is not available)/gi, message: "Naming an internal faculty (interiority)" },
  { pattern: /the body (?:has been |is |had been )?(?:making (?:a|its) case|running the argument|making an argument|making its case)/gi, message: "Narrator analyzing body ('the body makes a case')" },
  { pattern: /(?:cognitive )?dissonance is noted(?: and filed)?/gi, message: "Narrator analyzing a mental state ('noted and filed')" },
  { pattern: /noted and filed/gi, message: "Narrator analyzing a mental state ('noted and filed')" },
];

// ─── Orgasm spelling: "cum" not "come" (Important) ───────────────────────────
// House style: the climax verb is "cum"/"cums"/"cumming". Past tense stays
// "came". Only the person/body-climaxing sense — motion ("come in", "comes
// out") and the orgasm/it ARRIVING ("the orgasm comes") stay "come".

const ORGASM_SPELLING = [
  { pattern: /\b(?:make|makes|making|let|lets|letting)\s+(?:you|her|him|me)\s+come\b/gi },
  { pattern: /\b(?:going to|about to|gonna)\s+come\b/gi },
  { pattern: /\bcome\s+(?:hard|fast|first|again|twice|quietly|loud(?:ly)?|already|on (?:his|her|your)|against (?:his|her|your)|around (?:his|her|your))/gi },
  { pattern: /\b(?:you|he|she)\s+comes?\s+(?:hard|fast|first|again|twice|quietly|loud(?:ly)?|on (?:his|her|your)|against (?:his|her|your)|around (?:his|her|your)|before (?:he|she|you)|with a (?:grunt|groan|moan))/gi },
  { pattern: /\b(?:you'?re|he'?s|she'?s|i'?m)\s+coming\b(?!\s+(?:over|in|out|up|down|back|to|home|with|along|through|around|here|apart|undone|for|after|toward|with you))/gi },
];

// ─── AI erotic clichés (Critical) ────────────────────────────────────────────

const EROTIC_CLICHES = [
  { pattern: /bit (her|your) lip/gi, message: "AI erotic cliché" },
  { pattern: /heat building inside/gi, message: "AI erotic cliché" },
  { pattern: /couldn'?t help (her|your)self/gi, message: "AI erotic cliché" },
  { pattern: /explored (her|your) body/gi, message: "AI erotic cliché" },
  { pattern: /\bthrobbing\b/gi, message: "AI erotic cliché" },
  { pattern: /feminine core/gi, message: "AI erotic euphemism" },
  { pattern: /desire building/gi, message: "AI erotic cliché" },
  { pattern: /growing need/gi, message: "AI erotic cliché" },
];

// ─── AI prose tells (Critical) ───────────────────────────────────────────────

const AI_PROSE_TELLS = [
  // Em-dash reveals: "Not X, exactly — more like Y"
  {
    pattern: /[Nn]ot \w+,? exactly\s*[—–-]\s*more like/g,
    message: "Em-dash reveal pattern",
  },
  // "something — not quite X"
  {
    pattern: /something\s*[—–-]\s*not quite/g,
    message: "Em-dash reveal pattern",
  },
  // Over-naming: "the universal X nod/look/smile"
  {
    pattern: /the (?:universal|unspoken|familiar|inevitable|particular) [\w-]+ (?:nod|look|smile|gesture|shrug|pause|silence)/gi,
    message: "Over-naming — label the experience instead of showing it",
  },
  // Heart/pulse clichés
  { pattern: /heart skip/gi, message: "Heart/pulse cliché" },
  { pattern: /pulse quicken/gi, message: "Heart/pulse cliché" },
  { pattern: /shiver.{0,15}spine/gi, message: "Heart/pulse cliché" },
  { pattern: /heart pound/gi, message: "Heart/pulse cliché" },
  { pattern: /breath catch/gi, message: "Heart/pulse cliché (breath catch)" },
  // Emotion announcements
  {
    pattern: /[Yy]ou feel (nervous|anxious|confident|happy|sad|scared|excited|uncomfortable|uneasy|relieved|grateful|hopeful)/g,
    message: "Emotion announcement — show physical evidence instead",
  },
];

// ─── Overused words (Important — flag at 3+) ─────────────────────────────────

const OVERUSED_WORDS = [
  { pattern: /\bspecific(?:ally)?\b/gi, label: "specific/specifically" },
  { pattern: /\bsomething about\b/gi, label: "something about" },
  { pattern: /\bthe way\b/gi, label: "the way" },
  { pattern: /\ba (?:quality|certain)\b/gi, label: "a quality/a certain" },
  { pattern: /\byou (?:notice|realize)\b/gi, label: "you notice/realize" },
  { pattern: /\bsomehow\b/gi, label: "somehow" },
  { pattern: /\bdeliberate(?:ly)?\b/gi, label: "deliberate/deliberately" },
  { pattern: /\bsomething shifts\b/gi, label: "something shifts" },
  { pattern: /\bthe weight of\b/gi, label: "the weight of" },
];

const OVERUSED_THRESHOLD = 3;

// ─── Full thoughts (Important) ───────────────────────────────────────────────
// Italicized text longer than ~4 words suggests a full thought, not a fragment.

const FULL_THOUGHT_PATTERN = /\*([^*]{25,})\*/g;

// ─── Player acts in intro (Critical) ─────────────────────────────────────────

const PLAYER_ACTION_VERBS = [
  "order", "sit down", "sit at", "walk to", "walk over", "grab", "open",
  "choose", "decide", "pick up", "nod", "shake hands", "get dressed",
  "commute", "take a seat", "put on", "change into", "head to", "go to",
  "step inside", "step into", "move to", "reach for", "pull out", "turn to",
  "lean", "signal", "wave", "gesture to", "set your", "set the", "take out",
  "take off", "zip", "unzip", "shoulder your", "stand up", "push open",
  "close your", "open your", "plug in", "boot up", "connect to",
];

const PLAYER_SPEECH_PATTERN = /[""][^""]*[""][\s,]*you (say|tell|ask|reply|answer|murmur|whisper|call|shout)/gi;

// Not flagged (sensory/involuntary):
const SAFE_INTRO_VERBS = new Set([
  "see", "feel", "hear", "notice", "smell", "sense", "look", "watch",
  "are", "can", "find", "realize", "remember", "recognize", "know",
]);

// ─── POV violations ──────────────────────────────────────────────────────────
// "She" at start of sentence in narration (outside Jinja2 tags and dialogue).
// This is a heuristic — will flag NPC descriptions too, but those are rare
// enough to be worth manual review.

const SHE_SUBJECT_PATTERN = /(?:^|\.\s+)She (?:walk|went|sat|stood|felt|looked|moved|ran|took|turned|ordered|decided|chose|grabbed|opened|stepped|said|asked|told|thought|knew|realized|noticed|watched|smiled|laughed|frowned|sighed|shrugged)/gm;

// ─── Anaphoric repetition ────────────────────────────────────────────────────
// Two consecutive sentences starting with the same structure: "It X. It Y."

const ANAPHORIC_PATTERN = /(?:^|\n)(It \w[^.]{5,}\.\s*It \w[^.]{5,}\.)/gm;

// ─── Staccato closers ────────────────────────────────────────────────────────
// Short sentence (< 35 chars) as the last line of a paragraph, starting with
// "The" or "A" — atmospheric filler.

function findStaccatoClosers(text) {
  const findings = [];
  const paragraphs = text.split(/\n\n+/);
  for (const para of paragraphs) {
    const trimmed = para.trim();
    if (!trimmed) continue;
    // Skip Jinja2 blocks
    if (trimmed.startsWith("{%")) continue;
    const sentences = trimmed.split(/(?<=[.!?])\s+/);
    if (sentences.length < 2) continue;
    const last = sentences[sentences.length - 1].trim();
    if (
      last.length < 35 &&
      last.length > 3 &&
      /^(The|A|An|It) /.test(last) &&
      !/{[%{]/.test(last)
    ) {
      findings.push(last);
    }
  }
  return findings;
}

// ─── Branch block extractor ──────────────────────────────────────────────────
// Parses {% if %}...{% elif %}...{% else %}...{% endif %} blocks and measures
// the prose length in each branch. Only tracks top-level blocks (depth 1).

function extractBranchBlocks(text) {
  const blocks = [];
  let currentBlock = null;
  let currentBranch = null;
  let depth = 0;

  for (const line of text.split("\n")) {
    const trimmed = line.trim();

    if (/\{%\s*if\b/.test(trimmed)) {
      depth++;
      if (depth === 1) {
        currentBlock = { branches: [] };
        currentBranch = { textLength: 0 };
      }
    } else if (/\{%\s*elif\b/.test(trimmed) && depth === 1) {
      if (currentBranch) currentBlock.branches.push(currentBranch);
      currentBranch = { textLength: 0 };
    } else if (/\{%\s*else\s*%\}/.test(trimmed) && depth === 1) {
      if (currentBranch) currentBlock.branches.push(currentBranch);
      currentBranch = { textLength: 0 };
    } else if (/\{%\s*endif\b/.test(trimmed)) {
      if (depth === 1 && currentBlock) {
        if (currentBranch) currentBlock.branches.push(currentBranch);
        blocks.push(currentBlock);
        currentBlock = null;
        currentBranch = null;
      }
      depth = Math.max(0, depth - 1);
    } else if (depth === 1 && currentBranch) {
      currentBranch.textLength += trimmed.length;
    }
  }

  return blocks;
}

// ─── Section parser ──────────────────────────────────────────────────────────

const SECTION_HEADER = /^(INTRO|ACTION|NPC_ACTION):\s*(.*)/;

function parseSections(text) {
  const sections = [];
  let current = null;
  const lines = text.split(/\r?\n/);

  for (let i = 0; i < lines.length; i++) {
    const match = lines[i].match(SECTION_HEADER);
    if (match) {
      if (current) sections.push(current);
      current = {
        type: match[1],
        id: match[2].trim() || null,
        startLine: i + 1,
        lines: [],
      };
    } else if (current) {
      current.lines.push({ num: i + 1, text: lines[i] });
    }
  }
  if (current) sections.push(current);
  return sections;
}

// ─── Strip Jinja2 tags and dialogue for clean text checks ────────────────────

function stripJinja(text) {
  return text.replace(/\{%.*?%\}/gs, "").replace(/\{\{.*?\}\}/gs, "");
}

function stripDialogue(text) {
  return text.replace(/[""][^""]*[""]|"[^"]*"/g, "___DIALOGUE___");
}

// ─── Run all checks ──────────────────────────────────────────────────────────

function lint(text) {
  const findings = [];
  const sections = parseSections(text);
  const allLines = text.split(/\r?\n/);
  const cleanText = stripJinja(text);
  const noDialogue = stripDialogue(cleanText);

  // Helper: find line number for a match offset in a string
  function lineForOffset(src, offset) {
    const before = src.slice(0, offset);
    return (before.match(/\n/g) || []).length + 1;
  }

  // Helper: add a finding at a specific line
  function add(severity, rule, line, matchText, message) {
    findings.push({ severity, rule, line, text: matchText.trim(), message });
  }

  // 1. Banned phrases
  for (const { pattern, message } of BANNED_PHRASES) {
    pattern.lastIndex = 0;
    let m;
    while ((m = pattern.exec(cleanText)) !== null) {
      add("critical", "banned_phrase", lineForOffset(cleanText, m.index), m[0], message);
    }
  }

  // 2. AI erotic clichés
  for (const { pattern, message } of EROTIC_CLICHES) {
    pattern.lastIndex = 0;
    let m;
    while ((m = pattern.exec(cleanText)) !== null) {
      add("critical", "erotic_cliche", lineForOffset(cleanText, m.index), m[0], message);
    }
  }

  // 3. AI prose tells
  for (const { pattern, message } of AI_PROSE_TELLS) {
    pattern.lastIndex = 0;
    let m;
    while ((m = pattern.exec(cleanText)) !== null) {
      add("critical", "ai_prose_tell", lineForOffset(cleanText, m.index), m[0], message);
    }
  }

  // 3b. Orgasm spelling — climax verb should be "cum", not "come"
  for (const { pattern } of ORGASM_SPELLING) {
    pattern.lastIndex = 0;
    let m;
    while ((m = pattern.exec(cleanText)) !== null) {
      add("important", "orgasm_spelling", lineForOffset(cleanText, m.index), m[0], "Use 'cum'/'cums'/'cumming' for the orgasm sense (not 'come'); 'came' stays");
    }
  }

  // 4. Full thoughts (italicized sentences > 25 chars)
  {
    FULL_THOUGHT_PATTERN.lastIndex = 0;
    let m;
    while ((m = FULL_THOUGHT_PATTERN.exec(cleanText)) !== null) {
      // Allow fragments — only flag if it looks like a complete sentence
      const inner = m[1].trim();
      if (/[.!?]$/.test(inner) || /\b(I'm|I am|I was|I have|I need|I want|I can|I don't|I didn't|I won't)\b/i.test(inner)) {
        add("important", "full_thought", lineForOffset(cleanText, m.index), `*${inner}*`, "Full articulated thought — inner voice must be fragments");
      }
    }
  }

  // 5. Player acts in intro
  for (const section of sections) {
    if (section.type !== "INTRO") continue;
    const introText = section.lines.map((l) => l.text).join("\n");
    const cleanIntro = stripDialogue(stripJinja(introText));

    // Player action verbs
    for (const verb of PLAYER_ACTION_VERBS) {
      const re = new RegExp(`\\b[Yy]ou ${verb}\\b`, "gi");
      let m;
      while ((m = re.exec(cleanIntro)) !== null) {
        const lineOffset = lineForOffset(cleanIntro, m.index);
        const absoluteLine = section.startLine + lineOffset;
        add(
          "critical",
          "player_acts_in_intro",
          absoluteLine,
          m[0],
          `Player performs action "${verb}" in intro — intro describes the world, not player choices`,
        );
      }
    }

    // Player speech in intro
    {
      PLAYER_SPEECH_PATTERN.lastIndex = 0;
      let m;
      while ((m = PLAYER_SPEECH_PATTERN.exec(cleanIntro)) !== null) {
        const lineOffset = lineForOffset(cleanIntro, m.index);
        const absoluteLine = section.startLine + lineOffset;
        add("critical", "player_speaks_in_intro", absoluteLine, m[0], "Player speaks in intro — speech belongs in actions");
      }
    }
  }

  // 6. POV violations
  {
    SHE_SUBJECT_PATTERN.lastIndex = 0;
    let m;
    while ((m = SHE_SUBJECT_PATTERN.exec(noDialogue)) !== null) {
      add("critical", "pov_violation", lineForOffset(noDialogue, m.index), m[0].trim(), "Possible POV violation — must be second-person 'you'");
    }
  }

  // 7. Anaphoric repetition
  {
    ANAPHORIC_PATTERN.lastIndex = 0;
    let m;
    while ((m = ANAPHORIC_PATTERN.exec(cleanText)) !== null) {
      add("important", "anaphoric_repetition", lineForOffset(cleanText, m.index), m[1].trim(), "Anaphoric repetition — two sentences with same structure");
    }
  }

  // 8. Staccato closers
  {
    const closers = findStaccatoClosers(cleanText);
    for (const closer of closers) {
      // Find line number
      const idx = cleanText.indexOf(closer);
      const line = idx >= 0 ? lineForOffset(cleanText, idx) : 0;
      add("important", "staccato_closer", line, closer, "Possible staccato closer — atmospheric filler");
    }
  }

  // 9. Overused words
  const overusedStats = {};
  for (const { pattern, label } of OVERUSED_WORDS) {
    pattern.lastIndex = 0;
    const matches = cleanText.match(pattern);
    const count = matches ? matches.length : 0;
    if (count >= OVERUSED_THRESHOLD) {
      overusedStats[label] = count;
    }
  }

  for (const [label, count] of Object.entries(overusedStats)) {
    add("important", "overused_word", 0, `${label} (${count}x)`, `Overused: "${label}" appears ${count} times (threshold: ${OVERUSED_THRESHOLD})`);
  }

  // 10. Structural depth checks (only on complete scene drafts)
  const hasIntro = sections.some((s) => s.type === "INTRO");
  const hasAction = sections.some((s) => s.type === "ACTION");
  const isCompleteDraft = hasIntro && hasAction;

  for (const section of isCompleteDraft ? sections : []) {
    const sectionText = section.lines.map((l) => l.text).join("\n");
    const sectionLabel = section.id ? `${section.type}:${section.id}` : section.type;

    // Count Jinja2 if-blocks (top-level only)
    const ifCount = (sectionText.match(/\{%\s*if\b/g) || []).length;

    // Unbranched section check
    if (ifCount === 0) {
      if (section.type === "INTRO") {
        add("important", "unbranched_intro", section.startLine, sectionLabel,
          "Intro has no trait/skill branches — at least one {% if %} recommended");
      } else if (section.type === "ACTION") {
        add("minor", "unbranched_action", section.startLine, sectionLabel,
          "Action prose has no trait branches — consider branching on key traits");
      }
    }

    // Branch length analysis — detect adjective swaps
    const branchBlocks = extractBranchBlocks(sectionText);
    for (const block of branchBlocks) {
      if (block.branches.length < 2) continue;

      const lengths = block.branches.map((b) => b.textLength);
      const allShort = lengths.every((len) => len < 80);
      const maxLen = Math.max(...lengths);
      const minLen = Math.min(...lengths);

      if (allShort && block.branches.length >= 2) {
        add("important", "shallow_branch", section.startLine, sectionLabel,
          `Branch block has ${block.branches.length} branches, all < 80 chars ` +
          `(${lengths.join(", ")} chars). Likely adjective swaps — ` +
          "branches should change what HAPPENS (60+ words each).");
      } else if (maxLen > 0 && minLen > 0 && minLen < maxLen * 0.25 && minLen < 60) {
        // One branch is much shorter — uneven effort
        add("minor", "uneven_branches", section.startLine, sectionLabel,
          `Branch lengths vary widely (${lengths.join(", ")} chars). ` +
          "Shortest branch may need more depth.");
      }
    }

    // Section length checks
    const proseOnly = stripJinja(sectionText).trim();
    if (section.type === "INTRO" && proseOnly.length < 200 && proseOnly.length > 0) {
      add("important", "short_intro", section.startLine, sectionLabel,
        `Intro is only ${proseOnly.length} chars — needs more world-building (min 200)`);
    }
    if (section.type === "ACTION" && proseOnly.length < 80 && proseOnly.length > 0) {
      add("important", "short_action", section.startLine, sectionLabel,
        `Action prose is only ${proseOnly.length} chars — needs more depth (min 80)`);
    }
    if (section.type === "NPC_ACTION" && proseOnly.length < 60 && proseOnly.length > 0) {
      add("important", "short_npc_action", section.startLine, sectionLabel,
        `NPC action prose is only ${proseOnly.length} chars — NPC behavior needs substance (min 60)`);
    }
  }

  // 11. Section presence checks (always run if multiple sections detected)
  if (sections.length >= 2) {
    const introSections = sections.filter((s) => s.type === "INTRO");
    const actionSections = sections.filter((s) => s.type === "ACTION");

    if (introSections.length === 0) {
      add("critical", "missing_intro", 0, "INTRO", "No INTRO section — scene must establish the world before choices");
    }
    if (actionSections.length === 0) {
      add("critical", "missing_actions", 0, "ACTION", "No ACTION sections — players need choices");
    }
  }

  // 12. Branch density checks (only on complete drafts)
  if (isCompleteDraft) {
    const actionSections = sections.filter((s) => s.type === "ACTION");

    if (actionSections.length === 1) {
      add("important", "single_action", 0, "ACTION", "Only 1 ACTION section — a single choice is not a choice");
    }

    // Count total branches across all sections
    const totalBranches = sections.reduce((sum, s) => {
      const text = s.lines.map((l) => l.text).join("\n");
      return sum + (text.match(/\{%\s*if\b/g) || []).length;
    }, 0);

    if (totalBranches === 0) {
      add("critical", "no_branches", 0, "ALL", "Scene has zero trait/skill branches — every scene needs branching for depth");
    } else if (totalBranches < 2 && actionSections.length >= 2) {
      add("important", "few_branches", 0, "ALL",
        `Only ${totalBranches} branch(es) across ${sections.length} sections — more branching needed`);
    }
  }

  // Determine pass/fail
  const hasCritical = findings.some((f) => f.severity === "critical");
  const pass = !hasCritical;

  return {
    pass,
    total: findings.length,
    critical: findings.filter((f) => f.severity === "critical").length,
    important: findings.filter((f) => f.severity === "important").length,
    findings,
    overused_words: overusedStats,
  };
}

// ─── Self-test ───────────────────────────────────────────────────────────────

function selfTest() {
  const cases = [
    {
      name: "clean prose passes",
      input: `INTRO:\nDonovan's is half-empty on a Tuesday, which is probably why you picked it. Warm lighting, fryer oil and hops. A couple sharing nachos in the corner booth. Three guys at the end of the bar with a game on TV. The bartender looks up when you come in.\n\n{% if w.hasTrait("BEAUTIFUL") %}\nOne of the guys at the end has stopped watching the game. You can feel it on the side of your face before you look over.\n{% endif %}\n\nThe coaster is waiting.\n\nACTION: order_drink\n{% if w.hasTrait("SHY") %}\nYou clear your throat. "Um. A beer?" She waits. "Whatever's on tap is fine." The words stack on each other.\n{% else %}\n"Whiskey, neat," you say. She pours it.\n{% endif %}\n\nACTION: leave\nYou step back outside into the cold.\n`,
      expectPass: true,
    },
    {
      name: "banned phrase caught",
      input: `INTRO:\nNone of this was conscious. Your body is making calculations.\n`,
      expectPass: false,
      expectRule: "banned_phrase",
    },
    {
      name: "erotic cliché caught",
      input: `ACTION: kiss\nShe bit her lip and looked away.\n`,
      expectPass: false,
      expectRule: "erotic_cliche",
    },
    {
      name: "player acts in intro caught",
      input: `INTRO:\nThe bar is warm. You order a beer and sit down.\n`,
      expectPass: false,
      expectRule: "player_acts_in_intro",
    },
    {
      name: "player action in ACTION section is fine",
      input: `ACTION: order\nYou order a beer.\n`,
      expectPass: true,
    },
    {
      name: "emotion announcement caught",
      input: `INTRO:\nYou feel nervous as you walk in.\n`,
      expectPass: false,
      expectRule: "ai_prose_tell",
    },
    {
      name: "full thought caught",
      input: `ACTION: think\n*I'm here and I'm fine and I haven't decided yet.*\n`,
      expectPass: true, // full_thought is "important" not "critical"
      expectFinding: "full_thought",
    },
    {
      name: "fragment thought OK",
      input: `ACTION: react\n*Huh.* You look up.\n`,
      expectPass: true,
    },
    {
      name: "orgasm 'come' flagged",
      input: `ACTION: x\nYou come hard around his fingers and he keeps going.\n`,
      expectPass: true, // orgasm_spelling is "important", not critical
      expectFinding: "orgasm_spelling",
    },
    {
      name: "motion 'come' not flagged as orgasm",
      input: `INTRO:\nHe comes over and sits down. The bartender looks up when you come in.\n\n{% if w.hasTrait("SHY") %}\nYou wait quietly because that is the kind of thing a shy person does at a bar on a slow Tuesday night.\n{% endif %}\n\nACTION: leave\nYou step back outside into the cold night air.\n`,
      expectPass: true,
      expectNotFinding: "orgasm_spelling",
    },
    {
      name: "interiority 'noted and filed' flagged",
      input: `ACTION: x\nThe cognitive dissonance is noted and filed.\n`,
      expectPass: false,
      expectRule: "banned_phrase",
    },
    {
      name: "em-dash reveal caught",
      input: `ACTION: look\nNot danger, exactly — more like being placed.\n`,
      expectPass: false,
      expectRule: "ai_prose_tell",
    },
    {
      name: "overused words flagged",
      input: `INTRO:\nSomething about the way he looks. Something about the room. Something about the light. Something about the sound.\n`,
      expectPass: true, // overused is "important"
      expectFinding: "overused_word",
    },
    // ── Depth checks ──────────────────────────────────────────────────────────
    {
      name: "no branches = critical",
      input: `INTRO:\nThe bar is warm and welcoming. People are drinking. The bartender wipes the counter. You can hear the game on TV. Outside it's raining.\n\nACTION: order\nYou order a beer. She pours it.\n\nACTION: leave\nYou walk out.\n`,
      expectPass: false,
      expectRule: "no_branches",
    },
    {
      name: "shallow branches flagged",
      input: `INTRO:\nThe bar is warm.\n\n{% if w.hasTrait("BEAUTIFUL") %}\nHe looks.\n{% else %}\nHe doesn't.\n{% endif %}\n\nACTION: order\nYou order.\n\nACTION: leave\nYou leave.\n`,
      expectPass: true, // shallow_branch is "important" not critical
      expectFinding: "shallow_branch",
    },
    {
      name: "deep branches pass clean",
      input: `INTRO:\nDonovan's is half-empty on a Tuesday, which is probably why you picked it. Warm lighting, fryer oil and hops. A couple sharing nachos in the corner booth, leaned in close enough that whatever they're saying is just for them. Three guys at the far end of the bar with a pitcher and a game on TV.\n\n{% if w.hasTrait("BEAUTIFUL") %}\nOne of the guys at the end has stopped watching the game. You can feel it on the side of your face before you look over. His friend follows his eyes to you. Now there are two. The third one is still watching the game but his posture has shifted.\n{% endif %}\n\nThe coaster is waiting. The bartender is waiting.\n\nACTION: order_drink\n{% if w.hasTrait("CONFIDENT") %}\n"Gin and tonic," you say before she can ask. Your voice comes out clear, the way you'd order in a meeting. She nods and turns to the well. No small talk. You appreciate that about bars — the transaction is clean. You have a drink now. The glass is cold in your hand.\n{% elif w.hasTrait("SHY") %}\nYou have to clear your throat. "Um. A beer?" She waits. "Whatever's on tap is fine." The words come out too fast, stacked on top of each other. She pulls a pint without comment. When she sets it down, the foam spills slightly over the edge and you watch it track down the glass because that's easier than looking at anything else.\n{% else %}\n"Whiskey, neat," you say. Something simple. She pours it and the amber catches the light. You wrap your hand around it. The glass is heavier than you expected, or your hand is lighter.\n{% endif %}\n\nACTION: leave\nYou step back outside.\n`,
      expectPass: true,
    },
    {
      name: "missing intro = critical",
      input: `ACTION: order\nYou order a beer.\n\nACTION: leave\nYou leave.\n`,
      expectPass: false,
      expectRule: "missing_intro",
    },
    {
      name: "short intro flagged",
      input: `INTRO:\nThe bar.\n\n{% if w.hasTrait("SHY") %}\nYou sit quietly and wait for something to happen because you are shy and that is what shy people do in bars on Tuesday evenings.\n{% endif %}\n\nACTION: order\nYou order a beer. The glass is cold.\n\nACTION: leave\nYou leave through the front door.\n`,
      expectPass: true, // short_intro is "important"
      expectFinding: "short_intro",
    },
  ];

  let passed = 0;
  let failed = 0;

  for (const tc of cases) {
    const result = lint(tc.input);
    let ok = true;

    if (tc.expectPass !== undefined && result.pass !== tc.expectPass) {
      ok = false;
      process.stderr.write(`FAIL: ${tc.name} — expected pass=${tc.expectPass}, got ${result.pass}\n`);
      if (result.findings.length > 0) {
        process.stderr.write(`  findings: ${result.findings.map((f) => `${f.severity}:${f.rule}`).join(", ")}\n`);
      }
    }

    if (tc.expectRule && !result.findings.some((f) => f.rule === tc.expectRule)) {
      ok = false;
      process.stderr.write(`FAIL: ${tc.name} — expected rule ${tc.expectRule} not found\n`);
    }

    if (tc.expectFinding && !result.findings.some((f) => f.rule === tc.expectFinding)) {
      ok = false;
      process.stderr.write(`FAIL: ${tc.name} — expected finding ${tc.expectFinding} not found\n`);
    }

    if (tc.expectNotFinding && result.findings.some((f) => f.rule === tc.expectNotFinding)) {
      ok = false;
      process.stderr.write(`FAIL: ${tc.name} — unexpected finding ${tc.expectNotFinding} present\n`);
    }

    if (ok) {
      passed++;
    } else {
      failed++;
    }
  }

  process.stderr.write(`\nprose-lint self-test: ${passed} passed, ${failed} failed\n`);
  return failed === 0;
}

// ─── CLI ─────────────────────────────────────────────────────────────────────

async function readStdin() {
  if (process.stdin.isTTY) return "";
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  return Buffer.concat(chunks).toString("utf8");
}

async function main() {
  const args = process.argv.slice(2);

  if (args.includes("--test")) {
    const ok = selfTest();
    process.exitCode = ok ? 0 : 1;
    return;
  }

  if (args.includes("--help") || args.includes("-h")) {
    process.stdout.write(`Usage:
  node tools/prose-lint.mjs <file>       Lint a prose draft file
  node tools/prose-lint.mjs --stdin      Read from stdin
  node tools/prose-lint.mjs --test       Run self-tests

Output: JSON report to stdout.
Exit code: 0 if pass, 1 if critical findings.
`);
    return;
  }

  let input;
  if (args.includes("--stdin")) {
    input = await readStdin();
  } else if (args[0]) {
    input = await fs.readFile(args[0], "utf8");
  } else {
    process.stderr.write("Error: provide a file path or --stdin\n");
    process.exitCode = 1;
    return;
  }

  if (!input.trim()) {
    process.stderr.write("Error: input is empty\n");
    process.exitCode = 1;
    return;
  }

  const result = lint(input);
  process.stdout.write(JSON.stringify(result, null, 2) + "\n");

  if (!result.pass) {
    process.stderr.write(
      `[prose-lint] FAIL — ${result.critical} critical, ${result.important} important\n`,
    );
    process.exitCode = 1;
  } else if (result.important > 0) {
    process.stderr.write(
      `[prose-lint] PASS with warnings — ${result.important} important\n`,
    );
  } else {
    process.stderr.write("[prose-lint] PASS — clean\n");
  }
}

main().catch((error) => {
  process.stderr.write(`[prose-lint] ${error.message}\n`);
  process.exitCode = 1;
});
