"""Numeric check of reports/regression-curve-peak.md: the rate formulas (section 1), the path
identity (section 3) and the shape of m (section 2).

A model is its proxy levels t_0 < ... < t_K, base masses p0 and regression curve
m(t) = E0[g | r = t]. Everything is exact arithmetic on that representation except the
best-of-n Monte Carlo, which samples (r, g) from the joint to check E_n[g | r] = m(r)
independently. Logs are natural (nats).
"""

import bisect
import math
import random
from dataclasses import dataclass, field
from typing import Callable

Pdf = Callable[[float], float]


@dataclass
class Model:
    name: str
    t: list[float]
    p0: list[float]
    m: list[float]
    i0: float = float("nan")  # I(r; g) at the base distribution
    joint: list[tuple[int, float, float]] = field(default_factory=list[tuple[int, float, float]])  # (level, g, mass)


def normal_pdf(scale: float) -> Pdf:
    c = 1 / (scale * math.sqrt(2 * math.pi))
    return lambda x: c * math.exp(-0.5 * (x / scale) ** 2)


def student_t_pdf(nu: float, scale: float) -> Pdf:
    c = math.gamma((nu + 1) / 2) / (math.gamma(nu / 2) * math.sqrt(nu * math.pi) * scale)
    return lambda x: c * (1 + (x / scale) ** 2 / nu) ** (-(nu + 1) / 2)


def toy(eps: float = 1e-6) -> Model:
    # reports/goodhart-toy.md: bad (r 0.2, gold 0), good (r 0.9, gold 1), exploit (r 1, gold 0).
    return Model("toy eps=1e-6", [0.2, 0.9, 1.0], [0.7 - eps, 0.3, eps], [0.0, 1.0, 0.0])


def grid_model(name: str, gold: Pdf, noise: Pdf, r_max: float = 30.0,
               g_max: float = 8.0, dg: float = 0.05, dt: float = 0.1) -> Model:
    """r = g + noise on a grid: g in [-g_max, g_max], r in [-r_max, r_max]."""
    gs = [-g_max + dg * i for i in range(round(2 * g_max / dg) + 1)]
    ts = [-r_max + dt * j for j in range(round(2 * r_max / dt) + 1)]
    pg = [gold(g) for g in gs]
    rows = [[w0 * noise(t - g) for g, w0 in zip(gs, pg)] for t in ts]
    # Drop proxy levels whose mass underflows to 0 (far tails of Gaussian noise).
    total = sum(sum(row) for row in rows)
    keep = [j for j in range(len(ts)) if sum(rows[j]) / total > 0]
    g_marg = [sum(rows[j][i] for j in keep) / total for i in range(len(gs))]
    t: list[float] = []
    p0: list[float] = []
    m: list[float] = []
    joint: list[tuple[int, float, float]] = []
    i0 = 0.0
    for level, j in enumerate(keep):
        pr = sum(rows[j])
        t.append(ts[j])
        p0.append(pr / total)
        m.append(sum(g * w for g, w in zip(gs, rows[j])) / pr)
        for i, (g, w) in enumerate(zip(gs, rows[j])):
            joint.append((level, g, w))
            if w > 0:
                # Logs taken separately: the product of two tail masses underflows.
                i0 += w / total * (math.log(w) - math.log(pr) - math.log(g_marg[i]))
    return Model(name, t, p0, m, i0, joint)


def tilt(md: Model, lam: float) -> list[float]:
    top = lam * md.t[-1]  # levels ascend and lam >= 0, so the top level has the largest exponent
    w = [p * math.exp(lam * t - top) for p, t in zip(md.p0, md.t)]
    z = sum(w)
    return [x / z for x in w]


def dot(a: list[float], b: list[float]) -> float:
    return sum(x * y for x, y in zip(a, b))


def kernel(pi: list[float], t: list[float]) -> list[float]:
    """K(t_j) = E_pi[(R - E_pi R) 1{R > t_j}], for j < K."""
    mu = dot(pi, t)
    k = [0.0] * (len(t) - 1)
    acc = 0.0
    for j in range(len(t) - 1, 0, -1):
        acc += pi[j] * (t[j] - mu)
        k[j - 1] = acc
    return k


def dm(md: Model) -> list[float]:
    return [md.m[j + 1] - md.m[j] for j in range(len(md.m) - 1)]


def cdf(md: Model) -> list[float]:
    # 1 - upper tail, so F stays accurate (below 1) near the top levels.
    out = [1.0] * len(md.p0)
    acc = 0.0
    for j in range(len(md.p0) - 1, 0, -1):
        acc += md.p0[j]
        out[j - 1] = 1.0 - acc
    return out


def bon(md: Model, n: float) -> list[float]:
    """Law of the selected level under best-of-n: F_j^n - F_{j-1}^n."""
    f = cdf(md)
    return [f[j] ** n - (f[j - 1] ** n if j else 0.0) for j in range(len(f))]


def kl(p: list[float], q: list[float]) -> float:
    return sum(a * math.log(a / b) for a, b in zip(p, q) if a > 0)


def close(a: float, b: float, tol: float) -> bool:
    return abs(a - b) <= tol * max(1.0, abs(a), abs(b))


def check_tilt_rate(md: Model, lams: list[float], h: float = 1e-5) -> float:
    """Section 1: d/dlam E[g] = Cov(m(R), R) = sum_j dm_j K(t_j). Returns the worst gap."""
    worst = 0.0
    for lam in lams:
        fd = (dot(tilt(md, lam + h), md.m) - dot(tilt(md, lam - h), md.m)) / (2 * h)
        pi = tilt(md, lam)
        cov = dot(pi, [a * b for a, b in zip(md.m, md.t)]) - dot(pi, md.m) * dot(pi, md.t)
        formula = dot(dm(md), kernel(pi, md.t))
        assert close(cov, formula, 1e-9), (md.name, lam, cov, formula)
        assert close(fd, formula, 1e-5), (md.name, lam, fd, formula)
        worst = max(worst, abs(fd - formula))
    return worst


def check_bon_rate(md: Model, ns: list[int]) -> float:
    """Section 1: E_{n+1}[g] - E_n[g] = sum_j dm_j F_j^n (1 - F_j)."""
    f = cdf(md)
    worst = 0.0
    for n in ns:
        direct = dot(bon(md, n + 1), md.m) - dot(bon(md, n), md.m)
        formula = sum(d * f[j] ** n * (1 - f[j]) for j, d in enumerate(dm(md)))
        assert close(direct, formula, 1e-9), (md.name, n, direct, formula)
        worst = max(worst, abs(direct - formula))
    return worst


def check_tilt_path(md: Model, lam_end: float, steps: int = 4000) -> float:
    """Section 3: int_0^L K_lam(t_j) dlam = P_L(R > t_j) - P_0(R > t_j) (Simpson), every j."""
    hstep = lam_end / steps
    acc = [0.0] * (len(md.t) - 1)
    for i in range(steps + 1):
        wgt = 1 if i in (0, steps) else (4 if i % 2 else 2)
        for j, k in enumerate(kernel(tilt(md, i * hstep), md.t)):
            acc[j] += wgt * k * hstep / 3
    end = tilt(md, lam_end)
    worst = 0.0
    for j in range(len(md.t) - 1):
        exact = sum(end[j + 1:]) - sum(md.p0[j + 1:])
        worst = max(worst, abs(acc[j] - exact))
    assert worst < 1e-6, (md.name, worst)
    return worst


def check_limits(md: Model) -> tuple[float, float, float]:
    """Section 3: total change m(r_max) - E0 m = sum_j dm_j F_j; both optimizers reach it."""
    identity = sum(d * f for d, f in zip(dm(md), cdf(md)))
    budget = md.m[-1] - dot(md.p0, md.m)
    assert close(identity, budget, 1e-9), (md.name, identity, budget)
    bon_end = dot(bon(md, 1e12), md.m) - dot(md.p0, md.m)
    gap = md.t[-1] - md.t[-2]
    # Large enough that the top level holds all but e^-60 of the tilted mass.
    tilt_end = dot(tilt(md, (60 - 2 * math.log(md.p0[-1])) / gap), md.m) - dot(md.p0, md.m)
    return budget, bon_end, tilt_end


def mc_bon(md: Model, n: int, trials: int, rng: random.Random) -> tuple[float, float]:
    """Best-of-n sampled from the joint (ties go to the first draw, independent of g)."""
    cum: list[float] = []
    acc = 0.0
    for _, _, w in md.joint:
        acc += w
        cum.append(acc)
    vals: list[float] = []
    for _ in range(trials):
        best = max((md.joint[bisect.bisect_left(cum, rng.random() * acc)] for _ in range(n)),
                   key=lambda a: a[0])
        vals.append(best[1])
    mean = sum(vals) / trials
    sd = math.sqrt(sum((v - mean) ** 2 for v in vals) / (trials - 1))
    return mean, sd / math.sqrt(trials)


def sweep_peak(golds: list[float], kls: list[float], labels: list[float]) -> str:
    i = max(range(len(golds)), key=lambda k: golds[k])
    rising = all(golds[k + 1] >= golds[k] - 1e-12 for k in range(len(golds) - 1))
    if rising:
        return f"none (non-decreasing; end gold {golds[-1]:+.4f} at KL {kls[-1]:.2f})"
    return f"at {labels[i]:g}: gold {golds[i]:+.4f}, KL {kls[i]:.3f}; end gold {golds[-1]:+.4f} at KL {kls[-1]:.2f}"


def main() -> None:
    models = [
        toy(),
        grid_model("N(0,1) gold, N(0,.5) noise", normal_pdf(1.0), normal_pdf(0.5)),
        grid_model("N(0,1) gold, t3(.5) noise", normal_pdf(1.0), student_t_pdf(3, 0.5)),
        grid_model("t3(1) gold, N(0,1) noise", student_t_pdf(3, 1.0), normal_pdf(1.0)),
    ]
    print("Section 1 and 3 identities (worst absolute gap)")
    print(f"{'model':<30} {'tilt rate':<10} {'BoN rate':<10} {'tilt path':<10} "
          f"{'budget':<9} {'BoN 1e12':<9} {'tilt end':<9}")
    for md in models:
        lam_end = 300.0 if md.name.startswith("toy") else 2.0
        a = check_tilt_rate(md, [0.01, 0.3, 1.0, 3.0, 10.0])
        b = check_bon_rate(md, [1, 2, 5, 20, 100, 1000])
        c = check_tilt_path(md, lam_end, 4000 if md.name.startswith("toy") else 400)
        budget, bon_end, tilt_end = check_limits(md)
        print(f"{md.name:<30} {a:<10.1e} {b:<10.1e} {c:<10.1e} {budget:<+9.4f} {bon_end:<+9.4f} {tilt_end:<+9.4f}")

    print()
    print("Toy best-of-n against the closed form P(best is good) = (1-eps)^n - (0.7-eps)^n")
    md = models[0]
    eps = md.p0[-1]
    for n in (1, 3, 10, 100, 10**4, 10**6, 10**7):
        exact = (1 - eps) ** n - (0.7 - eps) ** n
        formula = dot(bon(md, n), md.m)
        assert close(exact, formula, 1e-9), (n, exact, formula)
        print(f"  n={n:<9} closed form {exact:.6f}  via m {formula:.6f}")

    print()
    print("Grid best-of-n, Monte Carlo from the joint vs the m formula (20000 trials)")
    rng = random.Random(20261019)
    for md in models[1:]:
        for n in (1, 4, 16, 64):
            mc, se = mc_bon(md, n, 20000, rng)
            formula = dot(bon(md, n), md.m)
            assert abs(mc - formula) < 4 * se, (md.name, n, mc, formula, se)
            print(f"  {md.name:<30} n={n:<3} MC {mc:+.4f} ± {se:.4f}  formula {formula:+.4f}")

    print()
    print("Shape of m(t) = E0[g | r = t]")
    for md in models[1:]:
        picks = [0.0, 1.0, 2.0, 4.0, 8.0, 16.0, 30.0]
        vals = [md.m[min(range(len(md.t)), key=lambda j: abs(md.t[j] - x))] for x in picks]
        nondecreasing = all(d >= -1e-12 for d in dm(md))
        top = max(range(len(md.t)), key=lambda j: md.m[j])
        shape = "non-decreasing" if nondecreasing else f"peaks at t={md.t[top]:.1f} (m={md.m[top]:.3f})"
        print(f"  {md.name:<30} " + " ".join(f"m({x:g})={v:+.3f}" for x, v in zip(picks, vals)) + f"  -> {shape}")
    gauss = models[1]
    slope = 1 / (1 + 0.5 ** 2)
    assert all(abs(m - slope * t) < 1e-6 for m, t in zip(gauss.m, gauss.t) if abs(t) < 5)
    assert all(d >= -1e-12 for d in dm(models[3]))
    heavy = models[2]
    top = max(range(len(heavy.t)), key=lambda j: heavy.m[j])
    assert 0 < top < len(heavy.t) - 1 and all(d <= 1e-12 for d in dm(heavy)[top:])

    print()
    print("Gold peaks (gold measured relative to E0 g)")
    for md in models:
        base = dot(md.p0, md.m)
        lams = [0.05 * i for i in range(401)] if not md.name.startswith("toy") else [0.1 * i for i in range(3001)]
        pis = [tilt(md, lam) for lam in lams]
        ns = [2 ** k for k in range(41)]
        bons = [bon(md, n) for n in ns]
        print(f"  {md.name}  (I0 = {md.i0:.3f} nats)" if not math.isnan(md.i0) else f"  {md.name}")
        print("    tilt: " + sweep_peak([dot(p, md.m) - base for p in pis], [kl(p, md.p0) for p in pis], lams))
        print("    BoN:  " + sweep_peak([dot(p, md.m) - base for p in bons], [kl(p, md.p0) for p in bons], ns))

    print()
    print("Truncation dependence of the tilt, t3 noise (the untruncated tilt does not exist)")
    for r_max in (10.0, 20.0, 30.0):
        md = grid_model("", normal_pdf(1.0), student_t_pdf(3, 0.5), r_max=r_max)
        lams = [0.01 * i for i in range(401)]
        pis = [tilt(md, lam) for lam in lams]
        print(f"  r_max={r_max:<5g} " + sweep_peak([dot(p, md.m) for p in pis], [kl(p, md.p0) for p in pis], lams))


if __name__ == "__main__":
    main()
