//! # dojo-types — the master testing vocabulary's shared border (M0)
//!
//! `dojo` (道場, the proving hall) unifies every fleet testing QUALITY as a
//! composable typed FACET. This crate is the thin border every testing primitive
//! depends on: [`Veredito`] (one verdict, lattice-composable), [`Evidence`] (the
//! deciding typed sum), [`Outcome`], [`Report`], and the [`DojoFacet`] contract.
//!
//! **M0 proves the load-bearing claim** (verify-pass R2): the
//! typed [`Evidence`] sum embeds sui's **8-way** `parity::Verdict` *losslessly* —
//! no `Falsified{evidence: String}` stringly-typed-discriminant. The tests are the
//! M0 **falsification gate**: if a lift is lossy, the shared type is not earned and
//! the pattern stays un-extracted. Here it is earned — every parity
//! variant round-trips through [`Veredito`] and back.
//!
//! Tier-honest: this M0 defines the SHAPES of the two facet source-verdicts
//! ([`ParityVerdict`] is the destination type `dojo-differential` will own once the
//! de-weld from sui lands; [`EnsaioVerdict`] mirrors ensaio's shipped 3-variant) so
//! the border is provably non-lossy standalone. Wiring the *real* `ensaio::Verdict`
//! / `sui::parity::Verdict` crates (their `From` impls landing in those crates as
//! they adopt `dojo-types`) is M2 — the honest next tier, not this slice.

use serde::{Deserialize, Serialize};

/// Typed evidence carried by a [`Veredito`]. **The deciding new algebra of dojo**:
/// a first-class sum, never an opaque `String`, so a rich source
/// discriminant (the 8-way parity verdict) survives the lift losslessly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Evidence {
    /// Free-text rationale (ensaio's shipped evidence shape).
    Message(String),
    /// A two-backend differential, re-embedding the full parity discriminant.
    ParityDifferential(ParityVerdict),
    /// Captured process output (the omote/sui `exec.rs` capture shape).
    Captured(CapturedOutput),
    /// No evidence (e.g. a `NotApplicable`).
    None,
}

/// Captured shell output — the richest capture in the fleet (lifted, shape-wise,
/// from sui `exec.rs` `CapturedOutput`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CapturedOutput {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// The ONE verdict every facet meets into. A 4-point severity lattice; `meet`
/// takes the worst (greatest-lower-bound). Evidence-carrying, never bare.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Veredito {
    /// The claim holds.
    Held { evidence: Evidence },
    /// The claim is refuted — a real found failure (the strongest negative signal).
    Falsified { evidence: Evidence },
    /// The facet could not run to a verdict (infra/harness fault, not a refutation).
    Errored { message: String },
    /// The facet did not apply to this subject (meets away).
    NotApplicable,
}

impl Veredito {
    /// Worst-severity-wins ordering. `NotApplicable` is the lattice identity (it
    /// meets away); `Falsified` is absorbing. Order:
    /// `Falsified > Errored > Held > NotApplicable`.
    #[must_use]
    pub fn severity(&self) -> u8 {
        match self {
            Veredito::NotApplicable => 0,
            Veredito::Held { .. } => 1,
            Veredito::Errored { .. } => 2,
            Veredito::Falsified { .. } => 3,
        }
    }

    /// The lattice `meet` — the suite verdict of two facets: the worse of the two.
    /// `{Held, NotApplicable} → Held` (a pass + a skip is a pass); `{Held,
    /// Falsified} → Falsified`; `{Held, Errored} → Errored`.
    #[must_use]
    pub fn meet(self, other: Veredito) -> Veredito {
        if other.severity() > self.severity() {
            other
        } else {
            self
        }
    }

    /// `meet` across a suite of facet verdicts (identity = `NotApplicable`).
    #[must_use]
    pub fn meet_all(verdicts: impl IntoIterator<Item = Veredito>) -> Veredito {
        verdicts
            .into_iter()
            .fold(Veredito::NotApplicable, Veredito::meet)
    }

    /// Did this verdict pass? (`Held` or `NotApplicable` — the gate-open condition.)
    #[must_use]
    pub fn is_held(&self) -> bool {
        matches!(self, Veredito::Held { .. } | Veredito::NotApplicable)
    }
}

// ── facet source-verdicts + their LOSSLESS lift adapters ───────────────────────

/// sui's shipped two-backend differential verdict — **8-way, `Ord`-ranked**. This
/// is the stress test for the border: collapsing it to a bare `Falsified` would
/// destroy an operationally-distinct discriminant (candidate-broke vs both-diverged
/// vs reference-infra-flake). `dojo-differential` owns this once de-welded from sui.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParityVerdict {
    /// Both backends produced the same observable.
    Match,
    /// The probe did not apply.
    NotApplicable,
    /// Both ran, observables diverged.
    Differ,
    /// The candidate failed where the reference succeeded.
    CandFailOnly,
    /// The reference failed where the candidate succeeded.
    RefFailOnly,
    /// Both backends failed.
    BothFail,
    /// The candidate timed out.
    CandTimeout,
    /// The reference timed out (infra flake — NOT a real divergence).
    RefTimeout,
}

impl From<ParityVerdict> for Veredito {
    /// LOSSLESS: the full 8-way discriminant rides in
    /// `Evidence::ParityDifferential`, so it can be recovered — never a String.
    fn from(p: ParityVerdict) -> Veredito {
        match p {
            ParityVerdict::Match => Veredito::Held {
                evidence: Evidence::ParityDifferential(p),
            },
            ParityVerdict::NotApplicable => Veredito::NotApplicable,
            // Every negative parity outcome is a Falsified, but the WHICH is
            // preserved in the evidence (the deciding non-lossy move).
            ParityVerdict::Differ
            | ParityVerdict::CandFailOnly
            | ParityVerdict::RefFailOnly
            | ParityVerdict::BothFail
            | ParityVerdict::CandTimeout
            | ParityVerdict::RefTimeout => Veredito::Falsified {
                evidence: Evidence::ParityDifferential(p),
            },
        }
    }
}

/// ensaio's shipped architectural-trial verdict (3-variant). Mirrored here so M0 is
/// standalone; the real `ensaio::Verdict` `From` impl lands in ensaio at M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EnsaioVerdict {
    Held { evidence: String },
    Falsified { evidence: String },
    Errored { message: String },
}

impl From<EnsaioVerdict> for Veredito {
    fn from(e: EnsaioVerdict) -> Veredito {
        match e {
            EnsaioVerdict::Held { evidence } => Veredito::Held {
                evidence: Evidence::Message(evidence),
            },
            EnsaioVerdict::Falsified { evidence } => Veredito::Falsified {
                evidence: Evidence::Message(evidence),
            },
            EnsaioVerdict::Errored { message } => Veredito::Errored { message },
        }
    }
}

// ── the facet contract + the suite (gate + meet) ──────────────────────────────

/// A single facet's result: its verdict + which facet produced it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Outcome {
    pub facet: String,
    pub verdict: Veredito,
}

/// The suite aggregate: the ordered facet outcomes + the meet verdict.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub outcomes: Vec<Outcome>,
    pub verdict: Veredito,
}

/// The ONE facet contract — the `apply<E: Env>(env) -> Veredito` shape every fleet
/// testing primitive already implements. The `Env` type stays **per-facet** (the
/// engines are kept); `DojoFacet` shares the *shape*, never the Env type.
pub trait DojoFacet {
    /// The facet's injected side-effect seam (omote `ShellEnvironment`, ensaio
    /// `EnsaioEnvironment`, …) — mockable for unit tests.
    type Env;
    /// The facet keyword (`case`/`parity`/`perf-lever`/…) for catalog reflection.
    fn kind(&self) -> FacetKind;
    /// Run the facet against its environment, producing one lattice verdict.
    fn run(&self, env: &Self::Env) -> Veredito;
}

/// The facet catalog — CATALOG-REFLECTION governs it (every kind ↔ an interpreter
/// arm; a kind with no arm, or an arm with no kind, is a CI failure). Generalizes
/// ensaio's `AssertionKind::ALL` bijection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FacetKind {
    Case,          // omote shell-invocation
    Ensaio,        // architectural trial
    Parity,        // two-backend differential
    PerfLever,     // perf-seal honesty gate
    Prova,         // attestation
    Property,      // proptest
    Laws,          // trait-law battery
    Matrix,        // mass-synthesis
    Compose,       // composition / determinism
    RebuildProbe,  // rebuild-shadow
}

impl FacetKind {
    /// The full facet population — the bijection anchor (CATALOG REFLECTION).
    pub const ALL: &'static [FacetKind] = &[
        FacetKind::Case,
        FacetKind::Ensaio,
        FacetKind::Parity,
        FacetKind::PerfLever,
        FacetKind::Prova,
        FacetKind::Property,
        FacetKind::Laws,
        FacetKind::Matrix,
        FacetKind::Compose,
        FacetKind::RebuildProbe,
    ];

    /// The `(def…)` authoring keyword for this facet.
    #[must_use]
    pub fn keyword(self) -> &'static str {
        match self {
            FacetKind::Case => "defcase",
            FacetKind::Ensaio => "defensaio",
            FacetKind::Parity => "defparity",
            FacetKind::PerfLever => "defperf-lever",
            FacetKind::Prova => "defprova",
            FacetKind::Property => "defproperty",
            FacetKind::Laws => "deflaws",
            FacetKind::Matrix => "defmatrix",
            FacetKind::Compose => "defcompose",
            FacetKind::RebuildProbe => "defrebuild-probe",
        }
    }
}

/// `gate_on`: run `next` only if `prev` held (the sole cross-facet edge — a
/// verdict predicate, not a data feed). Returns `next`'s verdict, or a skipping
/// `NotApplicable` when the gate is closed.
#[must_use]
pub fn gate_on<E>(prev: &Veredito, next: &dyn DojoFacet<Env = E>, env: &E) -> Veredito {
    if prev.is_held() {
        next.run(env)
    } else {
        Veredito::NotApplicable
    }
}

/// A minimal M0 suite runner: run facets in order, `gate_on` the previous verdict,
/// `meet` the results into one `Report`. (The M3 `(defsuite)` compiles to a full
/// `shigoto::Dag`; this is the 2-node vertical slice.)
#[must_use]
pub fn run_gated_suite<E>(facets: &[&dyn DojoFacet<Env = E>], env: &E) -> Report {
    let mut outcomes = Vec::new();
    let mut prev = Veredito::NotApplicable; // gate opens for the first facet
    for facet in facets {
        let verdict = gate_on(&prev, *facet, env);
        outcomes.push(Outcome {
            facet: facet.kind().keyword().to_string(),
            verdict: verdict.clone(),
        });
        prev = verdict;
    }
    let verdict = Veredito::meet_all(outcomes.iter().map(|o| o.verdict.clone()));
    Report { outcomes, verdict }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── M0 FALSIFICATION GATE: the 8-way parity verdict lifts LOSSLESSLY ──────
    #[test]
    fn every_parity_variant_round_trips_through_veredito() {
        let all = [
            ParityVerdict::Match,
            ParityVerdict::NotApplicable,
            ParityVerdict::Differ,
            ParityVerdict::CandFailOnly,
            ParityVerdict::RefFailOnly,
            ParityVerdict::BothFail,
            ParityVerdict::CandTimeout,
            ParityVerdict::RefTimeout,
        ];
        for p in all {
            let v: Veredito = p.into();
            // NotApplicable is the one variant that meets away (no evidence needed).
            if p == ParityVerdict::NotApplicable {
                assert_eq!(v, Veredito::NotApplicable);
                continue;
            }
            // The discriminant MUST be recoverable — no String round-trip.
            let recovered = match &v {
                Veredito::Held { evidence: Evidence::ParityDifferential(q) }
                | Veredito::Falsified { evidence: Evidence::ParityDifferential(q) } => Some(*q),
                _ => None,
            };
            assert_eq!(
                recovered,
                Some(p),
                "LOSSY LIFT for {p:?} — the border is NOT earned (M0 falsification gate fires)"
            );
        }
    }

    #[test]
    fn parity_reffailonly_and_differ_are_distinguishable_after_lift() {
        // The exact case the verify pass flagged: these must NOT collapse.
        let differ: Veredito = ParityVerdict::Differ.into();
        let reffail: Veredito = ParityVerdict::RefFailOnly.into();
        assert!(matches!(differ, Veredito::Falsified { .. }));
        assert!(matches!(reffail, Veredito::Falsified { .. }));
        assert_ne!(differ, reffail, "Differ and RefFailOnly must stay distinct in evidence");
    }

    #[test]
    fn ensaio_verdict_lifts() {
        let v: Veredito = EnsaioVerdict::Held { evidence: "ok".into() }.into();
        assert_eq!(v, Veredito::Held { evidence: Evidence::Message("ok".into()) });
    }

    // ── the severity lattice ──────────────────────────────────────────────────
    #[test]
    fn meet_is_worst_severity_wins() {
        let held = Veredito::Held { evidence: Evidence::None };
        let fals = Veredito::Falsified { evidence: Evidence::None };
        let err = Veredito::Errored { message: "x".into() };
        // pass + skip = pass
        assert_eq!(held.clone().meet(Veredito::NotApplicable), held);
        // pass + fail = fail
        assert_eq!(held.clone().meet(fals.clone()), fals);
        // pass + error = error; error + fail = fail
        assert_eq!(held.clone().meet(err.clone()), err);
        assert_eq!(err.clone().meet(fals.clone()), fals);
        // all-skip = skip (identity)
        assert_eq!(Veredito::meet_all([Veredito::NotApplicable, Veredito::NotApplicable]), Veredito::NotApplicable);
    }

    // ── the parity→perf gate_on hook (the one M0 hook) ────────────────────────
    struct MockEnv {
        parity: ParityVerdict,
    }
    struct ParityFacet;
    struct PerfFacet;
    impl DojoFacet for ParityFacet {
        type Env = MockEnv;
        fn kind(&self) -> FacetKind { FacetKind::Parity }
        fn run(&self, env: &MockEnv) -> Veredito { env.parity.into() }
    }
    impl DojoFacet for PerfFacet {
        type Env = MockEnv;
        fn kind(&self) -> FacetKind { FacetKind::PerfLever }
        // perf-seal gates the lever DECLARATION's honesty (claimed<=earned_tier),
        // NOT a speedup — here a mock honest lever holds.
        fn run(&self, _env: &MockEnv) -> Veredito { Veredito::Held { evidence: Evidence::Message("lever honest".into()) } }
    }

    #[test]
    fn parity_gates_perf_when_output_matches() {
        // parity Match → perf runs → suite Held
        let env = MockEnv { parity: ParityVerdict::Match };
        let facets: Vec<&dyn DojoFacet<Env = MockEnv>> = vec![&ParityFacet, &PerfFacet];
        let report = run_gated_suite(&facets, &env);
        assert!(matches!(report.verdict, Veredito::Held { .. }));
        assert_eq!(report.outcomes.len(), 2);
        assert_eq!(report.outcomes[1].verdict, Veredito::Held { evidence: Evidence::Message("lever honest".into()) });
    }

    #[test]
    fn parity_divergence_skips_perf_and_falsifies_suite() {
        // parity Differ → perf GATED OFF (never grade a lever on wrong output) → suite Falsified
        let env = MockEnv { parity: ParityVerdict::Differ };
        let facets: Vec<&dyn DojoFacet<Env = MockEnv>> = vec![&ParityFacet, &PerfFacet];
        let report = run_gated_suite(&facets, &env);
        assert!(matches!(report.verdict, Veredito::Falsified { .. }));
        // the perf facet did NOT run (gate closed) → NotApplicable
        assert_eq!(report.outcomes[1].verdict, Veredito::NotApplicable);
    }

    // ── CATALOG REFLECTION: kind ↔ keyword bijection ──────────────────────────
    #[test]
    fn every_facet_kind_has_a_unique_keyword() {
        let mut kws: Vec<&str> = FacetKind::ALL.iter().map(|k| k.keyword()).collect();
        let n = kws.len();
        kws.sort_unstable();
        kws.dedup();
        assert_eq!(kws.len(), n, "facet keywords must be globally unique");
        assert_eq!(n, 10, "the facet population is the bijection anchor");
    }

    // ── Evidence survives serialization (the typed sum is real, not opaque) ───
    #[test]
    fn evidence_serde_round_trips_the_parity_discriminant() {
        let v: Veredito = ParityVerdict::CandTimeout.into();
        let json = serde_json::to_string(&v).unwrap();
        let back: Veredito = serde_json::from_str(&json).unwrap();
        assert_eq!(v, back);
        assert!(json.contains("CandTimeout"), "the discriminant is in the wire form: {json}");
    }
}
