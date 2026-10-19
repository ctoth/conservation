---
title: "Towards a theory of biological robustness"
authors: "Hiroaki Kitano"
year: 2007
venue: "Molecular Systems Biology 3:137 (Editorial)"
doi_url: "https://doi.org/10.1038/msb4100179"
pages: "1-7"
affiliations: "Sony Computer Science Laboratories; The Systems Biology Institute, Tokyo; Cancer Institute of Japanese Foundation of Cancer Research"
note: "Page numbers below are the printed journal page numbers (1-7), which equal PDF page index + 1. Published 18 September 2007. The piece is an Editorial and has no abstract."
---

# Towards a theory of biological robustness

## One-Sentence Summary
An editorial that defines robustness as an integral of a normalized function-preservation score over a probability-weighted perturbation space (Eq. 1-2), and poses, without proving, the question of whether robustness is a conserved quantity, so that the robustness-fragility trade-off would read as a zero difference of total robustness between two systems (Eq. 3), by analogy with the Bode sensitivity integral (Eq. 4) and the MCA summation theorems. *(p.3-4)*

## Problem Addressed
"a mathematical foundation that provides a unified perspective on robustness is yet to be established" *(p.1)*. Kitano asks whether robustness and its trade-offs can be formalized so that "robustness is a conserved quantity or whether the trade-offs discussed above are bound to remain at the level of useful but empirical observations." *(p.3)*

## Key Contributions
- A working verbal definition: "robustness is a property that allows a system to maintain its functions against internal and external perturbations" (citing Kitano 2004a). *(p.1)*
- Separation of robustness (maintains *function*) from stability/homeostasis (maintains *state*); robustness may be achieved by switching steady states or by instability. *(p.1-2)*
- A formal definition of robustness R^s_{a,P} as a perturbation-probability-weighted integral of a relative-performance evaluation function (Eq. 1-2), and a comparison relation between systems. *(p.3)*
- Statement of three trade-off types: robustness-fragility, robustness-performance, robustness-resource (Figure 4). *(p.2, p.4-5)*
- A conjectured conservation law, Eq. 3 (ΔR = 0 over the whole perturbation space), presented as "reminiscent of the Bode integral formula", with the open question of whether any such conservation holds "in either relative or strict manner". *(p.4, p.5)*

## Study Design (empirical papers)
*Not applicable: editorial/theory piece, no data.*

## Methodology
Conceptual/definitional essay. Proposes formal definitions, illustrates them with coarse-grained perturbation grids (6 features x 7 degrees) and schematic performance curves, and draws analogies to control theory (robust control, Bode integral, gain-bandwidth product), Metabolic Control Analysis, reliability engineering, and HOT. *(p.1-6)*

## Key Equations / Statistical Models

$$
R^{s}_{a,P} = \int_{P} \psi(p)\, D^{s}_{a}(p)\, dp
$$
Where: R^s_{a,P} is robustness of system s with regard to function a against perturbation set P; ψ(p) is "the probability for perturbation 'p' to take place, and this should be 1 when all perturbation to take place at equal probability" (verbatim wording); D(p) is the evaluation function under perturbation p; P is "the entire perturbation space". *(p.3, Eq. 1)*

$$
D^{s}_{a}(p) = \begin{cases} 0, & p \in A \subset P \\ f_a(p)/f_a(0), & p \in P \setminus A \end{cases}
$$
Where: A is "a set of perturbations where the system failed to maintain its function"; f_a(p) is performance of function a under perturbation p, f_a(0) unperturbed. "For example, ATP production drop 20% under a certain perturbation compared with ATP production under unperturbed state, then 0.8 shall be returned." "Note that p in this equation represents a specific instance of a perturbation." *(p.3, Eq. 2)*

$$
R^{S1}_{a,Y} > R^{S2}_{a,Y}
$$
Where: "A system 'S1' can be said to be more robust than a system 'S2' with regard to a function 'a' against a certain set of perturbations 'Y', when" this holds. *(p.3)*

$$
\Delta R^{S1,S2}_{a,P} = \int_{P} \psi(p)\left(D^{S1}_{a}(p) - D^{S2}_{a}(p)\right) dp = R^{S1}_{a,P} - R^{S2}_{a,P} = 0
$$
Where: difference of robustness between two systems over the entire perturbation space P. Presented as what "should hold" under the robustness-fragility trade-off, and as what conservation would imply; not derived. *(p.4, Eq. 3)*

$$
\int_{0}^{\infty} \log|S(\omega)|\, d\omega \geq 0
$$
Where: "S(ω) is the sensitivity of a system at a frequency ω." Stated in the body as the Bode integral formula. No conditions (open-loop stability, relative degree, RHP poles) are stated in the paper. *(p.4, Eq. 4)*

$$
S = \frac{1}{1+G}
$$
Where: "the steady-state sensitivity (S) against a perturbation (d) of the system having feedback gain (G)" in a simple feedback amplifier circuit. *(p.6, Box 2)*

$$
S(\omega) = \frac{Y(\omega)}{D(\omega)}
$$
Where: Y(ω) is system output, D(ω) disturbance (following Yi et al, 2002). Base line sensitivity is log|S_0(ω)| with S_0 the open-loop sensitivity; "Normalized sensitivity, hence fragility of the system, can be obtained by subtracting the sensitivity of the feedback system and the base line sensitivity of the system without feedback." The normalized sensitivity is then written as the same ∫_0^∞ log|S(ω)| dω ≥ 0. *(p.6, Box 2)*

$$
G_a \times BW_a = G_b \times BW_b
$$
Where: gain-bandwidth product (GBWP) conservation in amplifier design; gain = performance, bandwidth = range of disturbance insensitivity. Worked example: gain 1000 at 1 kHz (GBWP = 1000 x 1 = 1000) can be extended via NFB to 100 kHz by reducing gain to 10 (GBWP = 100 x 10 = 1000) — numbers as printed. *(p.6, Box 2 panel C)*

$$
Y^{a} R^{a} = Y^{b} R^{b}
$$
Where: Y^a = f^a(0), performance of system A's function under no perturbation; R^a robustness of A over "some defined perturbations". "A robustness-performance trade-off would then imply that" this holds. *(p.5, Figure 4 caption)*

$$
\Delta R^{S1,S2}_{a,P} = U(m^{S1} - m^{S2})
$$
Where: m^{S1}, m^{S2} resources used by S1, S2; U "would relate the difference of resource use to the difference in robustness as a function of some design principles according to which resources are used." Posed as a question: "Whether it is at all possible to define such a function and, more fundamentally, whether such conservation actually exists, in either relative or strict manner, remains open." *(p.5, Figure 4 caption)*

## Parameters

| Name | Symbol | Units | Default | Range | Page | Notes |
|------|--------|-------|---------|-------|------|-------|
| Perturbation probability | ψ(p) | - | 1 (equiprobable case) | - | 3 | "should be 1 when all perturbation to take place at equal probability" |
| Evaluation function | D^s_a(p) | - | - | 0 or f_a(p)/f_a(0) | 3 | 0 on failure set A |
| Example relative ATP output | f(p)/f(0) | - | 0.8 | - | 3 | 20% ATP drop example |
| Figure 3 colour bands | f(p)/f(0) | - | - | >0.8; 0.8-0.6; 0.6-0.4; 0.4-0.0; D=0 | 4 | Coarse grid legend |
| Perturbation grid size | - | count | 6 features x 7 degrees | g1-g6, degree 0-6 | 4 | Figures 3, 4A |
| Amplifier gain, no NFB | G | - | 1000 | - | 6 | at 1 kHz bandwidth |
| Amplifier gain, with NFB | G | - | 10 | - | 6 | at 100 kHz bandwidth |
| Gain-bandwidth product | GBWP | - | 1000 | - | 6 | conserved across both cases |

## Effect Sizes / Key Quantitative Results
None; no empirical results.

## Focus passages (verbatim, with page)

### Conservation of robustness / fragility
- "A particularly interesting topic in the context of robustnesss is its trade-offs. What kind of trade-off exists in biological systems? Is robustness conserved? Does a trade-off between robustness and fragility indicate some kind of conservation principle as claimed by Csete and Doyle (2002)?" *(p.2)* — note: framed as questions.
- "The key issue is whether it is possible to find a formalism in which robustness and its trade-offs could be defined so that robustness is a conserved quantity or whether the trade-offs discussed above are bound to remain at the level of useful but empirical observations." *(p.3)*
- "However, considering an entire perturbation space (P), or sufficiently broad perturbation space, robustness-fragility trade-off should hold, thus difference of robustness (ΔR) between two systems shall be: [Eq. 3] which is reminiscent of the Bode integral formula." *(p.4)*
- "If robustness is conserved, then above equation should be zero with equiprobability over the perturbation space (see also Figure 4A). Assuming that S1 and S2 are the same system but with parameters optimized for different subset of perturbations, this equation implies that any increase in robustness against a subset of perturbation will be off-set by decrease of robustness against other perturbations." *(p.4)*
- "In fact, the notion that trade-offs between robustness and fragility represents a conservation of robustness (fragility) in biological systems was initially inspired by the so-called Bode Integral formula (Csete and Doyle, 2002)" *(p.4)*
- "Bode integral theorem and a set of theorems from metabolic analysis have already illustrated the conservation of robustness to some extent, and reliability engineering is a solid basis for component failure analysis." *(p.4)*
- Figure 4A caption: "If robustness is strictly conserved, then any increase in robustness for specific perturbation shall be compensated by increase in fragility elsewhere. ... if the system is tuned to cope better with perturbations of a subset of features (g1, g2, and g3), then robustness against other subset of perturbations are significantly reduced (right panel). Total robustness of both systems over this perturbation space remains equal." *(p.5)*
- Figure 4 caption end: "Whether it is at all possible to define such a function and, more fundamentally, whether such conservation actually exists, in either relative or strict manner, remains open." *(p.5)*
- Box 2: "Sensitivity of the system against perturbations (fragility) is conserved." *(p.6)* (said of the feedback amplifier/frequency-domain case)
- Box 2: "It has been argued that the same trade-off may apply to biological systems, and so increased robustness against certain perturbations inevitably results in extreme fragility elsewhere (Csete and Doyle, 2002)." *(p.6)*

### Bode integral
- Body: "Where S(ω) is the sensitivity of a system at a frequency ω. The Bode integral formula represents conservation of sensitivity of a negative feedback (NFB) system along the frequency axis (Bode, 1945) (the relevance of this theorem to biological systems is best described in (Csete and Doyle, 2002); see also Box 2). The Bode theorem indicates that an improvement of sensitivity gained by NFB in the low-frequency range is traded-off by increased instability in the high-frequency range." *(p.4)*
- Box 2: "In amplifier design, the trade-off between stability in specific frequency range provided by a NFB loop is compensated by increased instability in higher frequency region and less overall gain of the amplifier." *(p.6)*
- Box 2: "Therefore, a larger gain reduces the sensitivity and hence increases the robustness against perturbations. However, frequency domain analysis shows that such increase in robustness increases fragility in a specific frequency domain (A and B)." *(p.6)*
- Box 2: "An increase in feedback level reduces the sensitivity in a specific frequency range (A), but creates a region of instability elsewhere as shown by the peak of normalized fragility in the middle. With a larger feedback strength, sensitivity in a specific frequency range may be significantly reduced, but fragility would be larger as a result (B; adapted from Yi et al, 2002)." *(p.6)*
- Box 2: "This inequality is essential as it implies that feedback control cannot improve overall sensitivity; it only improves sensitivity in one place in a trade-off for fragility elsewhere." *(p.6)*
- Box 2 panel A label: "A = B" (area of reduced normalized fragility equals area of increased). *(p.6)*
- Box 2: "theories that integrate trade-offs between robustness and fragility in a feedback system with a feedback channel of limited capacity have been developed recently, thus expanding the horizon of intrinsic trade-offs involved (Martins et al, 2004, 2007)." *(p.6)*
- Observation (mine, not the paper's): Eq. 4 and Box 2 write the integral as "≥ 0" while Panel A labels the areas "A = B"; the paper states none of the hypotheses under which Bode's equality or inequality holds (open-loop stability, relative degree ≥ 2, unstable open-loop poles). *(p.4, p.6)*

### Csete & Doyle
- Cited as the source of the conservation claim: "as claimed by Csete and Doyle (2002)" *(p.2)*; "initially inspired by the so-called Bode Integral formula (Csete and Doyle, 2002)" *(p.4)*; relevance of Bode to biology "best described in (Csete and Doyle, 2002)" *(p.4)*; "increased robustness against certain perturbations inevitably results in extreme fragility elsewhere (Csete and Doyle, 2002)" *(p.6)*. Also cited generally (Csete and Doyle 2002, 2004) in the opening list *(p.1)*.

### HOT (Highly Optimized Tolerance)
- "Highly optimized tolerance (HOT) theory demonstrates, taking the example of a forest fire, that a system that is optimized for a specific perturbation inevitably entails extreme fragility for unexpected perturbations (Carlson and Doyle, 1999, 2002) (see Box 1)." *(p.2)*
- Box 1: "The HOT model argues that systems that have evolved to have a higher level of complexity are optimized for specific perturbations but, at the same time, are also inevitably extremely fragile against unexpected perturbations (Carlson and Doyle, 1999, 2002)." *(p.3)*
- Box 1: "Forest buffer zone patterns and tree planting patterns that are optimized for specific types of fire can be very fragile against unexpected types of fire, and may ignite from unexpected directions (A). They also argued that a design that can utilize a large degree of freedom can be more optimal for anticipated perturbations, and therefore can be extremely fragile against unexpected perturbations (Reynolds et al, 2002)." *(p.3)*
- Box 1: circular planting lets fire "spread and encircle the city" (B); cutting all trees makes the city "very fragile against the rainy season as flooding will be more likely" (C). "The point here is that such trade-off is inherent and cannot be avoided. This figure is inspired by the HOT theory (Carlson and Doyle, 1999, 2002)." *(p.3)*
- Engineering example: fly-by-wire jet airliners "highly robust against most component failures and atmospheric perturbations, but become extremely fragile against highly improbable events such as a total power failure"; the Wright Flyer "is a non-robust system but free from power failure problems". *(p.2)*

### Conditions under which Kitano says a trade-off holds
- Scope of perturbation space: the robustness-fragility trade-off "should hold" when "considering an entire perturbation space (P), or sufficiently broad perturbation space". *(p.4)*
- Weighting: ΔR "should be zero with equiprobability over the perturbation space" if robustness is conserved. *(p.4)*
- Same system, different tuning: the offset interpretation assumes "S1 and S2 are the same system but with parameters optimized for different subset of perturbations". *(p.4)*
- Resources/performance not changed: "A simultaneous increase of robustness and reduction of fragility may be achieved when additional resources are integrated properly into the system or if system performance is reduced." *(p.2-3)* (i.e., a robustness-fragility trade-off can be escaped by spending resources or performance). "Alternatively, a system's performance can be maximized by giving up robustness". "We should also note that these features are not independent." *(p.3)*
- "Thus, when robustness of the system against certain perturbations is increased, it may result in increased fragility against unexpected perturbation, increased resource demands, and degradation of performance." *(p.2)*
- Robustness-performance: "It is often the case that systems that are particularly well tuned for a specific task under a given environment are fragile against change in the environment. In contrast, systems with moderate performance tend to be more robust" *(p.4)*; Y^a R^a = Y^b R^b is what that trade-off "would then imply" "If the robustness-performance trade-off holds" *(p.5)*.
- Structural change excluded / future: "It will be a major challenge to find out under which conditions trade-offs exist and how to calculate system-level properties such as robustness or performance, when additional resource use is accompanied with changes in system configuration. In the long run, the theory should be extended to deal with major structural changes." *(p.4-5)*
- Unification unknown: "it is unclear whether formulations for each trade-off can be integrated into a single unified system of equations." *(p.4)*
- "much of the basic mathematics are already in place provided we deal with a well-chosen set." *(p.4)*

### Metabolic Control Analysis
- "within the theoretical framework of Metabolic Control Analysis, the summation and connectivity theorems represent constraints that are imposed on parametric changes in metabolic pathways (Fell, 1992), implying that the sensitivity of the network is conserved for changes in rate constants." *(p.4)*

## Figures of Interest
- **Fig 1 (p.2):** Design space of life bounded by environmental constraints, structural principles, fundamental principles, and evolution; organisms are instances of design.
- **Fig 2 (p.2):** Three steady states; return to SS1 = stability/homeostasis; transit to SS2 with function intact = robustness.
- **Box 1 (p.3):** HOT forest-fire countermeasures (buffer zones; circular planting; clear-cut leading to flood).
- **Fig 3 (p.4):** Two 6x7 perturbation grids with the same functional area but A better preserved than B: "Although the area the function is maintained is same in (A and B), (A) is considered more robust as the function is better preserved than (B)."
- **Fig 4 (p.5):** (A) strict conservation: a1 uniform vs a2 tuned to g1-g3, equal totals; (B) robustness-performance: low-performance high-robustness vs high-performance low-robustness curves; (C) robustness-resource: NFB loop with one resistor R vs two 2R in parallel, robustness rising concavely with resource used.
- **Box 2 (p.6):** (A) normalized fragility vs frequency with A = B areas; (B) larger feedback deepens low-frequency dip and raises peak; (C) gain-bandwidth product conserved.

## Results Summary
No results. The paper's positive claims are definitional (Eq. 1-2) and analogical; conservation of robustness is posed as conjecture and left open. *(p.3-5)*

## Limitations
- "The effort towards formalizing a theory of robustness and its trade-offs is still in its infancy and much remains to be completed to build a mature theory." *(p.3)*
- Robust control "cannot be directly applied to biological systems that have evolved and for which the desirable state of the system is not explicit"; existing mathematics focuses "on stability and performance of monostable systems". *(p.2)*
- "It is however challenging to derive an expression for more complex systems under various perturbations." *(p.5, Fig 4 caption)*
- Need to integrate with stability theory, evolution/genetics/game theory (Maynard-Smith, 1982), thermodynamics/nonequilibrium dissipative systems (Prigogine et al, 1974), and nonlinear dynamics/chaos. *(p.6-7)*

## Arguments Against Prior Work
- Homeostasis (Cannon 1932) "is clearly a property that maintains the state of the system rather than its functions"; robustness is the more general concept. *(p.1)*
- Control theory "assumes a system that is designed to meet given criteria" so does not directly fit evolved systems. *(p.2)*
- Thermodynamic/nonequilibrium theories "still do not take into account the heterogeneity and structured nature of biological systems as well as selection through evolution". *(p.6-7)*

## Design Rationale
- Definition chosen as "the most reasonable and appropriate definition, rather than creating yet another definition". *(p.1)*
- D(p) uses relative performance so that degree of preservation, not only area of function maintained, counts (Fig 3). *(p.3-4)*

## Testable Properties
- Under strict conservation with equiprobable ψ, R^{S1}_{a,P} = R^{S2}_{a,P} for two tunings of the same system over the full perturbation space. *(p.4)*
- Two systems with equal functional area but different degradation levels must differ in R (Fig 3). *(p.4)*
- If robustness-performance trade-off holds, Y^a R^a = Y^b R^b. *(p.5)*
- GBWP constant: 1000 x 1 = 10 x 100. *(p.6)*

## Relevance to Project
Primary source for the "conservation of robustness" framing that this project is studying. The key point for the Bode coverage study: Kitano's Eq. 3 is a conjecture whose stated conditions are only (i) the whole or a sufficiently broad perturbation space, (ii) equiprobable weighting, (iii) same system with re-tuned parameters, and implicitly (iv) no added resources or lost performance; he explicitly leaves open whether conservation exists "in either relative or strict manner". The Bode integral is cited without its control-theoretic hypotheses.

## Open Questions
- [ ] Eq. 3 writes "= 0" directly after "should hold"; the paper does not show that any system satisfies it.
- [ ] Eq. 4 is written "≥ 0" in a paragraph that calls it conservation; Box 2 panel A says "A = B". Which form Kitano intends is not stated.
- [ ] How ψ(p) normalizes when it is "1" for all p (not a density integrating to 1) is not specified.

## Related Work Worth Reading
- Csete & Doyle (2002) Reverse engineering of biological complexity. Science 295.
- Carlson & Doyle (1999, 2002) HOT papers.
- Yi, Sauro, Ingalls (2002) Limits of control and Bode's integral formula.
- Martins, Dahleh, Doyle (2007); Martins, Dahleh, Elia (2004).
- Bode (1945); Fell (1992) MCA survey; Kitano 2004a.

---
Provenance: read from page images pngs/page-000..006 (7/7 pages) of paper.pdf downloaded from https://www.embopress.org/doi/pdf/10.1038/msb4100179, 2026-10-19.
