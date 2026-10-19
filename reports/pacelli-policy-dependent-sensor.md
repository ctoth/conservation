# Pacelli's bound with a policy-dependent sensor

Question (Q, 2026-10-19): does Pacelli & Theodorou's free-energy bound survive when the sensor
h depends on the policy π, and what does that give for Goodhart?

Source: Pacelli & Theodorou 2026, arXiv 2603.18454 v3 (`papers/Pacelli_2026_FundamentalLimitsSensor-BasedControl/`).
Every equation number below is theirs; I read eqs. (1)–(16), Theorem 1, Propositions 1–3,
Definition 1 and Theorem 2 on the page images (PDF pp. 2–4). Claims marked **derived** are my
algebra on top of the paper and have not been checked by anyone else.

## 1. Where the fixed sensor enters Theorem 1

Theorem 1's proof (PDF p. 2–3):

1. (5) builds two path measures that share the dynamics: P^π uses the real sensor
   ∏ p(y_t | x_t); Q^π replaces it with arbitrary densities ∏ ν_t(y_t), so under Q^π the
   observations are independent of the state. Controls are u_t = π_t(y_{0:t}) in both.
2. (6) applies the Gibbs variational principle (GVP) with ℓ = J_trj[π].
3. (7): the dynamics cancel, so D[P^π ‖ Q^π] = Σ_t E_π D[p(y_t|x_t) ‖ ν_t(y_t)], minimized at
   ν_t = p^π(y_t), giving Σ_t I_π[x_t; y_t] ≤ Ī.
4. (8)–(9): under Q^π, a fixed y_{0:T} fixes u deterministically, so the expectation factors and
   is bounded by exp(−β inf_u{F_β^x(u) + J_trj^u(u)}).
5. (10): J[π] ≥ inf_u{F_β^x + J_trj^u} − Ī/β for every π and β; optimizing gives (4).

The sensor appears only in step 1's factor p(y_t | x_t). Nothing else in the proof uses h.

## 2. The proof with a policy-dependent sensor (derived)

**Global dependence (the channel's law is chosen with π, as in performative prediction).**
Fix π; its sensor p_π(y_t | x_t) is then one fixed channel. Steps 1–5 go through verbatim with
p_π in place of p, so

  J[π] ≥ inf_u{F_β^x(u) + J_trj^u(u)} − I(π)/β,  I(π) := Σ_t I_π[x_t; y_t],     (A)

with I_π computed under π's own channel and π's own state distribution.

**History dependence (the sensor at t depends on y_{0:t−1}, hence on past controls).**
Take the reference ν_t(y_t | y_{0:t−1}). Under Q^π the observation process still has a law of its
own, independent of x and of the noise ξ, so the factorization (8) still holds. The KL becomes
Σ_t E_π D[p_π(y_t | x_t, y_{<t}) ‖ ν_t(· | y_{<t})], minimized at ν_t = p^π(y_t | y_{<t}):

  I(π) := Σ_t I_π[x_t; y_t | y_{0:t−1}].                                              (B)

The sensor cannot depend on the current u_t: u_t = π_t(y_{0:t}) already depends on y_t.

**The per-policy bound.** Taking the supremum over β in (A) gives J[π] ≥ J_irr(I(π)) for each
policy separately, with J_irr as in (4). By Proposition 1 (needs J_ol > 0), J_irr is strictly
decreasing and J_irr(0) = J_ol. So

  J_ol − J[π] ≤ J_ol − J_irr(I(π)).                                                   (C)

The true-cost improvement any policy achieves over the best open-loop sequence is bounded by
an increasing function of the information its own channel carries, measured under its own
closed-loop distribution. A policy whose channel carries no information (I(π) = 0) cannot beat
open loop.

**What Theorem 1 as stated needs.** The bound on J* (rather than on one J[π]) needs Ī ≥ I(π) for
every π ∈ Π, because J* = inf_π J[π]. Theorem 2's route needs less: by Definition 1(b), Ī(J) only
has to cover policies with J[π] ≤ J.

## 3. The certificate

Definition 1(b) requires I(π) ≤ Ī(J) whenever J[π] ≤ J. Proposition 2's proof (p. 3) uses:
- (12): Assumptions 1–2 give a Σ_t tr(M_t) ≤ J + b(T+1), with M_t = E_π[x_t x_tᵀ];
- (13): Gaussian maximum entropy, I_π[x_t; y_t] ≤ ½ log det(I_p + Σ_v⁻¹ Cov(h(x_t))), which needs
  y_t = h(x_t) + v_t with v_t ~ N(0, Σ_v) independent of x_t (eq. 2);
- L-Lipschitz h: tr(Σ_v⁻¹ Cov(h(x_t))) ≤ L² tr(M_t)/λ_min.

**Derived:** with global dependence, Proposition 2 holds unchanged for any family {h_π} that is
uniformly L-Lipschitz with the same additive Gaussian noise: (12) does not involve h, (13) holds
per π, and tr Cov(h_π(x)) ≤ E‖h_π(x) − h_π(0)‖² ≤ L² tr(M_t) uses only the Lipschitz constant. For
history dependence the conditional form of (13) plus Jensen looks like it gives the same bound,
but I have not written that out.

Consequence: a uniform Lipschitz family cannot "game" the certificate. Moving h_π within the
family only lowers I(π), which only raises that policy's floor in (C). The fixed point of
Theorem 2 stays valid.

**Binary verifiers.** For y_t ∈ {0,1}, (13) does not apply, but I_π[x_t; y_t] ≤ H(y_t) ≤ log 2,
and → 0 as the pass rate → 1. Ī = (T+1) log 2 is a valid budget for Theorem 1. It is constant,
so it fails Definition 1(a) (strictly increasing) and Theorem 2 does not apply to it as stated.

## 4. What this does and does not give for Goodhart

Role map: x = the thing whose true cost is charged, y = the verifier's output, π = the optimizer.

**Gives:** (C) is a rigorous form of "a check can only buy as much true improvement as the
information it carries", and the information is measured under the optimized distribution,
not at initialization. That matches the β check against Gao et al. (task note): I estimated
I(proxy; gold) at the *initial* policy and the conjecture missed by ~two orders of magnitude.
(C) says the relevant I is the one under π.

**Does not give:**
1. *Why a proxy-optimizing policy has small I(π).* Pacelli's controllers minimize the true cost;
   (C) bounds any policy, but nothing in the paper links optimizing a proxy to losing channel
   information. That link is the Goodhart content and has to come from a model of the proxy.
2. *Deterministic verifiers.* Reward models are deterministic functions of the response. If x is
   the response and y = f(x) is continuous and noiseless, I_π[x; y] = ∞ and (C) is vacuous.
   Pacelli's noise v_t is what makes the information finite. A usable version must charge
   information to the cost-relevant part of x, not the whole state; for multi-step feedback I
   don't see how to coarsen the GVP step that way yet.
3. *State-dependent noise.* Eq. (2) needs v_t i.i.d. and independent of the process noise. A
   policy that shapes the channel can make the noise depend on the state; (13) then fails.

## 5. Side result for best-of-n via Russo & Zou (derived)

The one-step case avoids problem 2. Russo & Zou's Proposition 1 (checked, p. 3 of
`papers/Russo_2015_MuchDoesDataExploration/`) holds for any selection rule T. Take n i.i.d.
candidates with gold scores g_i (σ-sub-Gaussian, mean μ) and proxy scores p_i, and let T pick a
candidate using the proxies only. Then the gold gain E[g_T] − μ is Russo's bias for φ = g, so

  E[g_T] − μ ≤ σ √(2 I(T; g)),  and  I(T; g) ≤ min( H(T), I(p; g) ) ≤ min( log n, n·I(p_1; g_1) ),

by data processing (T depends on g only through p) and independence across candidates.
KL_bon = log n − (n−1)/n ≤ log n.

So the gold gain is limited by the smaller of the selection pressure (≈ KL) and the total
information the n proxy scores carry about gold, n·I. The crossover is at log n ≈ n·I, not at
KL ≈ I as in the original conjecture. This is an upper bound, not a prediction of where Gao's
peak sits, and it uses I at the initial distribution (fine here, since best-of-n draws its
candidates from the base policy).

## Open

- Check (B) and the history-dependent certificate in full.
- A proxy model that makes I(π) fall under proxy optimization (problem 1). Candidate: a binary
  verifier whose false-pass region the policy can enlarge (a toy model; code, needs Q's go).
- Charging information to the cost-relevant statistic for multi-step feedback (problem 2).
