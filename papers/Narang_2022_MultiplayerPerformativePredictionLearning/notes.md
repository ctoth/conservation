---
title: "Multiplayer Performative Prediction: Learning in Decision-Dependent Games"
authors: "Adhyyan Narang, Evan Faulkner, Dmitriy Drusvyatskiy, Maryam Fazel, Lillian J. Ratliff"
year: 2022
venue: "arXiv:2201.03398v2 (6 Apr 2022); journal version JMLR 24(202):1-56, 2023; preliminary version AISTATS 2022 (PMLR 151)"
doi_url: "https://arxiv.org/abs/2201.03398"
pages: 40
note: "Notes read from arXiv v2 (40 pp). The JMLR 2023 version (56 pp) was NOT read; its theorem numbering and content may differ. Page numbers below are the printed page numbers of arXiv v2 (page-NNN.png = printed p.NNN+1)."
---

# Multiplayer Performative Prediction: Learning in Decision-Dependent Games

## Version record
- Read: arXiv:2201.03398v2, dated April 8, 2022, 40 pages *(p.1)*.
- Footnote 1: "A preliminary version of this paper appeared in the Proceedings of the 25th International Conference on Artificial Intelligence and Statistics, 2022." *(p.2)*. Per a web search (not read), the AISTATS version is titled "Learning in Stochastic Monotone Games with Decision-Dependent Data", PMLR 151, pp. 5891-5912.
- Journal version: JMLR vol. 24, paper no. 202, pp. 1-56, 2023, same title and authors. Checked on the JMLR page https://jmlr.org/papers/v24/22-0131.html. I did not read the JMLR PDF.

## One-Sentence Summary
The paper defines an n-player game in which each player's loss is an expectation over a data distribution D_i(x) that depends on the *joint* action x of all players. It gives two solution concepts (performatively stable equilibrium, Nash equilibrium), proves that the stable equilibrium exists, is unique and can be found by retraining or gradient methods when the aggregate sensitivity constant rho = sqrt(sum_i (beta_i gamma_i / alpha)^2) < 1, and gives sufficient conditions for strong monotonicity plus Nash-seeking algorithms (derivative-free, SGD with known own-influence A_i, and an adaptive method that estimates A_i).

## Problem Addressed
"The existing strategic classification and performative prediction literature focuses solely on the interplay between a single learner and the population that reacts to the learner's actions. However, learning algorithms in practice are often deployed alongside other algorithms which may even be competing with one another." *(p.2)*. Examples: college admissions and loan procurement, where "the population reacts based on the admissions policies of all the colleges (or banks) simultaneously" *(p.2)*.

## Key Contributions
- "We formulate the first game theoretic model for decision-dependent learning in the presence of competition, called multi-player performative prediction." *(p.2)*
- Performatively stable equilibria exist and are unique "under reasonable smoothness, convexity, and Lipschitz assumptions"; repeated retraining and (stochastic) gradient methods find them, with rates that "reduce to state-of-the art guarantees in the single player setting" *(p.2)*.
- Transparent sufficient conditions for strong monotonicity (Section 5), giving Nash-seeking algorithms (Section 6) *(p.2)*.
- Derivative-free method O(d^2/eps^2); SGD with linear distribution dependence and known own-effects O(d/eps); adaptive algorithm that learns the distribution parameters, matching O(d/eps) up to variance terms *(p.3)*.
- Synthetic (election prediction) and semi-synthetic (Uber/Lyft Boston) experiments *(p.3, Sec. 7)*.

## Study Design
Theory paper with numerical illustrations; no empirical study design in the clinical sense.

## Methodology

### 2.1 Convex games and monotonicity *(p.5)*
Players i in [n], x = (x_1,...,x_n), x_i in R^{d_i}, d = sum d_i. Player i solves

$$
\min_{x_i \in \mathcal{X}_i} \mathcal{L}_i(x_i, x_{-i})
$$

*(p.5, eq. 1)*. Nash equilibrium: x*_i in argmin_{x_i} L_i(x_i, x*_{-i}) for each i. Vector of individual gradients:

$$
H(x) := (\nabla_1 \mathcal{L}_1(x), \ldots, \nabla_n \mathcal{L}_n(x))
$$

*(p.5)*. C^1-smooth convex game: X_i closed convex, L_i(., x_{-i}) convex, partial gradients continuous; Nash characterized by 0 in H(x*) + N_X(x*). alpha-strongly monotone:

$$
\langle H(x) - H(x'), x - x' \rangle \ge \alpha \|x - x'\|^2
$$

*(p.5)*. alpha = 0: monotone. Rosen 1965: alpha > 0 on convex closed bounded X gives a unique Nash equilibrium *(p.5)*.

### 2.2 Wasserstein-1 and gradient deviation *(p.5-6)*

$$
W_1(\mu,\nu) = \sup_{h \in \mathrm{Lip}_1} \left\{ \mathbb{E}_{X\sim\mu}[h(X)] - \mathbb{E}_{Y\sim\nu}[h(Y)] \right\}
$$

*(p.5, eq. 2)*.

Lemma 1 (Gradient deviation): if g(., z) is C^1 and z -> grad_x g(x,z) is beta-Lipschitz, then

$$
\sup_x \|\nabla f_\mu(x) - \nabla f_\nu(x)\| \le \beta \cdot W_1(\mu,\nu)
$$

*(p.6)*.

### 3 The decision-dependent game model *(p.6)*

$$
\min_{x_i \in \mathcal{X}_i} \mathcal{L}_i(x_i, x_{-i}) \quad \text{where} \quad \mathcal{L}_i(x) := \mathbb{E}_{z_i \sim \mathcal{D}_i(x)} \ell_i(x, z_i)
$$

*(p.6, eq. 3)*. l_i : R^d x Z_i -> R; D_i(x) in P(Z_i) "is a probability measure that depends on the joint decision x in X and the player i" *(p.6)*.

**How joint actions shift the distribution (quoted):** "Observe that the random variable z_i in the objective function of player i is governed by the distribution D_i(x), which crucially depends on the strategies x = (x_1,...,x_n) chosen by *all* players. This is worth emphasizing: the parameters chosen by one player have an influence on the data seen by all other players. This is one of the critical ways in which the problems for the different players are strategically coupled. The other is directly through the loss function l_i which also depends on the joint decision x." *(p.6)*

**Separable games:** if l_i(x, z_i) = l_i(x_i, z_i) "we will call the game *separable* (which refers to separable losses, not distributions). Thus, for separable games, the coupling among the players is due entirely to the distribution D_i(x) that depends on the actions of all the players." *(p.6)*

**Remark 1** *(p.6)*: decision dependence may encode strategic users reacting to the announced joint decision x (a Stackelberg layer); "This level of strategic interaction between decision-maker and strategic user is abstracted away to an aggregate level in D_i(x_i, x_{-i})." Both layers together left to future work.

Full information: "We assume that each player has full information of the other players' parameters ... In essense, these decisions are publicly announced." *(p.6)*

**Vignette 1 (Multiplayer forecasting):** shared distribution D = D_i = D_j for all i, j. Mapping apps forecast travel times; "the realized travel time is collectively influenced by all their forecasts"; realized travel time z ~ D on m road segments "observed by all firms" *(p.7)*.

**Vignette 2:** distinct D_i. (a) Multiplayer strategic classification: universities, applicants tailor applications to several universities, "distinct distributions D_i that depend on the joint decision rule x" *(p.7)*. (b) Ride-share demand forecasting: forecasted demand z_i ~ D_i(x) for platform i depends on own x_i and competitors' x_{-i} (drivers/passengers "price shopping") *(p.7)*.

### Solution concepts *(p.7-8)*
**Definition 1 (Nash Equilibrium):** x* in X with x*_i in argmin_{x_i in X_i} L_i(x_i, x*_{-i}) for each i *(p.7)*.

"The game (3) can easily fail to be monotone even if the game is separable and the loss functions l_i(., z) are strongly convex." *(p.7)*. The sufficient conditions of Section 5 "are necessarily quite restrictive" *(p.7)*.

Static game induced by y (distribution frozen at D_i(y)):

$$
\min_{x_i \in \mathcal{X}_i} \mathcal{L}^y_i(x_i, x_{-i}) \quad \text{where} \quad \mathcal{L}^y_i(x_i, x_{-i}) := \mathbb{E}_{z_i \sim \mathcal{D}_i(y)} \ell_i(x_i, x_{-i}, z_i)
$$

*(p.7, game G(y))*.

**Definition 2 (Performatively stable equilibria):** x* is a PSE of (3) "if it is a Nash equilibrium of the static game G(x*)" *(p.8)*. Meaning: "each player i has no incentive to deviate from x* having access only to data drawn from D(x*)" *(p.8)*. For separable games G(y) decouples across players *(p.8)*.

Gradient decomposition (chain rule):

$$
\nabla_i \mathcal{L}_i(x_i,x_{-i}) = \underbrace{\mathbb{E}_{z_i\sim\mathcal{D}_i(x)}[\nabla_i \ell_i(x_i,x_{-i},z_i)]}_{P_i} + \underbrace{\frac{d}{du_i}\mathbb{E}_{z_i\sim\mathcal{D}_i(u_i,x_{-i})}[\ell_i(x_i,x_{-i},z_i)]\Big|_{u_i=x_i}}_{Q_i}
$$

*(p.8, eq. 4)*. Nash: P_i + Q_i = 0 for all i. PSE (convex l_i): P_i = 0 for all i. "performative stability in essence ignores the term Q_i" *(p.8)*. Note Q_i differentiates only through player i's *own* action u_i, with x_{-i} held fixed. "It is an open question as to how these equilibrium concepts compare in terms of their efficiency relative to the social optimum. We explore this empirically in Section 7." *(p.8)*

### Standing assumptions and the sensitivity constant *(p.8)*
**Assumption 1 (Convexity and smoothness):** constants alpha > 0, beta_i > 0: (1) for any y, G(y) is alpha-strongly monotone; (2) l_i(x_i, x_{-i}, z_i) is C^1 in x_i and z_i -> grad_i l_i(x, z_i) is beta_i-Lipschitz for any x *(p.8)*. Separable case: G(y) is alpha-strongly monotone iff (sufficient) each E_{z~D_i(y)} l_i(x_i,z_i) is alpha-strongly convex in x_i. "Assumption 1 alone *does not* imply convexity of the objective functions L_i(x_i,x_{-i}) in x_i nor monotonicity of the game (3) itself." *(p.8)*

**Assumption 2 (Lipschitz distributions):** for each i there is gamma_i > 0 with

$$
W_1(\mathcal{D}_i(x), \mathcal{D}_i(y)) \le \gamma_i \cdot \|x - y\| \quad \forall x, y \in \mathcal{X}
$$

*(p.8)*. Here x, y are joint action vectors in R^d, so gamma_i bounds how much player i's distribution moves per unit movement of *everyone's* actions together (Euclidean norm on the stacked vector). It is not split into own-action and others'-action parts.

$$
\rho := \sqrt{\sum_{i=1}^n \left(\frac{\beta_i \gamma_i}{\alpha}\right)^2}
$$

*(p.8)*. "the constant rho is fundamentally important for algorithms, since it characterizes settings in which algorithms can be expected to work" *(p.8)*.

Notation *(p.9)*: g_i(x,z_i) := grad_i l_i(x,z_i); G_{i,y}(x) := E_{z_i~D_i(y)} g_i(x,z_i); G_y(x) := (G_{1,y}(x),...,G_{n,y}(x)) (eq. 5) = E_{z~D_pi(y)} g(x,z) with D_pi(y) := D_1(y) x ... x D_n(y) the product measure.

**Lemma 2 (Deviation in the vector of individual gradients):** under Assumptions 1-2,

$$
\|G_{i,y}(x) - G_{i,y'}(x)\| \le \beta_i \gamma_i \|y - y'\|
$$

$$
\|G_y(x) - G_{y'}(x)\| \le \sqrt{\textstyle\sum_{i=1}^n \beta_i^2 \gamma_i^2}\, \|y - y'\|
$$

*(p.9)*. Proof: Lemma 1 per player, then sum of squares over players.

## Key Equations / Results by section

### 4 Performatively stable equilibria *(p.9-14)*

**4.1 Repeated retraining.** PSE = fixed points of Nash(x) := {x' : x' is a Nash equilibrium of G(x)} *(p.10)*.

$$
x^{t+1} = \mathrm{Nash}(x^t)
$$

*(p.10, eq. 6)*; each round the players solve min_{y_i} E_{z_i~D_i(x^t)} l_i(y_i, y_{-i}, z_i) (eq. 7). "despite the fact that x^{t+1} is a Nash equilibrium of a game in iteration t, the collective decision x^{t+1} is **not** a Nash equilibrium for the multiplayer performative prediction problem (3)" *(p.10)*.

**Theorem 1 (Repeated retraining):** Assumptions 1-2, rho < 1: unique PSE x*, and

$$
\|x^{t+1} - x^\star\| \le \rho \|x^t - x^\star\|
$$

*(p.10)*. Proof: Nash(.) is rho-Lipschitz via strong monotonicity + Lemma 2 + Banach fixed point. "the interesting parameter regime is rho < 1, since outside of this setting performative equilibria may even fail to exist" *(p.11)*.

Separable case, each round: min_{y_i} E_{z_i~D_i(x^t)} l_i(y_i, z_i) (eq. 8), "entirely independent of each other" *(p.11)*.

**Theorem 2 (Improved contraction for separable games):** separable, L^y_i alpha_i-strongly convex for all y, Assumptions 1-2, regime sum_i (beta_i gamma_i / alpha_i)^2 < 1:

$$
\|x^{t+1} - x^\star\| \le \sqrt{\sum_{i=1}^n \left(\frac{\beta_i\gamma_i}{\alpha_i}\right)^2}\, \|x^t - x^\star\|
$$

*(p.11)*.

**4.2 Repeated gradient method.** x^{t+1} = proj_X(x^t - eta G_{x^t}(x^t)) *(p.12)*. **Assumption 3 (Smoothness):** G_y(x) is L-Lipschitz in x for every y *(p.12)*. **Theorem 3:** Assumptions 1-3, rho < 1/(2+sqrt(2)), eta = alpha/L^2:

$$
\|x^{t+1} - x^\star\| \le \left( \frac{1}{\sqrt{1 + \alpha^2/L^2}} + \frac{\alpha^2 \rho}{L^2} \right) \|x^t - x^\star\|
$$

*(p.12, eq. 9)*. Proof eqs. 10-13 *(p.12-13)*.

**4.3 Repeated stochastic gradient method.** Each player samples z_i^t ~ D_i(x^t), sets x_i^{t+1} = proj_{X_i}(x_i^t - eta grad_i l_i(x_i^t, x_{-i}^t, z_i^t)) *(p.13)*. **Assumption 4 (Finite variance):** E_{z~D_pi(x)} ||G_x(x) - g(x,z)||^2 <= sigma^2 *(p.13)*. Key observation: ||G_x(x) - G_{x*}(x)|| <= alpha rho ||x - x*||, so the method is SGD on the static game G(x*) "with a bias that is linearly bounded by the distance to x*" *(p.13)*.

**Theorem 4 (One-step improvement):** Assumptions 1-4, rho < 1, eta < alpha(1-rho)/(8L^2):

$$
\mathbb{E}_t\|x^{t+1}-x^\star\|^2 \le \frac{1 + 2\eta\alpha\rho + 2\eta^2\alpha^2\rho^2}{1 + 2\eta\alpha(\frac{1+\rho}{2})}\|x^t - x^\star\|^2 + \frac{4\eta^2\sigma^2}{1+2\eta\alpha(\frac{1+\rho}{2})}
$$

*(p.14)*; from Theorem 8 with C_t = D = 0, B = alpha rho.

**Corollary 1 (step-decay):** rho < 1/2, eta_0 := (alpha(1-rho)/4) min{1, 1/(2L^2)}, eta_k = 2^{-k} eta_0; T_0 = ceil(10/((1-rho) alpha eta_0) log(2R/eps)), T_k = ceil(10 log 4/((1-rho) alpha eta_k)), K = ceil(1 + log_2(40 eta_0 sigma^2/((1-rho) alpha eps))). Final iterate E||x - x*||^2 <= eps after at most

$$
\mathcal{O}\left( \frac{L^2}{(1-\rho)\alpha^2}\log\left(\frac{2R}{\varepsilon}\right) + \frac{\sigma^2}{(1-\rho)^2\alpha^2\varepsilon} \right)
$$

iterations *(p.14)*. "coincides with the standard efficiency estimate for the stochastic gradient method on static problems, up to multiplication by (1 - rho)^{-2}" *(p.14)*.

### 5 Monotonicity of decision-dependent games *(p.15-18)*
"the sufficient conditions we present are strong, and necessarily so because the game (3) is typically not monotone. When specialized to the single player setting n = 1, the sufficient conditions we derive are identical to those in [Miller et al., 2021] although the proofs are entirely different." *(p.15)*

**Assumption 5 (Smoothness of the distribution):** u_i -> E_{z_i~D(u_i,x_{-i})} l_i(x,z_i) differentiable at u_i = x_i, derivative continuous in x *(p.15)*.

$$
H_{i,x}(y) := \frac{d}{du_i}\mathbb{E}_{z_i\sim\mathcal{D}(u_i,x_{-i})}\ell_i(y,z_i)\Big|_{u_i=x_i}
$$

*(p.15)*. grad_i L_i(x) = G_{i,x}(x) + H_{i,x}(x); full pseudo-gradient D(x) := G_x(x) + H_x(x). Game monotone iff D monotone *(p.15)*.

Condition: x -> H_x(y) monotone for any y, i.e. for any y the auxiliary game where player i solves min_{x_i} E_{z_i~D_i(x_i,x_{-i})} l_i(y, z_i) is monotone; for n = 1 this is convexity of x -> E_{z~D(x)} l(y,z), "reducing exactly to the requirement in [Miller et al., 2021, Theorem 3.1]" *(p.15)*.

**Lemma 3:** Assumptions 1, 2, 5: for any x, H_x(y) is Lipschitz in y with parameter sqrt(sum beta_i^2 gamma_i^2) *(p.15; proof p.16 eq. 15)*.

**Theorem 5 (Monotonicity of the decision-dependent game):** Assumptions 1, 2, 5, rho < 1/2, and x -> H_x(y) monotone for any y. Then game (3) is strongly monotone with parameter

$$
(1 - 2\rho)\,\alpha
$$

*(p.16)*. Proof: <G_x(x) - G_{x'}(x'), x - x'> >= (1-rho) alpha ||x-x'||^2 (eqs. 17-18), and <H_x(x) - H_{x'}(x'), x - x'> >= -sqrt(sum beta_i^2 gamma_i^2) ||x-x'||^2 = -alpha rho ||x - x'||^2 (eqs. 19-21) *(p.16)*.

**Example 1 (Revenue maximization in ride-share):** l_i(x_i,z_i) = -z_i^T x_i + (lambda_i/2)||x_i||^2; z_i = zeta_i + A_i x_i + A_{-i} x_{-i}, zeta_i ~ P_i; A_i, A_{-i} = own and competitor price elasticities. H_{i,x}(y) = -A_i^T y_i, constant in x, "trivially monotone" *(p.17)*.

**Example 2 (Strategic prediction, election platforms):** z_i | theta ~ phi_i(theta) + A_i x_i + A_{-i} x_{-i} + w_i; l_i = (1/2)||z_i - theta^T x_i||^2. x -> H_x(y) monotone as long as

$$
\sqrt{n-1}\cdot\max_{i\in[n]}\|A_{-i}^\top A_i\|_{op} \le \min_{i\in[n]}\lambda_{\min}(A_i^\top A_i)
$$

*(p.17)*. "The interpretation of this condition is that the performative effects due to interaction with competitors are small relative to any player's own performative effects." *(p.17)*. Proof: H_{i,x}(y) affine in x; V(x) = Diag(A_1^T A_1,...,A_n^T A_n)x, W(x) = (A_1^T A_{-1} x_{-1},...,A_n^T A_{-n} x_{-n}); with s := max_i ||A_i^T A_{-i}||_op, ||W(x)||^2 <= (n-1) s^2 ||x||^2 *(p.18)*. (As printed the condition uses ||A_{-i}^T A_i|| on p.17 and ||A_i^T A_{-i}|| on p.18; these have the same operator norm.)

### 6 Algorithms for Nash equilibria *(p.18-27)*
Nash gradient (both terms):

$$
\nabla_i\mathcal{L}_i(x_i,x_{-i}) = \mathbb{E}_{z_i\sim\mathcal{D}_i(x)}[\nabla_i\ell_i(x,z_i)] + \frac{d}{du_i}\mathbb{E}_{z_i\sim\mathcal{D}_i(u_i,x_{-i})}[\ell_i(x,z_i)]\Big|_{u_i=x_i}
$$

*(p.18, eq. 22)*. PSE algorithms use only the first term. Estimating the second term is the main difficulty *(p.18)*.

**6.1 Derivative-free method** ("bandit feedback", loss queries only) *(p.19, eq. 23)*: sample v_i^t on unit sphere S_i; sample z_i^t ~ D_i(x^t + delta v^t); x_i^{t+1} = proj_{(1-delta)X_i}(x_i^t - eta_t (d_i/delta) l_i(x^t + delta v^t, z_i^t) v_i^t). **Proposition 1:** under reasonable smoothness and bounded variance, E||x - x*||^2 <= eps after at most O(d^2/eps^2) iterations *(p.19)*; stated informally, from Bravo et al. 2018 / Drusvyatskiy et al. 2021.

**6.2 Stochastic gradient method.** **Assumption 6 (Parametric assumption):**

$$
z_i \sim \mathcal{D}_i(x) \iff z_i = \zeta_i + A_i x_i + A_{-i} x_{-i}, \quad \zeta_i \sim \mathcal{P}_i
$$

with mu_i, Sigma_i the mean and covariance of zeta_i *(p.20)*. "the distribution used by player i is a 'linear perturbation' of some base distribution P_i. We can interpret the matrices A_i and A_{-i} as quantifying the performative effects of player i's decisions and the rest of the players' decisions, respectively, on the distribution D_i governing player i's data." *(p.20)*

$$
\mathcal{L}_i(x) = \mathbb{E}_{\zeta_i\sim\mathcal{P}_i}\ell_i(x, \zeta_i + A_i x_i + A_{-i}x_{-i})
$$

*(p.20, eq. 24)*

$$
\nabla_i\mathcal{L}_i(x) = \mathbb{E}_{z_i\sim\mathcal{D}_i(x)}[\nabla_i\ell_i(x,z_i) + A_i^\top\nabla_{z_i}\ell_i(x,z_i)]
$$

*(p.20, eq. 25)*. Unbiased estimator w_i(x,z_i) = grad_i l_i(x,z_i) + A_i^T grad_{z_i} l_i(x,z_i). Update eq. 26: sample z_i^t ~ D_i(x^t), x_i^{t+1} = proj(x_i^t - eta_t w_i(x^t, z_i^t)) *(p.20)*. Requires knowledge of own A_i only (not A_{-i}) *(p.20)*.

**Assumption 7:** (grad_1 L_1,...,grad_n L_n) is L-Lipschitz *(p.21)*. **Lemma 4:** under Assumption 6, if (x,z_i) -> grad_{i,z_i} l_i is xi_i-Lipschitz, then

$$
L = \sqrt{\sum_{i=1}^n \xi_i^2 \max\{1,\|A_i\|_{op}^2\}\,(1 + \|\bar A_i\|_{op}^2)}
$$

with bar A_i x = A_i x_i + A_{-i} x_{-i} *(p.21)*. **Assumption 8:** E||w(x,z) - E w(x,z')||^2 <= sigma^2 *(p.21)*. **Lemma 5:** sigma^2 = sum_i 2(s_1^2 + ||A_i||_op^2 s_2^2) *(p.21)*.

**Theorem 6 (Stochastic gradient play):** Assumptions 6-8, game alpha-strongly monotone, eta <= alpha/(2L^2):

$$
\mathbb{E}\|x^{t+1}-x^\star\|^2 \le \frac{1}{1+\alpha\eta}\mathbb{E}\|x^t-x^\star\|^2 + \frac{2\eta^2\sigma^2}{1+\eta\alpha}
$$

*(p.22, eq. 27)*, x* the Nash equilibrium. **Corollary 2:** eta_k = (alpha/(2L^2)) 2^{-k}; T_0 = ceil(2/(alpha eta_0) log(2R/eps)), T_k = ceil(2 log 4/(alpha eta_k)), K = ceil(1 + log_2(2 eta_0 sigma^2/(alpha eps))); total O(L^2/alpha^2 log(2R/eps) + sigma^2/(alpha^2 eps)) *(p.22)*.

**6.3 Adaptive gradient method** *(p.22-27)*. "adaptive" refers to estimating model parameters, not step sizes (footnote 4, p.22). Each player keeps an estimate hat A_i^t of bar A_i. Uses E[q_i^t - z_i^t | u^t] = bar A_i u^t with q_i^t ~ D_i(x^t + u^t) *(p.23)*; least-squares step

$$
\hat A_i^{t+1} = \hat A_i^t + \nu_t (q_i^t - z_i^t - \hat A_i^t u^t)(u^t)^\top
$$

*(p.23)*. **Algorithm 1** *(p.24)*: for t, for i: draw z_i^t ~ D_i(x^t), q_i^t ~ D_i(x^t + u^t); x_i^{t+1} = proj_{X_i}(x_i^t - eta_t(grad_i l_i(x^t,z_i^t) + (hat A_ii^t)^T grad_{z_i} l_i(x^t,z_i^t))), hat A_ii^t = columns of hat A_i^t indexed by player i; estimation update as above.

**Assumption 9 (Injected noise):** u^t zero-mean, independent of x^t and of previous injected noise; 0 < c_l I <= E[v_i v_i^T], E||v_i||^2 <= c_{u,i}, E[||v_i||^2 v_i v_i^T] <= R^2 E[v_i v_i^T]. Gaussian u ~ N(0, I_d): c_l = 1, c_{u,i} = d_i, R^2 = 3 max_i d_i *(p.23)*.

**Lemma 6 (Estimation error):** nu_t in (0, 2/R^2):

$$
\frac{1}{2}\mathbb{E}_t\|\hat A_i^{t+1} - \bar A_i\|_F^2 \le \frac{1 - c_l\nu_t(2 - \nu_t R^2)}{2}\|\hat A_i^t - \bar A_i\|_F^2 + \nu_t^2\,\mathrm{tr}(\Sigma_i)\,c_{u,i}
$$

*(p.23, eq. 28)*; with nu_t = 2/(c_l(t + 2R^2/c_l)), E||hat A^t - bar A||_F^2 <= max{(1 + 2R^2/c_l)||hat A_1 - bar A||_F^2, 8 sum tr(Sigma_i) c_{u,i}/c_l^2}/(t + 2R^2/c_l) *(p.23)*.

**Assumption 10:** E_{z~D_pi(x)} sqrt(sum ||grad l_i(x,z_i)||^2) <= delta *(p.24)*. **Lemma 7:** E_t||hat v^t - v^t|| <= delta ||hat A^t - bar A||_F^2 (as printed; the proof via Hölder gives ||B^t||_F, i.e. a non-squared norm, so the square in the statement looks like a typo in v2) *(p.25)*. **Assumption 11:** variance of grad_{i,z_i} l_i <= sigma^2 *(p.25)*.

**Theorem 7 (Convergence of the adaptive method):** Assumptions 6, 7, 9, 10, alpha-strongly monotone; k_0 = 1 + 8L^2/alpha^2, q_0 = 2R^2/c_l, eta_t = 2/(alpha(t + k_0 - 2)), nu_t = 2/(c_l(t + q_0)):

$$
\mathbb{E}\|x^t - x^\star\|^2 \le \frac{\max\{\frac12\alpha^2(1+k_0)\|x_1-x^\star\|^2,\ 32(1+2\|\bar A\|_F^2)\sigma^2 + 8\delta^2 Z \max\{\frac{1+k_0}{1+q_0},1\}\}}{\alpha^2(t+k_0)} + \frac{\max\{\frac12\alpha^2(1+k_0)^{3/2}\|x_1-x^\star\|^2,\ 64\sigma^2 Z\max\{\frac{1+k_0}{1+q_0},1\}\}}{\alpha^2(t+k_0)^{3/2}}
$$

with Z = max{(1 + 2R^2/c_l)||hat A^1 - bar A||_F^2, 8 sum tr(Sigma_i) c_{u,i}/c_l^2} *(p.25; proof p.25-26, eq. 29)*. Gaussian case with d_i = C_i d and equal traces gives rate O(d/t) *(p.27)*.

### 7 Numerical examples *(p.27-33)*
**7.1 Strategic prediction (synthetic):** Example 2 model; A_i, A_{-i} generated with scipy.sparse.random under the Example 2 monotonicity condition *(p.27)*. "We set d = 5 and m = 100 for the experiments in Figure 1" *(p.28)* (the Figure 1 caption itself says m_i = 10, d_i = 2 for panel (a) and m_i = 100, d_i = 2 for panel (b); the text and caption disagree as printed). SGM and AGM eta = 0.01; AGM nu = 2/(t + 6d); RSGM eta = 0.1 *(p.28)*.

**7.2 Ride-share (semi-synthetic, Kaggle Uber/Lyft Boston data):** l_i(x_i,z_i) = -(1/2) z_i^T x_i + (lambda_i/2)||x_i||^2 (p.29; Example 1 on p.17 has no 1/2); separable; A_i <= 0 (own elasticity negative), A_{-i} >= 0; lambda_1 = lambda_2 = 1; 11 locations; prices binned at $5 (5 floor(p/5)) *(p.29)*. Elasticities from the heuristic "a 50% increase in price by any firm leads to a 75% decrease in demand"; no cross-location correlation in the presented experiments; $10 nominal price bin *(p.30)*.
- Experiment 1 (iteration complexity): 20 random initial conditions; SGM eta = 5e-5; AGM eta_t = eta_0/t, nu_t = nu_0/t, eta_0 = 5e-5, nu_0 = 1e-4; DFM delta = 5, eta_t = 2/t *(p.30)*. AGM and SGM nearly identical, both beat DFM (Fig. 2, p.29).
- Experiment 2 (social efficiency): social cost S(x) = sum_i L_i(x); x^so by SGD on S with eta = 0.001; NE by SGM, PSE by RSGM, eta = 0.001 *(p.31)*. PoA(x) = max_{x in Eq(G)} S(x) / S(x^so) *(p.31)*. "The empirical price of anarchy for the Nash equilibrium of the ride-share game is PoA-hat(x^ne) = 0.899 while the empirical price of anarchy for the peformatively stable equilibrium is PoA-hat(x^ps) = 0.829" *(p.31)*. (S is negative here because losses are negative revenue, so a ratio below 1 means less total revenue than the social optimum; closer to 1 is better, p.30 Fig. 3 caption.)
- Experiment 3 (ignoring performativity): myopic g_i^t = lambda_i x_i^t - (1/2) zeta_i^t; partially myopic g_i^t = -(A_i - lambda_i I)^T x_i^t - (1/2) zeta_i^t; non-myopic SGM g_i^t = -(A_i - lambda_i I)^T x_i^t - (1/2)(zeta_i^t + A_{-i} x_{-i}^t) *(p.31)*; eta = 0.001 *(p.33)*. "when at least one player is completely myopic, then at least one player is worse off at the Nash equilibrium ... Interestingly, the player that is worse off at the Nash is the non-myopic player. ... when at least one player is partially myopic, the Nash equilibrium always is better for both players" *(p.33)*. Per-location (Fig. 4): most locations see lower demand but higher price and revenue *(p.32-33)*.

### Appendix A: SGD with bias *(p.36-38)*
VI 0 in G(x) + N_X(x) (eq. 30), G L-Lipschitz, alpha-strongly monotone; x^{t+1} = proj_X(x^t - eta g^t) (eq. 31). **Assumption 12:** ||E_t g^t - G(x^t)|| <= C_t + B||x^t - x*||; E_t||g^t - E_t g^t||^2 <= sigma_t^2 + D^2||x^t - x*||^2 *(p.36)*. **Theorem 8:** (Benign bias, C_t = 0, rho := B/alpha in (0,1), eta < alpha(1-rho)/(8L^2)):

$$
\mathbb{E}_t\|x^{t+1}-x^\star\|^2 \le \frac{1+2\eta\alpha\rho + 4\eta^2D^2 + 2\eta^2\alpha^2\rho^2}{1+2\eta\alpha(\frac{1+\rho}{2})}\|x^t-x^\star\|^2 + \frac{4\eta^2\sigma_t^2}{1+2\eta\alpha(\frac{1+\rho}{2})}
$$

(eq. 32); (Offset bias, B = 0, eta <= alpha/(4L^2)):

$$
\mathbb{E}_t\|x^{t+1}-x^\star\|^2 \le \frac{1+2\eta^2D^2}{1+\eta\alpha}\|x^t-x^\star\|^2 + \frac{2\eta^2\sigma_t^2}{1+\eta\alpha} + \frac{2\eta C_t^2}{\alpha(1+\eta\alpha)}
$$

(eq. 33); zero bias: eta <= alpha/(2L^2) suffices *(p.36-37)*. Proof eqs. 34-45 *(p.37-38)*.

### Appendix B, C *(p.39-40)*
Lemma 8: D_{t+1} <= (1 - 2/(t+t_0))D_t + a/(t+t_0)^2 implies D_t <= max{(1+t_0)D_1, a}/(t+t_0) (eqs. 46-47). Lemma 9: with extra b/(t+t_0)^3 term, eq. 49 *(p.39)*. Lemma 10 (online least squares, following Dieuleveut et al. 2017): for nu in (0, 2/R^2), B+ = B - nu(By - b)y^T gives (1/2)E||B+ - V||_F^2 <= ((1 - lambda_1 nu(2 - nu R^2))/2)||B - V||_F^2 + nu^2 sigma^2 lambda_2/2 *(p.39-40)*.

## Parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Number of players | n | - | - | >=1 | 5 | |
| Strong monotonicity of frozen game G(y) | alpha | - | - | >0 | 8 | Assumption 1 |
| Lipschitz constant of z -> grad_i l_i | beta_i | - | - | >0 | 8 | Assumption 1 |
| W1 sensitivity of D_i to joint action | gamma_i | - | - | >0 | 8 | Assumption 2 |
| Aggregate sensitivity ratio | rho | - | - | <1 (retraining), <1/(2+sqrt2) (rep. gradient), <1/2 (Cor 1, Thm 5) | 8,10,12,14,16 | sqrt(sum (beta_i gamma_i/alpha)^2) |
| Smoothness of G_y / pseudo-gradient | L | - | - | >0 | 12, 21 | Assumptions 3, 7 |
| Gradient noise variance | sigma^2 | - | - | >=0 | 13, 21, 25 | Assumptions 4, 8, 11 |
| Repeated gradient step size | eta | - | alpha/L^2 | - | 12 | Thm 3 |
| Strong monotonicity of full game (Thm 5) | - | - | (1-2rho)alpha | - | 16 | |
| Gaussian noise constants | c_l, c_{u,i}, R^2 | - | 1, d_i, 3 max d_i | - | 23 | Assumption 9 |
| Exp 7.1 dims | d, m | - | 5, 100 | - | 28 | caption says m_i=10 or 100, d_i=2 |
| Exp 7.1 step sizes | eta (SGM/AGM), nu, eta (RSGM) | - | 0.01, 2/(t+6d), 0.1 | - | 28 | |
| Ride-share regularization | lambda_i | - | 1 | >=0 | 29 | |
| Ride-share locations | - | count | 11 | - | 29 | |
| Price bin width | - | USD | 5 | - | 29 | $10 bin reported |
| SGM step (Exp 1 ride-share) | eta | - | 5e-5 | - | 30 | |
| AGM steps (Exp 1) | eta_0, nu_0 | - | 5e-5, 1e-4 | - | 30 | eta_t = eta_0/t |
| DFM query radius / step | delta, eta_t | - | 5, 2/t | - | 30 | |
| Social-optimum / NE / PSE solver step | eta | - | 0.001 | - | 31 | |

## Effect Sizes / Key Quantitative Results

| Outcome | Measure | Value | CI | p | Population/Context | Page |
|---------|---------|-------|----|---|--------------------|------|
| Price of anarchy, Nash | empirical PoA | 0.899 | - | - | ride-share, $10 bin | 31 |
| Price of anarchy, PSE | empirical PoA | 0.829 | - | - | ride-share, $10 bin | 31 |
| Derivative-free complexity | iterations | O(d^2/eps^2) | - | - | theory | 19 |
| SGD (known A_i) complexity | iterations | O(L^2/alpha^2 log(2R/eps) + sigma^2/(alpha^2 eps)) | - | - | theory | 22 |
| Adaptive method rate | E||x^t-x*||^2 | O(d/t) | - | - | Gaussian injection | 27 |
| Repeated SGD to PSE | iterations | standard SGD rate x (1-rho)^{-2} | - | - | theory | 14 |

## Figures of Interest
- **Fig 1 (p.28):** strategic prediction; (a) SGM vs AGM error to Nash, same rate; (b) RSGM error to PSE.
- **Fig 2 (p.29):** ride-share Experiment 1; AGM, SGM, DFM error to Nash (DFM plateaus near 10^0, the others near 10^-2); RSGM error to PSE.
- **Fig 3 (p.30):** loss, revenue and PoA at SO, NE, PS; total revenue SO > NE > PS.
- **Fig 4 (p.32):** per-location price/demand/revenue change from modeling decision dependence.
- **Fig 5 (p.32):** myopic and partially myopic variants, change in revenue/demand/price per player.

## Results Summary
- PSE exists, is unique and is found linearly by retraining when rho < 1 (Thm 1, p.10); separable games: sum (beta_i gamma_i/alpha_i)^2 < 1 (Thm 2, p.11).
- Repeated gradient converges linearly when rho < 1/(2+sqrt2) (Thm 3, p.12); repeated SGD has the static-SGD rate times (1-rho)^{-2} (Cor 1, p.14).
- Game is (1-2rho)alpha strongly monotone when rho < 1/2 and the auxiliary map x -> H_x(y) is monotone (Thm 5, p.16).
- Nash-seeking: DFO O(d^2/eps^2); SGD with known own A_i O(1/eps) (O(d/eps) per p.3); adaptive O(d/t) (p.27).
- Experiments: NE more socially efficient than PSE (PoA 0.899 vs 0.829) but close (p.31).

## Limitations
- Monotonicity conditions "are strong, and necessarily so because the game (3) is typically not monotone" *(p.15)*.
- Players are assumed to have full information of the other players' parameters (announced decisions) *(p.6)*.
- The user-level Stackelberg game is abstracted into D_i(x); both layers together left to future work *(p.6)*.
- Proposition 1 stated only informally *(p.19)*.
- Efficiency of PSE vs NE relative to the social optimum is an open question, explored only empirically *(p.8, p.31)*.
- Future work: more general parametric D_i, Stackelberg variant, mechanism design (e.g. a regulator constraining platforms from "manipulating users' preferences to make their prediction tasks easier") *(p.33)*.

## Arguments Against Prior Work
- Single-learner performative prediction / strategic classification ignores competing learners *(p.2)*.
- "Prior formulations of decision dependent learning do not readily extend to the settings described in the vignettes without a game theoretic model." *(p.7)*
- Retraining without considering decision dependence "can lead to unintended consequences, including reinforcing bias" *(p.1)*.
- Classical stochastic programming with decision-dependent uncertainty uses structural assumptions and so "sharply deviate[s]" from the performative prediction framework *(p.4)*.

## Design Rationale
- W1 Lipschitz sensitivity + Lemma 1 lets distribution shift be converted into gradient bias, so every algorithm is analyzed as SGD on a static game with bias (follows Drusvyatskiy & Xiao 2020; Tatarenko & Kamgarpour) *(p.4, p.13)*.
- Linear (location-family) Assumption 6 makes the Q_i term estimable with only own A_i *(p.20)*.
- Adaptive method borrowed from adaptive control; injected isotropic noise for exploration *(p.22-23)*.

## Testable Properties
- If rho < 1, repeated retraining contracts with factor rho to a unique PSE *(p.10)*.
- Separable games: contraction factor sqrt(sum (beta_i gamma_i/alpha_i)^2) *(p.11)*.
- At a PSE (convex l_i), P_i = 0 for all i; at a Nash, P_i + Q_i = 0; they differ whenever Q_i != 0 at the PSE *(p.8)*.
- With rho < 1/2 and monotone x -> H_x(y), strong monotonicity modulus >= (1 - 2rho) alpha *(p.16)*.
- Example 2: monotone when sqrt(n-1) max ||A_{-i}^T A_i|| <= min lambda_min(A_i^T A_i) *(p.17)*.
- Gaussian injected noise: c_l = 1, c_{u,i} = d_i, R^2 = 3 max d_i *(p.23)*.

## Relevance to Project (verifier information budget shared across optimizers)
What the paper does and does not say, against the question of whether an optimization budget is shared across several optimizers:

1. **The shift a player sees is driven by the joint action, and the sensitivity is per distribution, not per mover.** Assumption 2 bounds W1(D_i(x), D_i(y)) <= gamma_i ||x - y|| with x, y the stacked joint actions *(p.8)*. The paper never splits gamma_i into "own" and "others'" parts at this level. In Vignette 1 all players share one distribution D *(p.7)*, so every player's data moves when anyone moves.

2. **The convergence/existence budget is aggregate, in quadrature across players.** rho = sqrt(sum_i (beta_i gamma_i/alpha)^2) *(p.8)*; Theorem 1 needs rho < 1 *(p.10)*; for separable games the condition is sum_i (beta_i gamma_i / alpha_i)^2 < 1 *(p.11)*. So with n identical players each at ratio r = beta gamma / alpha, the condition is n r^2 < 1, i.e. each player's allowed sensitivity ratio shrinks like 1/sqrt(n). This is a sum over players' *distributions* (one term per D_i). It comes from stacking per-player gradient biases in Euclidean norm (Lemma 2 proof, p.9), not from a conservation law. The paper does not state it as a budget.

3. **Monotonicity (needed for Nash) requires cross-effects to shrink with n.** Example 2: sqrt(n-1) max ||A_{-i}^T A_i||_op <= min lambda_min(A_i^T A_i) *(p.17)*: "performative effects due to interaction with competitors are small relative to any player's own performative effects." The sqrt(n-1) comes from the (n-1) other players adding up in ||W(x)||^2 <= (n-1)s^2||x||^2 *(p.18)*. That is an aggregate-vs-per-player statement: the total competitor-induced shift must stay below each player's own effect.

4. **Stable vs Nash = whether a player accounts for its own shift of the distribution.** Q_i in eq. 4 differentiates D_i through u_i only, holding x_{-i} fixed *(p.8)*. Neither concept has a player account for how its action moves *other* players' distributions; the social optimum (S = sum L_i, p.31) is the only object that internalizes cross-shifts. PoA: NE 0.899, PSE 0.829 *(p.31)*.

5. **Metric is W1/Lipschitz, not information.** Nothing in the paper is stated in KL, mutual information or nats; I found no information-theoretic budget. Any mapping from gamma_i (W1 per unit action) to "nats of optimization" would be our own construction, not the paper's.

6. **Myopia result (Exp 3, p.33):** when one player ignores performative effects, the non-myopic player is the one worse off. This is an empirical observation in one semi-synthetic two-player game, not a theorem.

## Open Questions
- [ ] Does the JMLR 2023 version (56 pp) change any theorem, constant, or the Lemma 7 squared-norm typo? Not checked.
- [ ] Is there a natural KL/f-divergence analogue of Assumption 2 under which rho becomes an information quantity? The paper does not address this.
- [ ] Figure 1 text (d=5, m=100) vs caption (m_i=10/100, d_i=2) mismatch *(p.28)*.

## Related Work Worth Reading
- Perdomo, Zrnic, Mendler-Dünner, Hardt 2020, Performative prediction (ICML): single-player origin. → NOW IN COLLECTION: [Performative Prediction](../Perdomo_2020_PerformativePrediction/notes.md)
- Miller, Perdomo, Zrnic 2021, Outside the echo chamber: convexity conditions this paper generalizes.
- Drusvyatskiy & Xiao 2020, Stochastic optimization with decision-dependent distributions: the biased-SGD view.
- Rosen 1965, Existence and uniqueness of equilibrium points for concave n-person games.
- Drusvyatskiy, Fazel, Ratliff 2021, Improved rates for derivative free gradient play in monotone games.
- Mendler-Dünner et al. 2020, Stochastic optimization for performative prediction (NeurIPS).

## Collection Cross-References

### Already in Collection
- [Performative Prediction](../Perdomo_2020_PerformativePrediction/notes.md) - cited as the single-player origin of performative prediction and of the performative stability vs optimality distinction (p.2); this paper's rates "reduce to state-of-the art guarantees in the single player setting" (p.2).

### Now in Collection (previously listed as leads)
- [Performative Prediction](../Perdomo_2020_PerformativePrediction/notes.md) - Perdomo et al. define the distribution map θ -> D(θ), ε-sensitivity W1(D(θ), D(θ')) ≤ ε||θ − θ'||, and performative stability vs optimality. Repeated risk minimization is an εβ/γ-contraction that converges linearly to a unique stable point when ε < γ/β, tight in the worst case (Perdomo Thm 3.5, Prop 3.6), and stable points lie within 2L_z ε/γ of performative optima (Perdomo Thm 4.3). At n = 1 this paper's ρ = βγ/α is the same quantity in other letters (this paper's γ_i is Perdomo's ε, its α is Perdomo's γ, its β_i is Perdomo's β in z), so Theorem 1's ρ < 1 matches Perdomo's ε < γ/β. No conflict found.

### New Leads (Not Yet in Collection)
- Miller, Perdomo & Zrnic (2021) - "Outside the echo chamber: Optimizing the performative risk" - convexity conditions; Theorem 5 reduces to their Theorem 3.1 at n = 1 (p.15).
- Drusvyatskiy & Xiao (2020) - "Stochastic optimization with decision-dependent distributions" - decision-dependent SGD as static SGD with bias, the analysis template used throughout (p.4, p.13).
- Rosen (1965) - "Existence and uniqueness of equilibrium points for concave n-person games" - strong monotonicity gives a unique Nash equilibrium (p.5).
- Drusvyatskiy, Fazel & Ratliff (2021) - "Improved rates for derivative free gradient play in monotone games" - source of the Proposition 1 rates (p.19).
- Mendler-Dünner, Perdomo, Zrnic & Hardt (2020) - "Stochastic optimization for performative prediction" - stochastic methods for the single-player case.

### Supersedes or Recontextualizes
- [Performative Prediction](../Perdomo_2020_PerformativePrediction/notes.md) - extends single-player performative prediction to n players whose distributions D_i(x) depend on the joint action; the existence and convergence condition becomes the aggregate ρ = sqrt(Σ_i (β_i γ_i/α)²) < 1 (Thm 1, p.10), which reduces to Perdomo's ε < γ/β at n = 1.

### Cited By (in Collection)
- (none found). Perdomo's notes list this paper under Related Work as already in collection, but Perdomo et al. (2020) predates it and does not cite it.

### Conceptual Links (not citation-based)
- [How much does your data exploration overfit? Controlling bias via information usage](../Russo_2015_MuchDoesDataExploration/notes.md) - Moderate. Two different rules for how a budget aggregates across several optimizing steps or actors. Here the stability condition sums squared per-distribution sensitivities across players in quadrature (ρ² = Σ_i (β_i γ_i/α)², p.8), so with n identical players each player's allowed ratio shrinks like 1/sqrt(n). In Russo & Zou's multi-step analyst, information usage accumulates additively over adaptive steps, I(T_{k+1}; φ) ≤ Σ_i I(Y_{T_i}; φ_{T_i} | H_{i−1}, T_i) (Russo Lemma 1, p.11), and bias is bounded by σ·sqrt(2I). Russo's per-statistic step KL ≥ Δ²/(2σ²) for a σ-sub-Gaussian statistic (Russo p.13) plays the role for KL that this paper's Lemma 1 (gradient deviation ≤ β·W1) plays for Wasserstein-1, which is the ingredient this paper's open question about a KL analogue of Assumption 2 would need; neither paper makes that connection.
