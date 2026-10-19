"""Toy Goodhart model: a fixed binary verifier whose channel to gold quality moves with the policy.

Three response kinds: good (gold 1), bad (gold 0), exploit (gold 0, always passes).
The optimizer is the KL-regularized pass-rate maximizer pi_lam(x) ∝ p0(x) exp(lam * s(x)),
where s(x) is the pass probability. Everything is computed exactly; logs are natural (nats).
"""

import math

Dist = dict[str, float]

GOLD: Dist = {"good": 1.0, "bad": 0.0, "exploit": 0.0}


def policy(p0: Dist, s: Dist, lam: float) -> Dist:
    # Subtract the max exponent so large lam does not overflow.
    top = max(lam * s[k] for k in p0)
    w = {k: p0[k] * math.exp(lam * s[k] - top) for k in p0}
    z = sum(w.values())
    return {k: v / z for k, v in w.items()}


def kl(p: Dist, q: Dist) -> float:
    return sum(p[k] * math.log(p[k] / q[k]) for k in p if p[k] > 0)


def mutual_information(pi: Dist, s: Dist) -> float:
    """I(verdict; gold) under pi, for the induced joint over {pass, fail} x {gold 0, gold 1}."""
    joint: dict[tuple[int, float], float] = {}
    for k, mass in pi.items():
        g = GOLD[k]
        joint[(1, g)] = joint.get((1, g), 0.0) + mass * s[k]
        joint[(0, g)] = joint.get((0, g), 0.0) + mass * (1 - s[k])
    pv = {v: sum(m for (vv, _), m in joint.items() if vv == v) for v in (0, 1)}
    pg = {g: sum(m for (_, gg), m in joint.items() if gg == g) for g in (0.0, 1.0)}
    return sum(m * math.log(m / (pv[v] * pg[g])) for (v, g), m in joint.items() if m > 0)


def false_pass(pi: Dist, s: Dist) -> float:
    """p_pi(pass | gold 0): the channel the optimizer moves without touching the verifier."""
    bad = pi["bad"] + pi["exploit"]
    return (pi["bad"] * s["bad"] + pi["exploit"] * s["exploit"]) / bad


def sweep(eps: float, good: float = 0.3, a: float = 0.9, b: float = 0.2) -> list[dict[str, float]]:
    p0 = {"good": good, "bad": 1 - good - eps, "exploit": eps}
    s = {"good": a, "bad": b, "exploit": 1.0}
    # The exploit overtakes good once eps * e^lam > good * e^(a lam); run well past that.
    lam_max = 2 * math.log(good / eps) / (1 - a) + 20
    rows: list[dict[str, float]] = []
    for i in range(4001):
        lam = lam_max * i / 4000
        pi = policy(p0, s, lam)
        d = kl(pi, p0)
        rows.append({
            "lam": lam,
            "kl": d,
            "pass": sum(pi[k] * s[k] for k in pi),
            "gold": pi["good"],
            "mi": mutual_information(pi, s),
            "fp": false_pass(pi, s),
            # d/dlam E_pi[gold] = Cov_pi(gold, s) = Cov_pi(gold, verdict) for the tilted family.
            "cov": pi["good"] * s["good"] - pi["good"] * sum(pi[k] * s[k] for k in pi),
            # Binary gold is 1/2-sub-Gaussian, so gold gain <= (1/2) sqrt(2 KL) for any policy.
            "bound": good + 0.5 * math.sqrt(2 * d),
        })
    return rows


def main() -> None:
    good = 0.3
    print(f"log(1/p_good) = {math.log(1 / good):.3f}")
    print("eps     log(1/eps)  I_0     KL*     gold*   KL collapse  KL end   fp_0    fp*     fp end")
    for eps in (1e-2, 1e-3, 1e-4, 1e-6, 1e-9):
        rows = sweep(eps, good)
        assert all(r["gold"] <= r["bound"] + 1e-12 for r in rows)
        peak = max(rows, key=lambda r: r["gold"])
        i = rows.index(peak)
        # Covariance changes sign at the gold peak.
        assert rows[i - 1]["cov"] > 0 > rows[i + 1]["cov"], (eps, rows[i - 1]["cov"], rows[i + 1]["cov"])
        after = [r for r in rows if r["lam"] > peak["lam"] and r["gold"] < good]
        collapse = f"{after[0]['kl']:.3f}" if after else "none"
        print(f"{eps:<7.0e} {math.log(1 / eps):<11.3f} {rows[0]['mi']:<7.4f} {peak['kl']:<7.3f} "
              f"{peak['gold']:<7.4f} {collapse:<12} {rows[-1]['kl']:<8.3f} {rows[0]['fp']:<7.4f} "
              f"{peak['fp']:<7.4f} {rows[-1]['fp']:.4f}")
    print()
    print("trajectory, eps=1e-6")
    print("lam      KL       pass     gold     I(verdict;gold)  p(pass|gold 0)  Cov(gold,verdict)")
    for r in sweep(1e-6, good)[::250]:
        print(f"{r['lam']:<8.1f} {r['kl']:<8.3f} {r['pass']:<8.4f} {r['gold']:<8.4f} "
              f"{r['mi']:<16.4f} {r['fp']:<15.4f} {r['cov']:+.2e}")


if __name__ == "__main__":
    main()
