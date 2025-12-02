# Contributing to Mesu

ma-tu — thank you for wanting to help grow Mesu!

## How You Can Contribute

### 📝 Documentation & Translations

- Fix typos or unclear explanations
- Translate documentation into other natural languages
- Create learning materials (guides, flashcards, exercises)
- Write example texts in Mesu

### 🔤 Vocabulary Proposals

Mesu has 48 roots. Everything else is built through composition. If you encounter a concept that feels awkward to express, you can propose:

1. **New compounds** — combinations of existing roots
2. **Refined definitions** — clearer semantic boundaries for existing roots
3. **Edge case guidance** — how to handle ambiguous situations

**Note:** New *roots* are unlikely to be added. The 48-root constraint is fundamental to Mesu's design philosophy. But creative compounds are always welcome.

### 🖋️ Script Development

- Font development for Print and Cursive scripts
- Keyboard layouts for typing Mesu glyphs
- Rendering tools and converters
- Calligraphy samples and style guides

### 🛠️ Tools & Code

- Text-to-glyph converters
- Learning apps or websites
- Parser/validator for Mesu text
- Pronunciation audio generation

## Contribution Process

### For Documentation/Text

1. Fork the repository
2. Create a branch: `git checkout -b improve/your-improvement`
3. Make your changes
4. Submit a Pull Request with a clear description

### For Vocabulary Proposals

Open an Issue with the tag `vocabulary` containing:

```markdown
## Concept
[The meaning you want to express]

## Proposed Compound
[Your suggested Mesu construction]

## Derivation
[How it breaks down: root + root = meaning]

## Example Usage
[A sentence using the new compound]

## Alternatives Considered
[Other ways you thought about expressing this]
```

### For Tools/Code

1. Open an Issue first to discuss the approach
2. Fork and develop
3. Include documentation
4. Submit PR

## Style Guidelines

### For Mesu Text

- Use lowercase romanization (mi, tu, mesu — not MI, TU, MESU)
- Separate compounds with hyphens (wo-lum, not wolum)
- Use interpuncts (·) for sentence boundaries in Print
- Include interlinear glosses for non-trivial examples:

```
mi i wo-wi a tu
I [VERB] big-feel [OBJ] you
"I love you"
```

### For Documentation

- Keep explanations accessible to non-linguists
- Use examples liberally
- Include both romanized and glyph versions where relevant
- Be concise — Mesu values efficiency

### For Code

- Clear comments explaining Mesu-specific logic
- Handle Unicode properly (Mesu uses box-drawing characters)
- Include tests for any parsers or converters

## What We're Not Looking For

- Fundamental changes to the phonology (13 phonemes is final)
- New root words (48 roots is final)
- Changes to the grammar particle system
- "Fixing" perceived inefficiencies that are actually design choices

The constraint *is* the design. Mesu isn't trying to be maximally expressive — it's trying to be minimally complex while remaining fully functional.

## Recognition

Contributors will be acknowledged in the repository. Significant contributions may be noted in the CHANGELOG.

## Questions?

- Open an Issue with the `question` tag
- Email: contact@whispr.dev

---

## A Note on AI Contributions

This language was co-created with Claude. We welcome continued AI-assisted contributions — whether that's using AI to help draft proposals, generate examples, or develop tools.

What matters is the quality and thoughtfulness of the contribution, not whether a human or AI (or both together) produced it. Just be transparent about your process.

---

```
mi la i lu a mesu
I toward [VERB] know [OBJ] Mesu
"I will learn Mesu"

kan tu o
and you [QUESTION]
"And you?"
```

**ma-la — good journey — goodbye (for now)**
