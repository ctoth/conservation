---
title: "How much does your data exploration overfit? Controlling bias via information usage"
authors: "Daniel Russo, James Zou"
year: 2015
venue: "arXiv:1511.05219v3 (8 Oct 2019); extended version of AISTATS 2016 'Controlling Bias in Adaptive Data Analysis Using Information Theory', PMLR 51:1232-1240"
doi_url: "https://arxiv.org/abs/1511.05219"
pages: 23
note: "Read from arXiv v3 (23 pp). The AISTATS conference version was NOT read. Page numbers below are the printed page numbers at the top right of each page (= PDF page index + 1)."
---

# How much does your data exploration overfit? Controlling bias via information usage

**Version read:** arXiv:1511.05219v3 [stat.ML], dated 8 Oct 2019. Footnote p.1: "Manuscript received January 30, 2017; revised May 30, 2018; accepted September 12, 2019. Part of this work was presented at AISTATS 2016." The arXiv title differs from the AISTATS title ("Controlling Bias in Adaptive Data Analysis Using Information Theory"); same authors, same arXiv ID. *(p.1)*

## One-Sentence Summary
If each candidate statistic's deviation φ_i − μ_i is σ-sub-Gaussian, the bias of whichever statistic a data-dependent rule T selects obeys |E[φ_T − μ_T]| ≤ σ·sqrt(2·I(T;φ)), proved by applying the Donsker–Varadhan variational form of KL to each conditional law P(φ_i | T=i); for sub-exponential deviations the bound degrades to one *linear* in I(T;φ). *(pp.3, 12-13)*

## Problem Addressed
Adaptive data exploration (choosing which statistic to report after looking at the data) invalidates classical inference and produces selection bias ("Winner's Curse"), but the size of the bias varies by procedure and dataset. The paper gives a single, data-distribution-dependent quantity — the mutual information between the selection and the noisy statistics — that upper-bounds (and in natural cases lower-bounds) the bias of an arbitrary selection procedure. *(pp.1-3)*

## Key Contributions
- Defines **information usage** I(T;φ): mutual information between selection rule T and the vector of estimates φ = (φ_1..φ_m). *(p.3)*
- Prop 1: bias ≤ σ·sqrt(2 I(T;φ)) under sub-Gaussianity. *(p.3)*
- Extensions: unequal variances (Prop 8), sub-exponential (Prop 9), absolute and squared error (Prop 2), KL between selected and fresh-replicate distributions (Prop 10). *(pp.4, 13-15)*
- Lower bounds: for argmax of unit-variance Gaussians, E[(φ_T − μ_T)²] = Θ(1 + H(T)) (Prop 3); threshold selection lower bounds for Gaussian (Thm 1) and shifted-exponential (Thm 2) variables. *(pp.4, 15-18)*
- Applications: filtering by marginal statistics, strong data processing (Prop 4), clustering/visualization, rank selection with signal, LARS, differential privacy, classification overfitting with VC bound (Prop 5), Markov-chain data splitting, FDR (Prop 6), small p-values (Prop 11), Bayesian regret (Prop 12). *(pp.5-9, 18-19)*
- Randomization: Gibbs / exponential-weights selection; Gaussian-noise protocol for a k-step adaptive analyst with error O(σ k^{1/4} / n^{1/2}) (Prop 7 / Prop 13). *(pp.9-11, 20-21)*
- Comparison with max-information: max-information can *increase* with signal while bias decreases. *(pp.21-22)*

## Study Design (empirical papers)
Not an empirical study; theory paper with three small simulations (Figs 1-3). *(pp.6, 7, 10)*

## Methodology
Framework *(p.2)*: dataset D ~ P over 𝒟; analyst considers m functions φ_1..φ_m : 𝒟 → ℝ; reports φ_{T(D)}(D) with T : 𝒟 → {1..m}. m may be exponential in sample size; the φ_i are all estimators the analyst *potentially* could have considered. True value μ_i ≡ E[φ_i(D)]. Bias is E[φ_T − μ_T]; other metrics E|φ_T − μ_T|, E[(φ_T − μ_T)²]. Expectation over D and any internal randomness of T. *(p.2)*

General statement *(p.3)*: φ = (φ_1..φ_m) : Ω → ℝ^m and T : Ω → {1..m} are **any** random variables on a common probability space; μ = E[φ]. No independence among the φ_i is assumed; the bound "do[es] not assume independence and appl[ies] to structured data" *(p.3)*.

Footnote 1 *(p.2)*: if m_0 ≤ m results are reported, bound their average bias by letting T be a random draw from the m_0 selected.

## Key Equations / Statistical Models

### Sub-Gaussian definition (p.3 and footnote 3 p.4)
Main text, p.3: "a real-valued random variable X is σ–sub-Gaussian if for all λ ∈ ℝ,"

$$
\mathbf{E}[e^{\lambda X}] \le e^{\lambda^2\sigma^2/2}
$$

Footnote 3, p.4 (centered form):

$$
\mathbf{E}\left[e^{\lambda(X-\mathbf{E}[X])}\right] \le e^{\sigma^2\lambda^2/2} \quad \text{for all } \lambda
$$

"Zero-mean Gaussian random variables are sub-Gaussian, as are bounded random variables." *(p.3)* "The condition that φ_i is σ-sub-Gaussian ensures that its tail is not too heavy" *(p.4)*.

### Proposition 1 (main bias bound) — exact statement, p.3
"**Proposition 1.** If φ_i − μ_i is σ–sub-Gaussian for each i ∈ {1,...,m}, then,"

$$
\left|\mathbf{E}\left[\phi_T - \mu_T\right]\right| \le \sigma\sqrt{2I(T;\boldsymbol{\phi})}
$$

"where I denotes mutual information." Footnote 2 *(p.3)*:

$$
I(X;Y) = \sum_{x,y}\mathbf{P}(x,y)\log\left(\frac{\mathbf{P}(x,y)}{\mathbf{P}(x)\mathbf{P}(y)}\right)
$$

**Assumptions, exhaustively, as stated:** (i) each centered statistic φ_i − μ_i is σ-sub-Gaussian with a *common* σ (relaxed in Prop 8); (ii) T takes values in a finite set {1..m}; (iii) φ and T are arbitrary random variables on a common probability space — T may use φ, extra data, or independent randomness; no independence across i is required. Nothing else. Note σ is a bound on the MGF *of each marginal*; the joint law of φ is unrestricted. *(p.3)*

What I(T;φ) is: "the dependence of the selection process on the noise in the estimates" — called **information usage** or "bad information usage". *(p.3)* When T is a deterministic function of φ, I(T;φ) = H(T) *(p.4)*. Note: it is information between the selection and the *noisy statistics* φ, not between T and the true means μ (μ is a constant here).

Sample-mean corollary *(p.4)*: if φ_i = n^{-1} Σ_j f_i(X_j) with iid X_j and f_i(X_j) − E[f_i(X_j)] σ-sub-Gaussian, then φ_i − μ_i is σ/√n-sub-Gaussian and

$$
|\mathbf{E}[\phi_T] - \mathbf{E}[\mu_T]| \le \sigma\sqrt{\frac{2I(T;\boldsymbol{\phi})}{n}}
$$

*(p.4)*

### Fact 1 — Donsker–Varadhan variational form (p.12)
Cited as Theorem 5.2.1 of Gray, *Entropy and Information Theory* [34]. "**Fact 1.** Fix two probability measures P and Q defined on a common measurable space (Ω, 𝓕). Suppose that P is absolutely continuous with respect to Q. Then"

$$
D(\mathbf{P}\|\mathbf{Q}) = \sup_X\left\{\mathbf{E}_\mathbf{P}[X] - \log\mathbf{E}_\mathbf{Q}[e^X]\right\}
$$

"where the supremum is taken over all random variables X such that the expectation of X under P is well defined, and e^X is integrable under Q." *(p.12)* (The paper does not call this "Donsker–Varadhan"; it calls it "the following variational form of Kullback–Leibler divergence".)

### Proof of Proposition 1 (pp.12-13), step by step
Step 1 — MI as average KL, then drop to marginals (information processing) *(p.12)*:

$$
I(T;\boldsymbol{\phi}) = \sum_{i=1}^{n}\mathbf{P}(T=i)D\left(\mathbf{P}(\boldsymbol{\phi}=\cdot|T=i)\,\|\,\mathbf{P}(\boldsymbol{\phi}=\cdot)\right)
$$

$$
I(T;\boldsymbol{\phi}) \ge \sum_{i=1}^{n}\mathbf{P}(T=i)D\left(\mathbf{P}(\phi_i=\cdot|T=i)\,\|\,\mathbf{P}(\phi_i=\cdot)\right)
$$

(Sum index printed as n, though T ranges over {1..m}.)

Step 2 — change of measure: "Applying Fact 1 with P = P(φ_i = ·|T = i), Q = P(φ_i = ·), and X = λ(φ_i − μ_i), we have" *(p.13)*

$$
D\left(\mathbf{P}(\phi_i=\cdot|T=i)\,\|\,\mathbf{P}(\phi_i=\cdot)\right) \ge \sup_\lambda \lambda\Delta_i - \lambda^2\sigma^2/2
$$

"where Δ_i ≡ E[φ_i | T = i] − μ_i." (The −λ²σ²/2 term is the sub-Gaussian MGF bound on log E_Q[e^{λ(φ_i−μ_i)}] under the *unconditional* law Q.) "Taking the derivative with respect to λ, we find that the optimizer is λ = Δ_i/σ²." This gives *(p.13)*:

$$
2\sigma^2 I(T;\boldsymbol{\phi}) \ge \sum_{i=1}^{n}\mathbf{P}(T=i)\Delta_i^2 = \mathbf{E}[\Delta_T^2]
$$

Step 3 — tower property + Jensen *(p.13)*:

$$
\mathbf{E}[\phi_T - \mu_T] = \mathbf{E}[\Delta_T] \le \sqrt{\mathbf{E}[\Delta_T^2]} \le \sigma\sqrt{2I(T;\boldsymbol{\phi})}
$$

Because λ ranges over all of ℝ, the per-i bound holds for Δ_i of either sign, which gives the absolute value in the statement.

**Remark on looseness (p.13, verbatim gist):** step 1 uses D(P(φ|T=i) || P(φ)) ≥ D(P(φ_i|T=i) || P(φ_i)) (information processing), which "is not tight in general and can lead to gaps between the actual bias and our upper bound based on I(T;φ)". Counterexample: φ_1 ~ Uniform[0,1], T a deterministic function of φ_1 choosing among {2..m} (T=2 if φ_1 ∈ [0,1/(m−1)], etc.); I(T;φ) = log m, but if φ_j (j≠1) are independent of each other and of φ_1, D(P(φ_i|T=i)||P(φ_i)) = 0 and bias is 0. "In cases where there is a gap, we may study P(T=i)D(P(φ_i=·|T=i)||P(φ_i=·)) directly." *(p.13)*

### Proposition 8 — unequal variances (p.13)
"Suppose that for each i ∈ {1,...,m}, φ_i − μ_i is σ_i–sub-Gaussian. Then,"

$$
|\mathbf{E}[\phi_T] - \mathbf{E}[\mu_T]| \le \sqrt{\mathbf{E}[\sigma_T^2]}\sqrt{2I(T;\boldsymbol{\phi})}
$$

Proof: per i, sup_λ λΔ_i − λ²σ_i²/2 with optimizer λ_i = Δ_i/σ_i², giving

$$
\Delta_i \le \sigma_i\sqrt{2D\left(\mathbf{P}(\phi_i=\cdot|T=i)\,\|\,\mathbf{P}(\phi_i=\cdot)\right)}
$$

then E[Δ_T] = Σ Δ_i P(T=i) ≤ Σ σ_i P(T=i) sqrt(2D(...)) and Cauchy–Schwarz gives sqrt(Σ σ_i² P(T=i)) × sqrt(2 Σ P(T=i) D(...)) = sqrt(E[σ_T²]) sqrt(2I(T;φ)). (The printed Cauchy–Schwarz line writes "σ_i² P(φ_i = ·|T = i)" inside the first root and "P(φ_i = ·|T=i) D(...)" in the second; from context these weights are P(T=i). Typo in the paper.) *(p.13)*

### Sub-exponential definition and Proposition 9 — the heavier-tail result (p.13-14)
Definition *(p.13)*: "X is sub-exponential with parameters (σ, b) if"

$$
\mathbf{E}[e^{\lambda(X-\mathbf{E}[X])}] \le e^{\lambda^2\sigma^2/2} \quad \text{whenever } |\lambda| < 1/b
$$

Example: X ~ χ²_n with n ≥ 1 is sub-exponential with parameters (2√n, 4). *(p.13)*

"**Proposition 9.** Suppose that for each i ∈ {1,...,m}, φ_i − μ_i is sub-exponential with parameters (σ, b). Then"

$$
\mathbf{E}[\phi_T - \mu_T] \le bI(T;\boldsymbol{\phi}) + \frac{\sigma^2}{2b}
$$

"Moreover, if b < 1, we also have"

$$
\mathbf{E}[\phi_T - \mu_T] \le \sqrt{b}I(T;\boldsymbol{\phi}) + \frac{\sigma^2}{2\sqrt{b}}
$$

*(p.13)*

Proof *(pp.13-14)*: same DV step, but λ restricted:

$$
D\left(\mathbf{P}(\phi_i=\cdot|T=i)\,\|\,\mathbf{P}(\phi_i=\cdot)\right) \ge \sup_{\lambda<1/b} \lambda\Delta_i - \lambda^2\sigma^2/2
$$

"The RHS is greater than the value from setting λ = 1/b":

$$
D\left(\mathbf{P}(\phi_i=\cdot|T=i)\,\|\,\mathbf{P}(\phi_i=\cdot)\right) \ge \frac{\Delta_i}{b} - \frac{\sigma^2}{2b^2}
$$

Multiply by P(T=i), sum:

$$
I(T;\boldsymbol{\phi}) \ge \frac{\mathbf{E}[\phi_T - \mu_T]}{b} - \frac{\sigma^2}{2b^2}
$$

When b < 1, λ = 1/√b < 1/b is feasible and gives the second bound. *(p.14)*

**What this says (the paper's own result):** with only a finite MGF in a window |λ| < 1/b, the bias bound is affine in I(T;φ), not square-root. Note the paper evaluates at a *fixed* λ (1/b or 1/√b) and does not optimize λ over the window; it does not state the two-regime (sqrt for small I, linear for large I) form. [Our observation, not stated in the paper: optimizing sup_{λ<1/b} λΔ − λ²σ²/2 gives Δ²/(2σ²) when Δ ≤ σ²/b, recovering σ·sqrt(2I) for small enough Δ, and the linear regime only beyond that. Unverified beyond this algebra.]

**No result in the paper for distributions without any finite MGF** (e.g., polynomial / power-law tails, Pareto, Student-t, infinite variance). Sub-exponential is the heaviest class treated. The Appendix overview calls sub-exponential "heavier tales" [sic] *(p.12)*. The lower-bound side does treat shifted exponentials (Thm 2), where bias is shown to be at least *linear* in entropy H(T) — see below.

### Proposition 2 — absolute and squared error (p.4; proofs pp.14)
Main-text statement *(p.4)*: "Suppose for each i ∈ {1,...,m}, φ_i − μ_i is σ sub-Gaussian. Then"

$$
\mathbf{E}[|\phi_T - \mu_T|] \le \sigma + c_1\sigma\sqrt{2I(T;\boldsymbol{\phi})}
$$

$$
\mathbf{E}[(\phi_T - \mu_T)^2] \le 1.25\sigma^2 + c_2\sigma^2 I(T;\boldsymbol{\phi})
$$

"where c_1 < 36 and c_2 ≤ 10 are universal constants." *(p.4)*

Part (1) appendix statement *(p.14)*: E[|φ_T − μ_T|] ≤ σ + c·σ sqrt(2I(T;φ)), c < 36. Proof: U_i = φ_i − μ_i, γ_i = E|φ_i − μ_i|, Y_i = |U_i| − γ_i; show Y_i is cσ-sub-Gaussian, c ≤ 36, then by Prop 1 and data processing:

$$
\mathbf{E}[|\phi_T - \mu_T| - \gamma_T] = \mathbf{E}[Y_T] \le c\sigma\sqrt{2I(T;\boldsymbol{Y})} \le \sqrt{2I(T;\boldsymbol{\phi})}
$$

(As printed, the last term drops the cσ factor; the next line restores it: E|φ_T − μ_T| ≤ σ + 36σ sqrt(2I(T;φ)). Typo in the paper.) *(p.14)* Uses Facts from Wainwright [35]: Fact 1: if P(|Y| ≥ s) ≤ c P(|Z| ≥ s) for Z ~ N(0,τ²), c ≥ 1, then Y is sqrt(2c)τ sub-Gaussian; Fact 2: zero-mean σ-sub-Gaussian Y has P(|Y| ≥ s) ≤ √(8e) P(|Z| ≥ s), Z ~ N(0,2σ²). Result: Y is sub-Gaussian with parameter 2(√(8e) + 1/P(|Z| ≥ γ))σ; with γ ≤ σ, P(|Z| ≥ γ) > P(|Z| ≥ σ) > 0.1, giving 36σ. *(p.14)* Sample-mean version: E|φ_T − μ_T| ≤ σ/√n + c·(σ/√n) sqrt(2I(T;φ)). Interpreted as "similar to a bias-variance decomposition". *(p.14)*

Part (2) *(p.14)*:

$$
\mathbf{E}[(\phi_T - \mu_T)^2] \le \sigma^2\left(1.25 + 10I(T;\boldsymbol{\phi})\right)
$$

Proof uses Fact 3 [35]: for zero-mean σ-sub-Gaussian Y,

$$
\mathbf{E}\left[e^{\frac{\lambda Y^2}{2\sigma^2}}\right] \le \frac{1}{\sqrt{1-\lambda}} \quad \text{for all } \lambda\in[0,1)
$$

then

$$
\mathbf{E}\left[e^{\frac{\lambda(Y^2-\gamma)}{2\sigma^2}}\right] \le \frac{1}{\sqrt{1-\lambda}} \le e^{10\lambda^2} \quad \text{for } \lambda\in[0,0.1)
$$

("the last inequality can be verified numerically"), so Y² − γ is sub-exponential with parameters (√5 σ², 10σ²). Applying Prop 9 to Y_i²:

$$
\mathbf{E}[(\phi_T - \mu_T)^2] \le \sigma^2 + 10\sigma^2 I(T;\boldsymbol{Y}^2) + \frac{\sigma^2}{4} \le \sigma^2\left(1.25 + 10I(T;\boldsymbol{\phi})\right)
$$

*(p.14)* — i.e., the squared-error result is itself an instance of the *linear-in-I* sub-exponential bound.

### Proposition 10 — KL between selected and replicate (p.15)
"Let φ̃ denote a random variable drawn from the marginal distribution of φ, but drawn independently of T and φ. Then"

$$
D\left(\mathbf{P}(\phi_T=\cdot)\,\|\,\mathbf{P}(\tilde\phi_T=\cdot)\right) \le I(T;\boldsymbol{\phi})
$$

Proof: data processing for KL, joint (φ_T, T), chain rule, then D(P(φ_i|T=i)||P(φ_i)) ≤ D(P(φ|T=i)||P(φ)), summing to I(T;φ). *(p.15)* Context: φ̃ = "these same estimates computed on a fresh replication data-set D̃". *(pp.14-15)*

### Corollary 1 — top-m_0 average (p.4)
"Suppose for each i ∈ {1,...,m}, φ_i is a zero-centered sub-Gaussian random variable with parameter σ. Let φ_(1) ≥ φ_(2) ≥ ... ≥ φ_(m) denote the values of φ_i sorted from the largest to the smallest. Then"

$$
\mathbf{E}\left[\frac{1}{m_0}\sum_{i=1}^{m_0}\phi_{(i)}\right] \le \sigma\sqrt{2\log\frac{m}{m_0}}
$$

From I(T;φ) = H(T) − H(T|φ) = log m − log m_0 for uniform random pick among top m_0. Tight as m/m_0 → ∞ (Appendix C-A, p.15: random partition into m_0 groups, group leaders converge to σ sqrt(2 log m/m_0)). *(pp.4, 15)*

### Max of Gaussians example (p.4)
φ_i iid N(0,σ²), T = argmax: I(T;φ) = H(T) = log m; Prop 1 gives E[φ_T] ≤ σ sqrt(2 log m), "the well known inequality for the maximum of Gaussian random variables", approaching equality as m grows, "implying that the information usage I(T;φ) precisely measures the bias of max-selection in this setting". *(p.4)*

### Proposition 3 — lower bound, argmax of unit Gaussians (p.4; proof pp.15-16)
"Let T = argmax_{1≤i≤m} φ_i where φ ~ N(μ, I). There exist universal numerical constants c_1 = 1/8, c_2 < 2.5, c_3 = 10, and c_4 = 1.5 such that for any m ∈ ℕ and μ ∈ ℝ^m,"

$$
c_1H(T) - c_2 \le \mathbf{E}[(\phi_T - \mu_T)^2] \le c_3H(T) + c_4
$$

*(p.4)* Entropy *(p.5)*:

$$
H(T) = \sum_i\mathbf{P}(T=i)\log\left(\frac{1}{\mathbf{P}(T=i)}\right)
$$

Intuition *(p.5)*: m=2, φ_1 = x, φ_2 ~ N(0,1): log(1/P(T=2)) = log(1/P(φ_2 ≥ x)) ≈ x²/2, so surprise scales with squared gap; as x → ∞, H(T_x) ~ P(T_x=2) log(1/P(T_x=2)) ~ P(T_x=2) x² ~ E[(φ_{T_x} − μ_{T_x})²]. Covariance N(μ, I) means *independent* unit-variance Gaussians. Proof uses Fact 2 (max of Gaussians is 1-sub-Gaussian; M_{−i} = max_{j≠i} φ_j likewise) and Fact 3 (Mills-type lower tail P(X > x) ≥ (1/√(2π)) (x/(x²+1)) e^{−x²/2}). Splits i ∈ I ≡ {i : E[M_{−i}] ≥ μ_i + 1} vs not; c_{−I} = log(1/p(1)) < 5; final H(T) ≤ c_{−I} + 5 + 8 + 8||φ_T − μ_T||², i.e. ||φ_T − μ_T||² ≥ c_1 H(T) − c_2 with c_1 = 1/8, c_2 = (c_{−I} + 13)/8 < 2.5. Statement: "E[(φ_T − μ_T)²] = Θ(1 + H(T))". *(pp.15-16)*

### Theorem 1 — threshold-M selection, Gaussian (p.17)
Setup *(p.16)*: each φ_i Gaussian with variance 1, arbitrary means, may be correlated. Threshold-M rule: if at least one φ_i > M, pick uniformly among those (excluding φ_{−1}); else report a fixed φ_{−1}. N_{−i} = |{φ_j ≥ M, j ≠ i, j ≠ −1}|; N̂(M) = max_i E[N_{−i} | φ_i ≥ M].

"**Theorem 1.** Suppose"

$$
M - \max_i\mu_i \ge \sqrt{2\log[2\pi(1+\mathbf{E}[N_{-i}|\phi_i\ge M])(M-\max_i\mu_i)] + 3}
$$

"then"

$$
\mathbf{E}[(\phi_T - \mu_T)^2] \ge H(T)
$$

*(p.17)* Uses (φ_i ~ N(μ_i,1), M > μ_i):

$$
\frac{(M-\mu_i)^2}{2} \ge \log\frac{1}{\mathbf{P}(\phi_i\ge M)} - \log(M-\mu_i) - \frac{1}{2}\log(2\pi) - \frac{1}{(M-\mu_i)^2}
$$

*(p.17)* Remark: can be improved using I(T|Φ) = H(T) − H(T|Φ), H(T|Φ) = E[log N | N ≥ 1] with N = |{φ_i ≥ M}| (independent case). *(p.17)*

### Theorem 2 — threshold-M selection, shifted exponential (p.17-18)
Setup: φ_i = λ_i + exp(1), density e^{−(x−λ_i)} for x ≥ λ_i; may be correlated; μ_i = λ_i + 1. *(p.17)*

"**Theorem 2.** Suppose M − max λ_i ≥ 4 + 2 log(1 + N̂(M)),"

$$
\mathbf{E}[\phi_T - \mu_T] \ge H(T)/2
$$

*(p.17)* Proof: log 1/P(φ_i > M) = M − λ_i, so H(T) = Σ p_i (M − λ_i) + Σ p_i log(1/E[1/(1+N_{−i}) | φ_i ≥ M]) + (1−p) log(1/(1−p)); and E[φ_T − μ_T] ≥ Σ p_i (M − μ_i) = Σ p_i (M − λ_i) − p ≥ H(T) − Σ log(1 + E[N_{−i} | φ_i ≥ M]) − 2p ≥ H(T)/2. *(pp.17-18)*

**Relevance:** for exponential-tailed statistics, the *bias itself* (not squared error) is lower-bounded linearly in H(T) (= I(T;φ) here since T is a function of φ). So the linear-in-I upper bound of Prop 9 is the right order for exponential tails in this family; the σ·sqrt(2I) shape fails for them. The paper does not state this juxtaposition explicitly; it follows from Prop 9 + Thm 2 as stated.

### Filtering by marginal statistics (p.5)

$$
I(T;\boldsymbol{\phi}) = H(T) - H(T|\boldsymbol{\phi}) \le H(T) - I(T;D|\boldsymbol{\phi}) = (1-\alpha)H(T)
$$

α = I(T;D|φ)/H(T) = fraction of uncertainty in T explained by data beyond φ. Variance-based filtering T = argmax_i V_i, V_i = Σ_j (X_{i,j} − φ_i)²: for iid Gaussian samples, sample means are independent of sample variances, so I(T;φ) = 0 and no bias. "the bias bound depends on I(T;φ) instead of I(T;D)". Markov chain T − ψ − φ gives I(T;φ) ≤ I(ψ;φ). *(pp.5-6)*

### Definition 1 — strong data processing (p.6)
"A pair of random variables (X,Y) satisfies a strong data-processing inequality with contraction coefficient η ∈ [0,1] if for all random variables U with U − X − Y"

$$
I(U;Y) \le \eta I(U;X)
$$

η_XY smallest such constant. Tensorizes; η_XZ ≤ η_YZ for X − Y − Z. Subsample of k of n iid points: η_{ψφ} ≤ η_{ψD} ≤ k/n [25]. BSC with error δ: η = (1−2δ)² [26]. *(p.6)*

### Proposition 4 (p.6)
"Suppose φ_i − μ_i is σ sub-Gaussian for each i ∈ {1,..,m}. Then if the selection T is independent of φ conditioned on ψ,"

$$
\mathbf{E}[\phi_T - \mu_T] \le \sigma\sqrt{2\eta_{\psi\phi}I(T;\psi)}
$$

*(p.6)*

### Proposition 5 — classification overfitting (p.8; proof p.18)
Fixed inputs x_i, labels Y_i ∈ {−1,1} independent. Training error L̂(f) = (1/n) Σ 1(f(x_i) ≠ Y_i); true error L(f) = E[L̂(f)] = (1/n) Σ P(f(x_i) ≠ Y_i). *(p.7)* "Let x ≡ (x_1..x_n), Y ≡ (Y_1..Y_n), f̂(x) ≡ (f̂(x_1)..f̂(x_n)) and log_+(z) ≡ max{1, log(z)}. Then,"

$$
\mathbf{E}[L(\hat f) - \hat L(\hat f)] \le \sqrt{\frac{I(\hat f(\mathbf{x});\mathbf{Y})}{2n}}
$$

"If 𝓕 has VC-dimension d < ∞, then"

$$
I(\hat f(\mathbf{x});\mathbf{Y}) \le d\log_+\left(\frac{ne}{d}\right)
$$

*(p.8)* Proof (p.18): Bernoulli centered is sub-Gaussian with parameter < 1/4 [36]; average sub-Gaussian ≤ 1/(4√n) [printed "σ ≤ 1/2n"]; Sauer: S_𝓕(n) ≤ 2^n if n < d, (en/d)^d if n ≥ d. (Note log_+ defined as max{1, log z} on p.8 but max{0, log x} elsewhere, p.9.) *(pp.8, 18)*

### Markov-chain data splitting (p.8)
Uniform mixing:

$$
\max_s D(\mathbf{P}(s_\tau=\cdot|s_1=s)\,\|\,\pi) \le c_0e^{-c_1\tau} \quad \forall\tau\in\mathbb{N}
$$

Then

$$
I(T;\boldsymbol{\phi}) \le c_0e^{-c_1(n_2-n_1)}
$$

*(p.8)*

### Proposition 6 — FDR (p.9; proof p.18)

$$
I(T;\boldsymbol{\phi}) \le h(\mathrm{FDR}) + (1-\mathrm{FDR})\cdot\log\left(\frac{1}{1-\beta}\right) + \mathrm{FDR}\cdot\log\left(\frac{1}{\alpha}\right) + \xi
$$

h binary entropy; α = E[α̂], β = E[β̂]; α̂ = #(H_0 ∩ S_1)/#H_0, β̂ = #(H_1 ∩ S_0)/#H_1; FDR = P(T ∈ H_0) with T uniform over selected S_1.

$$
\xi = \mathbf{E}\left[\log_+\left(\frac{1-\beta}{1-\hat\beta}\right)\right] + \mathbf{E}\left[\log_+\left(\frac{\alpha}{\hat\alpha}\right)\right]
$$

log_+(x) ≡ max{0, log(x)}. *(p.9)*

### Gibbs / exponential-weights randomized selection (p.9)
Maximize H(π) over π ∈ ℝ^m_+ subject to Σ_{i=1}^k π_i φ_i ≥ b, Σ π_i = 1. Solution:

$$
\pi_i^* \propto e^{\beta\phi_i} \qquad i\in\{1,..,m\}
$$

(eq. 2), β > 0 chosen so Σ π_i* φ_i = b. "one of the most widely studied algorithms in the field of online learning [31], where it is often called exponential weights. It is also known as the exponential mechanism in differential privacy." *(p.9)* Unless degenerate:

$$
I(T;\boldsymbol{\phi}) = H(T) - H(T|\boldsymbol{\phi}) \le \log(m) - H(T|\boldsymbol{\phi}) < \log(m)
$$

Caveat: "the Gibbs mechanism described above does not reduce bias or information usage for all possible data–generating distributions because it could increase entropy H(T)." *(p.10)* Simulation scheme: sample K indices without replacement from π*; bias (1/K) Σ (φ_{T_i} − μ_{T_i}); accuracy |{T_i : T_i ≤ N_1}|/K. N_1 = 1000, n − N_1 = 100000, β = 2, μ from 1 to 5, K = 100. *(p.10)*

### Multi-step adaptive analyst (pp.10-11, 20-21)
Model: step k, choose T_k from {Y_{T_1}, T_1, ..., Y_{T_{k−1}}, T_{k−1}}; after K steps select φ_T ≡ φ_{T_{K+1}}. Markov chain T_{k+1} ← H_k ← D → φ. *(pp.10-11)*

Lemma 1 *(p.11)*:

$$
I(T_{k+1};\boldsymbol{\phi}) \le I(H_k;\boldsymbol{\phi}) = \sum_{i=1}^{k}I(Y_{T_i};\phi_{T_i}|H_{i-1},T_i)
$$

"information budget" I_b; per-step cost I(Y_{T_i}; φ_{T_i} | H_{i−1}, T_i) accumulates additively. *(p.11)*

Distortion / selection-bias split *(p.11)*:

$$
\mathbf{E}[|Y_{T_k} - \mu_{T_k}|] - \sigma \le \mathbf{E}[|Y_{T_k} - \phi_{T_k}|] + c\sigma\sqrt{2I(T_k;\boldsymbol{\phi})}
$$

Lemma 2 *(p.11)*: X ~ N(0,σ_1²), Y = X + W, W ~ N(0,σ_2²) independent:

$$
I(X;Y) = \frac{1}{2}\log(1+\beta) \le \frac{\beta}{2}
$$

β = σ_1²/σ_2² (SNR). Lemma 3 *(p.20)*: X real with variance σ_X², W ~ N(0,σ_W²) independent: I(X; X+W) ≤ σ_X²/σ_W² (proof via convexity of KL).

Proposition 7 *(p.11)*: φ_i ~ N(μ_i, σ²/n), jointly Gaussian; Y_{T_j} = φ_{T_j} + W_j, W_j ~ N(0, σ²√j/n) independent of φ; then for every k,

$$
\mathbf{E}[|Y_{T_{k+1}} - \mu_{T_{k+1}}|] \le c\left(\frac{\sigma k^{1/4}}{n^{1/2}}\right)
$$

Without noise, error can be Ω(σ sqrt(k/n)) (Example 1 p.21: θ̂ ~ N(0, σ²/n I), X = θ̂/||θ̂||, E[X^T θ̂] = Θ(σ sqrt(k/n))). k^{1/4} "worst-case price of adaptivity"; all queries accurate as long as k = o(n²). *(pp.11-12, 21)*

Proposition 13 *(p.20)* (general form): ω_j general,

$$
\mathbf{E}[|Y_{T_{k+1}} - \mu_{T_{k+1}}|] \le \frac{\sigma}{\sqrt{n}} + c_1\left(\frac{\omega_{k+1}}{\sqrt{n}} + \sigma^2\sqrt{\frac{\sum_{j=1}^{k}w_j^{-2}}{n}}\right)
$$

with ω_j = σ j^{1/4} giving c_2 σ k^{1/4}/n^{1/2}. Proof uses I(Y_{T_i}; φ_{T_i} | H_{i−1}, T_i) ≤ σ²/(2ω_i²), so I(T_{k+1}; φ) ≤ (σ²/2) Σ ω_i^{−2}. *(pp.20-21)*

### Proposition 11 — small p-values (p.19)
"Define Z_{ε,i} = 1(φ_i < ε) and let Z_ε = (Z_{ε,1},...,Z_{ε,m}). If φ_i ~ Uniform(0,1) for all i then"

$$
\mathbf{P}(p_T < \epsilon) \le \epsilon + \sqrt{\frac{I(T;\mathbf{Z}_\epsilon)}{\log(1/2\epsilon)}}
$$

Uses Bernoulli(p) sub-Gaussian parameter σ = sqrt((1−2p)/(2 log((1−p)/p))) ≤ sqrt(1/(2 log(1/2p))) [36]. *(p.19)* Note: this is a case where sub-Gaussian parameter of an indicator shrinks as p → 0, so I can be large while bias stays small.

### Proposition 12 — Bayesian regret / value of information (p.19)
"If for each x ∈ 𝒳, f_θ(x) − μ(x) is σ sub-Gaussian, then"

$$
\mathbf{E}[\max_{x\in\mathcal{X}} f_\theta(x)] - \max_{x\in\mathcal{X}}\mu(x) \le \sigma\sqrt{2H(X^*)}
$$

X* ∈ argmax f_θ; proof: ≤ E[f_θ(X*)] − E[μ(X*)] ≤ σ sqrt(2I(X*;θ)) = σ sqrt(2H(X*)). *(p.19)*

### Max-information comparison (pp.21-22)

$$
I_\infty(T;\boldsymbol{\phi}) = \max_{i,\boldsymbol{y}}\log\left(\frac{\mathbf{P}(\boldsymbol{\phi}=\boldsymbol{y},T=i)}{\mathbf{P}(\boldsymbol{\phi}=\boldsymbol{y})\mathbf{P}(T=i)}\right) = \max_i\log\left(\frac{1}{\mathbf{P}(T=i)}\right)
$$

For the planted-signal example, I_∞ = log((m−1)/P(T ≠ I*)), which *increases* toward infinity as μ grows, while I(T;φ) = H(T) and bias decrease. Approximate max-information:

$$
I_\infty^\beta(T;\boldsymbol{\phi}) := \max_{\mathcal{O}\subset[m]\times\mathbb{R}^m,\ \mathbf{P}((T,\boldsymbol{\phi})\in\mathcal{O})\ge\beta}\log\left(\frac{\mathbf{P}((T;\boldsymbol{\phi})\in\mathcal{O})-\beta}{\mathbf{P}((T;\tilde{\boldsymbol{\phi}})\in\mathcal{O})}\right)
$$

Lemma 4: T = f(φ) deterministic ⇒ I^β_∞(T;φ) ≥ max_{i: P(T=i) ≥ 2β} log(1/P(T=i)) − log 2. *(pp.21-22)*

## Parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Sub-Gaussian scale | σ | units of φ | - | >0 | 3 | Common across i in Prop 1 |
| Sub-exponential window | b | units of φ | - | >0 | 13 | MGF bound only for abs(λ) < 1/b |
| Abs-error constant | c_1 (Prop 2) | - | <36 | - | 4, 14 | universal |
| Squared-error constant | c_2 (Prop 2) | - | ≤10 | - | 4, 14 | universal; variance term 1.25σ² |
| Prop 3 lower slope | c_1 | - | 1/8 | - | 4 | argmax of N(μ, I) |
| Prop 3 lower offset | c_2 | - | <2.5 | - | 4 | |
| Prop 3 upper slope | c_3 | - | 10 | - | 4 | |
| Prop 3 upper offset | c_4 | - | 1.5 | - | 4 | |
| Prop 3 c_{-I} | c_{-I} | - | <5 | - | 16 | log(1/p(1)) |
| Theorem 2 threshold margin | M − max λ_i | units of φ | - | ≥ 4 + 2 log(1 + N̂(M)) | 17 | exponential case |
| Noise schedule (Prop 7/13) | ω_j | units of φ | σ j^{1/4} | - | 11, 20 | W_j ~ N(0, ω_j²/n) |
| χ²_n sub-exponential params | (σ, b) | - | (2√n, 4) | n ≥ 1 | 13 | example |
| Y² − γ sub-exponential params | (σ, b) | - | (√5σ², 10σ²) | - | 14 | for σ-sub-Gaussian Y |
| Gibbs simulation β | β | 1/units of φ | 2 | - | 10 | Fig 3 |
| Gibbs simulation sizes | N_1, n − N_1, K | count | 1000, 100000, 100 | - | 10 | Fig 3 |
| Rank-selection sim m | m | count | 1000 | - | 6 | Fig 1, 1000 runs, μ ∈ [1,4] |
| LARS sim design | X | - | 100×1000 iid N(0,1) | - | 19 | 20 nonzero β = s |
| LARS signal levels | s | - | 0.04, 0.06, 0.08 | - | 20 | low/medium/high; noise N(0, I_100/10) |

## Effect Sizes / Key Quantitative Results

| Outcome | Measure | Value | CI | p | Population/Context | Page |
|---------|---------|-------|----|---|--------------------|------|
| Max of m iid N(0,σ²) | bias bound | σ sqrt(2 log m) | - | - | approaches equality as m grows | 4 |
| Adaptive p-value, min of 5 | false rejection prob at .05 | 1 − .95^5 ≈ .23 | - | - | | 19 |
| Min of m iid U(0,1) | E[φ_T] | 1/(m+1) | - | - | P(φ_T ≤ 1/m) → 1 − 1/e | 19 |
| Multi-step with Gaussian noise | abs error | O(σ k^{1/4}/n^{1/2}) | - | - | vs Ω(σ sqrt(k/n)) with no noise | 11-12 |

Figures report simulations qualitatively (see below); no numeric tables.

## Methods & Implementation Details
- To estimate information usage empirically in LARS, they "use bootstrapping" *(p.7)*; details not given beyond Appendix G setup *(pp.19-20)*.
- LARS bias quantified on univariate regression coefficients β̂_{T_i} − β*_{T_i} *(p.20)*.
- Randomized max-entropy scheme: inputs β, K, φ; sample K indices without replacement from π* ∝ e^{βφ_i} *(p.10)*.

## Figures of Interest
- **Fig 1 (p.6):** Rank selection with one planted signal, m=1000: H(T) (red, ~6.9 down to ~2.4), information bound sqrt(2I(T;φ)) (blue, ~3.7 to ~2.2), and bias (black, ~3.2 to ~1.2) all decrease as μ goes 1 → 4. Bound tracks bias with a roughly constant gap.
- **Fig 2 (p.7):** LARS, bias (solid) vs information bound (dashed) per number of selected features 1-20, three SNRs; both decrease with signal and with number of features; bound above bias throughout (bias ~0.10-0.15, bound ~0.135-0.16).
- **Fig 3 (p.10):** Accuracy-bias tradeoff; max-selection (blue) vs max-entropy (red) across μ = 1..5; max-entropy has lower bias at every μ (e.g. ~1.4-2.0 vs ~1.75-3.25).

## Results Summary
Bias of an arbitrary selection is controlled by I(T;φ) at rate σ sqrt(2I) under sub-Gaussianity *(p.3)*; this is tight for max of iid Gaussians *(p.4)* and of matching order for squared error under argmax of independent unit Gaussians *(p.4)*. Under sub-exponential tails the bound is affine in I *(p.13)*, and for shifted-exponential threshold selection bias is ≥ H(T)/2 *(p.17)*. Signal reduces H(T) and hence bias *(p.6)*; randomization reduces I *(pp.9-11)*.

## Limitations
- Information processing step (joint → marginal KL) "is not tight in general" *(p.13)*.
- Gibbs randomization does not reduce bias for all distributions since it can raise H(T) *(p.10)*.
- Lower bounds only established when selection is an optimization (T = argmax) / threshold rule; "An interesting direction of research is to understand more general exploration procedures in which information usage provides a tight approximation to bias." *(p.12)*
- Stylized clustering example; "if the analyst uses additional information beyond the number of clusters K, then the bias could increase" *(p.6)*.
- Filtering bound via I(ψ;φ) "may be quite loose" *(p.6)*.
- No treatment of tails heavier than sub-exponential (stated by absence; nothing in pp.1-23 covers it).

## Arguments Against Prior Work
- Selective inference [8-11] requires P(φ_i = · | T = i) known and T mathematically tractable and pre-specified *(p.3)*.
- Algorithmic stability is worst-case and distribution-agnostic, traditionally for prediction *(p.3)*.
- Differential privacy / max-information are worst-case, adversarial; DP algorithms have low information usage for *all* datasets — "a much stricter condition" *(p.7)*; max-information can increase as bias falls *(p.21)*.
- VC bounds depend only on 𝓕 and cannot reflect the data distribution or fitting procedure *(p.8)*.

## Design Rationale
- Use I(T;φ), not I(T;D): selection may depend heavily on D while carrying little information about the reported statistics *(p.5)*.
- Mutual information chosen because it is amenable to calculation (data processing, chain rule, SDPI, composition) *(pp.6, 11, 12)*.
- Average-case (on average over selected) guarantees remain meaningful as m → ∞ *(p.9)*.

## Testable Properties
- If T ⟂ φ then E[φ_T] = E[μ_T] *(p.4)*.
- Sub-Gaussian σ: |E[φ_T − μ_T]| ≤ σ sqrt(2 I(T;φ)) for any T *(p.3)*.
- Sub-exponential (σ,b): E[φ_T − μ_T] ≤ b I + σ²/(2b); if b<1 also ≤ √b I + σ²/(2√b) *(p.13)*.
- Max of m iid N(0,σ²): E[max] ≤ σ sqrt(2 log m) *(p.4)*.
- Top-m_0 average ≤ σ sqrt(2 log(m/m_0)) *(p.4)*.
- E[(φ_T − μ_T)²] ≤ σ²(1.25 + 10 I) *(p.14)*.
- Argmax of N(μ, I): H(T)/8 − 2.5 ≤ E[(φ_T − μ_T)²] ≤ 10 H(T) + 1.5 *(p.4)*.
- Shifted exponentials, threshold M with margin ≥ 4 + 2 log(1 + N̂(M)): E[φ_T − μ_T] ≥ H(T)/2 *(p.17)*.
- KL(law of φ_T || law of fresh φ̃_T) ≤ I(T;φ) *(p.15)*.
- Gaussian noise with ω_j = σ j^{1/4}: E|Y_{T_{k+1}} − μ_{T_{k+1}}| ≤ c σ k^{1/4}/√n *(p.11)*.

## Relevance to Project
Context: conjecture that a verifier with I nats of mutual information about true quality can carry ~I nats of KL optimization against it, and that heavy tails break this.

What the paper actually establishes, mapped carefully:
1. **The quantity is different.** The paper's I is I(T;φ): information the *selection* carries about the *noisy statistics* φ (the noise, since μ is fixed). It is not mutual information between a verifier and true quality. Bias here = how much the selected noisy score exceeds that item's true mean — closest analogue is proxy overestimation of the chosen item. *(pp.2-3)*
2. **The proof is a change-of-measure argument per arm:** DV with X = λ(φ_i − μ_i), P = law of φ_i given T=i, Q = unconditional law; the cumulant generating function under Q is controlled by sub-Gaussianity, so KL ≥ Δ²/(2σ²). *(p.13)* This is exactly the "KL budget buys at most σ·sqrt(2·KL) of mean shift" transportation inequality; the I in the bound enters because I(T;φ) = E_T[KL(P(φ|T) || P(φ))]. A KL-regularized optimizer's tilt KL(π || π_0) plays the role of the per-selection KL, so the paper supports "gain ≤ σ sqrt(2·KL)" for sub-Gaussian noise; it does not itself make the RLHF/verifier connection.
3. **Heavy tails:** the only heavier class treated is sub-exponential, where the bound becomes *linear* in I *(p.13)*, and Thm 2 shows linear-in-H(T) bias is actually attained for shifted exponentials under threshold selection *(p.17)*. So exponential tails already break the square-root law (bias per nat no longer shrinks). For tails with no finite MGF (power laws), the DV argument as written gives nothing — log E_Q[e^{λ(φ−μ)}] = ∞ for all λ > 0 — and the paper offers no result. This is consistent with (but does not prove) "heavy tails break it".
4. **Gibbs/exponential weights** π* ∝ e^{βφ} appears as the max-entropy selection *(p.9)*: the same form as the KL-regularized optimum, used here to reduce I. Useful bridge but not quantified in KL terms.

## Open Questions
- [ ] The paper does not optimize λ within the sub-exponential window; the two-regime bound (sqrt for small Δ, linear beyond σ²/b) is our algebra, unverified against any source.
- [ ] No result for infinite-MGF tails; need another source (e.g., Bu/Zou/Veeravalli 2020, Jiao et al. 2017, or CVaR/Rényi-based variants) for heavy-tailed generalization bounds.
- [ ] Is the AISTATS version's Prop 1 identical? Not checked (conference version not read).

## Related Work Worth Reading
- Dwork et al. 2015 "Generalization in adaptive data analysis and holdout reuse" [18]; "The reusable holdout" [19]; STOC 2015 [28] — max-information.
- Gray, *Entropy and Information Theory* [34] Thm 5.2.1 — the variational KL formula used.
- Wainwright "Basic tail and concentration bounds" [35] — sub-Gaussian/sub-exponential facts.
- Polyanskiy & Wu 2016 [26]; Anantharam et al. [24]; Kamath & Nair [25] — strong data processing.
- McAllester PAC-Bayes tutorial [15].
- Cesa-Bianchi & Lugosi [31] — exponential weights.
