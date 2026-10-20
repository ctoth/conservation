"""Analytical and independent numerical regressions for the research experiment."""

import math
import unittest
from collections import defaultdict

from matched_kl_gaussian import Model, bernoulli_kl, bon_kl, gold_kl_bounds, logistic


def simpson_bon(model: Model, n: int, divergence: bool = False) -> float:
    """Independent smooth-density integral; wide Gaussian bounds, not score conditioning."""
    a, b, steps = -6.0, 10.0, 16000
    h = (b - a) / steps

    def density(r: float) -> float:
        base = math.fsum(w * math.exp(-(r - g) ** 2 / (2 * v)) / math.sqrt(2 * math.pi * v)
                         for w, g, v in model.components() if divergence or g == 1)
        log_weight = (n - 1) * model.log_cdf(r)
        return n * base * math.exp(log_weight) * (math.log(n) + log_weight if divergence else 1)

    return h / 3 * math.fsum((1 if i in (0, steps) else 4 if i % 2 else 2) * density(a + i * h)
                             for i in range(steps + 1))


class MatchedGaussianTests(unittest.TestCase):
    def test_complete_base_error_marginal_matches_but_proxy_marginal_differs(self) -> None:
        marginals = []
        for model in (Model(False), Model(True)):
            error = defaultdict(float)
            for w, _, v in model.components():
                error[v] += w
            marginals.append(error)
        for variance in marginals[0]:
            self.assertAlmostEqual(marginals[0][variance], marginals[1][variance], places=15)
        third = []
        for model in (Model(False), Model(True)):
            parts = model.components()
            self.assertAlmostEqual(math.fsum(w * g for w, g, _ in parts), 0.3)
            self.assertAlmostEqual(math.fsum(w * (g * g + v) for w, g, v in parts), 0.47)
            third.append(math.fsum(w * (g**3 + 3 * g * v) for w, g, v in parts))
        self.assertAlmostEqual(third[0], 0.39)
        self.assertAlmostEqual(third[1], 0.453)

    def test_analytic_tilt_and_finite_moments(self) -> None:
        for independent in (False, True):
            model = Model(independent)
            for lam in (0, 0.2, 2, 5, 10, 20, 30):
                with self.subTest(independent=independent, lam=lam):
                    expected = logistic(math.log(0.3 / 0.7) + lam
                                        - (0 if independent else 0.1 * lam * lam / 2))
                    value = model.tilt(lam)
                    self.assertAlmostEqual(value.gold, expected, places=13)
                    self.assertTrue(all(math.isfinite(x) for x in
                                        (value.log_normalizer, value.mean_score, value.second_score)))
                    self.assertGreater(value.second_score - value.mean_score**2, 0)
                    self.assertLessEqual(bernoulli_kl(value.gold, model.p), value.divergence + 2e-12)

    def test_heteroscedastic_peak(self) -> None:
        model = Model(False)
        peak = model.tilt(10)
        self.assertAlmostEqual(peak.gold, 0.9845214751227107, places=13)
        self.assertAlmostEqual(peak.divergence, 6.188373236412815, places=11)
        self.assertGreater(peak.gold, model.tilt(9).gold)
        self.assertGreater(peak.gold, model.tilt(11).gold)
        self.assertAlmostEqual(model.tilt(20).gold, model.p, places=13)
        self.assertLess(model.tilt(30).gold, 2e-7)

    def test_matching_finite_joint_proxy_kl(self) -> None:
        for model in (Model(False), Model(True)):
            for n in (1, 2, 1024, 10**12, 10**24):
                budget = bon_kl(n)
                matched = model.match_kl(budget)
                self.assertLess(abs(matched.value.divergence - budget), 1e-9)
                self.assertLessEqual(model.tilt(matched.lambda_lower).divergence, budget + 1e-12)
                self.assertGreaterEqual(model.tilt(matched.lambda_upper).divergence, budget - 1e-12)
                self.assertLess(matched.gold_upper - matched.gold_lower, 1e-10)

    def test_best_of_n_proxy_kl_by_density_integration(self) -> None:
        for model in (Model(False), Model(True)):
            for n in (2, 16, 10**6, 10**12, 10**24):
                self.assertAlmostEqual(simpson_bon(model, n, divergence=True), bon_kl(n), places=9)

    def test_tilt_density_ratio_and_conditional_gold_are_score_only(self) -> None:
        for model in (Model(False), Model(True)):
            lam = 4.0
            tilt = model.tilt(lam)
            for r in (-1, 0, 1, 2, 4):
                base, selected, selected_good = [], [], []
                for w, g, v in model.components():
                    base.append(w * math.exp(-(r - g)**2 / (2*v)) / math.sqrt(2*math.pi*v))
                    new_weight = math.exp(math.log(w) + lam*g + lam*lam*v/2 - tilt.log_normalizer)
                    part = new_weight * math.exp(-(r-g-lam*v)**2 / (2*v)) / math.sqrt(2*math.pi*v)
                    selected.append(part)
                    if g == 1:
                        selected_good.append(part)
                self.assertAlmostEqual(math.log(math.fsum(selected) / math.fsum(base)),
                                       lam*r - tilt.log_normalizer, places=12)
                self.assertAlmostEqual(math.fsum(selected_good) / math.fsum(selected),
                                       model.posterior(r), places=12)

    def test_best_of_two_against_pairwise_gaussian_comparison(self) -> None:
        for model in (Model(False), Model(True)):
            if model.independent:
                noise = ((0.7, 0.2), (0.3, 0.1))
                wins = math.fsum(w * u * (1 + math.erf(1 / math.sqrt(2 * (v + z)))) / 2
                                 for w, v in noise for u, z in noise)
            else:
                wins = (1 + math.erf(1 / math.sqrt(2 * 0.3))) / 2
            exact = 0.3**2 + 2 * 0.3 * 0.7 * wins
            estimate = model.best_of_n(2, 2e-5)
            self.assertLessEqual(estimate.lower, exact)
            self.assertGreaterEqual(estimate.upper, exact)
            self.assertLess(estimate.half_width, 2.01e-5)

    def test_probability_bins_against_independent_density_quadrature(self) -> None:
        for model in (Model(False), Model(True)):
            for n in (2, 16, 1024, 10**6, 10**12, 10**24):
                with self.subTest(model=model.name, n=n):
                    estimate = model.best_of_n(n)
                    reference = simpson_bon(model, n)
                    self.assertLessEqual(estimate.lower, reference + 1e-9)
                    self.assertGreaterEqual(estimate.upper, reference - 1e-9)
                    self.assertLess(estimate.half_width, 1.01e-4)
                    self.assertLess(estimate.tail_mass, 2.51e-5)

    def test_large_n_keeps_survival_and_gold_kl_separate(self) -> None:
        for model in (Model(False), Model(True)):
            # The ordinary binary64 CDF is one here, while the survival is not zero.
            self.assertEqual(math.exp(model.log_cdf(5)), 1.0)
            self.assertLess(model.log_cdf(5), 0.0)
            estimate = model.best_of_n(10**24)
            low, high = gold_kl_bounds(estimate.lower, estimate.upper, model.p)
            self.assertLessEqual(low, high)
            self.assertLess(high, bon_kl(10**24))
            self.assertLess(estimate.half_width, 1.01e-4)
            if model.independent:
                self.assertGreater(estimate.lower, 0.99)
            else:
                self.assertLess(estimate.upper, 0.01)

    def test_tighter_bounds_and_base_policy(self) -> None:
        for model in (Model(False), Model(True)):
            self.assertEqual(model.best_of_n(1).value, model.p)
            coarse = model.best_of_n(256, 2e-4)
            fine = model.best_of_n(256, 5e-5)
            self.assertLess(fine.half_width, coarse.half_width)
            self.assertLessEqual(max(coarse.lower, fine.lower), min(coarse.upper, fine.upper))

    def test_invalid_inputs(self) -> None:
        for n in (0, -1, True, 1.5):
            with self.assertRaises(ValueError):
                bon_kl(n)
        for budget in (-1, math.inf, math.nan):
            with self.assertRaises(ValueError):
                Model(False).match_kl(budget)
        with self.assertRaises(ValueError):
            Model(False, bad_variance=0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
