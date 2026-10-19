---
title: "Fundamental Limits for Sensor-Based Control via the Gibbs Variational Principle"
authors: "Vincent Pacelli, Evangelos A. Theodorou"
year: 2026
venue: "IEEE Control Systems Letters (accepted); arXiv:2603.18454v3 [math.OC], 7 Jul 2026"
doi_url: "https://doi.org/10.1109/LCSYS.2026.3710535"
pages: 6
affiliation: "Daniel Guggenheim School of Aerospace Engineering, Georgia Institute of Technology"
note: "Read from arXiv v3 PDF. Page citations p.N are PDF pages 1-6 (the preprint has no printed page numbers)."
---

# Fundamental Limits for Sensor-Based Control via the Gibbs Variational Principle

## One-Sentence Summary
A lower bound on the optimal expected cost of any causal output-feedback controller, obtained by applying the Gibbs variational principle (GVP) to the joint state/observation path measure against a reference measure in which observations are decoupled from state; the sensor's information budget is then made self-consistent with the cost itself (a good controller concentrates the state, which caps the mutual information the sensor can carry), giving a fixed-point bound J_sc that is unique, found by bisection, and strictly tighter than using the open-loop information budget. *(p.1, p.3, p.4)*

## Problem Addressed
General-purpose tools for lower-bounding achievable feedback performance under noisy sensing are scarce. The prior general approach (TRIP, refs [5],[6]) needs bounded costs and a finite control space and "computes mutual information (MI) using the *open-loop* state distribution—which arises in the absence of feedback". Because good feedback controllers concentrate the state in low-cost regions, "the open-loop distribution vastly overestimates the required sensor information. This structural issue is worst when feedback control is the most effective." *(p.1)*

## Key Contributions
- **TRFE (task-relevant free energy) lower bound** (Theorem 1) on the minimum expected cost of any causal feedback controller under partial observations; applies to nonlinear, nonholonomic, hybrid dynamics with unbounded costs and continuous control spaces. *(p.1, p.2)*
- **Self-consistent refinement** (Theorem 2): fixed-point equation J = Φ(J) with a unique solution that improves on the open-loop estimate. *(p.1, p.4)*
- **Conditions for certifiable computation** (Theorem 3): strict convexity of the free energy in u under matched actuator noise + semiconvexity, so the numerical bound is a certified lower bound. *(p.1, p.4)*
- Shows TRIP's generalized-Fano bound is a special case of GVP (Remark 1): replace J^x_trj by an indicator and take β → ∞. The free energy "retains the full cost distribution through the partition function rather than collapsing it to a binary threshold, and is therefore never looser". *(p.1, p.3)*
- Validation: scalar LQG (SC bound captures >50% of J* by σ_v ≈ 2 and >80% by σ_v ≈ 10) and Dubins car figure-eight tracking (SC informative across all noise levels; open-loop-MI bound vacuous at low noise). *(p.5, p.6)*

## Study Design (empirical papers)
*Not an empirical study; theory paper with two numerical examples (see Results).*

## Methodology
1. Write closed-loop path measure P^π over τ = (x_{0:T}, y_{0:T}) and a reference measure Q^π with the same state dynamics and controller but observations drawn from arbitrary densities ν_t(y_t) independent of state. *(p.2)*
2. Apply GVP with ℓ = trajectory cost: E_P[ℓ] ≥ free energy under Q minus (1/β)·KL(P‖Q). *(p.2)*
3. Minimizing KL over ν gives Σ_t I_π[x_t; y_t] (per-time state/observation MI under the closed-loop law), bounded by a budget Ī. *(p.2)*
4. Under Q^π observations are independent of state, so for each realized y_{0:T} the control sequence is deterministic (open-loop); the free energy factors and is lower-bounded by the best open-loop free energy. *(p.2-3)*
5. Get Ī from the cost: a cost bound J bounds second moments M_t (coercivity), which bounds the Gaussian-max-entropy MI bound — a "cost-information certificate" Ī(J). *(p.3)*
6. Close the loop: J* ≥ J_irr(Ī(J*)) =: Φ(J*); solve J = Φ(J) by bisection on [0, J_ol]. *(p.4)*

## Key Equations / Statistical Models

### Problem formulation (p.1-2)

$$
x_{t+1} = f(x_t, u_t, w_t), \qquad (x_0, w_{0:T}) \sim P_w. \quad (1)
$$
Where: 𝕏 = ℝ^n state space, 𝕌 ⊆ ℝ^m control space, P_w joint distribution of initial condition and process noise. *(p.2; text of the sentence begins p.1)*

$$
y_t = h(x_t) + v_t, \qquad v_t \sim \mathcal{N}(0, \Sigma_v), \quad (2)
$$
Where: 𝕐 = ℝ^p output space; measurement noise v_t i.i.d., independent of process noise w_t. *(p.2)*

Note on the channel: h and Σ_v are fixed problem data; the controller cannot change the channel map. What depends on the policy is the distribution of x_t fed into the channel, and therefore the MI the channel carries. *(p.2-3)*

Assumption 1 (Separability): "The cost function decomposes c_t(x, u) into a state cost and a non-negative control cost: i.e., c_t(x, u) = q_t(x) + r_t(u)." *(p.2)*

Assumption 2 (Coercivity): "The state cost is coercive, namely: q_t(x) ≥ a‖x‖² − b for constants a > 0, b ≥ 0." *(p.2)*

Rationale given: separability makes control cost deterministic given u (needed for the free-energy factorization in Theorem 1) and lets state cost define a CI certificate; coercivity makes a finite cost budget constrain the second moment of the state distribution, "which is essential for establishing a self-consistent bound". *(p.2)*

ξ = (x_0, w_{0:T−1}) is a realization of the stochastic primitives. *(p.2)*

$$
J_{\mathrm{trj}}(\boldsymbol{u};\boldsymbol{\xi}) := J^x_{\mathrm{trj}}(\boldsymbol{u};\boldsymbol{\xi}) + J^u_{\mathrm{trj}}(\boldsymbol{u}) \quad (3)
$$

$$
J^x_{\mathrm{trj}}(\boldsymbol{u};\boldsymbol{\xi}) := \sum_{t=0}^{T} q_t(x_t), \qquad J^u_{\mathrm{trj}}(\boldsymbol{u}) := \sum_{t=0}^{T} r_t(u_t),
$$
Where: x_{0:T} satisfies (1) given open-loop sequence u ∈ 𝕌^T and ξ. A causal output-feedback controller is π = π_{0:T} with π_t mapping y_{0:t} to u_t ∈ 𝕌; Π is the set of all such; 𝒥_trj[π] is the trajectory cost with u_t = π_t(y_{0:t}); E_π[·] expectations under π. *(p.2)*

$$
J^\star := \underset{\boldsymbol{\pi}\in\Pi}{\mathrm{minimize}} \;\; \mathbb{E}_{\boldsymbol{\pi}}\big[\mathcal{J}_{\mathrm{trj}}[\boldsymbol{\pi}]\big] =: \mathbb{J}[\boldsymbol{\pi}]. \quad (\mathrm{OPT})
$$
"We seek a lower bound on J* computable from the problem data alone." Footnote 2: no assumptions beyond density regularity are placed on P_w. *(p.2)*

### Gibbs variational principle (p.2)
Partition function ℤ_β[ℓ] := E_Q[exp(−βℓ)], free energy 𝔽_β[ℓ] := −(1/β) log ℤ_β[ℓ]. For P ≪ Q:

$$
\mathbb{F}_\beta[\ell] - \frac{1}{\beta}\mathbb{D}[P\|Q] \le \mathbb{E}_P[\ell], \quad (\mathrm{GVP})
$$
"with equality if and only if P(x)/Q(x) ∝ exp(−βℓ)". β > 0 is an inverse temperature interpolating between average-case (β → 0⁺) and worst-case (β → ∞); optimize over β. *(p.2)*

Footnote 3: unlike risk-sensitive variational formulations [10],[11] where the optimization variable is on the left of the KL, here the true closed-loop P is on the left and the reference Q (containing free variable ν_t) on the right, "because we bound the cost from below rather than solve for the optimal controller." *(p.2)*

### Uniform MI budget (p.2)
"a uniform mutual information (MI) budget Ī ≥ 0 satisfying Σ_{t=0}^T 𝕀_π[x_t; y_t] ≤ Ī for all π ∈ Π, where"

$$
\mathbb{I}_{\boldsymbol{\pi}}[x_t; y_t] := \mathbb{E}_{\boldsymbol{\pi}}\Big[\log \frac{p(x_t, y_t)}{p(x_t)\,p(y_t)}\Big]
$$
is the MI between state and observation at time t. **This MI is taken under the closed-loop law of policy π**: p(x_t) is the closed-loop state marginal, so the same sensor (h, Σ_v) carries different MI under different controllers. *(p.2)*

### Theorem 1 (TRFE Bound) (p.2)
"Under Assumption 1, let Ī ≥ 0 be a uniform MI budget. Then,"

$$
J_{\mathrm{irr}}(\bar I) := \sup_{\beta>0}\Big\{\inf_{\boldsymbol{u}\in\mathbb{U}^T} F^x_\beta(\boldsymbol{u}) + J^u_{\mathrm{trj}}(\boldsymbol{u}) - \frac{\bar I}{\beta}\Big\} \le J^\star, \quad (4)
$$
"where F^x_β(u) := −(1/β) log E_ξ[exp(−β J^x_trj(u; ξ))] is the free energy of the state cost." *(p.2)*

Proof equations:

$$
P^{\boldsymbol{\pi}}(\tau) = p_0(x_0)\prod_{t=0}^{T-1} p(x_{t+1}\mid x_t,u_t)\prod_{t=0}^{T} p(y_t\mid x_t), \quad (5)
$$

$$
Q^{\boldsymbol{\pi}}(\tau) = p_0(x_0)\prod_{t=0}^{T-1} p(x_{t+1}\mid x_t,u_t)\prod_{t=0}^{T} \nu_t(y_t),
$$
Where: τ = (x_{0:T}, y_{0:T}); u_t = π_t(y_{0:t}) in both; ν_t(y_t) arbitrary density over 𝕐. *(p.2)*

$$
\mathbb{J}[\boldsymbol{\pi}] \ge -\tfrac{1}{\beta}\log \mathbb{E}_{Q^{\boldsymbol{\pi}}}\big[\exp(-\beta\,\mathcal{J}_{\mathrm{trj}}[\boldsymbol{\pi}])\big] - \tfrac{1}{\beta}\mathbb{D}[P^{\boldsymbol{\pi}}\|Q^{\boldsymbol{\pi}}]. \quad (6)
$$
Since P^π and Q^π share dynamics, KL reduces to Σ_{t=0}^T E_π[𝔻[p(y_t | x_t)‖ν_t(y_t)]], minimized at ν*_t = p^π(y_t) (the closed-loop observation marginal), yielding *(p.2)*

$$
\inf_{\nu_{0:T}} \mathbb{D}[P^{\boldsymbol{\pi}}\|Q^{\boldsymbol{\pi}}] = \textstyle\sum_{t=0}^{T} \mathbb{I}_{\boldsymbol{\pi}}[x_t; y_t] \le \bar I. \quad (7)
$$
*(p.2)*

Under Q^π observations are independent of state; for each fixed y_{0:T}, u(y_{0:T}) = π(y_{0:T}) is deterministic, only randomness in state cost is ξ:

$$
\mathbb{E}_{Q^{\boldsymbol{\pi}}}\big[\exp(-\beta\,\mathcal{J}_{\mathrm{trj}}[\boldsymbol{\pi}])\big] = \mathbb{E}_{y_{0:T}}\Big[\exp\big(-\beta\,J^u_{\mathrm{trj}}(\boldsymbol{u}(y_{0:T}))\big)\,\mathbb{Z}_\beta\big[J^x_{\mathrm{trj}}(\boldsymbol{u}(y_{0:T});\cdot)\big]\Big]. \quad (8)
$$
*(p.2)*

Since F^x_β(u(y_{0:T})) + J^u_trj(u(y_{0:T})) ≥ inf_u{F^x_β(u) + J^u_trj(u)}, the log in (8) is bounded above by exp(−β inf_u{F^x_β(u) + J^u_trj(u)}); applying −β^{-1} log(·):

$$
-\tfrac{1}{\beta}\log \mathbb{E}_{Q^{\boldsymbol{\pi}}}\big[\exp(-\beta\,\mathcal{J}_{\mathrm{trj}}[\boldsymbol{\pi}])\big] \ge \inf_{\boldsymbol{u}} F^x_\beta(\boldsymbol{u}) + J^u_{\mathrm{trj}}(\boldsymbol{u}). \quad (9)
$$
*(p.3)*

$$
\mathbb{J}[\boldsymbol{\pi}] \ge \inf_{\boldsymbol{u}}\{F^x_\beta(\boldsymbol{u}) + J^u_{\mathrm{trj}}(\boldsymbol{u})\} - \bar I/\beta. \quad (10)
$$
Optimizing over π ∈ Π and β > 0 gives (4). *(p.3)*

Remark 1: "The TRIP bound [5], [6] is a special case: replacing J^x_trj with 𝟏[J^x_trj > θ] and taking β → ∞ recovers the generalized Fano inequality." *(p.3)*

J_irr(Ī) is the "irreducible task cost: the minimum cost remaining after accounting for the sensor's information budget". Footnote 4: "More precisely, J_irr(Ī) is a lower bound on the irreducible cost, since tighter bounds on J* may exist." *(p.3)*

### Proposition 1 (p.3)
Optimal open-loop task cost:

$$
J_{\mathrm{ol}} := \inf_{\boldsymbol{u}\in\mathbb{U}^T} \mathbb{E}_{\boldsymbol{\xi}}\big[J_{\mathrm{trj}}(\boldsymbol{u};\boldsymbol{\xi})\big]. \quad (11)
$$
"When J_ol > 0, J_irr(Ī) has the following properties for Ī ≥ 0: (i) J_irr(Ī) is strictly decreasing, (ii) J_irr(0) = J_ol, and (iii) J_irr(Ī) ≤ J_ol." Proof: Ī ↦ F^x_β(u) + J^u_trj(u) − Ī/β strictly decreasing, preserved by inf over u and sup over β; J_irr(0) = J_ol since F^x_β(u) → E_ξ[J^x_trj(u; ξ)] as β → 0⁺ (L'Hôpital). *(p.3)*

### Cost-information certificates (Section IV-A, p.3)
"The TRFE bound (Theorem 1) requires a uniform MI budget Ī ≥ 0. Identifying one is nontrivial: different controllers induce different trajectory distributions and therefore different MI values for the same sensor. However, the MI a sensor can extract is constrained by the cost the controller achieves—a performant controller concentrates the state distribution which bounds the MI." *(p.3)*

Under Assumption 1, r_t ≥ 0 implies any policy with 𝕁[π] ≤ J satisfies Σ_{t=0}^T E_π[q_t(x_t)] ≤ J. With Assumption 2, this constrains uncentered second moments M_t := E_π[x_t x_tᵀ]:

$$
a\sum_{t=0}^{T}\mathrm{tr}(M_t) \le J + b\,(T+1). \quad (12)
$$
When q_t(x) = xᵀQx with Q ≻ 0, this sharpens to Σ_{t=0}^T tr(QM_t) ≤ J. Since the Gaussian maximizes differential entropy for fixed covariance [15, Thm 8.6.5]: *(p.3)*

$$
\mathbb{I}_{\boldsymbol{\pi}}[x_t;\, y_t] \le \frac{1}{2}\log\det\big(I_p + \Sigma_v^{-1}\Sigma_{h_t}\big), \quad (13)
$$
Where: Σ_{h_t} = Cov(h(x_t)) — **the covariance of the sensor output's signal part under the closed-loop state distribution**. This is the explicit mechanism by which the channel's information depends on the closed-loop distribution. *(p.3)*

Definition 1 (Cost-Information Certificate): "A function Ī : [0, ∞) → [0, ∞) is a cost-information (CI) certificate for system (1) and (2) if: (a) Ī is continuous and strictly increasing; and (b) for π ∈ Π with 𝕁[π] ≤ J, Σ_{t=0}^T 𝕀_π(x_t; y_t) ≤ Ī(J)." *(p.3)*

Note: this weakens Theorem 1's "uniform over all π ∈ Π" requirement to "uniform over the sublevel set {π : 𝕁[π] ≤ J}".

### Proposition 2 (Nonlinear Sensors) (p.3)
"Let h(x) be L-Lipschitz with L > 0, and let λ_min denote the smallest eigenvalue of Σ_v. Under Assumptions 1 and 2, the function"

$$
\bar I(J) := \frac{(T+1)p}{2}\log\Big(1 + \frac{L^2(J + bT)}{ap\,(T+1)\,\lambda_{\min}}\Big) \quad (14)
$$
"is a cost-information certificate for system (1)–(2)." *(p.3)*

[Transcription note: (14) as printed has "J + bT" while (12) has "J + b(T+1)". Checked against a 300 dpi render; quoted as printed.]

Proof: AM-GM applied to (13) (footnote 5: for A ∈ 𝕊ⁿ₊, det(I_p + A) ≤ (1 + tr(A)/p)^p) and tr(Σ_v^{-1} Cov(h(x_t))) ≤ L² tr(M_t)/λ_min:

$$
\mathbb{I}_{\boldsymbol{\pi}}(x_t;\, y_t) \le \frac{p}{2}\log\Big(1 + \frac{L^2\,\mathrm{tr}(M_t)}{p\,\lambda_{\min}}\Big). \quad (15)
$$
Sum over t, Jensen on log(·), substitute (12) → property (b); (a) from strict monotonicity of log and L > 0. *(p.3)*

### Proposition 3 (Linear Sensors) (p.3)
"Let h(x) = Hx with Σ_v^{-1}H ≠ 0, and let q_t(x) = xᵀQx with Q ∈ 𝕊ⁿ₊, Q ≠ 0. Under Assumption 1, the function"

$$
\bar I(J) := \max_{M_t \succeq 0}\; \sum_{t=0}^{T} \frac{1}{2}\ln\det\big(I_p + \Sigma_v^{-1} H M_t H^{\mathsf T}\big) \;\; \text{s.t.} \;\; \mathrm{tr}(QM_t) \le J \quad (16)
$$
"is a cost-information certificate for system (1)–(2)." *(p.3)*

[Transcription note: the constraint is printed as "tr(QM_t) ≤ J" without a sum over t, while the preceding text gives the sharpened constraint as Σ_{t=0}^T tr(QM_t) ≤ J and says the optimal allocation is uniform over time. Checked against a 300 dpi render; quoted as printed. The summed form appears to be the intended one, but I have not confirmed this.]

"Program (16) is convex and admits a closed-form solution via water-filling across the modes of Q^{-1/2}HᵀΣ_v^{-1}HQ^{-1/2}, with the optimal second-moment allocation uniform over time [15]." (b) follows from feasibility of M^π_t in (16) for any π with 𝕁[π] ≤ J and (13); (a) from strict monotonicity of ln det and Σ_v^{-1}H ≠ 0. *(p.3)*

### Self-consistent fixed point (Section IV-B, p.4)
"Define Φ(J) := (J_irr ∘ Ī)(J), where Ī(J) is a certificate. Since the optimal policy achieves cost J*, applying Theorem 1 with budget Ī(J*) gives J* ≥ Φ(J*). This suggests searching for the largest J consistent with this constraint, i.e., the fixed point J_sc = Φ(J_sc)." *(p.4)*

Theorem 2 (Self-Consistent TRFE Bound): "Let Ī be a certificate for system (1)–(2) with Ī(J_ol) > 0, and suppose J_ol > 0. Then:
(i) The function Φ(J) is continuous and strictly decreasing on [0, J_ol].
(ii) There is a unique solution J_sc ∈ (0, J_ol) to the fixed-point equation: J = Φ(J).
(iii) This solution lower-bounds the optimal cost: J_sc ≤ J*." *(p.4)*

Proof with g(J) := Φ(J) − J: (i) Ī continuous strictly increasing, J_irr continuous strictly decreasing, so Φ is too; g continuous with g′(J) < −1 where the derivative exists. (ii) g(0) = J_ol > 0; Ī(J_ol) > 0 ⇒ Φ(J_ol) < J_irr(0) = J_ol ⇒ g(J_ol) < 0; IVT gives root, unique by (i). (iii) Definition 1(b) gives Ī(J*) as a valid MI budget: J* ≥ Φ(J*), i.e. g(J*) ≤ 0; g strictly decreasing and g(J_sc) = 0 ⇒ J* ≥ J_sc. *(p.4)*

[Observation, not a claim of the paper: the proof of (iii) applies Theorem 1 with a budget that is only valid for the sublevel set {π : 𝕁[π] ≤ J*}, not for all π ∈ Π as Theorem 1 states. The step works because Theorem 1's proof (6)-(10) holds pointwise for each fixed π, so it suffices for the optimal π to satisfy the budget. The paper does not spell this out.]

Strict tightening over open-loop budget: "Theorem 2 holds for any CI certificate satisfying Definition 1, including the nonlinear bound (14). The linear certificate (16) yields a tighter result but is not required for convergence. The self-consistent bound is strictly tighter than substituting the open-loop MI budget directly into Theorem 1: since J_sc < J_ol and Ī is strictly increasing, Ī(J_sc) < Ī(J_ol), and therefore J_sc = Φ(J_sc) > Φ(J_ol)." *(p.4)*

So the paper's "open-loop MI budget" baseline (OL-MI) is Ī_ol = Ī(J_ol), the certificate evaluated at the optimal open-loop cost (p.5, Algorithm description: "For the open-loop MI bound, Φ is evaluated at the fixed budget Ī_ol = Ī(J_ol)").

### Convexity of the free energy (Section V, p.4)
Computing (4) needs global minimization over open-loop sequences; conditions under which F_β(u) := F^x_β(u) + J^u_trj(u) is strictly convex in u. Footnote 6: "If these conditions are not met, Theorems 1 and 2 remain valid. Convexity is needed only to certify the numerical computation of inf_u F_β(u)." *(p.4)*

Assumption 3 (Matched Actuator Noise and Control Cost): "The process noise enters through the control input channel:"

$$
x_{t+1} = f(x_t,\, u_t + w_t), \;\; w_t \sim \mathcal{N}(0, \Sigma_w), \;\; \Sigma_w \succ 0, \quad (17)
$$
"with f twice differentiable, so that the state trajectory depends on u and w only through the realized input ζ := u + w. The control cost is quadratic: r(u) := ½uᵀRu, R ≻ 0." *(p.4)*

Trajectory state cost S(ζ) := Σ_{t=0}^T q(x_t(ζ)), x_t(ζ) the state at t under input ζ from fixed x_0. *(p.4)*

Assumption 4 (Semiconvexity): "The trajectory cost S is α-semiconvex [16] in ζ: there exists a value α ≥ 0 such that ∂²_ζ S(ζ) ⪰ −α I_{mT} for all ζ ∈ ℝ^{mT}." *(p.4)*

Theorem 3 (Convexity of the Free Energy): "Under Assumptions 1, 3 and 4, when α < R̲ := λ_min(R), F_β(u) is strictly convex for β ∈ (0, β̄), where"

$$
\bar\beta := \frac{\underline{R} - \alpha}{\underline{R}\,\alpha\,\bar\sigma^2}, \qquad \bar\sigma^2 := \lambda_{\max}(\Sigma_w). \quad (18)
$$
"When α = 0 (i.e., S is convex in ζ), β̄ = +∞." *(p.4)*

Proof: Σ_W := I_T ⊗ Σ_w, P_u := 𝒩(u, Σ_W); likelihood ratio P_u/P_0 = exp(uᵀΣ_W^{-1}ζ − ½uᵀΣ_W^{-1}u):

$$
F_\beta(\boldsymbol{u}) = \underbrace{-\tfrac{1}{\beta}\log \mathbb{E}_{P_0}\big[\exp\big(-\beta S(\boldsymbol\zeta) + \boldsymbol{u}^{\mathsf T}\Sigma_W^{-1}\boldsymbol\zeta\big)\big]}_{-h(\boldsymbol u)} + \underbrace{\tfrac{1}{2}\boldsymbol{u}^{\mathsf T}\Big(\tfrac{1}{\beta}\Sigma_W^{-1} + R\Big)\boldsymbol{u}}_{g(\boldsymbol u)}. \quad (19)
$$
g strictly convex, h convex by Hölder, F_β = g − h difference of convex; convex iff ∂²g ≻ ∂²h. *(p.4)*

$$
\partial^2_{\boldsymbol u} h(\boldsymbol u) = \tfrac{1}{\beta}\Sigma_W^{-1}\mathrm{Cov}_\mu[\boldsymbol\zeta;\boldsymbol u]\Sigma_W^{-1}, \quad (20)
$$
Cov_μ[ζ; u] covariance of ζ under tilted measure μ(ζ; u) ∝ exp(−Ψ(ζ; u)) with *(p.4)*

$$
\Psi(\boldsymbol\zeta;\boldsymbol u) := \beta S(\boldsymbol\zeta) + \tfrac{1}{2}\boldsymbol\zeta^{\mathsf T}\Sigma_W^{-1}\boldsymbol\zeta - \boldsymbol u^{\mathsf T}\Sigma_W^{-1}\boldsymbol\zeta. \quad (21)
$$
Under Assumption 4, ∂²_ζΨ = β∂²_ζS + Σ_W^{-1} ⪰ Σ_W^{-1} − αβI_{mT}, positive definite when αβσ̄² < 1. Brascamp–Lieb [17, Thm 4.1] (footnote 7: if p(z) ∝ exp(−ℓ(z)) with ∂²_zℓ ⪰ K ≻ 0, then Cov_p(z) ⪯ K^{-1}) gives Cov_μ[ζ; u] ⪯ (Σ_W^{-1} − αβI_{mT})^{-1}. Then *(p.4)*

$$
\partial^2_{\boldsymbol u} F_\beta(\boldsymbol u) \succeq R - \alpha\Sigma_W^{-1}\big(\Sigma_W^{-1} - \alpha\beta I_{mT}\big)^{-1} \succeq \Big(\underline{R} - \frac{\alpha}{1-\alpha\beta\bar\sigma^2}\Big) I_{mT} \succ 0 \iff \beta < \bar\beta.
$$
*(p.4)*

$$
\lambda_{\max}\big(\Sigma_W^{-1}(\Sigma_W^{-1} - \beta\alpha I)^{-1}\big) = (1 - \beta\alpha\bar\sigma^2)^{-1}, \quad (22)
$$
"as commuting symmetric matrices share an eigenbasis." *(p.4)*

"Restricting the supremum in (4) to (0, β̄) preserves validity—it can only weaken the bound—and loses no tightness when β* ≤ β̄." *(p.4)*

Remark 2 (Scope of Assumptions 3 and 4): Assumption 3 covers control-affine systems x_{t+1} = f_0(x_t) + B(x_t)(u_t + w_t) with actuator noise, including the Dubins car. It excludes unmatched disturbances entering independently of the control input. Assumption 4 holds with α = 0 for linear dynamics with convex costs. For nonlinear dynamics with convex state cost, α depends on dynamics curvature ∂²f and is small near a tracking reference; estimable from open-loop rollouts. *(p.4-5)*

### Numerical examples (p.5)
Scalar LQG system:

$$
x_{t+1} = 0.95\,x_t + u_t + w_t, \qquad y_t = x_t + v_t, \quad (23)
$$
with w_t ∼ 𝒩(0, 0.01), v_t ∼ 𝒩(0, σ_v²), x_0 = 0, T = 100; per-step cost c(x, u) = x² + u²; J* via discrete-time Riccati (LQG optimal); free energy in closed form [19], no sampling. *(p.5)*

Dubins car dynamics:

$$
x_{t+1} = x_t + \Delta t\cdot\big[v\cos\theta_t \;\; v\sin\theta_t \;\; \omega_t + w_t\big]^{\mathsf T}, \quad (24)
$$
w_t ∼ 𝒩(0, σ_w²), σ_w = 0.1; noise only through steering (Assumption 3 with m = 1). Sensor observes position only: y_t = Hx_t + v_t, H = [I_2 0], v_t ∼ 𝒩(0, σ_v² I_2). State x = (p_x, p_y, θ), control u = ω, figure-eight reference, v = 1, T = 100, Δt = 0.1 s. *(p.5)*

Dubins cost: c(x, u) = ‖p − p_t^ref‖² + 0.1(1 − cos(θ − θ_t^ref)) + ½(ω − ω_t^nom)², ω^nom feedforward steering rate. Heading cost smooth and 2π-periodic. Water-filling certificate uses Q = diag(1, 1, 0) (position only); since Q is rank-deficient, (16) is computed on the range of Q, "yielding a valid but potentially conservative MI budget that excludes heading information. The non-quadratic heading term is retained but does not contribute to the MI certificate." *(p.5)*

## Parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Coercivity slope | a | cost/state² | - | a > 0 | 2 | Assumption 2 |
| Coercivity offset | b | cost | - | b ≥ 0 | 2 | Assumption 2 |
| Inverse temperature | β | 1/cost | - | (0, ∞); certified on (0, β̄) | 2, 4 | Optimized in (4) |
| Uniform MI budget | Ī | nats | - | ≥ 0 | 2 | Σ_t I_π[x_t;y_t] ≤ Ī |
| Lipschitz constant of h | L | - | - | L > 0 | 3 | Prop. 2 |
| Smallest eigenvalue of Σ_v | λ_min | - | - | > 0 | 3 | Prop. 2 |
| Semiconvexity constant | α | - | - | α ≥ 0, need α < λ_min(R) | 4 | Assumption 4 |
| Convexity threshold | β̄ | 1/cost | - | (R̲−α)/(R̲ α σ̄²) | 4 | eq. (18); +∞ if α = 0 |
| LQG drift coefficient | - | - | 0.95 | - | 5 | eq. (23) |
| LQG process noise variance | - | - | 0.01 | - | 5 | w_t ∼ 𝒩(0,0.01) |
| LQG horizon | T | steps | 100 | - | 5 | |
| Dubins speed | v | m/s | 1 | - | 5 | |
| Dubins horizon | T | steps | 100 | - | 5 | |
| Dubins timestep | Δt | s | 0.1 | - | 5 | |
| Dubins steering noise std | σ_w | rad/s | 0.1 | - | 5 | |
| Dubins heading cost weight | - | - | 0.1 | - | 5 | 0.1(1 − cos(θ−θ^ref)) |
| Dubins MC rollouts | N | - | 5×10⁴ | - | 5 | |
| Dubins β grid size | K | - | 60 | - | 5 | |
| Dubins estimated semiconvexity | α̂ | - | 0.44 | - | 5 | |
| Dubins estimated convexity threshold | β̄̂ | - | 127 | - | 5 | β* below this at all noise levels |
| Sensor noise std (sweep) | σ_v | - | - | 10⁻¹ to 10² | 6 | Fig. 1 x-axis |

## Effect Sizes / Key Quantitative Results

| Outcome | Measure | Value | CI | p | Population/Context | Page |
|---------|---------|-------|----|---|--------------------|------|
| SC bound fraction of J* | J_sc / J* | > 0.5 | - | - | Scalar LQG, σ_v ≈ 2 | 5 |
| SC bound fraction of J* | J_sc / J* | > 0.8 | - | - | Scalar LQG, σ_v ≈ 10 | 5 |
| SC bound fraction of LQG(EKF+LQR) upper bound | J_sc / J_LQG | ≈ 0.10 | - | - | Dubins, σ_v = 0.1 | 5 |
| OL-MI bound | sign | vacuous (negative) | - | - | Dubins, σ_v = 0.1 | 5 |
| OL-MI vs SC | ordering | OL-MI substantially looser at all noise levels | - | - | Both examples | 5, 6 |
| Large-noise limit | all curves | → J_ol (normalized 1) as σ_v → ∞ | - | - | Both examples | 6 |

Fig. 1 read-off (approximate, from plot, p.6): LinSys at σ_v = 10⁻¹: LQG ≈ 0.31, SC ≈ 0.06, OL-MI ≈ 0.0; LQG reaches 1.0 by σ_v ≈ 4. Dubins at σ_v = 10⁻¹: LQG(EKF+LQR) ≈ 0.26, SC ≈ 0.03, OL-MI ≈ 0.0 (OL-MI stays ≈ 0 up to σ_v ≈ 0.5). Values are visual estimates, not reported numbers.

## Methods & Implementation Details
- Partition function in F_β estimated via N Monte Carlo rollouts of open-loop dynamics, drawn once and reused across all β and bisection iterates, so Φ(J) is a deterministic function of J for stable root-finding. *(p.5)*
- Under Assumption 3, minimization over u reduces to an importance-weighted fixed-point iteration — "a deterministic analogue of MPPI [18]" — converging to the global minimum when β < β̄. *(p.5)*
- OL-MI: Φ evaluated at fixed budget Ī_ol = Ī(J_ol). SC: bisection over [0, J_ol] for root of J = Φ(J), with Ī(J) from water-filling certificate (16). *(p.5)*
- α estimated by sampling N_α noise realizations ξ^(i), Hessian ∇²_ζS at ζ^(i) = u_nom + ξ^(i) via automatic differentiation, α̂ = max(0, −min_i λ^(i)_min). Since sampled, α̂ ≤ α and β̄̂ ≥ β̄ — so the certificate is approximate; "A rigorous finite-sample certificate is deferred to future work." *(p.5)*
- Bound valid for any β > 0, but a suboptimal inner minimization can violate the lower-bound guarantee; convexity certification ensures validity. *(p.5)*
- Dubins: J_ol approximated by sample-average cost at nominal feedforward u_nom (may overestimate the true infimum), but J_ol enters only as the bisection bracket for Theorem 2; F*_k independently optimized at each β_k, so J_sc unaffected (footnote 8). Costs normalized by J_ol/T. *(p.5)*
- Dubins baseline upper bound on J*: EKF + time-varying LQR linearized about the reference, simulated on nonlinear dynamics. *(p.5)*

**Algorithm 1 (Estimating the TRFE Bound)** *(p.6)*
Require: Traj. cost J_trj, CI cert. Ī, samples N, grid {β_k}_{k=1}^K.
1. Draw noise ξ^(1:N).
2. for k = 1, …, K (ascending β with warm-start): F*_k ← min_u {F^x_{β_k}(u) + J^u_trj(u)}. (inner opt. in (4))
3. return Φ(Ī) := max_k {F*_k − Ī/β_k}. (outer opt. in (4))

## Figures of Interest
- **Fig. 1 (p.6):** Normalized cost J/J_OL vs sensor noise σ_v (log scale 10⁻¹ to 10²). Left: scalar LQG (optimal J* from Riccati). Right: Dubins car tracking a figure-eight (heading unobserved). Curves: LQG (optimal on left; EKF+LQR upper bound on right), SC (self-consistent TRFE bound, Theorem 2), OL-MI (TRFE with open-loop MI budget). "The SC bound captures a substantial fraction of J* at moderate-to-high noise in both settings; OL-MI is substantially looser at all noise levels."

## Results Summary
- Scalar LQG: TRIP "cannot be applied to this problem, as it requires both bounded costs and a finite control space." SC captures over half of J* by σ_v ≈ 2 and over 80% by σ_v ≈ 10; OL-MI substantially looser at all noise levels. *(p.5)*
- Dubins: J* lies between the LQG curve (upper) and SC curve (lower); at σ_v = 0.1 SC accounts for ≈10% of LQG cost while OL-MI is vacuous (negative). "The SC bound is tighter than OL-MI because it uses an adaptive information budget coupled to the achievable cost, whereas OL-MI uses a fixed budget computed from the uncontrolled state distribution—which vastly overestimates the required sensor information when feedback is effective." As σ_v → ∞ all curves converge to open-loop cost; bound is tight there. Persistent LQG–SC gap at low-to-moderate noise "suggests room for improvement over the linearized controller". *(p.5-6)*

## Limitations
- Semiconvexity α estimated from finite samples (α̂ ≤ α), so the convexity certificate is approximate; rigorous finite-sample analysis via concentration inequalities is the "primary next step" (already done for TRIP [5],[6]). *(p.5, p.6)*
- Assumption 3 excludes unmatched disturbances that enter independently of the control input. *(p.4-5)*
- Water-filling certificate (16) ignores Markov structure of the dynamics; adding it (e.g., covariance propagation constraint) would tighten the certificate. *(p.6)*
- In Dubins, rank-deficient Q means MI certificate excludes heading information — "valid but potentially conservative". *(p.5)*
- At low noise, large gap between bound and achievable cost (≈10% at σ_v = 0.1 in Dubins). *(p.5)*
- J_irr is only a lower bound on the irreducible cost; tighter bounds on J* may exist (footnote 4). *(p.3)*
- The certificates bound MI by the Gaussian max-entropy bound (13) using second moments only; the bound is loose whenever the closed-loop state law is far from Gaussian (my observation, follows from the derivation on p.3).

## Arguments Against Prior Work
- TRIP framework [5],[6] (Majumdar & Pacelli RSS 2022; Majumdar, Mei, Pacelli IJRR 2023): first general-purpose approach via generalized Fano [7], but requires bounded costs and finite control space. *(p.1)*
- **The key criticism (the one the caller asked about):** TRIP "computes mutual information (MI) using the *open-loop* state distribution—which arises in the absence of feedback. In most instances, good feedback controllers concentrate the state in low-cost regions, meaning the open-loop distribution vastly overestimates the required sensor information. This structural issue is worst when feedback control is the most effective." *(p.1)*
- Abstract version: "Existing information-theoretic approaches overestimate the information a sensor must provide by evaluating it against the uncontrolled system, producing bounds that degrade precisely when feedback is most valuable." The abstract does not cite by number; in the body the prior bounds named as evaluating MI under the open-loop/uncontrolled distribution are refs [5],[6] (TRIP). No other work is named for this specific criticism. *(p.1)*
- Fano-based derivation collapses the cost distribution to a binary threshold; GVP free energy retains the full cost distribution and is "never looser"; the self-consistency structure is "invisible in the Fano-based derivation". *(p.1)*
- Classical limits — Nyquist–Shannon [1], Bode sensitivity integral [2], data-rate theorems [3], perception-latency limits [4] — each address a particular problem class. Bode integrals "do not apply" to the nonlinear nonholonomic Dubins tracking problem. *(p.1, p.5)*

Note on the paper's own OL-MI baseline: although the text describes it as "a fixed budget computed from the uncontrolled state distribution" (p.5-6), Algorithm/Section VI defines it as Ī_ol = Ī(J_ol), the CI certificate evaluated at the optimal open-loop cost J_ol (p.5). These are related (J_ol is the cost of the best open-loop, i.e. feedback-free, input) but not literally the MI of the uncontrolled process.

## Design Rationale
- Reference measure Q^π decouples observations from state while keeping same dynamics and controller, so KL(P‖Q) is exactly the information used and observations under Q make the controller effectively open-loop. *(p.2)*
- KL direction reversed relative to risk-sensitive control (true P on the left) because the goal is a lower bound, not controller synthesis (footnote 3). *(p.2)*
- Separability → control cost deterministic given u → free-energy factorization (8). *(p.2)*
- Coercivity → cost budget bounds second moment → bounds MI via Gaussian max entropy. *(p.2-3)*
- Optimize β to interpolate average/worst-case. *(p.2)*
- Reuse the same noise samples across β and bisection so Φ is deterministic in J. *(p.5)*
- Convexity needed only to certify numerics, not for validity of Theorems 1-2 (footnote 6). *(p.4)*

## Testable Properties
- J_irr(Ī) ≤ J* for any uniform MI budget Ī. *(p.2)*
- J_irr is strictly decreasing in Ī; J_irr(0) = J_ol; J_irr(Ī) ≤ J_ol (when J_ol > 0). *(p.3)*
- Setting J^x_trj → 𝟏[J^x_trj > θ] and β → ∞ recovers generalized Fano / TRIP. *(p.3)*
- Φ continuous and strictly decreasing on [0, J_ol]; g(J) = Φ(J) − J has g′ < −1 where differentiable. *(p.4)*
- Unique J_sc ∈ (0, J_ol) with J_sc = Φ(J_sc), and J_sc ≤ J*. *(p.4)*
- J_sc > Φ(J_ol) (SC strictly tighter than OL-MI). *(p.4)*
- F_β strictly convex in u for β ∈ (0, β̄) when α < λ_min(R); β̄ = ∞ when α = 0. *(p.4)*
- As σ_v → ∞, all normalized curves → 1 (open-loop cost) and the bound is tight. *(p.6)*
- Linear sensor + quadratic cost: Ī(J) from (16) by water-filling, allocation uniform over time. *(p.3)*

## Relevance to Project
Context: Bode-type limits where the channel is endogenous to the policy (Goodhart).

- **What is endogenous here and what is not.** The sensor map h and noise Σ_v are fixed. What depends on the policy is the closed-loop state distribution, and so the MI I_π[x_t; y_t] the fixed channel delivers (eq. (7): the optimal reference marginal is ν*_t = p^π(y_t), the closed-loop observation marginal; eq. (13): MI ≤ ½ log det(I_p + Σ_v^{-1} Cov(h(x_t))) with Cov under the closed loop). *(p.2-3)*
- **Direction of the coupling.** Better control → state more concentrated → less MI the sensor *can* carry → bound gets tighter (and in their framing, cost floors rise). This is a self-limiting loop: the controller's success shrinks its own information supply. This is the paper's "self-consistent" mechanism: J* ≥ J_irr(Ī(J*)). *(p.3-4)*
- **Relation to Goodhart.** There is no adversary and no gaming of the channel; the channel does not degrade because it is being optimized against. The effect is purely that the information content of a fixed measurement depends on the variance of what it measures, and control removes that variance. It is a clean formal example of "the measured variable loses informativeness as it is controlled", which is one ingredient of a Goodhart-type limit, but not a model of the measure being corrupted by pressure on it.
- **Bode connection.** Paper explicitly positions itself as covering cases where Bode integrals do not apply (nonlinear nonholonomic tracking) and lists Bode [2] among classical, problem-class-specific limits. It derives no Bode-type integral itself. *(p.1, p.5)*
- **Reusable device.** The cost-information certificate (Definition 1) is a general recipe for turning "any policy with cost ≤ J" into "channel MI ≤ Ī(J)", then solving a fixed point. That template could transfer to settings where the channel itself is a function of the policy, provided one can bound the channel's information over the sublevel set {π : 𝕁[π] ≤ J}.
- Uses per-time MI Σ_t I[x_t; y_t] (not directed information), arising from the KL between P^π and the decoupled Q^π. *(p.2)*

## Open Questions
- [ ] Is the constraint in (16) intended to be Σ_t tr(QM_t) ≤ J (as the preceding text and "uniform over time" allocation suggest) rather than the printed per-t form?
- [ ] (14) prints J + bT whereas (12) has J + b(T+1); typo or intentional?
- [ ] Theorem 2(iii) uses Ī(J*) as a budget valid only for policies with cost ≤ J*; the pointwise-in-π validity of Theorem 1's proof covers this, but the theorem statement says "uniform over all π ∈ Π". Worth checking if one builds on it.
- [ ] Can the self-consistent construction extend to a channel whose map (h or Σ_v) depends on π, i.e., an endogenous channel in the Goodhart sense? The paper does not address it.
- [ ] Connection to Sagawa–Ueda generalized Jarzynski equality [20] flagged as future work. *(p.6)*

## Related Work Worth Reading
- [5] Majumdar & Pacelli, "Fundamental performance limits for sensor-based robot control and policy learning," RSS 2022 — the TRIP bound this generalizes.
- [6] Majumdar, Mei & Pacelli, "Fundamental limits for sensor-based robot control," IJRR 2023.
- [7] Gerchinovitz, Ménard & Stoltz, "Fano's inequality for random variables," Statistical Science 2020.
- [3] Nair, Fagnani, Zampieri & Evans, "Feedback control under data rate constraints: An overview," Proc. IEEE 2007.
- [2] Bode, Network Analysis and Feedback Amplifier Design, 1945.
- [14] Chen, Georgiou & Tannenbaum, "Stochastic control and nonequilibrium thermodynamics: Fundamental limits," TAC 2020.
- [20] Sagawa & Ueda, "Generalized Jarzynski equality under nonequilibrium feedback control," PRL 2010.
- [10] Theodorou & Todorov, "Relative entropy and free energy dualities," CDC 2012.

## Collection Cross-References

### Already in Collection
- (none; no reference in citations.md is in the collection)

### New Leads (Not Yet in Collection)
- Majumdar & Pacelli (2022) - "Fundamental performance limits for sensor-based robot control and policy learning" - the TRIP bound; evaluates sensor MI under the open-loop state distribution, and is the β → ∞, indicator-cost special case of this paper's bound (Remark 1, p.3).
- Majumdar, Mei & Pacelli (2023) - "Fundamental limits for sensor-based robot control" - journal version of TRIP.
- Gerchinovitz, Ménard & Stoltz (2020) - "Fano's inequality for random variables" - the generalized Fano inequality underlying TRIP.
- Bode (1945) - "Network Analysis and Feedback Amplifier Design" - classical problem-class-specific limit the paper positions against (p.1).
- Nair, Fagnani, Zampieri & Evans (2007) - "Feedback control under data rate constraints: An overview" - data-rate theorems, the other classical limit named (p.1).
- Chen, Georgiou & Tannenbaum (2020) - "Stochastic control and nonequilibrium thermodynamics: Fundamental limits" - thermodynamic limits of stochastic control.
- Sagawa & Ueda (2010) - "Generalized Jarzynski equality under nonequilibrium feedback control" - structural parallel flagged as future work (p.6).

### Supersedes or Recontextualizes
- (none)

### Cited By (in Collection)
- (none found)

### Conceptual Links (not citation-based)

**Feedback reducing the information an observation carries:**
- [Violations of robustness trade-offs](../Kitano_2010_ViolationsRobustnessTrade-offs/notes.md) - Strong. Kitano's supplement has feedback compensation relax selection on loop components so they can evolve to be "sloppy": a 40% non-linearity component gives 4% closed-loop non-linearity, inside a 5% spec that a 4% component (0.4% closed-loop) also passes, so the spec no longer separates them (Kitano 2010 S1-S3). Here a good controller concentrates the state, which caps the MI the fixed sensor can carry (eq. 13, p.3). Both are cases of successful feedback lowering what an observable reveals; Kitano's is qualitative and evolutionary with no information measure, this paper's is a formal bound with a fixed sensor and no selection. Both also limit Bode: Kitano confines it to the frequency axis of negative feedback (Kitano 2010 p.1, S3); this paper says Bode integrals "do not apply" to the Dubins problem (p.5).
- [Towards a theory of biological robustness](../Kitano_2007_TheoryBiologicalRobustness/notes.md) - Moderate. Kitano's Box 2 points to limited-capacity feedback channels (Martins et al. 2004, 2007) as extending intrinsic trade-offs beyond Bode (Kitano 2007 p.6); this paper's bound is a limit of that information-constrained kind. It derives no conservation integral, so it does not test Kitano's conjectured robustness conservation (Kitano 2007 Eq. 3).

**Policy-dependent distributions and variational bounds:**
- [Performative Prediction](../Perdomo_2020_PerformativePrediction/notes.md) - Strong. Both make the distribution that a fixed evaluation sees depend on the policy and then solve a self-consistency condition. Perdomo et al. assume θ -> D(θ) is ε-Lipschitz in W1 and study the retraining fixed point θ_PS = argmin_θ E_{D(θ_PS)} ℓ (Perdomo Def 2.3); here the sensor is fixed but its MI is taken under the closed-loop state law and bounded from the cost (eq. 12-13), closed into J_sc = Φ(J_sc) (Thm 2). Perdomo's shift is assumed and its fixed point is a learning equilibrium; this paper derives the information supply from the cost and its fixed point is a bound. Neither models a proxy diverging from true quality: Perdomo trains and evaluates with the same loss, and the channel here is not corrupted by pressure on it.
- [How much does your data exploration overfit? Controlling bias via information usage](../Russo_2015_MuchDoesDataExploration/notes.md) - Strong. Same variational inequality and same kind of decoupled reference law. This paper's GVP, F_β[ℓ] − (1/β)KL(P‖Q) ≤ E_P[ℓ] (p.2), is Russo & Zou's Fact 1, KL(P‖Q) = sup_X {E_P X − log E_Q e^X} (Russo p.12), with X = −βℓ. Both choose a reference in which the decision is independent of the noisy signal (here observations drawn independently of state, eq. 5; in Russo the unconditional law of φ_i against its law given T = i, Russo p.12-13), so that the KL becomes a mutual information (here inf_ν KL = Σ_t I_π[x_t; y_t], eq. 7; in Russo I(T; φ) = Σ_i P(T = i) KL, Russo p.12). Both then optimize the free parameter (β in eq. 4; λ = Δ_i/σ² in Russo p.13). The direction differs: Russo upper-bounds how far I nats can push a selected statistic's mean (σ·sqrt(2I) for sub-Gaussian noise); this paper lower-bounds the cost a controller cannot get below with Ī nats (J_irr(Ī), eq. 4).
