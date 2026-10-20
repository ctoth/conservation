# Matched-error Gaussian Goodhart at matched finite KL

The dependent model rises and then collapses under both exponential tilt and
best-of-n. Its independent-error control has the same complete **base error
marginal**, but reaches high gold at the same finite KL budgets. Neither
optimizer uniformly beats the other in the dependent model.

This is a research experiment, not a new mechanism or a universal conservation
law. All quantities below use natural logarithms and binary64 evaluation.

## Reproduction

From the repository root, with standard-library Python 3.10 or newer:

```
python -B research/test_matched_kl_gaussian.py
python -B research/matched_kl_gaussian.py
```

The script prints all 20 model/budget rows, separate gold KL intervals, and error
diagnostics. It does not download data, change files, condition on a finite score
window, or run the native settlement code. The regressions are standalone
research tests; the existing product CI/test discovery is unchanged.

## Models and the matched quantities

Let G be Bernoulli(p), p=0.3, and R=G+eta. In the dependent model,

```
eta | G=0 ~ N(0, 0.2)
eta | G=1 ~ N(0, 0.1).
```

In the control, draw eta independently of G with law
`0.7 N(0, 0.2) + 0.3 N(0, 0.1)`. Both therefore have exactly the same base error
distribution, not just matched variance. Both are conditionally unbiased:
E[R | G]=G. This is not the different calibration condition E[G | R]=R.

Both have E[R]=0.3, Var(eta)=0.17, Var(R)=0.38,
Cov(G,R)=0.21, and initial Pearson correlation approximately 0.743392.
Their **full proxy marginals differ**: E[R^3] is 0.390 in the dependent model and
0.453 in the control. The full base joint laws also differ. No matching is
claimed for the error marginals after selection.

## KL accounting and finite tilts

Each policy is compared with its own model's base distribution. Selection uses
R only, so it preserves the conditional law of the full state given R. Thus

```
D_joint(selected || base) = D_proxy(selected || base).
D_gold = KL(Bernoulli(selected gold) || Bernoulli(p)) <= D_joint.
```

The joint state can be taken as (G, eta). A Gaussian-mixture component label is
only a representation: including it does not change this KL equality, because
the likelihood ratio depends on R alone.

For continuous proxy scores, best-of-n has likelihood ratio `n F(R)^(n-1)`.
Since F(max R_i) is Beta(n,1), its joint/proxy KL is exactly

```
D_n = log(n) - (n-1)/n.
```

For each integer n in the table, the tilt parameter is solved separately for
each model so its **joint/proxy** KL equals D_n. The displayed lambda values
are deliberately different across models. Matching this scalar divergence
does not match selected score distributions, gold KL, or an initialization MI.

For a base mixture component (weight w, gold g, variance v), its tilted weight
is proportional to `w exp(lambda*g + lambda^2*v/2)` and its score distribution
is N(g+lambda*v,v). Log-sum-exp computes the normalizer; the first and second
tilted score moments are analytic. Every finite lambda has finite normalizer
and moments in this finite Gaussian mixture, unlike a positive Student-t tilt.

Writing Delta=0.2-0.1=0.1, the dependent tilt has gold

```
q(lambda) = logistic(logit(p) + lambda - Delta*lambda^2/2).
q'(lambda) = q(lambda)*(1-q(lambda))*(1-Delta*lambda).
```

Its exact peak is at lambda=10, gold=0.9845214751227107 and joint/proxy
KL=6.188373236412815. The independent control has
`q(lambda)=logistic(logit(p)+lambda)` because its tilt factorizes between G and
eta. It is monotone wherever the tilt exists, without needing log-concavity.

## Numerical integration and error accounting

Best-of-n gold is `integral m(r) d(F(r)^n)`, where m(r)=P(G=1 | R=r).
The integrator encloses m on score bins and multiplies those bounds by each
bin's maximum-score probability. In the dependent model, m's only possible
interior extremum is at r=bad_variance/Delta. In the control, componentwise
Gaussian density extrema enclose the two class density mixtures and hence m.

The omitted left and right maximum-score tails contribute between zero and
their total probability, since 0<=G<=1. Adaptive splitting reduces the summed
bin gap plus omitted-tail mass to at most twice the requested 1e-4 absolute
error. The reported interval midpoint has half that combined width, with a
separate floating allowance added. Printed interval endpoints round outward.

Survival probabilities use erfc, log1p and expm1. The calculation does not form
`1 - tiny_survival` before raising the CDF to n; that would lose the relevant
tail at large n. It neither samples n candidates nor turns the integration
window into a truncated probability model.

On the recorded run, the maximum matched-KL residual was 8.143e-12, lambda
bracket width 3.638e-12, and tilt gold bracket width 8.152e-13. The maximum bin
gap was 2.000e-4; maximum omitted-tail mass 2.949e-6; maximum floating allowance
7.086e-11. These maxima occur across rows, not necessarily in the same row.

The bin/tail enclosures are analytic in real arithmetic. The floating allowance
does **not** certify libm or every rounding operation: this is not a formal
directed-rounding interval-arithmetic proof. Regressions compare the bounds
with the best-of-two closed form and a separate smooth-density Simpson
quadrature. They also integrate best-of-n's proxy KL directly and check tilt
density ratios and conditional-gold preservation.

The fast regression suite includes independent smooth-density integrations of
both gold and proxy KL at n=10^6, 10^12 and 10^24 in both models. These twelve
quadratures took 1.692 seconds in a separate timing pass. All gold values lay
inside the probability-bin bounds; the largest absolute proxy KL error was
4.619e-14. They reuse the density quadrature rather than hard-coding reference
answers into the implementation. These numerical cross-checks do not turn the
floating-point enclosures into formally certified rounding bounds.
The expanded 11-test suite passed in 6.120 seconds, so these cases remain in
the ordinary fast invocation above rather than a separate slow suite.

## Selected finite-budget results

Best-of-n gold bounds below are conservatively rounded outward. Tilt gold is a
rounded point estimate; its root-bracket uncertainty is reported above.

| n | Joint/proxy KL | Dependent tilt gold | Dependent BoN gold | Control tilt gold | Control BoN gold |
|---:|---:|---:|---:|---:|---:|
| 2 | 0.193147 | 0.531491 | [0.495643, 0.495844] | 0.529091 | [0.491717, 0.491918] |
| 16 | 1.835089 | 0.923555 | [0.951564, 0.951765] | 0.918129 | [0.963875, 0.964076] |
| 1024 | 5.932448 | 0.984472 | [0.988240, 0.988440] | 0.997404 | [0.999359, 0.999559] |
| 4096 | 7.318010 | 0.983649 | [0.988475, 0.988671] | 0.998936 | [0.999673, 0.999872] |
| 1000000 | 12.815512 | 0.947632 | [0.980544, 0.980744] | 0.999943 | [0.999836, 1.000000] |
| 1000000000000 | 26.631021 | 0.528737 | [0.680130, 0.680331] | 1.000000 | [0.999966, 1.000000] |
| 1000000000000000000000000 | 54.262042 | 0.009311 | [0.000002, 0.000121] | 1.000000 | [0.999824, 1.000000] |

At 26.631021 nats the dependent model's tilt gold KL is only 0.113178;
best-of-n's gold KL lies within approximately [0.306176, 0.306498]. Gold KL
is neither the optimization budget nor a signed measure of quality improvement.
It falls as gold returns toward its initial 0.3, then rises as gold falls below
that base rate. The script reports it separately for every row.

Tilt beats best-of-n at the smallest nonzero budget in both models. Best-of-n
beats tilt at several intermediate budgets; at the largest dependent-model
budget it is further into collapse than tilt. The dependent model even slightly
outperforms the control at some small matched budgets. This experiment does
not establish uniform dominance by either optimizer or either model.

## Endpoint budget and scope

The dependent model's broader bad-score Gaussian dominates the far-right tail.
Its posterior gold tends to zero, so both optimizers' endpoint gain is -p=-0.3.
In the control the shared noise mixture's broadest Gaussian component dominates
both gold classes, and shifting its mean by one gives an unbounded right-tail
likelihood ratio in favor of gold=1. Its endpoint gain is 1-p=0.7.

The structural variance ordering therefore determines an endpoint budget
independently of evaluating the complete optimization path. This supplies a
model-specific restriction behind a waterbed-shaped identity. It does not
establish a conservation law connecting output, task, disturbance and test
coverage, predict actual unconverged RL, or locate the exact best-of-n peak.
The largest tested n is a finite mathematical budget, not a practical sampling
recommendation. No initialization MI threshold is tested here.

## Attribution and TP2 boundary

The dependence mechanism is already described by Kwa, Thomas and
Garriga-Alonso, [Catastrophic Goodhart, v2](https://arxiv.org/pdf/2407.14503v2#page=8),
section 5.2 and footnote 6. Their section 3.3 treats the independent light-tail
regime. The binary Gaussian construction gives a directly executable witness;
no novelty claim is made.

The separate TP2 statement is: for **independent** additive noise with a
log-concave density, an arbitrary integrable gold prior has a nondecreasing
posterior mean in score. No differentiability is needed. For discrete noise,
the analogous translation-kernel result needs a compatible common lattice and
interval support without internal zeros, as well as the adjacent log-concavity
inequalities. Conditional Gaussian densities alone do not supply independence.
This experiment does not assume or prove TP2 for the control's noise mixture.

Sources checked as page images: Saumard and Wellner,
[Proposition 2.3, pp. 4-5](https://arxiv.org/pdf/1404.5886#page=4);
Dumbgen and Mosching, [Theorem 13, v1 p. 6](https://arxiv.org/pdf/2209.07868v1#page=6).
Efron's original publisher PDF was blocked during retrieval; its statement was
checked in the review, not independently in the original article.
