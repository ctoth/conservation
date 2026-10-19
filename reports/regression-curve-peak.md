# The gold peak is set by the regression curve of gold on proxy

Status: **derived by hand, unchecked numerically and unreviewed.** Answers open questions 2, 3
(partly), 4, 8 (partly) and 13 from `agent-notes/bode-coverage.md`, for two optimizers: the
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
If the noise has heavier tails than g (e.g. Gaussian g, Student-t noise), m is redescending:
m(t) → E[g] as t → ∞. This is the Bayesian outlier-rejection result (Dawid 1973; O'Hagan 1979;
cited from memory, not checked). Then a peak exists and gold eventually returns to its base mean
(see §3: the boundary term is 0, so every gain is given back). If g has the heavier tails, m keeps
increasing and there is no overoptimization. Which side has the heavier tail decides between
diminishing returns and catastrophic Goodhart. Kwa et al. 2024 ("Catastrophic Goodhart", unread)
should be checked against this.

**Q13, the KL-penalized optimum is the tilt.** max_π E_π[r] − β·KL(π ‖ p₀) is the Gibbs
variational principle; its equality condition, "with equality if and only if P(x)/Q(x) ∝
exp(−βℓ)" (checked, Pacelli & Theodorou p. 2, eq. GVP), gives π* ∝ p₀·e^{r/β}, i.e. λ = 1/β.

## 3. A path-independent, Bode-shaped identity (Q8, partly)

Integrate either kernel over the whole optimization path:
- **Tilt:** d/dλ P_λ(R > t) = Cov_λ(1{R > t}, R) = K_λ(t), and P_λ(R > t) runs from 1 − F₀(t) to 1
  as λ goes from 0 to ∞ (for t below the top of the support), so ∫₀^∞ K_λ(t) dλ = F₀(t).
- **Best-of-n:** Σ_{n≥1} uⁿ(1 − u) = u = F₀(t).

So for **both** optimizers, from the base distribution to the top of the proxy's support,

  g(end) − g(start) = ∫ m′(t)·F₀(t) dt = m(r_max) − E₀[m(R)].

The Bode correspondence:

| Bode | here |
|---|---|
| ln\|S(jω)\|, sensitivity at frequency ω | m′(t), gold gained per unit proxy at proxy level t |
| dω (with Bode's weighting) | F₀(t) dt |
| π Σ Re p_unstable | m(r_max) − E₀[m(R)]: what the proxy's top is worth above average |
| independent of the controller (beyond the poles) | independent of the optimizer (tilt or BoN) |

Waterbed reading: when the proxy's top is worth no more than average (m(r_max) ≤ E₀m, the
heavy-tailed-noise case), every unit of gold gained where m rises is given back where m falls,
weighted by F₀.

**Caveat.** Mathematically this is integration by parts. Its content is the path independence
and the identification of the budget term; whether that counts as a conservation law is open.

## Martins & Dahleh check (Q12)

Typeset page (IEEE TAC 53(7), p. 1609), Definition 5.1: channel z(k) = f(k, vᵏ, cᵏ), with f a given
map and the noise process c "assumed mutually independent of x(0) and d"; the encoder chooses
only the input v. The channel's law is fixed. Confirms the reading in the task note.

## To check

- Numerically verify §1 (both rate formulas) and §3 (path identity) on the toy and on a
  Gaussian-gold / Student-t-noise model, including the redescending m. Code; needs Q's go.
- Retrieve Dawid 1973 / O'Hagan 1979 and Kwa et al. 2024.
