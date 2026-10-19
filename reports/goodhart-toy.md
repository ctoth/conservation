# Goodhart toy: a fixed verifier whose channel moves with the policy

Script: `research/goodhart_toy.py` (stdlib only, exact arithmetic, no sampling). Run:
`python research/goodhart_toy.py`. Built on Q's "make it so", 2026-10-19, to probe gap 1 of
`reports/pacelli-policy-dependent-sensor.md`: why proxy optimization would destroy the
verifier's information.

## Model

- Response kinds: good (gold 1, base mass 0.3, passes w.p. a = 0.9), bad (gold 0, mass 0.7 − ε,
  passes w.p. b = 0.2), exploit (gold 0, mass ε, always passes).
- Verifier: a binary verdict with pass probability s(x). It never changes.
- Optimizer: the KL-regularized pass-rate maximizer π_λ(x) ∝ p₀(x)·e^{λ s(x)}, swept over λ.
- The channel p_π(verdict | gold) is induced by π. Q's framing: optimization acts on the
  channel, not the verifier.

## Results (from the run)

| ε | log(1/ε) | I₀ (nats) | KL at gold peak | gold peak | KL when gold < base | p(pass \| gold 0) at peak |
|---|---|---|---|---|---|---|
| 1e-2 | 4.61 | 0.221 | 1.125 | 0.924 | 2.98 | 0.900 |
| 1e-3 | 6.91 | 0.228 | 1.193 | 0.989 | 4.59 | 0.900 |
| 1e-4 | 9.21 | 0.229 | 1.203 | 0.999 | 6.21 | 0.901 |
| 1e-6 | 13.82 | 0.229 | 1.204 | 1.000 | 9.43 | 0.900 |
| 1e-9 | 20.72 | 0.229 | 1.204 | 1.000 | 14.30 | 0.901 |

I₀ = 0.2291 also checked by hand from the initial joint (0.27, 0.03, 0.14, 0.56).

1. **The peak is not at KL ≈ I₀.** It sits at KL ≈ log(1/p_good) = 1.204, about 5× I₀. In this toy
   the peak is set by how much mass the optimizer has to move onto good responses, not by the
   verifier's initial information. Same direction as the β check on Gao et al.

2. **Exact: the gold peak is where the verdict stops correlating with gold under π.** For the
   tilted family, d/dλ E_π[gold] = Cov_π(gold, s), and Cov_π(gold, s) = Cov_π(gold, verdict)
   because E[verdict | x] = s(x). So gold rises while the verdict is positively correlated with
   gold under the current policy and peaks where that covariance is zero. With binary verdict
   and binary gold, zero covariance is independence, so I_π(verdict; gold) = 0 exactly at the
   peak. The script asserts the covariance changes sign at the peak for every ε (passes), and the
   peak sits at pass rate = a = 0.9, where p_π(pass | bad) = p_π(pass | good). Algebra: at
   E_π[s] = a, π_exploit(1 − a) = π_bad(a − b), so p_π(pass | bad) = (0.2 + 7)/8 = 0.9.

3. **The channel breaks before gold falls.** The false-pass rate among bad responses goes
   0.20 → 0.90 by the peak and → 1.00 after, while gold is still ≈ 1 (bad responses have almost no
   mass, so nobody sees it). Gold drops below its base rate only at KL ≈ 0.69·log(1/ε), 2–13 nats
   later.

4. **Unsigned mutual information cannot detect Goodhart.** I_π(verdict; gold) goes 0.229 → 0 at the
   peak, then *rises* to 0.036 while gold collapses (the verdict is now anti-informative:
   covariance negative, passing indicates bad), then → 0 again once gold is constant at 0. MI is
   zero both at the best policy and at the worst. The signed quantity, Cov_π(gold, verdict), is the
   one that tracks the optimum.

5. The transportation bound gold ≤ 0.3 + ½√(2·KL) (binary gold is ½-sub-Gaussian) holds everywhere
   (asserted) and is loose (1.076 at the peak, vs gold ≤ 1).

## What carries beyond the toy

Finding 2 uses only the tilted form of π, so it holds for any proxy s and gold g, not just this
three-point model: along π_λ ∝ p₀·e^{λ s}, gold peaks where Cov_π(g, s) = 0. The KL-penalized RL
objective's optimal policy has this tilted form (standard; from memory, not re-derived here), so
the prediction for a Gao-style replication is: **along the KL-regularized optimum, the gold
peak falls where the proxy–gold covariance under the optimized policy crosses zero.** That
replaces "KL ≈ I₀". It says nothing about best-of-n (not a tilt) or about RL runs that have not
converged to the tilted optimum. For non-binary scores, zero covariance does not mean zero MI.

## Not shown

- Nothing here links the peak's KL to any information quantity at initialization; in this toy
  the peak KL is set by the base rate of good responses.
- Heavy tails: the toy's gold is bounded. The exploit's collapse is the discrete analogue of a
  heavy right tail in proxy error.
