# Abstract

## Original Text (Verbatim)

Learning problems commonly exhibit an interesting feedback mechanism wherein the population data reacts to competing decision makers' actions. This paper formulates a new game theoretic framework for this phenomenon, called *multi-player performative prediction*. We focus on two distinct solution concepts, namely (*i*) performatively stable equilibria and (*ii*) Nash equilibria of the game. The latter equilibria are arguably more informative, but can be found efficiently only when the game is monotone. We show that under mild assumptions, the performatively stable equilibria can be found efficiently by a variety of algorithms, including repeated retraining and the repeated (stochastic) gradient method. We then establish transparent sufficient conditions for strong monotonicity of the game and use them to develop algorithms for finding Nash equilibria. We investigate derivative free methods and adaptive gradient algorithms wherein each player alternates between learning a parametric description of their distribution and gradient steps on the empirical risk. Synthetic and semi-synthetic numerical experiments illustrate the results.

(arXiv:2201.03398v2, p.1)

---

## Our Interpretation

The paper extends single-learner performative prediction to several competing learners whose joint actions move each player's data distribution. Its central constant, rho = sqrt(sum_i (beta_i gamma_i/alpha)^2), adds per-player sensitivity ratios in quadrature, so the regime in which retraining converges shrinks as players are added. This bears on whether a shift budget is shared across optimizers, but it is stated in Wasserstein-1 terms, not information.
