# The gold peak is set by the regression curve of gold on proxy

Status: **derived by hand; §1 and §3 checked numerically (see "Numeric check" below); not
reviewed by anyone else.** Corrected 2026-10-21 after an outside check (§2 Q3, §3, peak KL).
Answers open questions 2, 3 (partly), 4 and 13 from `agent-notes/bode-coverage.md` (not 8; see
§3), for two optimizers: the
KL-regularized tilt and best-of-n. Follows `reports/goodhart-toy.md`.

## Setup

Responses x ~ p₀. Proxy score r(x) with base CDF F₀; gold g(x). Define the base regression curve

  m(t) := E₀[g | r = t].

Both optimizers select using r only:
- **Tilt:** π_λ(x) ∝ p₀(x)·e^{λ r(x)}.
- **Best-of-n:** draw n i.i.d. candidates from p₀, keep the one with the largest r.

**Key fact.** Because the selection depends on x only through r, the conditional law of g given r
is unchanged: E_π[g | r] = m(r) for every π_λ and every n. All the optimizer changes is the
distribution of r.

## 1. Rate of gold change: a non-negative average of m′

**Tilt.** d/dλ E_λ[g] = Cov_λ(g, R) (derivative of an exponential family), and by the key fact
Cov_λ(g, R) = Cov_λ(m(R), R). Hoeffding's covariance identity then gives

  d/dλ E_λ[g] = ∫ m′(t)·K_λ(t) dt,  K_λ(t) := E_λ[(R − E_λR)·1{R > t}] ≥ 0.

(K_λ(t) ≥ 0: for t ≥ E_λR the integrand is non-negative; for t < E_λR,
K_λ(t) = −E_λ[(R − E_λR)·1{R ≤ t}] ≥ 0.)

**Best-of-n.** In quantile coordinates u = F₀(r), m̃(u) := m(F₀⁻¹(u)), the selected quantile has
density n·u^{n−1}, so E_n[g] = ∫₀¹ m̃(u)·n·u^{n−1} du. Since (n+1)uⁿ − n·u^{n−1} = d/du(u^{n+1} − uⁿ)
and u^{n+1} − uⁿ vanishes at 0 and 1, integration by parts gives

  E_{n+1}[g] − E_n[g] = ∫₀¹ m̃′(u)·uⁿ(1 − u) du,

a Beta(n+1, 2)-weighted average of the slope, concentrated near the top 1/n quantile.

## 2. Consequences

**Monotone criterion.** If m is non-decreasing, neither optimizer can ever lower gold, at any λ
or n. Overoptimization requires m to decrease somewhere. Jointly Gaussian (r, g): m is linear, so
no peak, as derived earlier. The toy (`reports/goodhart-toy.md`): r ∈ {0.2, 0.9, 1} with
m = {0, 1, 0}, non-monotone, so it peaks.

**Q4, predicting the peak.** The peak is where the optimizer's kernel (K_λ, or uⁿ(1 − u)) moves
onto the region where m′ < 0, so it is fully determined by the initial joint law of (r, g)
through m and F₀. A single scalar such as I₀ = I(r; g) cannot determine it: two joint laws with
the same mutual information can have monotone and non-monotone m. This is why the β check on
Gao et al. and the toy both found the peak far from KL ≈ I₀.

**Q2, non-binary scores.** The quantity that marks the peak is the kernel-weighted slope of m,
not mutual information. Cov_π(g, r) is how it appears for the tilt. With binary gold,
m(t) = P(good | r = t).

**Q3, heavy tails (mechanism, partly from memory).** Take r = g + noise, noise independent of g.
The first version of this paragraph said "noise heavier-tailed than g ⇒ m redescending". That is
false: Laplace(0.5) noise is heavier-tailed than Gaussian g, yet m is non-decreasing and
saturates at 1/b = 2 (checked numerically, see "Numeric check"). The split the grid models
support is whether the noise is log-concave, giving three regimes for Gaussian g:

| noise | m | optimization |
|---|---|---|
| Gaussian (log-concave, light) | linear, unbounded | gains without limit |
| Laplace (log-concave, exponential tails) | non-decreasing, bounded (→ 1/b) | diminishing returns, nothing given back |
| Student-t₃ (not log-concave) | redescending, m(t) → E[g] | peak, then every gain given back |

Redescending m under t-noise is the Bayesian outlier-rejection result (Dawid 1973; O'Hagan 1979;
cited from memory, not checked). "Log-concave noise ⇒ m non-decreasing for every gold law" is
Karlin's total positivity, also from memory (see "Numeric check"). Kwa et al. 2024 ("Catastrophic
Goodhart") is still unread. A secondhand reading of its abstract splits heavy-tailed from
light-tailed error. Laplace error has an MGF near 0, so it counts as light-tailed, yet its gain is
capped. Check the theorem's tail definition against this before using either split.

**Q13, the KL-penalized optimum is the tilt.** max_π E_π[r] − β·KL(π ‖ p₀) is the Gibbs
variational principle; its equality condition, "with equality if and only if P(x)/Q(x) ∝
exp(−βℓ)" (checked, Pacelli & Theodorou p. 2, eq. GVP), gives π* ∝ p₀·e^{r/β}, i.e. λ = 1/β.

## 3. An endpoint identity, not a conservation law (Q8 stays open)

**Correction (2026-10-21).** By the key fact, E_π[g] = E_π[m(R)] for every π an r-only optimizer
reaches. So any path from p₀ to the point mass at r_max changes gold by exactly
m(r_max) − E₀[m(R)], and the F₀(t) weighting below is just integration by parts of that
difference. The identity constrains only the two endpoints. Bode's integral constrains every
admissible controller at every operating point. Nothing here is an analogue of that, so this
section does not answer Q8. The derivation is kept because the kernels it integrates are
checked.

Integrate either kernel over the whole optimization path:
- **Tilt:** d/dλ P_λ(R > t) = Cov_λ(1{R > t}, R) = K_λ(t), and P_λ(R > t) runs from 1 − F₀(t) to 1
  as λ goes from 0 to ∞ (for t below the top of the support), so ∫₀^∞ K_λ(t) dλ = F₀(t).
- **Best-of-n:** Σ_{n≥1} uⁿ(1 − u) = u = F₀(t).

So for **both** optimizers, from the base distribution to the top of the proxy's support,

  g(end) − g(start) = ∫ m′(t)·F₀(t) dt = m(r_max) − E₀[m(R)].

The first version set this beside Bode's integral (m′ ↔ ln|S|, F₀ dt ↔ dω, m(r_max) − E₀m ↔
π Σ Re p_unstable) and read it as a waterbed. That reading holds only at the endpoint, which
light-tailed noise never reaches in practice (Numeric check, n = 10¹² row). It is withdrawn.

## Martins & Dahleh check (Q12)

Typeset page (IEEE TAC 53(7), p. 1609), Definition 5.1: channel z(k) = f(k, vᵏ, cᵏ), with f a given
map and the noise process c "assumed mutually independent of x(0) and d"; the encoder chooses
only the input v. The channel's law is fixed. Confirms the reading in the task note.

## Numeric check

Script: `research/regression_curve_check.py` (stdlib; Q's "make it so", 2026-10-19). Run:
`python research/regression_curve_check.py` (~7 s). Models: the toy (ε = 1e-6), and r = g + noise
on a grid (g ∈ [−8, 8] step 0.05, r ∈ [−30, 30] step 0.1) for three gold/noise pairs. A grid model
is a discrete model, so the identities should hold to rounding.

**§1 and §3 hold.** Worst absolute gaps over λ ∈ {0.01, 0.3, 1, 3, 10} and n ∈ {1, 2, 5, 20, 100, 1000}:

| model | tilt rate: central difference vs Σ Δm·K_λ (Cov agrees to 1e-9) | BoN increment | tilt path: ∫₀^Λ K_λ dλ vs ΔP(R > t) | budget m(r_max) − E₀m | tilt at the top |
|---|---|---|---|---|---|
| toy | 3.9e-12 | 8.3e-17 | 5.4e-9 | −0.3000 | −0.3000 |
| N(0,1) gold, N(0, 0.5) noise | 2.3e-11 | 2.5e-16 | 4.5e-12 | +7.998 | +7.998 |
| N(0,1) gold, t₃(0.5) noise | 1.4e-9 | 8.6e-17 | 3.7e-11 | +0.134 | +0.134 |
| t₃(1) gold, N(0,1) noise | 7.0e-10 | 3.2e-16 | 3.3e-11 | +7.974 | +7.974 |

Best-of-n at n = 10¹² reaches the budget for the toy and the t₃-noise model but not for the
two Gaussian-noise models (6.36, 7.85). There the top levels carry mass ~10⁻³⁰⁰, so no
practical n reaches them. The identity is a limit statement, and with light-tailed noise the
limit is effectively unreachable.

**The key fact E_π[g | r] = m(r) holds independently.**
- Toy: best-of-n via m matches the closed form P(best is good) = (1−ε)ⁿ − (0.7−ε)ⁿ for
  n from 1 to 10⁷.
- Grid models: Monte Carlo best-of-n sampled from the joint (20000 trials, n ∈ {1, 4, 16, 64})
  is within 4 standard errors of the m formula in all 12 cases. The largest deviation is
  2.4 se (t₃ gold, n = 64).

**Shape of m (§2).**
- Gaussian noise: m(t) = 0.8t to 1e-6 for |t| < 5, the linear 1/(1+σ²) slope, as predicted.
- t₃ gold with Gaussian noise: non-decreasing on the whole grid.
- Gaussian gold with t₃ noise: m rises to 1.666 at t = 3.1, then falls monotonically:
  m(8) = 0.54, m(16) = 0.25, m(30) = 0.13. It is redescending toward E g = 0, as predicted.
  The +0.134 budget is m at the grid edge, an artifact of truncation; untruncated, the budget is
  lim m − E g = 0.

The Gaussian-noise rows fit a more general statement than the one in §2: log-concave noise
gives a monotone likelihood ratio in (t, g), so m is non-decreasing for every gold law. Under
that statement, log-concave noise means no overoptimization, whatever gold's tails are. This
is Karlin's total-positivity result, which I'm citing from memory and have not checked.

Laplace noise (added 2026-10-21, one-off probe through this script's `grid_model`, Gaussian
gold, b = 0.5): m is non-decreasing on the whole grid, m(1) = 0.731, m(3) = 1.790, and m = 2.000
from t = 6 to 30. That is the saturation value 1/b. The t₃ model in the same probe: m(3) = 1.662,
m(6) = 0.777, m(30) = 0.134. So heavier-than-gold noise tails are not enough for
redescending m; §2 Q3 is corrected accordingly. The Laplace model is not yet in the script.

**Gold peaks.** Gold is measured relative to E₀g. BoN is swept over n = 2^k, so its peak is
located only to a factor of 2.

| model | I₀ (nats) | tilt peak | BoN peak |
|---|---|---|---|
| toy | 0.229 | +0.70 at KL 1.204 | +0.70 at KL 1.204 |
| Gaussian noise | 0.805 | none (monotone) | none (monotone) |
| t₃ noise | 0.581 | +0.346 at KL ≈ 1.6 (λ ≈ 0.386; see below) | +1.47 at KL 3.17 (n = 64) |
| t₃ gold | 0.560 | none (monotone) | none (monotone) |

The t₃ tilt's KL at the peak is ill-conditioned. Near the peak, gold is flat to 3e-4 over
λ ∈ [0.38, 0.41], while KL climbs from 1.43 to 2.72 as tilted mass runs to the truncation edge
(P(r > 20) goes from 0.135 to 0.252). The script's λ step of 0.05 lands on λ = 0.40, KL 2.215. The
truncation table's step of 0.01 lands on λ = 0.39, KL 1.785. Same model, different sweep step.
A step of 0.0025 puts the peak at λ ≈ 0.385–0.3875, KL ≈ 1.6–1.7. Since the untruncated tilt does
not exist (below), the tilt's peak KL for this model carries no information beyond "of order 1–2
nats on this grid".

The BoN KL values are the exact discrete KL of the selected-level law on the grid, not the
atomless formula log n − (n−1)/n. At n = 64 they differ by 0.002 (3.1726 vs 3.1745).

1. Overoptimization appears only in the redescending case, under both optimizers, as the
   monotone criterion says.
2. The peak is again not at KL ≈ I₀. KL*/I₀ is 5.3 (toy), about 5.5 (t₃ BoN), and about 2.8 (t₃
   tilt, with the conditioning caveat above).

**New: the tilt does not exist under heavy-tailed noise.** A t₃ proxy has E₀[e^{λR}] = ∞ for
every λ > 0. So the KL-penalized objective sup_π E_π r − β·KL equals β·log E₀e^{r/β} = ∞, and it
has no optimum. On the grid the tilt exists only because r is truncated, and its peak moves with
the truncation point:

| r_max | λ at peak | gold at peak | KL at peak |
|---|---|---|---|
| 10 | 0.93 | +0.77 | 2.47 |
| 20 | 0.53 | +0.46 | 1.87 |
| 30 | 0.39 | +0.35 | 1.79 |

Best-of-n stays well defined. So with heavy-tailed noise, the "tilt" half of §1–§3 describes a
truncated model, not the real one. This looks like the setting of Kwa et al. 2024, which is
still unread.

## To check

- Retrieve Dawid 1973 / O'Hagan 1979 (redescending posterior mean), Karlin (log-concave ⇒ MLR ⇒
  monotone posterior mean), and Kwa et al. 2024. Compare Kwa's claim with the tilt
  non-existence above and with the Laplace (bounded, monotone) regime.
- Manheim & Garrabrant 2018: does "monotone m with slope < 1" match their regressional Goodhart,
  and "non-monotone m" their extremal Goodhart? Suggested in an outside check from memory;
  unchecked.
