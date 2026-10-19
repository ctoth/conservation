---
title: "Violations of robustness trade-offs"
authors: "Hiroaki Kitano"
year: 2010
venue: "Molecular Systems Biology 6:384 (Perspective)"
doi_url: "https://doi.org/10.1038/msb.2010.40"
pages: "8 (main) + 3 (Supplementary Material)"
note: "Read from page images of the PMC open-access PDF (PMC2913402) and the Supplementary Material PDF msb201040-s1.pdf. Main-text page numbers are the printed page numbers (1-8; printed page = PDF index + 1). Supplementary pages are unnumbered in the source; cited here as S1-S3 in reading order."
---

# Violations of robustness trade-offs

## One-Sentence Summary

A qualitative Perspective arguing, by analogy with Markowitz portfolio theory and Pareto efficiency, that robustness trade-offs (robustness vs. fragility, robustness vs. performance) bind only on the efficient frontier, i.e. only once a system is sufficiently optimized; a suboptimal system can gain robustness and performance together; the paper gives no theorem, no equation, and no data, only toy examples (a resistor circuit, a hypothetical cell culture) and proposed experiments.

## Problem Addressed

Prior work (Csete & Doyle 2002; Kitano 2004, 2007) claims intrinsic trade-offs among robustness, fragility, performance and resource demand, supported by the Bode integral and the metabolic-control-analysis summation theorem. Kitano asks whether these trade-offs hold unconditionally or only under specific conditions *(p.1)*. He also wants to explain observations that seem to breach the trade-off: suboptimal metabolism in *B. subtilis* and *E. coli*, and "cost-free" compensatory evolution of drug resistance *(p.1, p.5-7)*.

## Key Contributions

- The central conjecture: trade-offs "hold only when design and implementation are sufficiently optimized", so "the system can be made more robust without undermining other features" when it is not *(p.1)*.
- A circuit toy example (Box 1) showing a design change that improves robustness to component failure with no loss elsewhere *(p.2)*.
- A translation of portfolio selection theory into biology ("genetic portfolio selection"): each organism or cell maps to a point in yield-risk space (expected growth vs. s.d. of growth across perturbations); the trade-off is the efficient frontier *(p.3-5)*.
- A reinterpretation of indifference curves as a reflection of the perturbation level imposed on cells, not of investor preference *(p.5)*.
- Testable predictions for artificial-evolution / cell-culture experiments, and implications for drug screening and iPS-cell selection *(p.5, p.7-8)*.
- Supplementary Material: a separate trade-off between feedback gain and component-failure risk, which Kitano says "is different from Bode's theorem" *(S3)*.

## The exact central claim (verbatim, with pages)

- Abstract: "we reassess this concept and argue that biological robustness may hold only when a system is sufficiently optimized and that it may not be conserved when there is room for optimization in its design." *(p.1)* Note: the abstract says "biological robustness may hold", not "robustness trade-offs may hold"; the body text (below) makes clear the trade-off is what is meant.
- Introduction: "Such trade-offs may thus hold only when design and implementation are sufficiently optimized. This means that the system can be made more robust without undermining other features (see Box 1)." *(p.1)*
- Introduction: "It should be noted that the system can be optimized by redesign and reimplementation of engineering systems. In biological systems, an evolutionary selection is required for such design optimizations. Although qualitative observations exist for this trade-off, quantitative experimental verification of this trade-off has not been conducted." *(p.1)*
- "In this paper, we examine the idea that such trade-offs appear only when the system is sufficiently optimized, and thus may not be observed when systems are yet to be fully optimized. This implies that there should be cases in which descendents of organisms can be more robust and perform as well as or better than their ancestors, which is not possible if robustness-performance trade-off holds universally." *(p.2)*
- Predictions section: "A trade-off emerges only at the efficient frontier." ... "Only when the population is on the efficient frontier do changes in yield and risk affect each other, hence a trade-off emerges. Therefore, robustness (or fragility) is not always conserved; it is conserved only when the system is on the efficient frontier." *(p.7)*
- Box 2 lead-in: "Changes in the distribution of projected positions in the yield-risk space for randomly sampled cells will test the conjecture that the robustness-performance trade-off holds only at the efficient frontier." *(p.4)*

## Scope of the Bode / summation theorems as Kitano states it

- "Principles such as the Bode integral formula (Bode, 1945) and the summation theorem in metabolic control analysis (Fell, 1997) underscore this trade-off in certain conditions. Although such theorems provide a basis for understanding robustness trade-offs, their applications are limited to specific aspects of the system. The Bode integral formula applies specifically to conservation of the sensitivities of negative feedback circuits on a frequency axis, and the summation theorem assumes linearization for minor perturbations. In addition, real systems are likely to exhibit more complex robustness-fragility trade-offs because of the involvement of component failures and other aspects not taken into account for these theorems (see Supplementary information)." *(p.1)*
- The paper never writes the Bode integral or the summation theorem as an equation. The only statement of Bode's content is verbal: "According to Bode's theorem, S+ and S- are generally equal (illustrated in the inset)" *(S2)*, and "Bode's theorem, however, indicates that this reduction of sensitivity causes extreme sensitivity elsewhere (Figure S1, inset). This is specifically applied for the frequency axis of the negative feedback system." *(S1)*

## Formal content: what is and is not stated

There is **no formal theorem, no definition with quantifiers, and no equation** in the main text or the supplement. The "formal" content is:

1. **Definitions (verbal).**
   - Robustness: "'robustness' means an individual organism's capability of tolerating external and internal perturbations, such as environmental fluctuations, the addition of drugs, and mutations." *(p.1)*
   - Robustness-performance trade-off: "when two individuals are compared, one is found to be more robust than the other but is outperformed by the other; thus, no individual can be more robust and at the same time exhibit higher performance than others." *(p.1)* This is a pairwise-incomparability (Pareto) definition.
   - Projected position: "Each organism or cell can be mapped onto the yield-risk space. A position of the yield-risk space that characterizes the biological entity X can be called 'a projected position of X.'" *(p.3)*
   - Yield: "an expected performance, such as reproduction speed or biomass production rate." Risk "(equivalent to fragility) is the degree to which a growth rate or biomass production rate is affected by perturbations. In general, it can be represented by s.d. and calculated by assuming possible perturbations, their probabilities of occurrence, and expected yield under each perturbation (Box 2)." *(p.3)*
   - Efficient frontier: "a set of points that represent an optimal combination of assets ... that maximizes the return for any given level of s.d." *(p.2)*
   - Pareto efficiency: "For a Pareto-efficient solution, no individual parameter can be improved without undermining another parameter. A set of Pareto-efficient solutions constitute a Pareto surface, also called a Pareto frontier." *(p.2)*
2. **The trade-off's condition.** The trade-off holds at (and only at) points on the efficient/Pareto frontier. Off the frontier, "any portfolio not on the efficient frontier can improve its yield without increasing risk, or reduce risk without undermining the expected yield. However, on the efficient frontier, any change in yield affects risk and vice versa. Trade-off between risk and yield takes place on an optimal portfolio that is on the efficient frontier." *(p.2)* This is the definition of Pareto optimality restated; the paper adds no further condition (e.g. convexity, smoothness, a fixed perturbation set) explicitly, though it implicitly fixes the perturbation distribution (see Box 2 and Fig. 4C, where a new drug moves the frontier).
3. **Frontier shift.** The frontier itself depends on the perturbation set: "The efficient frontier has changed because of the presence of a drug that was not a factor in determining the original efficient frontier (Trajectory E or F in Figure 4C)." *(p.7)*

Implied formula (NOT in the paper; my restatement of what "risk = s.d." means given the Box 2 description): for a subtype with growth rate g_k under perturbation k occurring with probability π_k,

$$
Y = \sum_k \pi_k g_k
$$

$$
R = \sqrt{\sum_k \pi_k (g_k - Y)^2}
$$

Where: Y is expected yield, R is risk (s.d.), k ranges over "no perturbation" and perturbations 1-4. Basis: p.3 ("calculated by assuming possible perturbations, their probabilities of occurrence, and expected yield under each perturbation") and Box 2 *(p.4)*. The paper does not write these.

## Box 1: Design suboptimality and robustness-fragility trade-offs (circuit toy example) *(p.2)*

Figure: three non-inverting op-amp feedback amplifiers. Input to + terminal; feedback network from output to - terminal with a resistor to GND.
- **Design A:** one resistor in the feedback loop.
- **Design B:** two resistors in series in the feedback loop.
- **Design C:** two resistors in parallel in the feedback loop.

Verbatim caption (key sentences):
- "A simple toy example using electric circuits is presented here to show that the trade-off holds only when system design and implementation are sufficiently optimized. In other words, a circuit can be made more robust without scarifying [sic] other features."
- "Assume a simple feedback amplifier in which a feedback loop consists of resistors (Design A). There is clearly a robustness-fragility trade-off due to the feedback loop, as depicted in the Bode theorem (Bode, 1945)."
- "Without affecting the trade-off due to the feedback, the actual implementation of the amplifier can be changed to have two serially connected resistors in its feedback loop (Design B). Failure of one of these resisters may cause dysfunction in the feedback loop and undermine the system stability. Alternatively, the use of resistors connected in parallel would reduce such a failure risk (Design C)."
- "Assuming that a probability of failure of each resister is P, Design B is susceptive to component failure, as a probability of failure of the feedback loop simply doubles (2P). Design A reduces such a risk to half of that of Design B because it only uses one resister; hence, probability of failure is P. Design C improves robustness against component failure on feedback loops because of parallel construction. Now it has only P^2, whereas resource demand is equivalent to Design B."
- "For example, a design change from Design B to Design C improves the robustness of the system against component failures without increasing fragility elsewhere, undermining performance, or requiring major additional resources. Thus, robustness is improved without substantial trade-offs. Change from Design B to A actually reduces resource demands slightly and improves robustness. Change from Design A to C improves robustness with minimum increase in resource demands."
- "However, attempts to totally eliminate component failure, not only for a feedback loop but for every aspect of the system, would require multiple redundancies for every aspect of the system, which would require major resources. How design affects the system vulnerability to component failure is a complex issue, and biological cases need to be further explored. It will be further complicated when feedback loops are involved (see Supplementary information)."

Box 1 quantities:

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Per-resistor failure probability | P | - | - | 0-1 | 2 | Assumed equal for every resistor |
| Feedback-loop failure probability, Design A (single) | - | - | P | - | 2 | One resistor |
| Feedback-loop failure probability, Design B (series) | - | - | 2P | - | 2 | Paper's value; exact for independent failures is 1-(1-P)^2 = 2P-P^2, so 2P is the small-P approximation (my note) |
| Feedback-loop failure probability, Design C (parallel) | - | - | P^2 | - | 2 | Requires both to fail; implicitly assumes independence and that one surviving resistor keeps the loop functional (my note) |
| Resource demand, Design A | - | resistors | 1 | - | 2 | "reduces resource demands slightly" vs B |
| Resource demand, Designs B and C | - | resistors | 2 | - | 2 | "resource demand is equivalent to Design B" |

What Box 1 does and does not show (my reading, not the paper's words):
- The Bode/feedback trade-off is held fixed across A, B, C ("Without affecting the trade-off due to the feedback"). Box 1 varies a *different* axis, component-failure robustness, and shows that axis is not on a frontier for Design B: C dominates B (lower failure, same resource), A dominates B (lower failure, fewer resources). So Box 1 is a dominance argument on the (failure probability, resource) plane, not a violation of Bode's integral.
- The caption does not address that in a real amplifier the series/parallel choice changes the resistance value (hence the gain) unless resistor values are rescaled, nor what failure mode (open vs. short) is assumed. The paper is silent; flagged as an open question below.
- The final sentence concedes a cost frontier does exist eventually: eliminating all component failure "would require major resources".

## Supplementary Material: "Design sub-optimality and robustness-fragility trade-offs with feedback regulation" *(S1-S3)*

Purpose: "While BOX1 illustrates a simple case of design optimization and fragility, this material provides description of cases component failures and feedback regulations are coupled." *(S1)*

Key statements (verbatim):
- "The impact of component failure will be different depending on which component fails. In a feedback amplifier, sensitivities to non-linearity of components and to noise can be further reduced by a higher feedback level. Bode's theorem, however, indicates that this reduction of sensitivity causes extreme sensitivity elsewhere (Figure S1, inset). This is specifically applied for the frequency axis of the negative feedback system. The level of feedback is called 'feedback gain'." *(S1)*
- "Failure of feedback components in a high feedback gain amplifier is more serious than that in a low feedback gain amplifier because the system's function is more dependent on the feedback control. Failure of components not involved in the feedback loop equally affects all systems regardless of feedback gain, at least in theory." *(S1)*
- "Design optimization using redundancy and other mechanisms will significantly reduce sensitivity of the system to component failure without affecting sensitivity elsewhere." *(S1)*
- Sloppy components: "Situations may be more complicated in biological systems because some components may be evolved to be 'sloppy' (unreliable, noisy, etc.) when a feedback loop compensates for their 'sloppiness.' This may be possible as selection pressure for these components is mitigated due to a compensatory effect of the feedback. If this is the case, a system with larger feedback may entail 'sloppier' components in its feedback loop. Failure of components in a feedback loop may be catastrophic for these systems as some components have already evolved to be unreliable and, therefore, dependent on the feedback." *(S1)*
- "According to Bode's theorem, S+ and S- are generally equal (illustrated in the inset), as represented by the straight solid line. With A as a reference point of the system, a design with higher component failure will deviate from the system's operating point to C, which has higher sensitivity. Here, S+ is the sum of sensitivity due to feedback and probability of system malfunction due to component failure." *(S2)*
- "Increasing the level of feedback further reduces sensitivity in a relatively low frequency region, but the trade-off of increasing sensitivity in a higher frequency region exists. Thus, B, a higher feedback gain version of A, is also on the solid line. Reduction of feedback gain from B may only result in shifting the operating point to A if identical components have been used for both A and B." *(S2)*
- Numerical example: "Assume the system can tolerate 5% non-linearity of a component and the system initially uses a component of 4% non-linearity. Negative feedback can be set to reduce the component non-linearity to 1/10 to achieve 0.4% non-linearity. In an engineering system, the design is fixed. However, in biological systems, non-linearity of this component may evolve to be 40%, for example, yet the system operates with 4% non-linearity with negative feedback. If there is no fitness difference between 0.4 and 4% non-linearity, such evolution is likely to happen. However, the failure of a feedback loop with a 40% non-linearity component is catastrophic." *(S2-S3)*
- "Failure of feedback loop components for a high feedback gain system would impact the system much more than low feedback gain systems due to dependency on the feedback. This trade-off is different from Bode's theorem, which describes specifically on the trade-off along the frequency axis. The trade-off discussed here is a combined problem of negative feedback and component failure risk." *(S3)*

Supplementary parameters:

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Tolerable component non-linearity | - | % | 5 | - | S2 | System spec in the example |
| Initial component non-linearity | - | % | 4 | - | S2 | Before feedback |
| Feedback reduction factor | - | - | 1/10 | - | S2 | Non-linearity attenuated tenfold |
| Closed-loop non-linearity with original component | - | % | 0.4 | - | S2 | 4% x 1/10 |
| Evolved ("sloppy") component non-linearity | - | % | 40 | - | S2 | Example value |
| Closed-loop non-linearity with sloppy component | - | % | 4 | - | S2 | 40% x 1/10; still under 5% spec |

**Figure S1 (S2): "System's dependency on negative feedback control and failure risk."** Axes S+ (horizontal) vs S- (vertical). Inset: sensitivity vs frequency, with a negative region S- at low frequency and a positive bump S+ at higher frequency (the waterbed picture). Main plot: a solid 45-degree line through the origin labelled "Sensitivity is conserved", with "Lower feedback gain" toward the origin and "Higher feedback gain" away from it; points A (low gain) and B (high gain) lie on it, double arrow between them. Dashed parallel line to the right through C and C' labelled "Sensitivity increased due to higher component-failure risk". Further dotted lines of lower slope from the origin pass through D, E, F (at A's S- level) and D', E', F' (at B's S- level), highlighted, labelled "Increased unreliability due to evolution under feedback compensation"; the horizontal spread is larger at the higher-gain level (D'-F' spread wider than D-F), depicting that the extra S+ from sloppy components grows with feedback gain. The paper does not define S+ and S- beyond the inset and the sentence "S+ is the sum of sensitivity due to feedback and probability of system malfunction due to component failure" *(S2)*; the plot has no scale.

Reading (mine): in the supplement, "conservation" is pictured as S+ = S- (a line), and off-line points (C, D-F') are *worse* than the conserved line (more S+ for the same S-). That is, component-failure risk adds to the Bode budget rather than violating it. This is consistent with the main text's claim that Bode-type conservation holds for the optimized design and that suboptimal designs sit strictly inside (worse than) the frontier.

## Box 2: Mapping cell culture onto yield-risk space *(p.4)*

- (A) Bar charts of growth rate (axis 0.5-1.5, dashed reference line at 1.0) for subtypes A, B, C under "No perturbation" and perturbations 1-4. Qualitatively: C is fastest without perturbation and collapses under perturbations 2 and 3; B is moderate everywhere and highest under perturbation 3; A is intermediate. (Bar heights are not given numerically.)
- (B) Histograms of cell counts by fate (cell death, non-dividing, division time 48 h, 36 h, 24 h) for subtype A under perturbation 4 vs. under no perturbation 4: "Selection is imposed at an individual level." *(p.4)*
- (C) Scatter of expected yield vs s.d. for subtypes A (circle), B (triangle), C (cross) under three perturbation regimes:

| Condition | No : P1 : P2 : P3 : P4 (probabilities) | Page |
|-----------|----------------------------------------|------|
| Condition 1 (red) | 80% : 20% : 0% : 0% : 0% | 4 |
| Condition 2 (blue) | 50% : 50% : 0% : 0% : 0% | 4 |
| Condition 3 (green) | 20% : 20% : 20% : 20% : 20% | 4 |

Approximate scatter coordinates (my visual read of the plot, +/- ~0.03; not stated in the text):

| Subtype | Condition | s.d. (approx.) | Expected yield (approx.) |
|---------|-----------|----------------|--------------------------|
| A | 1 | 0.34 | 1.07 |
| B | 1 | 0.17 | 1.01 |
| C | 1 | 0.45 | 1.22 |
| A | 2 | 0.49 | 0.86 |
| B | 2 | 0.26 | 0.96 |
| C | 2 | 0.55 | 0.96 |
| A | 3 | 0.57 | 0.76 |
| B | 3 | 0.41 | 0.86 |
| C | 3 | 0.81 | 0.43 |

Caption, verbatim key sentences: "Under the condition that 80% of the time there is no perturbation and 20% of the time perturbation 1 occurs, each subtype can be mapped on the yield-risk space as shown by red symbols. In this case, a combination of subtypes B and C reduces risk without major sacrifice in yield. Because of the covariance of B and C that has a very small positive value (0.068), mitigation of risk over the risk of subtype B is limited." ... "The expected yields of subtypes B and C are now equal [Condition 2]; thus, a combination of B and C only reduces risk, yield is not reduced." ... "[Condition 3] In this case, subtypes A and C suffer seriously from perturbations; thus, the population will be dominated by subtype B, which has robust yet low-yield characteristics." *(p.4)*

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Covariance of subtypes B and C growth (Condition 1) | cov(B,C) | (growth rate)^2 | 0.068 | - | 4 | "very small positive value"; limits risk reduction of a B+C mix below B's risk |

## Methodology

Conceptual analogy, no experiments or simulations. Steps:
1. Box 1 dominance example in engineering *(p.2)*.
2. Primer on Markowitz portfolio selection: yield = expected return, risk = s.d.; efficient frontier; Pareto efficiency; indifference curves (risk-averse, risk-neutral, risk-preferring) *(p.2-3)*.
3. Translation: organism/cell = asset; subtype mixture = portfolio; risk-aversion level of the indifference curve = level of perturbation in the environment *(p.3-5)*.
4. Use the translation to reinterpret published observations (Fischer & Sauer 2005; Ibarra et al. 2002; Andersson 2006) *(p.5-7)*.
5. State predictions *(p.7)*.

## Key differences between portfolio theory and biology (as Kitano states them) *(p.5)*

- "First, portfolio selection assumes that there are investors and fund managers making decisions regarding the composition of assets selected for the portfolio. This is clearly not the case in biology. The population composition changes because of the relative growth rate of each cell subtype that is the aggregated effect of individual cell reproduction cycles. Individual cells and organisms will be the subject of selection."
- "Second, in portfolio selection theory, investors decide on the indifference curve to be used on the basis of their appetite for risk taking. In the biological translation, indifference curves are only a reflection of the level of perturbation imposed on organisms and cells (Figure 4B). When organisms and cells are cultured in a highly stationary environment, the use of the risk-neutral curve may best predict their possible evolutionary paths. Risk-aversion curves represent the situation in which perturbations are imposed on organisms and cells."
- Fig. 1 caption: the capital market line is "not considered in biological applications because there is no zero-risk-fixed-yield genotype." *(p.3)*
- Fig. 1 caption links diversity of assets to HOT: "Availability of a larger number of assets with different yield-risk characteristics improves the overall portfolio, analogous to an increase in the degree of design of freedom in highly optimized tolerance (Carlson and Doyle, 1999; Reynolds et al, 2002)." *(p.3)*

## Figures of Interest

- **Box 1 (p.2):** Designs A/B/C op-amp circuits; see above.
- **Fig. 1 (p.3):** (A) E(r)-sigma(r) space with efficient frontier (red); Portfolio X inside the frontier, dominated by Y (higher return, same risk) and Z (lower risk, same return); possible-space regions for combining assets A,B; B,C; A,B,C. (B) Return distributions of each asset and portfolio.
- **Fig. 2 (p.3):** Indifference curves at 80/90/100% utility: (A) risk-aversion (rising with s.d.), (B) risk-neutral (horizontal), (C) risk-preference (falling with s.d.).
- **Box 2 (p.4):** see above.
- **Fig. 3 (p.5):** Distributions of randomly sampled cells in yield-risk space. (A) initial broad scatter inside the frontier; (B) after passages under stationary conditions, shift toward high yield; (C) further passages: points constrained against the frontier. (D) the alternative hypothesis: "If the robustness-performance trade-off holds at any time, even without being at the efficient frontier, the center of gravity of all randomly sampled points after passages will simply shift toward the upper right. This contrasts to the case that the trade-off holds only at the efficient frontier. This difference can be experimentally verified." *(p.5)* Caption also: "If there is a trade-off between robustness and performance, hence an inverse of risk and yield holds, distribution will be constrained within a certain envelope represented by the efficient frontier." *(p.5)*
- **Fig. 4 (p.6):** (A) Cultures 1-5 with subtype compositions A-D moving across yield-risk space (Culture 1: A,B dominant; 2: C rises; 3: D appears and rises; 4: more perturbation, D falls, C rises; 5: stable environment, D dominates). (B) Trajectories under risk-neutral (A), risk-averse (B), and from inside the frontier (C); strong risk aversion. (C) Drug resistance: Trajectories D (?), E, F to a "New efficient frontier"; Trajectories G (chemotherapy pushes the population inside the frontier, heterogeneous end points); Trajectories H (tumour regains proliferation under drug). Caption: tumour cells "may be too optimized for this specific therapeutic intervention, which implies possible efficacy when therapeutic regimens are switched. This may explain the collateral sensitivity of drug resistance tumor cells (Skipper et al, 1972)." *(p.6)*

## Results Summary (reinterpretations of prior data)

- *B. subtilis* suboptimal metabolism: Fischer & Sauer (2005) report regulatory mutants with improved biomass efficiency were "'almost exclusively regulators of not-yet-activated adaptive responses, suggesting that B. subtilis invests valuable resources in anticipation of changing environmental conditions at the expense of optimal growth'" *(p.5)*. Kitano: "it is likely that the projected position of B. subtilis is on the efficient frontier and optimized for the risk-aversion indifference curve ... Thus, although Fischer and Sauer argue that B. subtilis exhibits suboptimal metabolism, it can be considered optimal for the risk-aversion indifference curve." *(p.6)*
- *E. coli* adaptive evolution (Ibarra et al. 2002): in a chemostatic lab setup "a risk-neutral indifference curve can be applied"; *E. coli* "may quickly arrive at one of the projected positions on the efficient frontier and move along the trajectory of maximum biomass production through evolution (Trajectory A in Figure 4B)"; Ibarra's line of optimality "may correspond to the efficient frontier under stationary condition", and "their Figure 4 shows that the yield of bacteria evolves toward the line of optimality that is consistent with Figure 3 (A-C)". Caveat in text: "the line of optimality does not reflect the risk factor against perturbations. It only illustrates yield optimization. Therefore, it can be speculated that the E. coli that achieved maximum biomass production is more fragile against environmental change than those that have not evolved." *(p.6)*
- Drug resistance and "cost-free evolution" (Andersson 2006): resistant pathogens usually have lower growth fitness, read as Trajectory B (lower-left along the frontier) *(p.7)*. Compensatory evolution: "Although the existence of 'cost-free evolution' seems to breach the concepts of robustness-performance trade-off, it is consistent because this is a process of moving toward a new efficient frontier (hence, the population is evolving), and robustness-performance trade-off can be observed only on the efficient frontier. The efficient frontier has changed because of the presence of a drug that was not a factor in determining the original efficient frontier (Trajectory E or F in Figure 4C)." *(p.7)*

No effect sizes; no new data. The only numeric result is the illustrative covariance 0.068 in Box 2 *(p.4)*.

## Testable Properties / Predictions

- P1. Growth rate or biomass production "can be improved through evolution without increasing fragility until the efficient frontier is reached. A trade-off emerges only at the efficient frontier. This can be tested by random sampling of cells in artificial evolution experiments under a stationary culture condition, as shown in Figure 3." *(p.7)*
- P2. Same at population level: yield "can be improved without increasing their fragility against perturbations. Only when the population is on the efficient frontier do changes in yield and risk affect each other." *(p.7)*
- P3 (discriminating test, Fig. 3D). Frontier-only trade-off predicts sampled points pile up against an envelope (Fig. 3C); universal trade-off predicts the cloud's centre of gravity just shifts "toward the upper right" (higher yield and higher risk). "This difference can be experimentally verified." *(p.5)*
- P4. Once on the frontier in a stable condition, the population moves "along the efficient frontier to the right for higher yield at the cost of robustness", because of faster-growing subtypes *(p.7)*.
- P5. Gene-level signature distinguishes the two motions: toward the frontier from inside (Trajectory C): "mutations and gene up- or downregulations can be observed in broader functional classes of genes"; along the frontier to the right (Trajectory A): "mutations that generate high-risk high-yield phenotypes and downregulation of genes that accounts for perturbations may be observed". Tested by sequencing and expression profiling of populations and single cells *(p.7)*.
- P6. Under constantly imposed multiple perturbations, "Genes that are accountable for environmental perturbations will be upregulated and genes that attain a higher yield may be downregulated." *(p.7)*
- P7. *E. coli* evolved to maximum biomass is predicted more fragile against environmental change than unevolved strains (speculation) *(p.6)*.
- P8. Drug-resistant tumour cells optimized for one therapy may be sensitive when the regimen is switched (collateral sensitivity) *(p.6, Fig. 4C caption)*.
- P9 (supplement). Systems with higher feedback gain may evolve sloppier components inside the loop; failure of loop components is then more damaging in high-gain systems *(S1, S3)*.

## Implications stated *(p.7-8)*

- Long-cultured cells "may have the problem of being optimized for a culture-specific condition and higher growth rate rather than for robustness against broader perturbations." *(p.7)*
- Drug screening on cancer cell lines selects drugs that hit cells optimized for culture growth; in vivo cancer cells "may be optimized for surviving under various perturbations, but may not for growth rate"; proposes "a multiple perturbation culture system that may improve the screening process" *(p.7)*.
- Induced stem cells: selection for reprogramming efficiency may select "Cells with undermined robustness"; "It remains to be seen whether characteristics coselected for high-yield subtypes entail any unwanted characteristics for therapeutic use." *(p.7-8)*
- Closing: "With the introduction of the portfolio selection concept, observed breaches of trade-offs and enigma of performance suboptimality can be explained." *(p.8)*

## Limitations

- Stated by the author: "Although qualitative observations exist for this trade-off, quantitative experimental verification of this trade-off has not been conducted." *(p.1)* "How design affects the system vulnerability to component failure is a complex issue, and biological cases need to be further explored." *(p.2)* Both emergence and rebalancing scenarios occur, "but for the sake of explanation, only the wild type and its mutational variants are used as subtypes" *(p.5)*. "Although there is controversy with regard to this concept of 'cost-free evolution'" *(p.7)*.
- Not stated, observed by me: no mathematical model, so "sufficiently optimized" has no threshold or measure; the claim reduces to the definition of Pareto optimality (off the frontier, dominating moves exist). Whether a biological population is "on the frontier" is not measurable independently of the trade-off itself, which risks circularity in P1-P3. Risk as s.d. depends on the assumed perturbation distribution (Box 2 shows the ranking flips across Conditions 1-3), so the frontier is relative to an environment model. The Bode integral itself is never claimed to be violated; the "violations" are of trade-offs along axes Bode does not cover (component failure, yield vs. s.d.).

## Arguments Against Prior Work

- Against reading the robustness-fragility / robustness-performance trade-offs as universal (Csete & Doyle 2002; Kitano 2004, 2007): the trade-offs hold only on the efficient frontier *(p.1-2)*.
- Bode and the summation theorem are too narrow to underwrite a general biological trade-off: Bode is "specifically ... conservation of the sensitivities of negative feedback circuits on a frequency axis"; summation "assumes linearization for minor perturbations"; neither accounts for component failure *(p.1; S1, S3)*.
- Against Fischer & Sauer's "suboptimal metabolism" reading of *B. subtilis*: it is optimal under a risk-averse indifference curve *(p.6)*.
- Against Ibarra et al.'s line of optimality as a full optimality criterion: it "does not reflect the risk factor against perturbations" *(p.6)*.

## Design Rationale

- Portfolio theory chosen because it already has a precise notion of where a risk-return trade-off binds (the efficient frontier) and of dominated points inside it *(p.2)*.
- Risk measured as s.d. of growth over a perturbation distribution because it can be measured "by tracing behaviors of individual cells and their biomass production or lineage ... Alternatively ... by the growth of the population under various perturbations in which the population can be kept monoclonal" *(p.3)*; at population level, "Risk is an s.d. of growth rate under various possible perturbations. Experimentally, it can be measured by repeated perturbation experiments." *(p.4)*
- Indifference curves reinterpreted as environment, not preference, because there is no investor in biology *(p.5)*.

## Relevance to Project (Bode coverage study)

- Directly bears on the conjecture that a Bode-type conservation law governs coverage under optimization pressure. Kitano's position: conservation (a binding trade-off) is a property of optimized systems only; off the frontier, robustness can be bought for free. In the supplement's picture, suboptimal designs sit strictly worse than the conserved S+ = S- line (Fig. S1), i.e. the conservation law is an inequality whose equality case is the optimized design *(S2)*. That matches the standard reading of Bode's integral as a lower bound on the waterbed area that a design can only meet or exceed; Kitano does not write it that way, so this connection is my inference.
- Useful distinction for the study: Kitano separates (a) the frequency-axis sensitivity conservation (Bode) from (b) component-failure robustness and (c) yield-vs-s.d. robustness. He claims only (b) and (c) are "violated" off-frontier; he never claims a violation of (a) *(p.1; S3)*.
- The supplement's "sloppy component" mechanism (feedback compensation relaxes selection on components, which then degrade, making the system more dependent on the loop) is an optimization-pressure-creates-fragility mechanism relevant to the Goodhart framing in the task note: the verifier (closed-loop non-linearity of 4% within a 5% spec) is unchanged while what it certifies about component quality falls *(S2-S3)*.
- Frontier relative to the perturbation set (Fig. 4C, new drug -> new frontier) parallels the endogenous-channel point: the frontier moves when the disturbance distribution moves *(p.6-7)*.

## Open Questions

- [ ] Box 1: what failure mode (open or short) and what resistor values are assumed? Series vs parallel changes the feedback resistance (and gain) unless values are rescaled; the paper is silent *(p.2)*.
- [ ] Box 1 series failure 2P is first-order; exact independent-failure value 1-(1-P)^2. Parallel P^2 assumes independent failures and that one remaining resistor preserves function. Not stated *(p.2)*.
- [ ] S+ and S- in Fig. S1 are undefined quantities beyond the inset; how "probability of system malfunction due to component failure" is added to a sensitivity is not specified *(S2)*.
- [ ] How to identify "on the frontier" independently of observing the trade-off (circularity of P1-P3).
- [ ] Did anyone run the Fig. 3 experiment? Follow citing literature (Europe PMC lists 32 citations as of retrieval).

## Related Work Worth Reading

- Bode H (1945) Network Analysis and Feedback Amplifier Design — the integral itself.
- Csete ME, Doyle JC (2002) Reverse engineering of biological complexity. Science 295:1664-1669 — "robust yet fragile".
- Carlson JM, Doyle J (1999) Highly optimized tolerance. Phys Rev E 60:1412-1427; Reynolds, Carlson, Doyle (2002) Design degrees of freedom. Phys Rev E 66:016108.
- Kitano H (2004) Biological robustness. Nat Rev Genet 5:826-837; Kitano H (2007) Towards a theory of biological robustness. Mol Syst Biol 3:137 (already in collection: papers/Kitano_2007_TheoryBiologicalRobustness).
- Ibarra RU, Edwards JS, Palsson BO (2002) E. coli K-12 adaptive evolution to in silico predicted optimal growth. Nature 420:186-189.
- Fischer E, Sauer U (2005) Rigidity and suboptimal performance of B. subtilis metabolism. Nat Genet 37:636-640.
- Markowitz H (1991) Portfolio Selection.
