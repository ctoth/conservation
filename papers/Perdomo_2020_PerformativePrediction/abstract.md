# Abstract

## Original Text (Verbatim)

When predictions support decisions they may influence the outcome they aim to predict. We call such predictions *performative*; the prediction influences the target. Performativity is a well-studied phenomenon in policy-making that has so far been neglected in supervised learning. When ignored, performativity surfaces as undesirable distribution shift, routinely addressed with retraining.

We develop a risk minimization framework for performative prediction bringing together concepts from statistics, game theory, and causality. A conceptual novelty is an equilibrium notion we call performative stability. Performative stability implies that the predictions are calibrated not against past outcomes, but against the future outcomes that manifest from acting on the prediction. Our main results are necessary and sufficient conditions for the convergence of retraining to a performatively stable point of nearly minimal loss.

In full generality, performative prediction strictly subsumes the setting known as *strategic classification*. We thus also give the first sufficient conditions for retraining to overcome strategic feedback effects.

---

## Our Interpretation

The paper treats a deployed model as choosing its own test distribution via a map D(θ), and asks when retraining on the data a model induces settles down. The answer is a contraction condition: the Wasserstein sensitivity ε of D(·) must be below the loss's inverse condition number γ/β, in which case retraining converges linearly to a stable point whose distance to the true performative optimum is O(L_z ε/γ). For a Goodhart study it supplies the cleanest bound on how far a model can move the distribution it is scored on, under an assumed Lipschitz response.
