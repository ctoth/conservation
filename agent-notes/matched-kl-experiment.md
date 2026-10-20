# Matched-KL Gaussian experiment

## Goal

Implement the approved matched-error, matched-finite-KL research experiment,
test it, and push tested intended files directly to the verified default branch.
No PR, release, settings changes, or edits to existing research functionality.

## State

- Origin is git@github.com:ctoth/conservation.git; remote HEAD is master at
  1dfe71548ed51562896e5d4f72625a3e27d917b3 (verified by ls-remote).
- The primary checkout is clean at e8544e7, with 15 unpublished research commits.
  This isolated branch starts at remote master to avoid publishing them incidentally.
- Implemented one standard-library experiment, 11 focused regressions, and a
  report. Existing scripts and reports in the primary checkout remain untouched.
- All 11 tests passed, including after the final outward-display rounding edit;
  the experiment completed all 20 rows in about one second. Worst KL residual
  8.143e-12; omitted tail 2.949e-6;
  bin gap 2.000e-4; floating allowance 7.086e-11.

## Decisions and why

- Gold is Bernoulli(0.3); bad/good conditional score variances are 0.2/0.1.
  The control draws the same full marginal error mixture independently of gold.
- Match finite joint/proxy KL using continuous best-of-n's exact divergence,
  log(n) - (n-1)/n. Gold KL is a separate, generally smaller quantity.
- Gaussian-mixture tilts are analytic. Best-of-n integrates posterior gold
  against the maximum-score law using adaptive probability bins with analytic
  posterior bounds and explicit omitted-tail mass. Binary64 evaluation is not
  a directed-rounding interval-arithmetic proof.
- No mechanism novelty claim. Kwa et al. 2407.14503v2, section 5.2 and footnote 6,
  already identify the dependence issue. TP2 claims require independent noise;
  the discrete version also needs compatible lattices and no internal zeros.

## Surprises

The models match the complete base error marginal and first two proxy moments,
but their full proxy marginals differ (including their third raw moments).

## Next step

No implementation remains. Inspect and publish only the four intended files
via normal HEAD:master push, and report the exact commit and remote verification
in the task conversation. The publication starts at remote master and excludes
the primary checkout's 15 older commits; their branch and files remain intact.
