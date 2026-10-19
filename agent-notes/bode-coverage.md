# Bode generalized: conservation laws for coverage under optimization pressure

## Goal

Study whether Bode's sensitivity integral (the waterbed: ∫ log|S(jω)| dω = π Σ Re p_unstable)
generalizes to a conservation law for *coverage* under optimization pressure. Coverage in all
four senses Q chose (2026-10-19): distributional/output, input/task-space,
disturbance/robustness, test/verification. Research only; no crate code until Q says
"make it so".

## State

- Work happens on `master` in this repo (Q, 2026-10-19). `papers/` lives here; `.gitignore`
  already excludes `*.pdf` and `*.png` (case-insensitive).
- Step 1 sweep done: `reports/research-bode-conservation-coverage.md` (corrected seed list,
  tiers A–F with identifiers, retrieval priority in its Recommendations). Claims checked by me
  against primary sources (2026-10-19): Kitano 2007 (MSB, full text: asks whether the
  robustness/fragility trade-off is a conservation principle, cites Bode as "conservation of
  sensitivity", says whether conservation exists "remains open"); Kitano 2010 (abstract:
  robustness may be conserved only when the system is sufficiently optimized); Martins &
  Dahleh 2008 (author PDF text: "The channel noise, the plant's initial condition and the
  disturbance are mutually independent"; Def. 5.1 channel = a given map plus a noise process
  independent of plant and disturbance; symbols lost in extraction, so the fixed-law reading
  still needs the typeset equations); Kent 2016 (0 matches for probab|entropy|measure);
  Pacelli & Theodorou 2026 (arXiv 2603.18454, IEEE L-CSS: bound where the closed-loop state
  distribution limits what the sensor can extract); Recht et al. 2019 (abstract: drops "not
  caused by adaptivity"). Everything else in the report is the sweep agent's claim, unchecked.
- Pending from Q: which "Narang et al." (message cut off). Guess, unconfirmed: Narang et al.
  2021, "Do Transformer Modifications Transfer…" (EMNLP).

## Plan

1. `research-papers:research` sweep → `reports/`; verify/correct the seed list (all seeds
   below were written from memory and are unverified).
2. Tiers A–D: `paper-retriever` + `paper-reader` notes only.
3. `reconcile --all`, then `process-leads` capped at ~10 per round.
4. Synthesis report in `reports/`: per coverage sense, the conserved quantity, the budget term
   (analogue of Σ Re p_unstable), and the assumptions; then: one budget or four laws?
5. Choose the load-bearing papers for full propstore ingestion.

Seed tiers (unverified):
- A, the law: Bode 1945; Freudenberg & Looze 1985; Seron, Braslavsky & Goodwin 1997; Stein 2003.
- B, information-theoretic: Iglesias 2001/2002; Martins & Dahleh 2008; Touchette & Lloyd
  2000/2004; Ashby 1956.
- C, optimization → fragility: Carlson & Doyle 1999, 2002; Csete & Doyle 2002; Chandra, Buzi
  & Doyle 2011.
- D, optimization-side: Schaffer 1994; Wolpert & Macready 1997; Manheim & Garrabrant 2018;
  Zhuang & Hadfield-Menell 2020; Gao, Schulman & Hilton 2023; Kirk et al. 2024.
- D, test strand: Inozemtseva & Holmes 2014; Bailey, Borwein, López de Prado & Zhu 2014
  (backtest overfitting); Benjamini & Hochberg 1995; Dwork et al. 2015 (reusable holdout);
  Recht et al. 2019.
- E, framework (Q, 2026-10-19): Kent, "Information Flow in Logical Environments", arXiv
  1603.03475 (verified via arXiv API): Barwise–Seligman channel theory over arbitrary logical
  environments (semantic institutions). Dependencies: Barwise & Seligman 1997; Goguen &
  Burstall 1992.

## Decisions and why

- Notes first, full ingestion later: full `paper-process` costs several times as much per
  paper; ingest only what the synthesis depends on.
- Bailey 2014 / BH 1995 do not apply to this repo's tests: assertions are exact-rational
  pass/fail and proptest is a counterexample search (a failure is a concrete replayable
  input). They do apply to any experiments the study runs, and belong in the corpus as
  instances of test coverage under selection pressure.

- Working framing (Q, 2026-10-19): Goodhart is optimization acting on the channel's
  regularities, not on the verifier; the verifier looks unchanged while its information about
  the goal falls toward zero. Martins & Dahleh assume a fixed channel (Q's claim; not yet
  checked against the paper), so they don't cover it. In Barwise–Seligman terms: the verifier
  is the classification, `pass ⊢ goal` is a constraint sound only on normal tokens, and
  optimization moves the token population onto abnormal ones. Floor: I(V;G) ≤ H(V) → 0 as
  pass rate → 1. With v = f(x) and g = h(x), p(v|g) is induced by p(x), which the optimizer
  controls: the channel is endogenous. Hypothesis to test: the budget term (analogue of
  Σ Re p_unstable) measures the abnormal-token set the optimizer can reach. Leads sent to the
  sweep: performative prediction (Perdomo et al. 2020), strategic classification (Hardt et al.
  2016).

## Surprises

- An untracked `conservation-dynamics/tests/compiled_conformance.proptest-regressions` held a
  failure seed (vault=10, safe=10, spend=0, share=0, drain=1). It passes on master 1dfe715;
  committed as a regression case.

## Next step

Step 2: retrieve + read notes, in the report's priority order, starting with Kitano 2007 and
2010 (closest prior statement of the thesis) and the Tier F endogenous-channel set
(Perdomo et al. 2020, Russo & Zou 2016, Pacelli & Theodorou 2026).
