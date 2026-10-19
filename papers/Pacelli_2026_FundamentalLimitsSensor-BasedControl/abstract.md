# Abstract

## Original Text (Verbatim)

Fundamental limits on the performance of feedback controllers are essential for benchmarking algorithms, guiding sensor selection, and certifying task feasibility—yet few general-purpose tools exist for computing them. Existing information-theoretic approaches overestimate the information a sensor must provide by evaluating it against the uncontrolled system, producing bounds that degrade precisely when feedback is most valuable. We derive a lower bound on the minimum expected cost of any causal feedback controller under partial observations by applying the Gibbs variational principle to the joint path measure over states and observations. The bound applies to nonlinear, nonholonomic, and hybrid dynamics with unbounded costs and admits a self-consistent refinement: any good controller concentrates the state, which limits the information the sensor can extract, which tightens the bound. The resulting fixed-point equation has a unique solution computable by bisection, and we provide conditions under which the free energy minimization is provably convex, yielding a certifiably correct numerical bound. On a scalar LQG problem the self-consistent bound captures over 80% of the known optimal cost at moderate sensor noise, and on a nonlinear Dubins car tracking problem it remains informative across all noise levels where a bound using the uncontrolled state distribution is vacuous.

---

## Our Interpretation

Prior information-theoretic control limits (TRIP, refs [5],[6]) count the sensor information a controller could use under the open-loop state distribution, which overstates it when feedback works. This paper bounds cost via a free-energy/KL argument and then makes the information budget depend on the cost itself: low cost forces a concentrated state, which caps the mutual information of a fixed sensor, giving a self-consistent fixed-point lower bound. For a study of limits where the information channel is endogenous to the policy, it is a worked example of the information a fixed channel carries being set by the closed-loop distribution.
