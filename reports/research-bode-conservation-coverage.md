# Research: Generalizing Bode's sensitivity integral into conservation laws for coverage under optimization pressure

Full brief: map (A) the Bode integral and its generalizations, (B) information-theoretic versions, (C) optimization that produces conserved fragility (HOT / robust-yet-fragile), and (D) conservation on the optimization side for four senses of coverage (distributional/output, input/task-space, disturbance/robustness, test/verification). Verify every seed citation, correct or drop wrong ones, find missing papers (especially any that already state a cross-domain "waterbed"/conservation law), and output a prioritized retrieval list per tier.

Note on this file's name: the requested filename (the whole brief, including `:` characters, over 2,000 characters) is not a legal Windows filename, so this report is saved as `reports/research-bode-conservation-coverage.md`.

## Summary

Every seed citation exists, but several of the remembered details were wrong or incomplete. Corrections: Iglesias's LTV paper is Automatica 2001 and has a 2002 continuous-time companion in *Linear Algebra and its Applications*. The nonlinear entropy extension is Zang & Iglesias 2003. Zhuang & Hadfield-Menell is NeurIPS 2020 but went up on arXiv in Feb 2021. The NeurIPS 2022 title of Skalse et al. is "Reward *Gaming*". The Bailey et al. "probability of backtest overfitting" paper is J. Computational Finance 2017, separate from the 2014 Notices paper. Recht et al. 2019 conclude that the accuracy drop is *not* explained by adaptive overfitting, which cuts against a naive "test-set coverage is conserved" claim. The only seed I could not confirm is "Yu & Mehta 2010": no record found, so I dropped it. Its nearest verified neighbour is Mehta, Vaidya & Banaszuk, IEEE TAC 2008, whose title I could confirm only from a secondary reference list.

On the main question (has anyone already stated a cross-domain conservation law?), the closest prior statements are these. Stein (2003) calls the Bode integral a conservation law. Touchette & Lloyd (2000) state feedback control as a zero-sum information trade-off. Schaffer (1994) states an explicit "conservation law for generalization performance", and Wolpert & Macready (1997) give the optimization analogue. **Kitano (2007), *Mol. Syst. Biol.*, explicitly proposes "conservation of robustness (fragility)", says it was inspired by the Bode integral via Csete & Doyle (2002), and leaves open whether such conservation exists.** Kitano (2010) then argues that the trade-off may hold *only when a system is sufficiently optimized* and can be violated when there is room for optimization. That is the strongest existing statement of "conservation under optimization pressure". My searches found no paper that unifies Bode, NFL, Goodhart and test-coverage results into one conservation law. That is an absence in the searches I ran, not proof that no such paper exists.

## Approaches Found

### A. Analytic (complex-analysis) conservation: the Bode sensitivity integral and its generalizations
**Source:** Freudenberg & Looze 1985 (doi:10.1109/tac.1985.1104004); Seron, Braslavsky & Goodwin 1997 (free PDF: https://www.eng.newcastle.edu.au/~jhb519/book/FLFC.pdf)
**Description:** For a stable, rational loop with relative degree at least 2, the integral of ln|S(jω)| over frequency is 0, or π times the sum of the unstable open-loop poles. Pushing sensitivity down in one band forces it up in another (the "waterbed"). Extensions cover RHP zeros (Poisson integrals), MIMO (Chen 2000), discrete time (Sung & Hara 1988), time-varying systems (Iglesias 2001/2002), time-periodic systems (Sandberg & Bernhardsson 2005) and nonlinear systems (Seron et al. 1999, Zang & Iglesias 2003).
**Pros:** Exact equality, a true conserved quantity with a cost term (unstable poles). Mature theory.
**Cons:** Needs analyticity / LTI structure (or an entropy reinterpretation) to carry over. The "measure" is frequency with a log weighting, which has no obvious analogue for task space or test space.
**Complexity:** Medium (math is classical; transplanting it is the hard part).

### B. Information-theoretic conservation: entropy rates, Bode-like integrals over channels, requisite variety
**Source:** Iglesias 2001; Zang & Iglesias 2003; Elia 2004; Martins, Dahleh & Doyle 2007; Martins & Dahleh 2008; Touchette & Lloyd 2000/2004; Ashby 1956/1958; Conant & Ashby 1970; Fang, Chen & Ishii 2017 (book); Lestas, Vinnicombe & Paulsson 2010.
**Description:** Bode's integral rewritten as an output-minus-input entropy-rate difference. This removes the need for LTI or frequency structure and works for LTV, nonlinear and stochastic loops and for loops over noisy channels. Elia: Bode integral = directed information = degree of instability. Touchette & Lloyd: each bit of controller information buys at most one bit of entropy reduction. Ashby: only variety can destroy variety.
**Pros:** The most portable form, because entropy is defined on any distribution. That gives a natural bridge to distributional/output coverage (diversity = entropy), and to Kirk et al. 2024 and Padmakumar & He 2024.
**Cons:** Equalities become inequalities. "Information" must be identified concretely in each new domain.
**Complexity:** Medium-High.

### C. Optimization-produced fragility (HOT, robust-yet-fragile)
**Source:** Carlson & Doyle 1999 (PRE), 2000 (PRL), 2002 (PNAS); Doyle & Carlson 2000 (PRL); Csete & Doyle 2002 (Science); Doyle et al. 2005 (PNAS); Chandra, Buzi & Doyle 2011 (Science); Doyle & Csete 2011 (PNAS); Kitano 2007, 2010 (MSB). Critique: Newman, Girvan & Farmer 2002 (PRL).
**Description:** Systems optimized for anticipated perturbations become hypersensitive to unanticipated ones and show heavy tails. Chandra et al. derive explicit hard robustness/efficiency trade-offs for glycolysis using the Bode-type integral. Kitano frames this as a conservation hypothesis.
**Pros:** Directly about *optimization pressure*. Ties to Bode explicitly (Chandra 2011; Kitano 2007 citing Csete & Doyle).
**Cons:** Outside the Chandra et al. control-theoretic case, mostly a modelling paradigm, not a theorem. Newman et al. 2002 show that the HOT power laws depend on a risk-neutral objective, and risk aversion changes the outcome. Kitano 2010 argues the trade-off is violated away from optimality.
**Complexity:** Medium.

### D1. Distributional/output coverage under optimization (Goodhart, reward over-optimization, diversity collapse)
**Source:** Goodhart 1975; Campbell 1979; Strathern 1997; Manheim & Garrabrant 2018; Zhuang & Hadfield-Menell 2020; Skalse et al. 2022; Pan et al. 2022; Gao, Schulman & Hilton 2023; Karwowski et al. 2024; Kirk et al. 2024; Padmakumar & He 2024; Shumailov et al. 2024; Kleinberg & Raghavan 2021; El-Mhamdi & Hoang 2024; Majka & El-Mhamdi 2025.
**Description:** Optimizing a proxy beyond some point lowers the true objective (Gao: scaling laws; Karwowski: geometric explanation plus early stopping). Unhackable proxy pairs essentially do not exist (Skalse). Optimizing on J < L attributes drives the ignored attributes arbitrarily bad (Zhuang & Hadfield-Menell). That is the closest thing on the ML side to "suppressed here, pushed up there". RLHF trades generalization for output diversity (Kirk). Heavy-tailed proxy/goal discrepancy decides weak vs strong Goodhart (El-Mhamdi & Hoang; Majka & El-Mhamdi).
**Pros:** Several formal results (Skalse, Zhuang, El-Mhamdi) plus empirical scaling laws (Gao).
**Cons:** None of these states a conservation *equality*. They are impossibility results or monotone-degradation results.
**Complexity:** Medium.

### D2. Input/task-space coverage (NFL, conservation of generalization)
**Source:** Schaffer 1994; Wolpert 1996; Wolpert & Macready 1997; Igel & Toussaint 2004/2005.
**Description:** Summed over all target functions under a uniform prior, a learner's generalization gain on some tasks is exactly cancelled by losses on others (Schaffer). Any optimizer's advantage on one class is offset on another (Wolpert & Macready). Igel & Toussaint give the conditions on non-uniform priors under which NFL still holds.
**Pros:** A literal *equality* conservation law, the closest discrete analogue to Bode's integral (task space plays the role of frequency).
**Cons:** Needs a uniform or permutation-closed prior. Rao, Gordon & Spears (ICML 1995) argue the measure is meaningful only under that uniform prior (seen in the ML Anthology listing; not retrieved).
**Complexity:** Low-Medium.

### D3. Disturbance/robustness coverage in learned models
**Source:** Tsipras et al. 2019; Yin et al. 2019; Dohmatob 2019.
**Description:** Yin et al. find that Gaussian augmentation and adversarial training improve robustness to high-frequency corruptions *and reduce it for low-frequency ones*. That is a direct empirical waterbed over the Fourier spectrum of the input, the closest ML analogue to Bode found in this search. Tsipras: a robustness-vs-accuracy trade-off that persists with infinite data. Dohmatob: a "strong no free lunch" for adversarial robustness.
**Pros:** Yin et al. even keep Bode's frequency axis.
**Cons:** Empirical, no integral identity. Yin et al. report that more diverse augmentation (AutoAugment) eases the trade-off, so it is not a hard conservation law.
**Complexity:** Medium.

### D4. Test/verification coverage under selection pressure
**Source:** Inozemtseva & Holmes 2014; Just et al. 2014; Böhme 2018 (STADS); Böhme & Falk 2020; Benjamini & Hochberg 1995; White 2000; Bailey et al. 2014 (Notices), 2017 (JCF); Bailey & López de Prado 2014 (DSR); Harvey, Liu & Zhu 2016; Dwork et al. 2015 (Science, STOC); Blum & Hardt 2015; Recht et al. 2019; Roelofs et al. 2019.
**Description:** Optimizing a coverage metric decouples it from fault detection (Inozemtseva & Holmes). Finding more bugs in the same time needs exponentially more machines (Böhme & Falk). Each adaptive query spends holdout validity, so the holdout behaves like a budget (Dwork; Blum & Hardt). Multiple trials must be charged for (Bailey's PBO/DSR, White, Harvey et al., BH-FDR).
**Pros:** The adaptive-data-analysis framing (a finite information budget on a holdout) is formally close to approach B.
**Cons:** Counter-evidence: Recht et al. 2019 and Roelofs et al. 2019 find little evidence that holdout reuse caused overfitting in ImageNet/CIFAR or across 100+ Kaggle competitions. The budget is real in theory but often slack in practice.
**Complexity:** Medium.

## Key Papers

Tags: [V] = bibliographic details confirmed in this session (Crossref/publisher/arXiv). [V-sec] = confirmed only from secondary sources. [U] = not verified.

### Tier A: Bode integral and generalizations
- [Bode (1945)] *Network Analysis and Feedback Amplifier Design*, Van Nostrand: the original. [U, from memory; not checked]
- [Freudenberg & Looze (1985)](https://doi.org/10.1109/tac.1985.1104004): "Right half plane poles and zeros and design tradeoffs in feedback systems", IEEE TAC 30(6):555-565. [V]
- [Freudenberg & Looze (1988)](https://doi.org/10.1007/BFb0044565): *Frequency Domain Properties of Scalar and Multivariable Feedback Systems*, LNCIS 104, Springer. [V] Monograph with MIMO extensions.
- [Seron, Braslavsky & Goodwin (1997)](https://www.eng.newcastle.edu.au/~jhb519/book/FLFC.pdf): *Fundamental Limitations in Filtering and Control*, Springer, doi:10.1007/978-1-4471-0965-5, free PDF from the author. [V] The standard reference, covering the linear, filtering and nonlinear parts.
- [Stein (2003)](https://doi.org/10.1109/mcs.2003.1213600): "Respect the unstable", IEEE Control Systems Magazine 23(4):12-25 (1989 Bode Lecture). [V] Frames Bode as a conservation law ("conservation of sensitivity dirt", per arXiv:1902.11302).
- [Sung & Hara (1988)]: discrete-time SISO sensitivity properties, Int. J. Control 48(6):2429-2439. [V-sec: reference list only]
- [Chen (2000)](https://doi.org/10.1109/9.863595): "Logarithmic integrals, interpolation bounds, and performance limitations in MIMO feedback systems", IEEE TAC 45(6):1098-1115. [V]
- [Iglesias (2001)](https://doi.org/10.1016/s0005-1098(01)00103-0): "Tradeoffs in linear time-varying systems: an analogue of Bode's sensitivity integral", Automatica 37(10):1541-1550. [V] Seed listed this as "Iglesias 2001/2002"; the 2002 paper is next.
- [Iglesias (2002)](https://doi.org/10.1016/s0024-3795(01)00360-3): "Logarithmic integrals and system dynamics: an analogue of Bode's sensitivity integral for continuous-time, time-varying systems", Linear Algebra Appl. 343-344:451-471. [V]
- [Seron, Braslavsky, Kokotović & Mayne (1999)](https://doi.org/10.1109/9.754828): "Feedback limitations in nonlinear systems: from Bode integrals to cheap control", IEEE TAC 44(4):829-833. [V]
- [Zang & Iglesias (2003)](https://doi.org/10.1016/s0167-6911(03)00119-1): "Nonlinear extension of Bode's integral based on an information-theoretic interpretation", Systems & Control Letters 50(1):11-19. [V]
- [Sandberg & Bernhardsson (2005)](https://doi.org/10.1109/TAC.2005.860247): "A Bode sensitivity integral for linear time-periodic systems", IEEE TAC 50(12):2034-2039. [V-sec]
- [Emami-Naeini & de Roover (2019)](https://arxiv.org/abs/1902.11302): "Bode's Sensitivity Integral Constraints: The Waterbed Effect Revisited", arXiv:1902.11302. [V] Recent tutorial; ties the integral to how much faster the closed loop is than the open loop.

### Tier B: Information-theoretic versions
- [Ashby (1956)]: *An Introduction to Cybernetics*, Chapman & Hall: the law of requisite variety. [V-sec]
- [Ashby (1958)]: "Requisite variety and its implications for the control of complex systems", Cybernetica (vol. 1(2):83-99 from memory, not confirmed); reprint doi:10.1007/978-1-4899-0718-9_28. [V-sec]
- [Conant & Ashby (1970)](https://doi.org/10.1080/00207727008920220): "Every good regulator of a system must be a model of that system", Int. J. Systems Sci. 1(2):89-97. [V] Missing from seed.
- [Touchette & Lloyd (2000)](https://arxiv.org/abs/chao-dyn/9905039): "Information-theoretic limits of control", PRL 84:1156. [V]
- [Touchette & Lloyd (2004)](https://doi.org/10.1016/j.physa.2003.09.007): "Information-theoretic approach to the study of control systems", Physica A 331:140-172, arXiv:physics/0104007. [V]
- [Elia (2004)](https://doi.org/10.1109/tac.2004.834119): "When Bode meets Shannon: control-oriented feedback communication schemes", IEEE TAC 49(9):1477-1488. [V] Missing from seed.
- [Martins, Dahleh & Doyle (2007)](https://doi.org/10.1109/tac.2006.887898): "Fundamental limitations of disturbance attenuation in the presence of side information", IEEE TAC 52(1):56-66. [V] Missing from seed.
- [Martins & Dahleh (2008)](https://doi.org/10.1109/tac.2008.929361): "Feedback control in the presence of noisy channels: 'Bode-like' fundamental limitations of performance", IEEE TAC 53(7):1604-1615. [V]
- [Mehta, Vaidya & Banaszuk (2008)]: "Markov chains, entropy, and fundamental limitations in nonlinear stabilization", IEEE TAC 53:784-791. [V-sec] Nearest verified substitute for the dropped "Yu & Mehta 2010".
- [Lestas, Vinnicombe & Paulsson (2010)](https://doi.org/10.1038/nature09333): "Fundamental limits on the suppression of molecular fluctuations", Nature 467:174-178. [V] Information limit on biological noise control; missing from seed.
- [Fang, Chen & Ishii (2017)](https://doi.org/10.1007/978-3-319-49289-6): *Towards Integrating Control and Information Theories*, LNCIS, Springer. [V] Book-length synthesis.
- [Fang & Zhu (2020)](https://arxiv.org/abs/2012.12174): "Fundamental limits of controlled stochastic dynamical systems: an information-theoretic approach", arXiv:2012.12174. [V] L_p bounds driven by conditional entropy of the disturbance.

### Tier C: Optimization producing conserved fragility
- [Carlson & Doyle (1999)](https://doi.org/10.1103/PhysRevE.60.1412): "Highly optimized tolerance: a mechanism for power laws in designed systems", PRE 60(2):1412-1427. [V]
- [Carlson & Doyle (2000)]: "Highly optimized tolerance: robustness and design in complex systems", PRL 84(11). [V-sec]
- [Doyle & Carlson (2000)](https://doi.org/10.1103/physrevlett.84.5656): "Power laws, highly optimized tolerance, and generalized source coding", PRL 84(24):5656-5659. [V]
- [Carlson & Doyle (2002)](https://doi.org/10.1073/pnas.012582499): "Complexity and robustness", PNAS 99(suppl 1):2538-2545. [V]
- [Csete & Doyle (2002)](https://doi.org/10.1126/science.1069981): "Reverse engineering of biological complexity", Science 295(5560):1664-1669. [V] (Its Bode/conservation passage was not checked; the PDF was protected.)
- [Newman, Girvan & Farmer (2002)](https://doi.org/10.1103/physrevlett.89.028301): "Optimal design, robustness, and risk aversion", PRL 89:028301. [V] Key critique of HOT; missing from seed.
- [Doyle et al. (2005)]: "The 'robust yet fragile' nature of the Internet", PNAS 102(41):14497-14502. [V-sec]
- [Kitano (2007)](https://doi.org/10.1038/msb4100179): "Towards a theory of biological robustness", Mol. Syst. Biol. 3:137. [V] **Explicit conservation-of-robustness/fragility hypothesis, inspired by Bode via Csete & Doyle; leaves open whether it holds.** Missing from seed; top priority.
- [Kitano (2010)](https://doi.org/10.1038/msb.2010.40): "Violations of robustness trade-offs", Mol. Syst. Biol. 6:384. [V-sec] Conservation may hold only at optimality. Missing from seed; top priority.
- [Chandra, Buzi & Doyle (2011)](https://doi.org/10.1126/science.1200705): "Glycolytic oscillations and limits on robust efficiency", Science 333(6039):187-192. [V]
- [Doyle & Csete (2011)](https://doi.org/10.1073/pnas.1103557108): "Architecture, constraints, and behavior", PNAS 108(suppl 3):15624-15630. [V-sec]

### Tier D1: Distributional/output coverage (Goodhart)
- [Goodhart (1975)]: "Problems of monetary management: the U.K. experience", RBA conference July 1975, printed in *Papers in Monetary Economics* (RBA, 1976); reprinted in *Monetary Theory and Practice* (1984). [V-sec]
- [Campbell (1979)](https://doi.org/10.1016/0149-7189(79)90048-x): "Assessing the impact of planned social change", Eval. Program Plann. 2(1):67-90. [V] Campbell's law.
- [Strathern (1997)](https://doi.org/10.1002/(sici)1234-981x(199707)5:3%3C305::aid-euro184%3E3.0.co;2-4): "'Improving ratings': audit in the British University system", European Review 5(3):305-321. [V] Source of the popular wording.
- [Manheim & Garrabrant (2018)](https://arxiv.org/abs/1803.04585): "Categorizing variants of Goodhart's law", arXiv:1803.04585. [V]
- [Zhuang & Hadfield-Menell (2020)](https://arxiv.org/abs/2102.03896): "Consequences of misaligned AI", NeurIPS 2020; arXiv:2102.03896 (posted Feb 2021). [V]
- [Kleinberg & Raghavan (2021)](https://doi.org/10.1073/pnas.2018340118): "Algorithmic monoculture and social welfare", PNAS 118(22). [V] Missing from seed.
- [Skalse, Howe, Krasheninnikov & Krueger (2022)](https://arxiv.org/abs/2209.13085): "Defining and characterizing reward hacking", arXiv:2209.13085; titled "Defining and Characterizing Reward *Gaming*" in NeurIPS 2022. [V] Missing from seed.
- [Pan, Bhatia & Steinhardt (2022)](https://arxiv.org/abs/2201.03544): "The effects of reward misspecification", arXiv:2201.03544 (ICLR 2022 venue not confirmed). [V arXiv / U venue]
- [Gao, Schulman & Hilton (2023)](https://proceedings.mlr.press/v202/gao23h.html): "Scaling laws for reward model overoptimization", ICML 2023, PMLR 202:10835-10866, arXiv:2210.10760. [V]
- [Karwowski et al. (2024)](https://arxiv.org/abs/2310.09144): "Goodhart's law in reinforcement learning", ICLR 2024, arXiv:2310.09144. [V] Missing from seed.
- [Kirk et al. (2024)](https://arxiv.org/abs/2310.06452): "Understanding the effects of RLHF on LLM generalisation and diversity", ICLR 2024, arXiv:2310.06452. [V]
- [Padmakumar & He (2024)](https://arxiv.org/abs/2309.05196): "Does writing with language models reduce content diversity?", ICLR 2024. [V]
- [Shumailov et al. (2024)](https://doi.org/10.1038/s41586-024-07566-y): "AI models collapse when trained on recursively generated data", Nature 631:755-759. [V]
- [El-Mhamdi & Hoang (2024)](https://arxiv.org/abs/2410.09638): "On Goodhart's law, with an application to value alignment", arXiv:2410.09638. [V]
- [Majka & El-Mhamdi (2025)](https://arxiv.org/abs/2505.23445): "The Strong, Weak and Benign Goodhart's law", arXiv:2505.23445. [V]

### Tier D2: Input/task-space coverage
- [Schaffer (1994)](https://mlanthology.org/icml/1994/schaffer1994icml-conservation): "A conservation law for generalization performance", ICML 1994, pp. 259-265, doi:10.1016/B978-1-55860-335-6.50039-8. [V-sec, ML Anthology]
- [Wolpert (1996)](https://doi.org/10.1162/neco.1996.8.7.1341): "The lack of a priori distinctions between learning algorithms", Neural Computation 8(7):1341-1390. [V] Missing from seed.
- [Wolpert & Macready (1997)](https://doi.org/10.1109/4235.585893): "No free lunch theorems for optimization", IEEE TEC 1(1):67-82. [V-sec]
- [Igel & Toussaint (2004)](https://doi.org/10.1023/b:jmma.0000049381.24625.f7): "A no-free-lunch theorem for non-uniform distributions of target functions", J. Math. Model. Algorithms 3(4):313-322. [V] Crossref also holds a duplicate 2005 DOI, 10.1007/s10852-005-2586-y.

### Tier D3: Disturbance/robustness in learned systems
- [Tsipras et al. (2019)](https://arxiv.org/abs/1805.12152): "Robustness may be at odds with accuracy", ICLR 2019. [V]
- [Yin et al. (2019)](https://arxiv.org/abs/1906.08988): "A Fourier perspective on model robustness in computer vision", NeurIPS 2019. [V] **Empirical waterbed across input frequency.** Missing from seed.
- [Dohmatob (2019)](https://proceedings.mlr.press/v97/dohmatob19a.html): "Generalized no free lunch theorem for adversarial robustness", ICML 2019, PMLR 97:1646-1654. [V]

### Tier D4: Test/verification coverage
- [Benjamini & Hochberg (1995)](https://doi.org/10.1111/j.2517-6161.1995.tb02031.x): "Controlling the false discovery rate: a practical and powerful approach to multiple testing", JRSS-B 57(1):289-300. [V]
- [White (2000)](https://doi.org/10.1111/1468-0262.00152): "A reality check for data snooping", Econometrica 68(5):1097-1126. [V] Missing from seed.
- [Inozemtseva & Holmes (2014)](https://doi.org/10.1145/2568225.2568271): "Coverage is not strongly correlated with test suite effectiveness", ICSE 2014, pp. 435-445. [V]
- [Just et al. (2014)](https://doi.org/10.1145/2635868.2635929): "Are mutants a valid substitute for real faults in software testing?", FSE 2014. [V] Missing from seed.
- [Bailey, Borwein, López de Prado & Zhu (2014)]: "Pseudo-mathematics and financial charlatanism: the effects of backtest overfitting on out-of-sample performance", Notices AMS 61(5):458-471. [V-sec]
- [Bailey & López de Prado (2014)](https://doi.org/10.3905/jpm.2014.40.5.094): "The deflated Sharpe ratio", J. Portfolio Management 40(5):94-107. [V]
- [Bailey, Borwein, López de Prado & Zhu (2017)](https://doi.org/10.21314/jcf.2016.322): "The probability of backtest overfitting", J. Computational Finance 20(4). [V-sec]
- [Dwork et al. (2015a)](https://doi.org/10.1126/science.aaa9375): "The reusable holdout: preserving validity in adaptive data analysis", Science 349(6248):636-638. [V]
- [Dwork et al. (2015b)](https://doi.org/10.1145/2746539.2746580): "Preserving statistical validity in adaptive data analysis", STOC 2015, pp. 117-126. [V] The theory paper behind (a).
- [Blum & Hardt (2015)](https://proceedings.mlr.press/v37/blum15.html): "The Ladder: a reliable leaderboard for machine learning competitions", ICML 2015, PMLR 37:1006-1014. [V]
- [Harvey, Liu & Zhu (2016)](https://doi.org/10.1093/rfs/hhv059): "... and the cross-section of expected returns", RFS 29(1):5-68. [V]
- [Böhme (2018)](https://doi.org/10.1145/3210309): "STADS: software testing as species discovery", TOSEM 27(2). [V] Coverage estimation as species estimation; missing from seed.
- [Böhme & Falk (2020)](https://doi.org/10.1145/3368089.3409729): "Fuzzing: on the exponential cost of vulnerability discovery", ESEC/FSE 2020, pp. 713-724. [V] Missing from seed.
- [Recht, Roelofs, Schmidt & Shankar (2019)](https://proceedings.mlr.press/v97/recht19a): "Do ImageNet classifiers generalize to ImageNet?", ICML 2019, PMLR 97:5389-5400, arXiv:1902.10811. [V] Attributes the drop to harder images, not adaptive overfitting.
- [Roelofs et al. (2019)](https://mlanthology.org/neurips/2019/roelofs2019neurips-metaanalysis): "A meta-analysis of overfitting in machine learning", NeurIPS 2019. [V-sec] Counter-evidence; missing from seed.

## Existing Implementations
- **pbo** (CRAN R package; README at https://cran.hafro.is/web/packages/pbo/readme/README.html): implements CSCV / probability of backtest overfitting. Language R; license not checked.
- **facebookresearch/rlfh-gen-div** (https://github.com/facebookresearch/rlfh-gen-div): code for Kirk et al. 2024, as named in the authors' ICLR slides. I did not open the repository; license not checked.
- **STADS / fuzzing dataset**: the Böhme & Falk data is on figshare (https://figshare.com/articles/dataset/Fuzzing_On_the_Exponential_Cost_of_Vulnerability_Discovery/11911287).
- I found no software that computes a Bode-style conservation quantity for ML or test coverage.

## Addendum 1: Tier E, framework substrate (channel theory and institutions)

Added at the coordinator's request: Kent's paper as a candidate formal substrate for stating one coverage conservation law across domains.

- [Kent (2016)](https://arxiv.org/abs/1603.03475): Robert E. Kent, "Information Flow in Logical Environments", arXiv:1603.03475, cs.LO, submitted 10 Mar 2016; the arXiv comment says "This paper was completed in 2008". No journal reference. [V] It defines Barwise-Seligman information flow over an arbitrary *logical environment* (a semantic version of a Goguen-Burstall institution). The stated motivation is semantic integration of ontologies and databases. **I extracted the PDF text and found no occurrences of "probab", "entropy" or "measure": the framework is purely qualitative.** It provides channels and infomorphisms, but no quantity that could be conserved. Its reference list (eight items): Barwise & Seligman 1997; Ganter & Wille 1999 (*Formal Concept Analysis*); Goguen 2005; Goguen 2006 draft; Goguen & Burstall 1992; Kent 2003 (IFF, CEUR vol. 82); Kent 2005 (Dagstuhl 04391); the IFF website.
- Core dependencies:
  - [Barwise & Seligman (1997)](https://doi.org/10.1017/cbo9780511895968): *Information Flow: The Logic of Distributed Systems*, Cambridge UP. [V] The definitions of classifications, infomorphisms, channels and local logics (ch. 12) with normal tokens.
  - [Goguen & Burstall (1992)](https://doi.org/10.1145/147508.147524): "Institutions: abstract model theory for specification and programming", JACM 39(1):95-146. [V]
  - [Goguen (2005)](https://doi.org/10.1093/jigpal/jzi050): "Data, schema, ontology and logic integration", Logic J. IGPL 13(6):685-715. [V]
  - Goguen, "Information integration in institutions": draft for the Jon Barwise memorial volume (ed. L. Moss, Indiana UP), cited as "to appear (2006)". [V-sec; I could not confirm it was ever published]
  - Kent (2018), "The Institutional Approach", arXiv:1810.08074 (seen in search results; not opened). Kent, "System Consequence", arXiv:1810.02929 (seen; not opened). [U]
  - [Kalfoglou & Schorlemmer (2003)](https://doi.org/10.1007/978-3-540-39733-5_5): "IF-Map: an ontology-mapping method based on information-flow theory", J. Data Semantics I, LNCS, pp. 98-127. [V] An application of channel theory.
- Work connecting channel theory to *quantitative* information (the bridge a conservation law would need):
  - [Allwein (2004/2005)](https://doi.org/10.1145/1065907.1066030): "A qualitative framework for Shannon information theories", NSPW '04 (Crossref lists Allwein alone; a secondary source lists Allwein, Moskowitz & Chang). Shannon theories "decorate" a Barwise-Seligman qualitative structure with measures. [V; author list differs between sources] A related NRL memo report, NRL/MR/5540--04-8748, is at https://apps.dtic.mil/sti/tr/pdf/ADA420108.pdf (returned 403; not read).
  - [Seligman (2009)](https://doi.org/10.1007/978-3-642-00659-3_8): "Channels: from logic to probability", in *Formal Theories of Information*, LNCS, pp. 193-233. [V] According to a secondary summary (not checked against the paper), it shows a structural similarity between Shannon channels and Tarski channels, but infinite Shannon channels cannot be captured as *strong* Barwise-Seligman information flow ("the strength problem"). If that holds, it is a direct obstacle to using channel theory as the substrate for an entropy-based conservation law.
  - [Baez, Fritz & Leinster (2011)](https://doi.org/10.3390/e13111945): "A characterization of entropy in terms of information loss", Entropy 13(11):1945-1957. [V] Characterizes Shannon entropy functorially, as information loss along measure-preserving maps. This is a category-theoretic way to attach an additive quantity to morphisms, the missing quantitative piece. My addition, not one of the requested leads.
- **Channel theory/institutions linked to Bode-type or information-theoretic control limits: none found.** Two searches turned up nothing that joins institution theory, Chu spaces or channel theory to Bode, requisite variety or control limits. The nearest is the folklore reading of Ashby's law as Shannon's 10th theorem (Ashby 1958 itself; arXiv:0809.1698, not opened).
- **Local logics (normal tokens, soundness) applied to distribution shift, specification gaming or Goodhart: none found** (one extended search). As a structural observation of mine, not something found in the literature: a Barwise-Seligman local logic is sound only on its *normal tokens*. Goodhart, in Q's framing, is optimization moving the population of tokens out of the normal set, so the verifier's sequents still hold syntactically but no longer carry information about the goal. That is a candidate formalization, not a published one.

## Addendum 2: Q's framing — Goodhart acts on the channel, not the verifier

Framing (from Q, via the coordinator): the verifier is unchanged, while optimization erodes the regularities that made the verifier's output informative about the goal. In other words, the channel from the goal to the verifier's output is *endogenous to the policy*.

### (1) The channel assumptions Martins & Dahleh (2008) actually make
Source: the full text, extracted from the authors' copy (http://www.mit.edu/~dahleh/pubs/Martins.8.2008.pdf) with pdftotext. The PDF's math symbols did not survive extraction; the quotes below are the surviving words, with symbols dropped where the gaps are.
- Sec. II-C, "Basic Feedback Paradigm and Assumptions", bullet: "The channel noise , the plant's initial condition and the disturbance are mutually independent."
- Same list: the feedback map "deterministically maps past and present inputs to present outputs. As such, if admits a state-space representation then the initial state must be deterministic."
- Sec. V-A, Definition 5.1 (Channel): "Let be a given channel input alphabet, along with a stochastic process which is assumed mutually independent of and . In addition, consider a map acting as , where and represent the channel output and input processes, respectively. The pair defines a channel, which may be either memory-less or with memory."
- Sec. IV: "Causality is the central assumption in obtaining a conservation law". Lemma 4.1 needs the two exogenous signals to be "mutually independent, for all".
- Remark 5.1: feedback capacity is defined over a *given* set of allowable channel-input processes, and is "an upper-bound to the information rate ... under the channel input constraint set".

**Reading (mine; check against the typeset equations):** the channel is a *given* fixed map plus a noise process that is independent of the plant's initial state and the disturbance. The designer chooses the encoder/controller, which sees past channel outputs. The channel map and noise law do not depend on that choice; the policy enters only through the channel *input*, constrained to an admissible set. So the answer to "is the channel independent of the controller/plant?" is yes, in the sense that matters for the Goodhart framing: the channel's transition law is fixed in advance and its noise is independent of plant and disturbance. A policy-dependent channel, one whose law shifts as the controller optimizes, is outside the paper's model. This supports Q's diagnosis of why the paper does not cover Goodhart. (The same holds for Martins, Dahleh & Doyle 2007, where side information enters as an exogenous channel; I did not re-read that paper's text.)

### (2) Control limits with policy-dependent, endogenous or adversarial channels: leads
No Bode-type integral for a policy-dependent channel turned up in three searches. Nearest work:
- [Bai, Pasqualetti & Gupta (2017)](https://doi.org/10.1016/j.automatica.2017.04.047): "Data-injection attacks in stochastic control systems: detectability and performance tradeoffs", Automatica 82:251-260. [V] An adversary alters the sensor channel. The performance-vs-detectability trade-off is stated through a KL-divergence bound, which makes it the closest "adversarial channel + information-theoretic limit" result. (Details are from my knowledge of the paper; only the bibliographic data was checked.)
- [Ranade & Sahai (2017)](https://arxiv.org/abs/1701.04187): "Control Capacity", arXiv:1701.04187. [V] Multiplicative actuation noise: the channel acts *on the control*, so its effect depends on what the controller does. The stabilization threshold is control capacity > log of the unstable eigenvalue.
- [Pacelli & Theodorou (2026)](https://arxiv.org/abs/2603.18454): "Fundamental limits for sensor-based control via the Gibbs variational principle", arXiv:2603.18454, v3 Jul 2026. [V] Explicitly criticizes bounds that evaluate the sensor against the *uncontrolled* system, and gives a self-consistent bound in which the closed-loop state distribution limits what the sensor can extract. This is a closed-loop-dependent observation channel, the nearest match to Q's framing.
- [Sayin, Akyol & Başar]: "Hierarchical multistage Gaussian signaling games in noncooperative communication and control systems", arXiv:1609.09448. [V arXiv; Automatica venue not confirmed] A strategic sender shapes the channel.
- [Nair (2013)]: "A nonstochastic information theory for communication and state estimation", IEEE TAC 58(6):1497-1510, doi:10.1109/TAC.2013.2241491. [V-sec] Worst-case (nonstochastic) channels.
- ML side, where the data channel shifts in response to the deployed model:
  - [Hardt, Megiddo, Papadimitriou & Wootters (2016)](https://arxiv.org/abs/1506.06980): "Strategic classification", ITCS 2016, pp. 111-122, doi:10.1145/2840728.2840730, arXiv:1506.06980. [V-sec, ACM citation via search] Agents alter features in response to the published classifier.
  - [Perdomo, Zrnic, Mendler-Dünner & Hardt (2020)](https://proceedings.mlr.press/v119/perdomo20a.html): "Performative prediction", ICML 2020, PMLR 119:7599-7609, arXiv:2002.06673. [V] The data distribution is a function of the deployed model; introduces performative stability; strictly subsumes strategic classification. This is the cleanest formal model of a policy-dependent channel.
  - Perdomo et al.'s survey "Performative prediction: past and future", arXiv:2310.16608 (seen; not opened). [U]

### (3) Adaptive data analysis read as a holdout's information wearing out
- Dwork et al. 2015 (Science and STOC): already verified above.
- [Hardt & Ullman (2014)](https://doi.org/10.1109/focs.2014.55): "Preventing false discovery in interactive data analysis is hard", FOCS 2014, pp. 454-463. [V] The lower-bound side: no efficient mechanism can answer too many adaptive queries accurately. That is the quantitative "wear-out" rate.
- [Russo & Zou (2016)](https://proceedings.mlr.press/v51/russo16.html): "Controlling bias in adaptive data analysis using information theory", AISTATS 2016, PMLR 51:1232-1240; extended version arXiv:1511.05219. [V] Bounds adaptive-analysis bias by the *mutual information* between the selection and the data. This turns "the holdout's information wears out" into an explicit information budget, the closest D4 analogue of an information-theoretic Bode inequality.
- Blum & Hardt 2015 (Ladder): already verified above.

## Complexity vs Quality Tradeoffs
- **Analogy level (low cost):** call each D-tier result a "waterbed" and cite Stein / Kitano. Cheap but weak: Kitano 2010 and Newman et al. 2002 show that such trade-offs fail away from the optimum or under other objectives.
- **Equality level (medium):** only NFL/Schaffer (D2) and Bode (A) give exact equalities. Each needs a measure over which the trade-off integrates (frequency with a log weighting; a uniform or permutation-closed prior over tasks). A real generalization has to name that measure for each coverage sense. For D3, Yin et al.'s Fourier axis is a ready candidate.
- **Information level (high):** the B-tier entropy form (Iglesias, Zang & Iglesias, Martins & Dahleh, Touchette & Lloyd) gives inequalities that hold without LTI structure. It maps onto output diversity (entropy) and the holdout budget (Dwork et al. bound validity through information/stability). This is the most general route and the most work.

## Recommendations
1. Retrieve first (priority order): Kitano 2007; Kitano 2010; Stein 2003; Seron/Braslavsky/Goodwin 1997 (free PDF); Schaffer 1994; Wolpert & Macready 1997; Yin et al. 2019; Zang & Iglesias 2003; Martins & Dahleh 2008; Touchette & Lloyd 2004; Chandra et al. 2011; Newman et al. 2002; Skalse et al. 2022; Zhuang & Hadfield-Menell 2020; Dwork et al. 2015 (STOC); Recht et al. 2019.
2. Treat Kitano 2007/2010 as the closest prior statement of the thesis, and check them first for overlap.
3. Read Newman et al. 2002, Kitano 2010, Recht et al. 2019 and Roelofs et al. 2019 as the counter-evidence any conservation claim has to survive.
4. Drop "Yu & Mehta 2010" unless a primary record turns up. Use Mehta, Vaidya & Banaszuk 2008 instead.
5. Tier E retrieval order: Barwise & Seligman 1997 (ch. 12, local logics); Kent 2016; Seligman 2009 (check the "strength problem" claim first, because it may rule channel theory out as a quantitative substrate); Allwein 2004; Baez, Fritz & Leinster 2011; Goguen & Burstall 1992; Goguen 2005.
6. Tier F (endogenous channel) retrieval order: Perdomo et al. 2020; Russo & Zou 2016; Pacelli & Theodorou 2026; Bai, Pasqualetti & Gupta 2017; Hardt et al. 2016; Hardt & Ullman 2014; Ranade & Sahai 2017. Together with the Martins & Dahleh fixed-channel finding above, these give the material for Q's framing: Bode-type limits assume a fixed channel law; Goodhart is the case where the policy moves that law.

## Estimated Implementation Effort
- **Minimal approach:** retrieve and read about 16 tier-1 papers (list above). Result: a defensible map of which coverage senses already have an equality law (task space), an information inequality (control, holdout), or only empirical trade-offs (output diversity, robustness frequency).
- **Full approach:** all of roughly 60 listed papers plus forward citations of Kitano 2007 and Stein 2003. Result: enough to judge whether a unified conservation law is new.

## Open Questions
- [ ] Does any paper citing Kitano 2007 formalize "conservation of robustness" (Kitano 2010 uses portfolio theory)? Forward-citation search not done.
- [ ] Exact Csete & Doyle 2002 wording on Bode/conservation (the PDF was password-protected).
- [ ] Ashby 1958 volume/pages; Goodhart 1975 primary text; Sung & Hara 1988 primary record; Pan et al. ICLR venue; Bode 1945 publisher details. All unverified.
- [ ] Is there a measure on test-input space under which coverage gains and losses integrate to a constant (the D4 analogue of Bode's frequency axis)? None found.
- [ ] Confirm my reading of Martins & Dahleh's Definition 5.1 against the typeset equations (symbols were lost in text extraction).
- [ ] Does Seligman 2009's "strength problem" block an entropy-valued conservation law stated in channel theory? (Secondary summary only.)
- [ ] Is there any Bode-type integral, or directed-information inequality, where the channel law depends on the policy? None found; Pacelli & Theodorou 2026 and Perdomo et al. 2020 are the nearest.
- [ ] Local logics / normal tokens applied to distribution shift or Goodhart: none found. The normal-token reading in Addendum 1 is mine, not from any source.
- [ ] Okano/Ishii-style Bode integrals for stochastic switched systems (Automatica, ScienceDirect S0005109812004621): seen in search results, authors and details not verified.

## References
- Freudenberg, J., Looze, D. (1985). IEEE TAC 30(6):555-565. https://doi.org/10.1109/tac.1985.1104004
- Freudenberg, J., Looze, D. (1988). LNCIS 104, Springer. https://doi.org/10.1007/BFb0044565
- Seron, M., Braslavsky, J., Goodwin, G. (1997). Springer. https://doi.org/10.1007/978-1-4471-0965-5 ; https://www.eng.newcastle.edu.au/~jhb519/book/FLFC.pdf
- Stein, G. (2003). IEEE CSM 23(4):12-25. https://doi.org/10.1109/mcs.2003.1213600
- Chen, J. (2000). IEEE TAC 45(6):1098-1115. https://doi.org/10.1109/9.863595
- Iglesias, P. (2001). Automatica 37(10):1541-1550. https://doi.org/10.1016/s0005-1098(01)00103-0
- Iglesias, P. (2002). Linear Algebra Appl. 343-344:451-471. https://doi.org/10.1016/s0024-3795(01)00360-3
- Seron, Braslavsky, Kokotović, Mayne (1999). IEEE TAC 44(4):829-833. https://doi.org/10.1109/9.754828
- Zang, G., Iglesias, P. (2003). SCL 50(1):11-19. https://doi.org/10.1016/s0167-6911(03)00119-1
- Sandberg, H., Bernhardsson, B. (2005). IEEE TAC 50(12):2034-2039. https://doi.org/10.1109/TAC.2005.860247
- Emami-Naeini, A., de Roover, D. (2019). arXiv:1902.11302. https://arxiv.org/abs/1902.11302
- Conant, R., Ashby, W.R. (1970). IJSS 1(2):89-97. https://doi.org/10.1080/00207727008920220
- Ashby, W.R. (1958). Cybernetica; reprint https://doi.org/10.1007/978-1-4899-0718-9_28
- Touchette, H., Lloyd, S. (2000). PRL 84:1156. https://arxiv.org/abs/chao-dyn/9905039
- Touchette, H., Lloyd, S. (2004). Physica A 331:140-172. https://doi.org/10.1016/j.physa.2003.09.007
- Elia, N. (2004). IEEE TAC 49(9):1477-1488. https://doi.org/10.1109/tac.2004.834119
- Martins, Dahleh, Doyle (2007). IEEE TAC 52(1):56-66. https://doi.org/10.1109/tac.2006.887898
- Martins, Dahleh (2008). IEEE TAC 53(7):1604-1615. https://doi.org/10.1109/tac.2008.929361
- Lestas, Vinnicombe, Paulsson (2010). Nature 467:174-178. https://doi.org/10.1038/nature09333
- Fang, Chen, Ishii (2017). Springer LNCIS. https://doi.org/10.1007/978-3-319-49289-6
- Fang, Zhu (2020). arXiv:2012.12174. https://arxiv.org/abs/2012.12174
- Carlson, Doyle (1999). PRE 60:1412. https://doi.org/10.1103/PhysRevE.60.1412
- Doyle, Carlson (2000). PRL 84:5656. https://doi.org/10.1103/physrevlett.84.5656
- Carlson, Doyle (2002). PNAS 99:2538. https://doi.org/10.1073/pnas.012582499
- Csete, Doyle (2002). Science 295:1664. https://doi.org/10.1126/science.1069981
- Newman, Girvan, Farmer (2002). PRL 89:028301. https://doi.org/10.1103/physrevlett.89.028301
- Kitano (2007). MSB 3:137. https://doi.org/10.1038/msb4100179
- Kitano (2010). MSB 6:384. https://doi.org/10.1038/msb.2010.40
- Chandra, Buzi, Doyle (2011). Science 333(6039):187-192. https://doi.org/10.1126/science.1200705 [V, Crossref]
- Doyle, Csete (2011). PNAS 108:15624. https://doi.org/10.1073/pnas.1103557108
- Campbell (1979). https://doi.org/10.1016/0149-7189(79)90048-x
- Strathern (1997). European Review 5(3):305-321.
- Manheim, Garrabrant (2018). https://arxiv.org/abs/1803.04585
- Zhuang, Hadfield-Menell (2020). https://arxiv.org/abs/2102.03896
- Kleinberg, Raghavan (2021). https://doi.org/10.1073/pnas.2018340118
- Skalse et al. (2022). https://arxiv.org/abs/2209.13085
- Pan, Bhatia, Steinhardt (2022). https://arxiv.org/abs/2201.03544
- Gao, Schulman, Hilton (2023). https://proceedings.mlr.press/v202/gao23h.html
- Karwowski et al. (2024). https://arxiv.org/abs/2310.09144
- Kirk et al. (2024). https://arxiv.org/abs/2310.06452
- Padmakumar, He (2024). https://arxiv.org/abs/2309.05196
- Shumailov et al. (2024). https://doi.org/10.1038/s41586-024-07566-y
- El-Mhamdi, Hoang (2024). https://arxiv.org/abs/2410.09638
- Majka, El-Mhamdi (2025). https://arxiv.org/abs/2505.23445
- Schaffer (1994). https://mlanthology.org/icml/1994/schaffer1994icml-conservation
- Wolpert (1996). https://doi.org/10.1162/neco.1996.8.7.1341
- Wolpert, Macready (1997). https://doi.org/10.1109/4235.585893
- Igel, Toussaint (2004). https://doi.org/10.1023/b:jmma.0000049381.24625.f7
- Tsipras et al. (2019). https://arxiv.org/abs/1805.12152
- Yin et al. (2019). https://arxiv.org/abs/1906.08988
- Dohmatob (2019). https://proceedings.mlr.press/v97/dohmatob19a.html
- Benjamini, Hochberg (1995). https://doi.org/10.1111/j.2517-6161.1995.tb02031.x
- White (2000). https://doi.org/10.1111/1468-0262.00152
- Inozemtseva, Holmes (2014). https://doi.org/10.1145/2568225.2568271
- Just et al. (2014). https://doi.org/10.1145/2635868.2635929
- Bailey, López de Prado (2014). https://doi.org/10.3905/jpm.2014.40.5.094
- Bailey et al. (2017). https://doi.org/10.21314/jcf.2016.322
- Dwork et al. (2015). https://doi.org/10.1126/science.aaa9375 ; https://doi.org/10.1145/2746539.2746580
- Blum, Hardt (2015). https://proceedings.mlr.press/v37/blum15.html
- Harvey, Liu, Zhu (2016). https://doi.org/10.1093/rfs/hhv059
- Böhme (2018). https://doi.org/10.1145/3210309
- Böhme, Falk (2020). https://doi.org/10.1145/3368089.3409729
- Recht et al. (2019). https://proceedings.mlr.press/v97/recht19a
- Roelofs et al. (2019). https://mlanthology.org/neurips/2019/roelofs2019neurips-metaanalysis
- Kent, R.E. (2016). Information Flow in Logical Environments. arXiv:1603.03475. https://arxiv.org/abs/1603.03475
- Barwise, J., Seligman, J. (1997). Information Flow. CUP. https://doi.org/10.1017/cbo9780511895968
- Goguen, J., Burstall, R. (1992). JACM 39(1):95-146. https://doi.org/10.1145/147508.147524
- Goguen, J. (2005). Logic J. IGPL 13(6):685-715. https://doi.org/10.1093/jigpal/jzi050
- Kalfoglou, Y., Schorlemmer, M. (2003). https://doi.org/10.1007/978-3-540-39733-5_5
- Allwein, G. (2004). NSPW '04. https://doi.org/10.1145/1065907.1066030
- Seligman, J. (2009). https://doi.org/10.1007/978-3-642-00659-3_8
- Baez, Fritz, Leinster (2011). Entropy 13(11):1945-1957. https://doi.org/10.3390/e13111945
- Bai, Pasqualetti, Gupta (2017). Automatica 82:251-260. https://doi.org/10.1016/j.automatica.2017.04.047
- Ranade, Sahai (2017). arXiv:1701.04187. https://arxiv.org/abs/1701.04187
- Pacelli, Theodorou (2026). arXiv:2603.18454. https://arxiv.org/abs/2603.18454
- Sayin, Akyol, Başar. arXiv:1609.09448. https://arxiv.org/abs/1609.09448
- Nair (2013). IEEE TAC 58(6):1497-1510. https://doi.org/10.1109/TAC.2013.2241491
- Hardt, Megiddo, Papadimitriou, Wootters (2016). ITCS. https://doi.org/10.1145/2840728.2840730 ; arXiv:1506.06980
- Perdomo, Zrnic, Mendler-Dünner, Hardt (2020). ICML, PMLR 119:7599-7609. https://proceedings.mlr.press/v119/perdomo20a.html
- Hardt, Ullman (2014). FOCS. https://doi.org/10.1109/focs.2014.55
- Russo, Zou (2016). AISTATS, PMLR 51:1232-1240. https://proceedings.mlr.press/v51/russo16.html
