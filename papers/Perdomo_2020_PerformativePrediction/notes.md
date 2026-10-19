---
title: "Performative Prediction"
authors: "Juan C. Perdomo, Tijana Zrnic, Celestine Mendler-Dünner, Moritz Hardt"
year: 2020
venue: "ICML 2020, PMLR 119:7599-7609"
doi_url: "https://arxiv.org/abs/2002.06673"
pages: 32
note: "Read from arXiv v4 (dated March 2, 2021). All (p.N) citations are arXiv v4 printed page numbers (page image index + 1), NOT the PMLR proceedings pagination. Theorem numbers are as printed in v4; I have not checked them against the PMLR version."
---

# Performative Prediction

## One-Sentence Summary
Formalizes prediction whose deployment changes the data distribution as a map θ -> D(θ), assumes that map is ε-Lipschitz from Euclidean parameter distance to Wasserstein-1 distance ("ε-sensitivity"), and proves that repeated retraining (RRM) converges linearly to a unique performatively stable point iff-tightly when ε < γ/β (strong convexity over joint smoothness), with stable points within 2L_z ε/γ of performative optima.

## Problem Addressed
When predictions drive decisions, the decisions change the outcome distribution the model is evaluated on ("performativity"). Standard risk minimization assumes a fixed D; ignoring performativity makes it look like distribution shift, which practitioners address with ad-hoc retraining. The paper asks: when do stable points of retraining exist, when does retraining converge, and when are stable points also nearly optimal? *(p.1-2)*

> "When predictions support decisions they may influence the outcome they aim to predict. We call such predictions *performative*; the prediction influences the target." *(p.1)*

> "When ignored, performativity can surface as a form of *distribution shift*. ... Retraining is often considered an undesired — yet necessary — cat and mouse game of chasing a moving target." *(p.2)*

## Key Contributions
- The distribution map D(·) from parameters to distributions over Z=(X,Y) as the key conceptual device; performative risk PR(θ) = E_{Z~D(θ)} ℓ(Z;θ). *(p.2)*
- Two solution concepts: performative optimality (minimize PR) and performative stability (fixed point of retraining); they are distinct in general. *(p.5-6)*
- ε-sensitivity (W1-Lipschitz distribution map) as the core regularity assumption. *(p.7)*
- Theorem 3.5: RRM is an εβ/γ-contraction; converges linearly to a unique stable point if ε < γ/β. *(p.8)*
- Proposition 3.6: tightness — dropping strong convexity, joint smoothness, or ε < γ/β each admits a counterexample where RRM fails to converge. *(p.9)*
- Theorem 3.8: same for repeated gradient descent (RGD) under a stricter ε bound. *(p.10)*
- Theorem 3.10: finite-sample versions (RERM, REGD). *(p.11)*
- Proposition 4.1: existence of stable points under convex, jointly continuous loss and compact Θ (Kakutani). *(p.12)*
- Proposition 4.2: PR can be concave even under all the nice assumptions. *(p.12)*
- Theorem 4.3: ||θ_PO − θ_PS|| ≤ 2 L_z ε / γ. *(p.12)*
- Strategic classification is a special case; Stackelberg equilibria = performative optima; first sufficient conditions for retraining to overcome strategic effects (Corollary 5.1). *(p.13)*
- Semi-synthetic credit-scoring simulation (Kaggle GiveMeSomeCredit). *(p.13-15, 21-23)*

## Study Design (empirical papers)
Not an empirical study in the clinical sense. One semi-synthetic simulation; see Methods.

## Methodology
Decision-theoretic/optimization analysis. Population-level analysis of fixed-point iterations (Banach contraction arguments using strong convexity + Kantorovich-Rubinstein duality + ε-sensitivity), then finite-sample extension via Fournier & Guillin (2015) W1 concentration of empirical measures. *(p.7-11, 23-30)*

## Formal Model of the Decision-Dependent Distribution D(θ)

### Setup
- Parameters θ ∈ Θ ⊆ R^d, Θ closed and convex. Instances z = (x, y), x ∈ R^{m−1}, y ∈ R (so z ∈ R^m). When argmin is non-unique, θ* is an arbitrary point of the argmin set. *(p.5)*
- "For a given choice of parameters θ, we think of D(θ) as the distribution over features and outcomes that results from making decisions according to the model specified by θ. This mapping from predictive model to distribution is the key conceptual device of our framework." *(p.2)*
- "we generally envision that the map D(·) is unknown to the decision maker." *(p.2)*
- Stationarity / no memory: "this mapping D(·) remains fixed and does not change over time or by intervention: deploying the same model at two different points in time must induce the same distribution over observations Z." *(p.4)*. Causal reading: D(θ) is "the interventional distribution over instances Z resulting from a do-intervention that sets the model parameters to θ in some underlying causal graph." *(p.4)*
- Related to concept drift: "an issue we circumvent by assuming that the population distribution is determined by the deployed predictive model." *(p.4)*
- Z-support union: 𝒵 := ∪_{θ∈Θ} supp(D(θ)). *(p.8)*

### Performative risk
$$
\mathrm{PR}(\theta) = \mathbb{E}_{Z \sim \mathcal{D}(\theta)} \ell(Z;\theta)
$$
*(p.2, p.5 Def 2.1)*

### Decoupled performative risk
$$
\mathrm{DPR}(\theta,\theta') = \mathbb{E}_{Z \sim \mathcal{D}(\theta)} \ell(Z;\theta')
$$
First argument: which model's deployment generated the data. Second: which model is evaluated. *(p.6 Def 2.3)*

### Sensitivity assumption (Definition 3.1, ε-sensitivity) *(p.7)*
> "We say that a distribution map D(·) is ε-sensitive if for all θ, θ' ∈ Θ:"

$$
W_1\big(\mathcal{D}(\theta),\mathcal{D}(\theta')\big) \le \varepsilon \lVert \theta - \theta' \rVert_2
$$
Where W1 is the Wasserstein-1 (earth mover's) distance. *(p.7)*

Rationale: "if decisions are made according to similar predictive models, then the resulting distributions over instances should also be similar." *(p.7)*; "Informally, it requires that a small change in model parameters θ does not have an outsized effect on the induced distribution D(θ)." *(p.3)*; "It is easy to see that one cannot make any guarantees on the convergence of retraining or the existence of stable points without making some regularity assumptions on D(·)." *(p.7)*

Dual form (Lemma C.3, Kantorovich-Rubinstein) *(p.23)*: D(·) is ε-sensitive iff for all θ, θ':
$$
\sup_{g\ 1\text{-Lipschitz}} \left| \mathbb{E}_{Z\sim\mathcal{D}(\theta)} g(Z) - \mathbb{E}_{Z\sim\mathcal{D}(\theta')} g(Z) \right| \le \varepsilon \lVert\theta-\theta'\rVert_2
$$

Lemma C.4 *(p.23-24)*: if f: R^n -> R^d is L-Lipschitz and W1(X,X') ≤ C then
$$
\lVert \mathbb{E}[f(X)] - \mathbb{E}[f(X')] \rVert_2 \le L C
$$

Examples of ε-sensitive maps:
- Remark 3.2 (Gaussian family): θ = (μ, σ_1..σ_p) ∈ R^{2p}, D(θ) = N(ε_1 μ, ε_2² diag(σ_1²..σ_p²)); ε-sensitive for ε = max{|ε_1|, |ε_2|}. *(p.7)*
- Strategic best response with linear utility u(θ,x) = −⟨θ,x⟩ and quadratic cost c(x',x) = (1/2ε)||x'−x||²: best response x'_S = x_S − ε θ_S, and W1(D(θ),D(θ')) ≤ ε||θ−θ'||₂ because every individual point moves by ε||θ_S − θ'_S||. Outcomes y unaffected. *(p.14, p.22)* Here ε is literally the inverse cost of manipulation.
- Counterexample constructions: Y ~ point mass at εθ (or 1+εθ), X invariant. *(p.9, p.25)*

## Performative Optimality vs Performative Stability

### Supervised-learning baseline *(p.5)*
$$
\theta_{SL} = \arg\min_{\theta\in\Theta} \mathbb{E}_{Z\sim\mathcal{D}} \ell(Z;\theta)
$$

### Definition 2.1 (performative optimality and risk) *(p.5)*
$$
\theta_{PO} = \arg\min_{\theta} \mathbb{E}_{Z\sim\mathcal{D}(\theta)} \ell(Z;\theta) = \arg\min_\theta \mathrm{PR}(\theta)
$$

### Definition 2.3 (performative stability and decoupled risk) *(p.6)*
$$
\theta_{PS} = \arg\min_{\theta} \mathbb{E}_{Z\sim\mathcal{D}(\theta_{PS})} \ell(Z;\theta)
$$
> "A performatively stable model f_θPS minimizes the expected loss on the distribution D(θ_PS) resulting from deploying f_θPS in the first place. Therefore, a model that is performatively stable eliminates the need for retraining after deployment since any retraining procedure would simply return the same model parameters. Performatively stable models are *fixed points* of risk minimization." *(p.6)*

### Remark 2.4 (both via DPR) *(p.6)*
- θ_PS performatively stable ⇔ θ_PS = argmin_θ DPR(θ_PS, θ)
- θ_PO performatively optimal ⇔ θ_PO = argmin_θ DPR(θ, θ)

> "Observe that performative optimality and performative stability are in general two distinct solution concepts. Performatively optimal models need not be performatively stable and performatively stable models need not be performatively optimal." *(p.6)*

> "We refer to this condition as *performative stability*, noting that it is neither implied by nor does it imply minimal performative risk." *(p.3)*

Abstract framing: "Performative stability implies that the predictions are calibrated not against past outcomes, but against the future outcomes that manifest from acting on the prediction." *(p.1)*

If D(θ) is constant, PO = PS = SL. *(p.6)*

### Example 2.2 (biased coin flip) *(p.5-6)*
X ∈ {±1}, Y | X ~ Bernoulli(1/2 + μX + εθX), μ ∈ (0, 1/2), ε < 1/2 − μ. Predictor f_θ(x) = θx + 1/2, squared loss ℓ(z;θ) = (y − f_θ(x))².
- ε = 0: θ_SL = μ.
$$
\theta_{PO} = \frac{\mu}{1-2\varepsilon}
$$
$$
\theta_{PS} = \frac{\mu}{1-\varepsilon}
$$
(both with θ ∈ [0,1]). PO and PS identify iff ε = 0; PO and SL "diverge as the performativity strength ε increases." *(p.6)*

## Algorithms

### Definition 3.3 (RRM, repeated risk minimization) *(p.8)*
$$
\theta_{t+1} = G(\theta_t) = \arg\min_{\theta\in\Theta} \mathbb{E}_{Z\sim\mathcal{D}(\theta_t)} \ell(Z;\theta)
$$

### Definition 3.7 (RGD, repeated gradient descent) *(p.10)*
$$
\theta_{t+1} = G_{gd}(\theta_t) = \Pi_\Theta\left(\theta_t - \eta\, \mathbb{E}_{Z\sim\mathcal{D}(\theta_t)} \nabla_\theta \ell(Z;\theta_t)\right)
$$
η > 0 fixed step; Π_Θ Euclidean projection. Requires only ∇_θ ℓ, not gradients of PR. *(p.10)*

### Definition 3.9 (RERM, REGD) *(p.11)*
Collect n_t samples from D(θ_t); D^{n_t}(θ_t) the empirical distribution.
$$
\theta_{t+1} = G^{n_t}(\theta_t) = \arg\min_\theta \mathbb{E}_{Z\sim\mathcal{D}^{n_t}(\theta_t)} \ell(Z;\theta)
$$
$$
\theta_{t+1} = G^{n_t}_{gd}(\theta_t) = \Pi_\Theta\left(\theta_t - \eta\, \mathbb{E}_{Z\sim\mathcal{D}^{n_t}(\theta_t)} \nabla_\theta \ell(Z;\theta_t)\right)
$$

## Loss Assumptions *(p.8)*
(A1) β-joint smoothness: ∇_θ ℓ(z;θ) is β-Lipschitz in θ AND in z:
$$
\lVert\nabla_\theta\ell(z;\theta)-\nabla_\theta\ell(z;\theta')\rVert_2 \le \beta\lVert\theta-\theta'\rVert_2
$$
$$
\lVert\nabla_\theta\ell(z;\theta)-\nabla_\theta\ell(z';\theta)\rVert_2 \le \beta\lVert z-z'\rVert_2
$$
for all θ,θ' ∈ Θ, z,z' ∈ 𝒵.

(A2) γ-strong convexity:
$$
\ell(z;\theta) \ge \ell(z;\theta') + \nabla_\theta\ell(z;\theta')^\top(\theta-\theta') + \frac{\gamma}{2}\lVert\theta-\theta'\rVert_2^2
$$
γ = 0 is plain convexity. β/γ is called the condition number. *(p.8)*

## Convergence Theorems (exact statements)

### Theorem 3.5 (RRM) *(p.8)*
> "Suppose that the loss ℓ(z;θ) is β-jointly smooth (A1) and γ-strongly convex (A2). If the distribution map D(·) is ε-sensitive, then the following statements are true:
> (a) ||G(θ) − G(θ')||₂ ≤ ε(β/γ)||θ − θ'||₂, for all θ, θ' ∈ Θ.
> (b) If ε < γ/β, the iterates θ_t of RRM converge to a unique performatively stable point θ_PS at a linear rate: ||θ_t − θ_PS||₂ ≤ δ for t ≥ (1 − εβ/γ)^{-1} log(||θ_0 − θ_PS||₂ / δ)."

$$
\lVert G(\theta)-G(\theta')\rVert_2 \le \varepsilon\frac{\beta}{\gamma}\lVert\theta-\theta'\rVert_2
$$
$$
t \ge \left(1-\varepsilon\frac{\beta}{\gamma}\right)^{-1}\log\left(\frac{\lVert\theta_0-\theta_{PS}\rVert_2}{\delta}\right)
$$
Per-step (eq. 4, p.24):
$$
\lVert\theta_t-\theta_{PS}\rVert_2 \le \left(\varepsilon\frac{\beta}{\gamma}\right)^t\lVert\theta_0-\theta_{PS}\rVert_2
$$
Proof (D.1, p.24): strong convexity gives −γ||G(θ)−G(θ')||² ≥ (G(θ)−G(θ'))ᵀ∇f(G(θ')); (G(θ)−G(θ'))ᵀ∇_θℓ(z;G(θ')) is ||G(θ)−G(θ')||β-Lipschitz in z, so by K-R duality and ε-sensitivity plus first-order optimality, (G(θ)−G(θ'))ᵀ∇f(G(θ')) ≥ −εβ||G(θ)−G(θ')|| ||θ−θ'||. Combine. (b) is Banach fixed point. *(p.9, p.24)*

> "Intuitively, strong convexity forces the iterates to contract after retraining, yet this contraction is offset by the distribution shift induced by changing the underlying model parameters. Joint smoothness and ε-sensitivity ensure that this shift is not too large." *(p.9)*

### Proposition 3.6 (tightness) *(p.9)*
> "Suppose that the distribution map D(·) is ε-sensitive with ε > 0. RRM can fail to converge at all in any of the following cases, for any choice of parameters β, γ > 0:
> (a) The loss is β-jointly smooth and convex, but not strongly convex.
> (b) The loss is γ-strongly convex, but not jointly smooth.
> (c) The loss is β-jointly smooth and γ-strongly convex, but ε ≥ γ/β."

Counterexamples:
- (a) ℓ((x,y);θ) = βyθ, θ ∈ [−1,1]; Y ~ point mass at εθ; DPR(θ,φ) = εβθφ; unique stable point 0; RRM from any nonzero start gives 1, −1, 1, −1, ... for all ε, β > 0. *(p.9)*
- (b) ℓ(z;θ) = C max(−1, yθ) + (γ/2)(θ−1)², Θ ⊇ [−1/(2ε), 1/(2ε)], Y ~ point mass at εθ; θ_0 = 2, large C: oscillates between 2 and −1/(2ε), for all γ, ε > 0. *(p.25)*
- (c) ℓ(z;θ) = (y−θ)², β = γ; Y ~ point mass at 1+εθ; W1 = ε|θ−θ'|; G(θ) = 1+εθ, so |G(θ)−G(θ')| = ε|θ−θ'| (exactly the Thm 3.5 bound). θ_PS = 1/(1−ε) for ε > 1 [sic, as printed]; ε = 1 no stable point; ε > 1 and θ_0 ≠ θ_PS: θ_t -> ±∞ via θ_t = (ε^t − 1)/(ε − 1) + θ_0 ε^t. *(p.25)*

> "strong convexity is in fact *necessary* to guarantee convergence of repeated risk minimization, even for arbitrarily smooth losses and an arbitrarily small sensitivity parameter." *(p.10)*

### Theorem 3.8 (RGD) *(p.10)*
> "Suppose that the loss ℓ(z;θ) is β-jointly smooth (A1) and γ-strongly convex (A2). If the distribution map D(·) is ε-sensitive with ε < γ/((β+γ)(1+1.5ηβ)), then RGD with step size η ≤ 2/(β+γ) satisfies the following:
> (a) ||G_gd(θ) − G_gd(θ')||₂ ≤ (1 − η(βγ/(β+γ) − ε(1.5ηβ² + β)))||θ − θ'||₂ < ||θ − θ'||₂.
> (b) The iterates θ_t of RGD converge to a unique performatively stable point θ_PS at a linear rate, ||θ_t − θ_PS||₂ ≤ δ for t ≥ (1/η)(βγ/(β+γ) − ε(1.5ηβ² + β))^{-1} log(||θ_0 − θ_PS||₂/δ)."

$$
\varepsilon < \frac{\gamma}{(\beta+\gamma)(1+1.5\eta\beta)}, \quad \eta \le \frac{2}{\beta+\gamma}
$$
$$
\lVert G_{gd}(\theta)-G_{gd}(\theta')\rVert_2 \le \left(1-\eta\left(\frac{\beta\gamma}{\beta+\gamma}-\varepsilon(1.5\eta\beta^2+\beta)\right)\right)\lVert\theta-\theta'\rVert_2
$$
Reduces to classical static rate (1 − ηβγ/(β+γ)) at ε = 0 (Nesterov Thm 2.1.15; Hardt-Recht-Singer Lemma 3.7). *(p.10)* Proof (D.3, p.25-28) assumes ε ≤ 1 at one step ("Canceling out ηβ, and assuming ε ≤ 1, it suffices to have 2γ/(β+γ) − 3ηεβ − 2ε > 0", p.27); uses Lemma C.2 (Bubeck Lemma 3.11) co-coercivity and Lemma C.4.

### Theorem 3.10 (finite samples) *(p.11)*
> "Suppose that the loss ℓ(z;θ) is β-jointly smooth (A1) and γ-strongly convex (A2), and that there exist α > 1, μ > 0 such that ξ_{α,μ} := ∫_{R^m} e^{μ|x|^α} dD(θ) is finite ∀θ ∈ Θ. Let δ ∈ (0,1) be a radius of convergence. Consider running RERM or RGD [sic; REGD intended] with n_t = O((1/(εδ)^m) log(t/p)) samples at time t.
> (a) If D(·) is ε-sensitive with ε < γ/(2β), then with probability 1 − p, RERM satisfies ||θ_t − θ_PS||₂ ≤ δ, for all t ≥ log((1/δ)||θ_0 − θ_PS||₂) / (1 − 2εβ/γ).
> (b) If D(·) is ε-sensitive with ε < γ/((β+γ)(1+1.5ηβ)), then with probability 1 − p, REGD with satisfies [sic], ||θ_t − θ_PS||₂ ≤ δ, for all t ≥ log((1/δ)||θ_0 − θ_PS||₂) / (η(βγ/(β+γ) − ε(3ηβ² + 2β))), for a constant choice of step size η ≤ 2/(β+γ)."

Note the finite-sample RERM needs ε < γ/(2β) (factor 2 vs population). Sample size exponential in data dimension m (n_t ∝ (εδ)^{-m}). Per proof D.4 (p.28): n_t ≥ (1/(c_2(εδ)^m)) log(t²π²c_1/(6p)) with c_1, c_2 depending on α, μ, ξ_{α,μ}, m (Fournier & Guillin 2015 Thm 2); per-step failure prob 6p/(π²t²), union bound sums to p. Case 1 (outside δ-ball): W1(D^{n_t}(θ_t), D(θ_PS)) ≤ 2ε||θ_t − θ_PS|| w.h.p. (eq. 6), giving ||θ_{t+1} − θ_PS|| ≤ (2εβ/γ)||θ_t − θ_PS||. Case 2 (inside): ||θ_{t+1} − θ_PS|| ≤ 2ε(β/γ)δ ≤ δ. *(p.28-29)* [Inconsistency in source: the proof overview on p.28 says θ_{t+1} "must be in a (εβ/2γ)δ-ball", while Case 2 on p.29 derives 2ε(β/γ)δ; the Case 2 derivation is the one carried through.]

> "Once this happens, contrary to population-level results, a contraction is no longer guaranteed due to the noise inherent in observing only finite-sample approximations of D(θ_t)." *(p.11)*

## Existence and Optimality-Stability Relation

### Proposition 4.1 (existence) *(p.12)*
> "Let the distribution map D(·) be ε-sensitive and Θ ⊂ R^d be compact. If the loss ℓ(z;θ) is convex and jointly continuous in (z,θ), then there exists a performatively stable point."
Proof via g(θ) = argmin_{θ'} DPR(θ,θ'), Berge's Maximum Theorem (upper hemicontinuous, compact nonempty convex values) and Kakutani's fixed point theorem. *(p.30)* Footnote: performative optima always exist over the extended reals θ ∈ (R ∪ {±∞})^d. *(p.11)*

### Proposition 4.2 (PR can be concave) *(p.12)*
> "The performative risk PR(θ) can be concave in θ, even if the loss ℓ(z;θ) is β-jointly smooth (A1), γ-strongly convex (A2), and the distribution map D(·) is ε-sensitive with ε < γ/β."
Construction (D.6, p.30): Example 2.2 with Θ = [0,1], ε ≥ 1/2, |μ+ε| ≤ 1/2; β/γ = 1;
$$
\mathrm{PR}(\theta) = \frac{1}{4} - 2\theta\mu + (1-2\varepsilon)\theta^2
$$
concave for ε ∈ [1/2, 1). "directly minimizing the performative risk is in general a more challenging problem than finding performatively stable points." *(p.12)*

### Theorem 4.3 (stable points near optima) *(p.12)*
> "Suppose that the loss ℓ(z;θ) is L_z-Lipschitz in z, γ-strongly convex (A2), and that the distribution map D(·) is ε-sensitive. Then, for every performatively stable point θ_PS and every performative optimum θ_PO:"
$$
\lVert\theta_{PO}-\theta_{PS}\rVert_2 \le \frac{2L_z\varepsilon}{\gamma}
$$
Holds without uniqueness of either. Proof (D.7, p.30-31): DPR(θ_PO,θ_PO) ≤ DPR(θ_PS,θ_PS) ≤ DPR(θ_PS,θ_PO); strong convexity + first-order optimality give DPR(θ_PS,θ_PO) − DPR(θ_PS,θ_PS) ≥ (γ/2)||θ_PO − θ_PS||²; ε-sensitivity + L_z-Lipschitz give DPR(θ_PS,θ_PO) − DPR(θ_PO,θ_PO) ≤ L_z ε ||θ_PO − θ_PS||. Note this theorem does NOT require ε < γ/β or smoothness.

### Corollary 5.1 (objective-value gap) *(p.13)*
> "Let the institution's loss ℓ(z;θ) be L_z- and L_θ-Lipschitz in z and θ respectively, β-jointly smooth (A1), and γ-strongly convex (A2). If the induced distribution map is ε-sensitive, with ε < γ/β, then RRM converges at a linear rate to a performatively stable classifier θ_PS that is 2L_z ε(L_θ + L_z ε)γ^{-1} close in objective value to the Stackelberg equilibrium."
$$
\mathrm{PR}(\theta_{PS}) - \mathrm{PR}(\theta_{SE}) \le \frac{2L_z\varepsilon(L_\theta+L_z\varepsilon)}{\gamma}
$$
Proof (D.8, p.31): |PR(θ_PS) − DPR(θ_PS,θ_SE)| ≤ L_θ||θ_SE − θ_PS|| and |DPR(θ_PS,θ_SE) − PR(θ_SE)| ≤ L_z ε ||θ_SE − θ_PS|| (K-R), then Thm 4.3. The proof uses nothing specific to strategic classification, so it applies to any θ_PO in place of θ_SE (my reading of the proof; the paper states it only for Stackelberg).

### Proposition E.1 (regularization for merely convex losses) *(p.31-32)*
> "Suppose that the loss ℓ(z;θ) is L_z-Lipschitz iz [sic] z and L_θ-Lipschitz in θ, β-jointly smooth (A1) and convex (but not necessarily strongly convex). Furthermore, suppose that distribution map D(·) is ε-sensitive with ε < 1, and that the set Θ is bounded with diameter 1. Then, there exists a choice of α, such that running RRM with loss ℓ^reg(z;θ) := ℓ(z;θ) + (α/2)||θ − θ_0||² converges to a performatively stable point θ^reg_PS which satisfies the following"
$$
\mathrm{PR}(\theta^{reg}_{PS}) \le \min_\theta \mathrm{PR}(\theta) + O\left(\frac{\sqrt{\varepsilon}}{1-\varepsilon}\right)
$$
Choice α = √ε β/(1−ε), so ε < α/(α+β). Explicit gap before substitution: 2(L_θ + α + εL_z)L_z ε/α + α/2. *(p.32)* At ε = 0 the limit is performatively optimal. *(p.31)*

## Bounds on How Much a Deployed Model Shifts Its Own Evaluation Distribution
(Collected for the Goodhart / verifier-channel question. Each item is what the paper states or what follows directly from a stated lemma; derivations that are mine are marked.)

1. **Distribution shift per unit parameter change** is bounded by assumption, not derived: W1(D(θ), D(θ')) ≤ ε||θ − θ'||₂ (Def 3.1, p.7). The paper never bounds ε from first principles except in constructed examples (Gaussian family p.7; quadratic-cost strategic response p.14/p.22, where ε is the inverse manipulation cost).
2. **Shift in any Lipschitz evaluation statistic**: |E_{D(θ)} g − E_{D(θ')} g| ≤ ε||θ−θ'|| for 1-Lipschitz g (Lemma C.3, p.23); vector version ||E f(X) − E f(X')|| ≤ L·C (Lemma C.4, p.23). So an L_z-Lipschitz loss evaluated on the induced distribution moves by at most L_z ε ||θ − θ'|| when the deployed model changes: |DPR(θ,φ) − DPR(θ',φ)| ≤ L_z ε ||θ − θ'|| (used in D.7, p.31, and D.8, p.31). [Caveat, mine: W1 controls only Lipschitz-in-z functionals. A 0-1 accuracy or threshold verifier is not Lipschitz, so ε-sensitivity alone does not bound its shift.]
3. **Gap between "calibrated to own induced distribution" and "best possible"**: ||θ_PO − θ_PS|| ≤ 2L_z ε/γ (Thm 4.3, p.12); PR gap ≤ 2L_z ε(L_θ + L_z ε)/γ (Cor 5.1, p.13).
4. **Retraining converges only below a threshold**: ε < γ/β for RRM (Thm 3.5, p.8), tight (Prop 3.6(c), p.9/p.25); stricter γ/((β+γ)(1+1.5ηβ)) for RGD (Thm 3.8, p.10); γ/(2β) for finite-sample RERM (Thm 3.10, p.11). Above threshold, the constructed example θ_t -> ±∞ (p.25), i.e. unbounded drift of the deployed model and hence of its induced distribution.
5. **SL solution vs performative solutions diverge with ε**: θ_SL = μ, θ_PS = μ/(1−ε), θ_PO = μ/(1−2ε) in Example 2.2 (p.5-6). The model trained on the static distribution is mis-specified for the distribution it induces by an amount growing with ε.
6. **Stationarity assumption** (p.4): D(·) is a fixed map with no memory; the same θ always induces the same distribution. Any channel whose response to θ drifts over time is outside this model.
7. The paper does not analyze a verifier/proxy distinct from the loss: the same ℓ both trains and evaluates. The "channel" in this paper is only θ -> D(θ).

## Parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Sensitivity of distribution map | ε | dist/param (W1 per unit L2) | - | ≥0 | 7 | Def 3.1 |
| Joint smoothness constant | β | - | - | >0 | 8 | (A1), Lipschitz of ∇θℓ in θ and z |
| Strong convexity constant | γ | - | - | ≥0 | 8 | (A2) |
| Condition number | β/γ | - | - | ≥1 | 8 | |
| RRM convergence threshold | γ/β | - | - | - | 8 | ε < γ/β; tight (Prop 3.6c) |
| RRM contraction factor | εβ/γ | - | - | <1 | 8 | Thm 3.5(a) |
| RGD step size bound | η | - | - | ≤ 2/(β+γ) | 10 | Thm 3.8 |
| RGD convergence threshold | - | - | - | ε < γ/((β+γ)(1+1.5ηβ)) | 10 | Thm 3.8; as η->0, ε < γ/(β+γ) (p.15) |
| RERM threshold (finite sample) | - | - | - | ε < γ/(2β) | 11 | Thm 3.10(a) |
| RERM per-round samples | n_t | samples | - | O((εδ)^{-m} log(t/p)) | 11 | m = dim of z |
| Lipschitz of loss in z | L_z | - | - | - | 12 | Thm 4.3 |
| Lipschitz of loss in θ | L_θ | - | - | - | 13 | Cor 5.1 |
| PO-PS distance bound | - | param L2 | 2L_z ε/γ | - | 12 | Thm 4.3 |
| PS objective gap vs PO/SE | - | loss | 2L_z ε(L_θ+L_z ε)/γ | - | 13 | Cor 5.1 |
| Regularization weight (App. E) | α | - | √ε β/(1−ε) | - | 32 | Prop E.1 |
| Simulation dataset size | n | points | 18,357 | - | 22 | Kaggle GiveMeSomeCredit subsample, 45% y=1 |
| Number of features | - | count | 10 | - | 22 | 3 strategic: credit-line utilization, # open credit lines, # real estate loans |
| Simulation strong convexity | γ | - | 1000/n | - | 22 | ℓ2 regularization on logistic loss |
| Simulation smoothness | β | - | max{2, (1/4n)Σ||x_i||² + γ} | - | 23 | worst case over trajectory |
| RRM inner solver tolerance | - | - | 1e-8 | - | 23 | GD with backtracking line search |
| RGD step size in sims | η | - | 2/(β+γ) | - | 23 | |
| Simulated sensitivities | ε | - | - | 0.01, 1, 100, 1000 | 14 | Fig 2; ε=80 in Fig 3; ε=25,100 in Fig 4 |

## Effect Sizes / Key Quantitative Results

| Outcome | Measure | Value | CI | p | Population/Context | Page |
|---------|---------|-------|----|---|--------------------|------|
| RRM convergence, ε=0.01 and ε=1 | iterates to numerical zero | < ~5 iterations | - | - | credit sim, Fig 2 left | 14 |
| RRM, ε=100 | normalized ||θ_{t+1}−θ_t|| | linear decrease to ~1e-14 plateau by ~iter 35 | - | - | Fig 2 left (read off plot) | 14 |
| RRM, ε=1000 | normalized ||θ_{t+1}−θ_t|| | flat ~1e-2, no convergence | - | - | Fig 2 left | 14 |
| RGD, ε=0.01, ε=1 | convergence | linear, to numerical zero ~1400 iters | - | - | Fig 2 right | 14 |
| RGD, ε=100 and 1000 | convergence | no convergence (ε=100 jumps up near iter ~1500) | - | - | Fig 2 right | 14 |
| PR during RRM, ε=80 | performative risk | spikes to ~0.623 after first deployment, damped to ~0.610 by iter 30 | - | - | Fig 3 | 15 |
| Accuracy during RRM, ε=80 | accuracy | ~69.2% dip after deployment, ~70.5% after retraining | - | - | Fig 3 | 15 |
| Theory condition satisfied in sims | - | RRM: only ε=0.01; RGD: never | - | - | | 23 |

Note: simulations converge at ε = 1 and ε = 100 for RRM even though the sufficient condition holds only at ε = 0.01 (p.23), so the threshold is loose for this instance (the tightness in Prop 3.6 is worst-case over losses/maps).

## Methods & Implementation Details
- Strategic distribution map sampling procedure (Figure 1): sample (x,y) ~ D; compute x_BR <- argmax_{x'} u(x',θ) − c(x',x); output (x_BR, y). *(p.13)*
- Best response with u(θ,x) = −⟨θ,x⟩, c(x',x) = (1/2ε)||x'−x||²: x'_S = x_S − εθ_S. *(p.14)*
- Base distribution treated as the population (no sampling); features standardized to zero mean, unit variance. *(p.14, p.22)*
- Objective (eq. 1): (1/n)Σ[−y_i θᵀx_i + log(1 + exp(θᵀx_i))] + (γ/2)||θ||². *(p.22)*
- Gradient ∇θℓ(z;θ) = yx + (exp(θᵀx)/(1+exp(θᵀx)))x + γθ [as printed; sign of yx term as in source], 2-Lipschitz in z. *(p.23)*
- Code: https://github.com/zykls/performative-prediction; simulator integrated into WhyNot. *(p.14)*

## Figures of Interest
- **Fig 1 (p.13):** Distribution map for strategic classification (sampling procedure box).
- **Fig 2 (p.14):** Convergence in domain of RRM (left) and RGD (right) for ε ∈ {0.01, 1, 100, 1000}; distance normalized by c = ||θ_{0,S}||^{-1}.
- **Fig 3 (p.15):** PR and accuracy across RRM rounds for ε = 80; green dashed segments show the jump caused by deployment-induced shift, blue the optimization phase.
- **Fig 4 (p.21):** PR surface over 2 features with RRM trajectory: ε = 25 converges to stable point near PO; ε = 100 oscillates.
- **Table 1 (p.20):** Concept-drift applications (Žliobaitė et al.) with performative ones highlighted: fraud detection, crime place prediction, recommendations, gaming the system, etc.

## Results Summary
Under β-joint smoothness, γ-strong convexity, and ε-sensitivity with ε < γ/β, RRM is a contraction with factor εβ/γ and converges linearly to a unique stable point (p.8); each assumption is necessary in the worst case (p.9). Stable points are within 2L_z ε/γ of every performative optimum (p.12). PR can be non-convex (concave) even when everything is nice (p.12), so stability is the tractable target. Simulations: small ε converges in a few rounds, large ε oscillates/diverges (p.14-15, 21).

## Limitations
- D(·) assumed fixed, stateless, and the same across time (p.4); generally unknown to the decision maker (p.2).
- Results need strong convexity; convexity alone is insufficient (p.10, p.15). Merely convex case requires added regularization and Θ of diameter 1 (p.31).
- RGD threshold γ/(β+γ) (as η->0) vs RRM γ/β: "It would be interesting to understand whether this gap is a fundamental difference between both procedures or an artifact of our analysis." (p.15)
- Finite-sample sample complexity exponential in data dimension m (p.11).
- In the simulations, theory's condition held only at ε=0.01 for RRM and never for RGD (p.23); RRM inner minimization is approximate (tolerance 1e-8) (p.23).
- Stable points are not optimal in general; no procedure for computing performative optima is given (p.12).

## Arguments Against Prior Work
- Concept drift literature considers arbitrary sources of shift, which "has led to a number of difficulties in creating a unified language and objective"; discusses retraining but "stops short of discussing the need for stability or analyzing the long-term behavior of retraining." *(p.3-4)*
- Strategic classification results usually depend on a specific cost function; this paper instead relies on a sensitivity assumption. *(p.4)*
- RL/contextual bandits subsume the problem but "we lose many of the intricacies of performativity which make the problem interesting and tractable to analyze." *(p.4)*

## Design Rationale
- W1 chosen because it is "a natural notion of distance between probability distributions that provides access to a rich technical repertoire" and can be verified in concrete settings. *(p.7)*
- Joint smoothness in z is required so that ε-sensitivity (a statement about distributions) translates into bounded movement of expected gradients via K-R duality. *(p.9)*
- RGD introduced as computationally efficient approximation not requiring an exact optimization oracle. *(p.7, p.10)*
- Retraining reframed: "our work interprets retraining as the natural equilibrating dynamic for performative prediction." *(p.15)*

## Testable Properties
- RRM map G is (εβ/γ)-Lipschitz under (A1),(A2), ε-sensitivity. *(p.8)*
- If ε < γ/β, RRM iterates satisfy ||θ_t − θ_PS|| ≤ (εβ/γ)^t ||θ_0 − θ_PS||. *(p.24)*
- For squared loss with Y = 1 + εθ (point mass), RRM diverges for ε > 1 = γ/β and has no stable point at ε = 1. *(p.25)*
- Linear loss βyθ with Y = εθ: RRM oscillates ±1 for all ε, β > 0. *(p.9)*
- ||θ_PO − θ_PS|| ≤ 2L_z ε/γ for every PS and PO. *(p.12)*
- Biased coin: θ_PO = μ/(1−2ε), θ_PS = μ/(1−ε); equal iff ε = 0. *(p.5-6)*
- PR(θ) = 1/4 − 2θμ + (1−2ε)θ² concave for ε ∈ [1/2, 1) with β/γ = 1. *(p.30)*
- RGD contraction factor reduces to 1 − ηβγ/(β+γ) at ε = 0. *(p.10)*
- If D(θ) constant, θ_PO = θ_PS = θ_SL. *(p.6)*

## Relevance to Project
For Goodhart-as-optimization-changing-the-channel: this is the canonical formalism where the thing being optimized (θ) changes the distribution it is scored on. Its central quantitative handle is ε (W1 shift per unit parameter change) against the curvature-to-smoothness ratio γ/β of the loss: retraining settles only when ε < γ/β, and the settled point's loss is within 2L_z ε(L_θ + L_z ε)/γ of the best achievable. The bounds are only as strong as the assumed ε and only control Lipschitz-in-z evaluation functions; nothing in the paper separates a proxy verifier from true quality — loss and evaluation are the same ℓ, and the map D(·) is stationary. So it bounds "self-induced distribution shift of the evaluation data" but not "divergence between proxy score and true quality."

## Open Questions
- [ ] Is the RGD vs RRM threshold gap (γ/(β+γ) vs γ/β) fundamental? (authors' open question, p.15)
- [ ] When do off-policy RL methods converge, via the RRM-as-off-policy-learning reading? (p.16)
- [ ] (Ours) What bound replaces W1-sensitivity for non-Lipschitz verifiers (0-1 pass/fail)?
- [ ] (Ours) Does the PMLR version number theorems the same way as arXiv v4? Not checked.

## Related Work Worth Reading
- Hardt, Megiddo, Papadimitriou, Wootters 2016, Strategic Classification [15].
- Miller, Milli, Hardt 2019, Strategic Classification is Causal Modeling in Disguise [32].
- Milli, Miller, Dragan, Hardt 2019, The Social Cost of Strategic Classification [33].
- Kleinberg & Raghavan 2019, How Do Classifiers Induce Agents to Invest Effort Strategically? [25].
- Fournier & Guillin 2015, W1 rate of convergence of empirical measure [13].
- Bechavod et al. 2020, Causal Feature Discovery through Strategic Modification [4].
- Shavit, Edelman, Axelrod 2020, Learning From Strategic Agents [37].
- Already in collection: Narang et al. 2022, Multiplayer Performative Prediction (papers/Narang_2022_MultiplayerPerformativePredictionLearning).

## Collection Cross-References

### Already in Collection
- (none; no reference in citations.md is in the collection)

### New Leads (Not Yet in Collection)
- Hardt, Megiddo, Papadimitriou & Wootters (2016) - "Strategic Classification" - the Stackelberg formulation this paper subsumes; agents gaming a classifier is the direct Goodhart case.
- Miller, Milli & Hardt (2019) - "Strategic Classification is Causal Modeling in Disguise" - separates gaming from improvement causally, the proxy-vs-true-quality split this paper does not model.
- Kleinberg & Raghavan (2019) - "How Do Classifiers Induce Agents to Invest Effort Strategically?" - which classifiers induce effort on the true target vs on the proxy.
- Fournier & Guillin (2015) - "On the Rate of Convergence in Wasserstein Distance of the Empirical Measure" - W1 concentration used for the finite-sample Theorem 3.10.
- Shavit, Edelman & Axelrod (2020) - "Learning From Strategic Agents: Accuracy, Improvement, and Causality" - accuracy vs improvement vs causality under strategic agents.

### Supersedes or Recontextualizes
- (none)

### Cited By (in Collection)
- [Multiplayer Performative Prediction: Learning in Decision-Dependent Games](../Narang_2022_MultiplayerPerformativePredictionLearning/notes.md) - cites this paper as the single-player origin and generalizes ε-sensitivity and the retraining contraction to n players with aggregate ρ = sqrt(Σ_i (β_i γ_i/α)²); at n = 1, ρ equals this paper's εβ/γ and Narang's ρ < 1 matches ε < γ/β (Narang Thm 1, p.10). Narang's description of this paper is accurate.

### Conceptual Links (not citation-based)
- [Fundamental Limits for Sensor-Based Control via the Gibbs Variational Principle](../Pacelli_2026_FundamentalLimitsSensor-BasedControl/notes.md) - Strong. Both make the distribution a fixed evaluation sees depend on the policy, then solve a self-consistency condition. Here θ -> D(θ) with the shift assumed ε-Lipschitz in W1, and a performatively stable point is a fixed point of retraining, θ_PS = argmin_θ E_{D(θ_PS)} ℓ (Def 2.3). In Pacelli & Theodorou the sensor (h, Σ_v) is fixed but the mutual information it carries is taken under the closed-loop state law and bounded through the cost (coercivity to second moments to Gaussian max-entropy, Pacelli eq. 12-13), closed into the fixed point J_sc = Φ(J_sc) (Pacelli Thm 2). The differences: here ε is an assumption; Pacelli derives the information supply from the cost. Here the fixed point is an equilibrium of a learning dynamic; Pacelli's is the largest cost consistent with the certificate, i.e. a bound. Neither models a proxy diverging from true quality: this paper trains and evaluates with the same ℓ (item 7 above), and Pacelli's channel is not corrupted by pressure on it (Pacelli, Relevance to Project).
- [How much does your data exploration overfit? Controlling bias via information usage](../Russo_2015_MuchDoesDataExploration/notes.md) - Strong. Two change-of-measure routes to the same question: how far can a shift in distribution move an evaluation statistic. Here W1(D(θ), D(θ')) ≤ ε||θ − θ'|| bounds the shift of any L-Lipschitz statistic by L·ε||θ − θ'|| (Lemmas C.3-C.4). Russo & Zou bound the mean shift of a σ-sub-Gaussian statistic between its selected and unconditional laws by σ·sqrt(2·KL) (Russo p.13), and since bounded variables are sub-Gaussian (Russo p.3) this covers 0-1 statistics such as training error (Russo Prop 5), which are the non-Lipschitz verifiers that ε-sensitivity cannot bound (the "(Ours)" open question above). The budgets are in different units (W1 per unit parameter change vs nats), and neither paper relates them. Russo also shows the square-root law becomes linear in I under sub-exponential tails (Russo Prop 9, Thm 2).

---

**See also:** [Multiplayer Performative Prediction: Learning in Decision-Dependent Games](../Narang_2022_MultiplayerPerformativePredictionLearning/notes.md) - extends this paper to n players whose distributions depend on the joint action; the convergence condition becomes ρ = sqrt(Σ_i (β_i γ_i/α)²) < 1, which reduces to ε < γ/β at n = 1 (Narang Thm 1, p.10).
