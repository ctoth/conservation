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
- Implementation commit bd52d66833cf93eb97539c98db38ce10aa3b3389 was pushed
  normally to origin/master and independently confirmed by ls-remote as both
  remote HEAD and refs/heads/master. Only the four intended added files were
  published; the primary checkout remains at e8544e7 with its old work intact.
- Final focused validation: 11 tests passed in 4.365s, and all 20 experiment rows
  completed. The implementation SHA's product CI run 39370172766 was in progress
  at the first post-push check; the new research regressions were run explicitly.
- CI run 39370319511 for 13ea78e completed successfully: Ubuntu stable/Python
  3.13, Ubuntu Rust 1.85/Python 3.10, and Windows stable/Python 3.13 all passed.
- Approved follow-up adds n=10^6, 10^12 and 10^24 to the existing independent
  density quadrature gold and proxy-KL regressions for both models. The twelve
  extra quadratures took 1.692s; all gold results were within reported bounds,
  and maximum absolute KL error was 4.619e-14. No implementation change.
- The expanded 11-test suite passed in 6.120s; the added cases remain in the
  ordinary fast invocation. Only the test parameter lists, report, and this
  task note changed in the follow-up.

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

Verification is complete. Publish this small regression/report/notes follow-up
normally and report the exact verified remote head. The original requested CI
run is successful. No implementation or product changes remain.
