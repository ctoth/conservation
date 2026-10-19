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
- Step 1 (research sweep) starting.

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

## Decisions and why

- Notes first, full ingestion later: full `paper-process` costs several times as much per
  paper; ingest only what the synthesis depends on.
- Bailey 2014 / BH 1995 do not apply to this repo's tests: assertions are exact-rational
  pass/fail and proptest is a counterexample search (a failure is a concrete replayable
  input). They do apply to any experiments the study runs, and belong in the corpus as
  instances of test coverage under selection pressure.

## Surprises

- An untracked `conservation-dynamics/tests/compiled_conformance.proptest-regressions` held a
  failure seed (vault=10, safe=10, spend=0, share=0, drain=1). It passes on master 1dfe715;
  committed as a regression case.

## Next step

Run the step 1 sweep.
