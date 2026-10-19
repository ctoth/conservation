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
- Tier F addition (Q, 2026-10-19; verified on arXiv): Narang, Faulkner, Drusvyatskiy, Fazel &
  Ratliff, "Multiplayer Performative Prediction: Learning in Decision-Dependent Games",
  arXiv 2201.03398. Population data reacts to several competing decision makers; solution
  concepts are performatively stable equilibria and Nash equilibria (efficiently findable
  only for monotone games). Relevance: several optimizers jointly shaping one channel.

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

- Conjecture (Q, 2026-10-19): true value per nat of KL decays once selection pressure (nats)
  exceeds I(verifier score; true quality); light tails → diminishing returns, heavy tails →
  catastrophic Goodhart. Candidate "Bode integral for performative channels": a check with
  I nats about quality safely carries ~I nats of optimization. Test: Gao setup, peak near
  KL ≈ I?
  - Checked (Gao et al. arXiv 2210.10760 §1): d := √KL; R_bon = d(α − βd), R_RL =
    d(α − β log d). Peaks (my algebra): BoN KL* = α²/4β²; RL d* = e^(α/β − 1). Gold RM =
    OpenAI-internal 6B InstructGPT RM (§2.1), so not reproducible as-is.
  - My derivation, unchecked: jointly Gaussian proxy/gold → BoN gold gain ≈ ρ√(2KL), no peak,
    noise share constant (1−ρ); small ρ: gain ≈ 2√(I·KL). So the KL ≈ I turnover must come
    from non-Gaussian error (Gao's β term). Published-coefficient filter: ρ ≈ α/√2,
    I ≥ −½log(1−α²/2) ≈ α²/4 ⇒ conjecture predicts β_bon ≈ 1 across RM sizes.
  - β check DONE (2026-10-19), values read off Gao Fig. 3 (p.5; no numeric table exists):
    α_bon ≈ 0.50 (≈3M RM) → 0.66 (3B); β_bon ≈ 0.12 → 0.09. So β_bon ≈ 0.1, not ≈ 1.
    Peak KL* = α²/4β² ≈ 4.3 nats (small RM) → 13.4 nats (3B). Gaussian I from ρ = α/(√2·c),
    c ∈ [0.92, 1] (ratio of E[max of n normals] to √(2·KL_bon)): I ≈ 0.07–0.08 nats → 0.12–0.15
    nats. KL*/I ≈ 54–65 (small) → 90–109 (3B): peak sits ~2 orders of magnitude past
    Gaussian-copula I, and the ratio grows with RM size. Gaussian I is a lower bound at fixed ρ,
    so the conjecture survives only if proxy–gold dependence is strongly non-Gaussian
    (I of 4–13 nats at ρ ≈ 0.4–0.5). Settling it needs a direct MI measurement.
  - Theory hook (from memory, unchecked): Donsker–Varadhan/transportation: σ-subgaussian gold ⇒
    gain ≤ σ√(2KL); Russo & Zou 2016 is the MI version.
  - Real test (needs "make it so"): open replication; measure I(proxy;gold) by a rank-based
    estimator under π_init (predicts peak) and under π_d (endogenous-channel decay).
  - Retrieval additions (unverified): Kwa et al. 2024 "Catastrophic Goodhart"; Coste et al.
    2024 (open Gao replication); Bobkov & Götze 1999.

## Surprises

- Step 2 retrieval (2026-10-19): six background agents, notes only, each told not to touch
  shared collection files or commit. Kitano 2007 done: `papers/Kitano_2007_TheoryBiologicalRobustness/`;
  five quoted conditions checked against the PDF text (all match). Kitano's ΔR = 0 needs the
  whole/broad perturbation space, equiprobable weighting, same system optimized for different
  subsets — same shape as Schaffer/NFL's uniform prior (my reading, for the synthesis).
  Pending: Kitano 2010, Perdomo 2020, Narang 2022, Russo & Zou 2016, Pacelli & Theodorou 2026.
  Shared-file steps (index.md, keymap, reconcile) were skipped by design; run them once all land.
- The Kitano 2007 agent sent Q's email to the Unpaywall API in a manual curl. Cause:
  research-papers-plugin `skills/paper-retriever/SKILL.md` Option 2 step 2 says to query
  Unpaywall without naming the email (the script uses `$UNPAYWALL_EMAIL` or a placeholder).
  Fixed the skill text in ~/code/research-papers-plugin (uncommitted; that repo's master has
  other uncommitted WIP). Running agents told the rule by message; installed plugin 4.9.0
  lacks the fix until updated.

- An untracked `conservation-dynamics/tests/compiled_conformance.proptest-regressions` held a
  failure seed (vault=10, safe=10, spend=0, share=0, drain=1). It passes on master 1dfe715;
  committed as a regression case.

## Next step

Wait for the five pending retrieval agents; spot-check each paper's notes against its PDF;
then run the shared-file steps (index, keymap, reconcile) once and commit `papers/`. Then
the next retrieval batch: Kwa et al. 2024 (Catastrophic Goodhart), Coste et al. 2024, the
rest of the report's priority list.
