"""Matched base-error marginals and finite joint/proxy KL, without score truncation.

Run: python -B research/matched_kl_gaussian.py
Tests: python -B research/test_matched_kl_gaussian.py

All logarithms are natural. Numerical values use binary64, not exact arithmetic.
The probability-bin enclosure is analytic in real arithmetic; math.erfc and the
other floating operations are not certified by directed interval rounding.
"""

import heapq
import math
from dataclasses import dataclass


def logsumexp(values: list[float]) -> float:
    top = max(values)
    return top + math.log(math.fsum(math.exp(x - top) for x in values))


def logistic(x: float) -> float:
    return 1 / (1 + math.exp(-x)) if x >= 0 else math.exp(x) / (1 + math.exp(x))


def bernoulli_kl(q: float, p: float) -> float:
    if not 0 <= q <= 1 or not 0 < p < 1:
        raise ValueError("Bernoulli probabilities are outside their domains")
    return math.fsum((q * math.log(q / p) if q else 0.0,
                     (1 - q) * math.log((1 - q) / (1 - p)) if q < 1 else 0.0))


def bon_kl(n: int) -> float:
    """Joint KL = proxy KL for continuous scores and score-only best-of-n."""
    if isinstance(n, bool) or not isinstance(n, int) or n < 1:
        raise ValueError("n must be a positive integer")
    return math.log(n) - (n - 1) / n


@dataclass(frozen=True)
class Tilt:
    lam: float
    gold: float
    divergence: float  # joint KL and proxy KL, each relative to this model's base
    mean_score: float
    second_score: float
    log_normalizer: float


@dataclass(frozen=True)
class MatchedTilt:
    value: Tilt
    lambda_lower: float
    lambda_upper: float
    gold_lower: float
    gold_upper: float


@dataclass(frozen=True)
class GoldEstimate:
    lower: float
    upper: float
    bin_gap: float
    tail_mass: float
    bins: int
    floating_allowance: float

    @property
    def value(self) -> float:
        return (self.lower + self.upper) / 2

    @property
    def half_width(self) -> float:
        return (self.upper - self.lower) / 2


@dataclass(frozen=True)
class Model:
    independent: bool
    p: float = 0.3
    bad_variance: float = 0.2
    good_variance: float = 0.1

    def __post_init__(self) -> None:
        if not 0 < self.p < 1:
            raise ValueError("gold base rate must be strictly between zero and one")
        if any(not math.isfinite(v) or v <= 0
               for v in (self.bad_variance, self.good_variance)):
            raise ValueError("Gaussian variances must be positive and finite")

    @property
    def name(self) -> str:
        return "independent" if self.independent else "dependent"

    def components(self) -> tuple[tuple[float, int, float], ...]:
        """(base probability, gold/score mean, score variance)."""
        if not self.independent:
            return ((1 - self.p, 0, self.bad_variance),
                    (self.p, 1, self.good_variance))
        return tuple((pg * pe, g, v)
                     for g, pg in ((0, 1 - self.p), (1, self.p))
                     for pe, v in ((1 - self.p, self.bad_variance),
                                   (self.p, self.good_variance)))

    def tilt(self, lam: float) -> Tilt:
        if not math.isfinite(lam) or lam < 0:
            raise ValueError("lambda must be nonnegative and finite")
        parts = self.components()
        logs = [math.log(w) + lam * g + lam * lam * v / 2 for w, g, v in parts]
        z = logsumexp(logs)
        weights = [math.exp(a - z) for a in logs]
        gold = math.fsum(w * g for w, (_, g, _) in zip(weights, parts))
        mean = math.fsum(w * (g + lam * v) for w, (_, g, v) in zip(weights, parts))
        second = math.fsum(w * (v + (g + lam * v) ** 2)
                           for w, (_, g, v) in zip(weights, parts))
        divergence = lam * mean - z if lam else 0.0
        if not all(math.isfinite(x) for x in (z, gold, mean, second, divergence)):
            raise ArithmeticError("tilted normalizer or moments exceed binary64 range")
        return Tilt(lam, gold, divergence, mean, second, z)

    def match_kl(self, budget: float) -> MatchedTilt:
        if not math.isfinite(budget) or budget < 0:
            raise ValueError("KL budget must be nonnegative and finite")
        if budget == 0:
            return MatchedTilt(self.tilt(0), 0, 0, self.p, self.p)
        lo, hi = 0.0, 1.0
        while self.tilt(hi).divergence < budget:
            hi *= 2
        for _ in range(100):
            mid = (lo + hi) / 2
            if self.tilt(mid).divergence < budget:
                lo = mid
            else:
                hi = mid
            if hi - lo <= 2e-13 * max(1, hi):
                break
        golds = [self.tilt(lo).gold, self.tilt(hi).gold]
        delta = self.bad_variance - self.good_variance
        if not self.independent and delta > 0 and lo <= 1 / delta <= hi:
            golds.append(self.tilt(1 / delta).gold)
        return MatchedTilt(self.tilt((lo + hi) / 2), lo, hi, min(golds), max(golds))

    def log_cdf(self, r: float) -> float:
        """Keep the upper-tail probability: 1 - tiny_tail would round to one."""
        parts = self.components()
        survival = math.fsum(w * math.erfc((r - g) / math.sqrt(2 * v)) / 2
                             for w, g, v in parts)
        if survival < 0.5:
            return math.log1p(-survival)
        cdf = math.fsum(w * math.erfc((g - r) / math.sqrt(2 * v)) / 2
                       for w, g, v in parts)
        return math.log(cdf) if cdf else -math.inf

    def posterior(self, r: float) -> float:
        logs = [(math.log(w) - math.log(2 * math.pi * v) / 2 - (r - g) ** 2 / (2 * v), g)
                for w, g, v in self.components()]
        good = logsumexp([a for a, g in logs if g == 1])
        bad = logsumexp([a for a, g in logs if g == 0])
        return logistic(good - bad)

    def posterior_bounds(self, a: float, b: float) -> tuple[float, float]:
        if not self.independent:
            vals = [self.posterior(a), self.posterior(b)]
            delta = self.bad_variance - self.good_variance
            if delta and a < self.bad_variance / delta < b:
                vals.append(self.posterior(self.bad_variance / delta))
            return min(vals), max(vals)
        # Bound each Gaussian on the interval, then its class density mixture.
        low: dict[int, list[float]] = {0: [], 1: []}
        high: dict[int, list[float]] = {0: [], 1: []}
        for w, g, v in self.components():
            center = min(b, max(a, float(g)))
            constant = math.log(w) - math.log(2 * math.pi * v) / 2
            low[g].append(constant - max((a - g) ** 2, (b - g) ** 2) / (2 * v))
            high[g].append(constant - (center - g) ** 2 / (2 * v))
        return (logistic(logsumexp(low[1]) - logsumexp(high[0])),
                logistic(logsumexp(high[1]) - logsumexp(low[0])))

    def best_of_n(self, n: int, tolerance: float = 1e-4) -> GoldEstimate:
        """Enclose integral m(r) d(F(r)^n), with gold in [0,1].

        Each bin has exact-form maximum-score mass and a posterior enclosure.
        Omitted tails can contribute between zero and their total probability.
        The reported floating allowance is a cushion, not formal interval rounding.
        """
        bon_kl(n)  # validate before the n=1 shortcut
        if not math.isfinite(tolerance) or not 1e-8 <= tolerance <= 0.01:
            raise ValueError("absolute tolerance must lie in [1e-8, 0.01]")
        if n == 1:
            return GoldEstimate(self.p, self.p, 0, 0, 0, 0)
        a, b, step = -1.0, 2.0, 1.0
        while math.exp(n * self.log_cdf(a)) > tolerance / 8:
            a -= step
            step *= 2
        step = 1.0
        while -math.expm1(n * self.log_cdf(b)) > tolerance / 8:
            b += step
            step *= 2
        tail = math.exp(n * self.log_cdf(a)) - math.expm1(n * self.log_cdf(b))

        def make_bin(left: float, right: float) -> tuple[float, ...]:
            x, y = n * self.log_cdf(left), n * self.log_cdf(right)
            if x > y:
                raise ArithmeticError("maximum-score CDF decreased")
            mass = 0.0 if y == -math.inf else math.exp(y) * (-math.expm1(x - y))
            low, high = self.posterior_bounds(left, right)
            return (-mass * (high - low), left, right, mass, low, high)

        heap = [make_bin(a, b)]
        gap = -heap[0][0]
        while gap + tail > 2 * tolerance:
            old = heapq.heappop(heap)
            mid = (old[1] + old[2]) / 2
            if mid in (old[1], old[2]) or len(heap) >= 200000:
                raise ArithmeticError("probability bins could not resolve the requested error")
            left, right = make_bin(old[1], mid), make_bin(mid, old[2])
            gap = math.fsum((gap, old[0], -left[0], -right[0]))
            heapq.heappush(heap, left)
            heapq.heappush(heap, right)
        gap = math.fsum(-item[0] for item in heap)
        floating = 64 * math.ulp(1.0) * len(heap)
        lower = max(0.0, math.fsum(item[3] * item[4] for item in heap) - floating)
        upper = min(1.0, math.fsum(item[3] * item[5] for item in heap) + tail + floating)
        return GoldEstimate(lower, upper, gap, tail, len(heap), floating)


def gold_kl_bounds(lower: float, upper: float, p: float) -> tuple[float, float]:
    ends = (bernoulli_kl(lower, p), bernoulli_kl(upper, p))
    return (0.0 if lower <= p <= upper else min(ends), max(ends))


def display_interval(lower: float, upper: float) -> str:
    # Round displayed bounds outward; ordinary rounding can shrink an enclosure.
    return f"[{math.floor(lower * 1e6) / 1e6:.6f}, {math.ceil(upper * 1e6) / 1e6:.6f}]"


def main() -> None:
    ns = (1, 2, 16, 256, 1024, 4096, 65536, 10**6, 10**12, 10**24)
    print("Base p=0.3; conditional bad/good variances=0.2/0.1; gold tolerance=1e-4")
    print("Joint KL = proxy KL = D. Gold KL is shown separately; bases differ across models.")
    print("| model | n | D | lambda | tilt gold | BoN gold interval | tilt gold KL | BoN gold KL interval |")
    print("|---|---:|---:|---:|---:|---:|---:|---:|")
    residuals, gaps, tails, floating = [], [], [], []
    lambda_widths, gold_widths = [], []
    for model in (Model(False), Model(True)):
        for n in ns:
            budget = bon_kl(n)
            matched = model.match_kl(budget)
            bon = model.best_of_n(n)
            low, high = gold_kl_bounds(bon.lower, bon.upper, model.p)
            residuals.append(abs(matched.value.divergence - budget))
            lambda_widths.append(matched.lambda_upper - matched.lambda_lower)
            gold_widths.append(matched.gold_upper - matched.gold_lower)
            gaps.append(bon.bin_gap)
            tails.append(bon.tail_mass)
            floating.append(bon.floating_allowance)
            print(f"| {model.name} | {n} | {budget:.6f} | {matched.value.lam:.6f} | "
                  f"{matched.value.gold:.6f} | {display_interval(bon.lower, bon.upper)} | "
                  f"{bernoulli_kl(matched.value.gold, model.p):.6f} | {display_interval(low, high)} |")
    print(f"max matched-KL residual: {max(residuals):.3e}")
    print(f"max lambda bracket width: {max(lambda_widths):.3e}; "
          f"max tilt gold bracket width: {max(gold_widths):.3e}")
    print(f"max bin gap: {max(gaps):.3e}; max omitted tail mass: {max(tails):.3e}")
    print(f"max floating allowance: {max(floating):.3e} (not directed interval arithmetic)")


if __name__ == "__main__":
    main()
