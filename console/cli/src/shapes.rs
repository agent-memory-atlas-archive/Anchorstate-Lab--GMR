use crate::error::CliError;

pub use gmr_coding_shapes::{
    ABSENT, ALL, DWELL, Dim, FLUX, MISSING, REPLACED, RETIRED, RETURNED, SETTLED, Shape, axes_of,
    name_of, of, rules, rules_of, transitions_of, watch_of,
};

pub fn get(name: &str) -> Result<&'static Shape, CliError> {
    gmr_coding_shapes::get(name).map_err(CliError)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{COORD_SCHEMA, reads_of, unmet};
    use std::collections::BTreeSet;

    #[test]
    fn every_shape_is_a_program_the_evaluator_accepts() {
        for shape in ALL {
            let transitions = crate::rules::transitions(&rules_of(shape))
                .unwrap_or_else(|e| panic!("shape `{}` does not parse: {e}", shape.name));
            assert_eq!(
                transitions,
                transitions_of(shape),
                "`{}`: the string form and the structured form must be the same table",
                shape.name
            );
            reads_of(&transitions)
                .unwrap_or_else(|e| panic!("shape `{}`'s reads do not parse: {e}", shape.name));
        }
    }

    fn obs(schema: &str, at: &[&str], facts: &[&str]) -> crate::probes::Obs {
        crate::probes::Obs {
            schema: schema.to_owned(),
            at: at.iter().map(|s| s.to_string()).collect(),
            identity: Vec::new(),
            facts: facts.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn reads_of_shape(shape: &Shape) -> BTreeSet<String> {
        reads_of(&transitions_of(shape)).unwrap()
    }

    #[test]
    fn roster_reads_only_report_level_fields() {
        for name in ["roster", "roster-dwell"] {
            assert_eq!(
                reads_of_shape(get(name).unwrap()),
                BTreeSet::from(["found", "candidates", "roll"].map(String::from)),
                "{name}"
            );
        }
    }

    #[test]
    fn roster_rides_any_coord_probe() {
        for name in ["roster", "roster-dwell"] {
            let reads = reads_of_shape(get(name).unwrap());
            assert!(
                unmet(&reads, &obs(COORD_SCHEMA, &[], &[])).is_empty(),
                "{name}"
            );
        }
    }

    #[test]
    fn fingerprint_takes_any_probe_that_names_a_fingerprint() {
        for name in ["fingerprint", "fingerprint-dwell"] {
            let reads = reads_of_shape(get(name).unwrap());
            for at in [
                &["file", "heading", "fingerprint"][..],
                &["path", "name", "fingerprint"][..],
            ] {
                assert!(
                    unmet(&reads, &obs(COORD_SCHEMA, at, &[])).is_empty(),
                    "{name}: {at:?}"
                );
            }
            assert_eq!(
                unmet(
                    &reads,
                    &obs(COORD_SCHEMA, &["name", "scope"], &["occurrences"])
                ),
                vec!["at.fingerprint"],
                "{name}"
            );
        }
    }

    #[test]
    fn an_unknown_shape_names_what_this_build_ships() {
        let e = get("nope").unwrap_err();
        assert!(e.to_string().contains("roster"), "{e}");
    }

    #[test]
    fn a_hand_written_rule_reading_an_undeclared_field_is_caught() {
        let transitions =
            crate::rules::transitions(&["obs.sha != state.sha => { status: \"moved\" }".into()])
                .unwrap();
        let reads = reads_of(&transitions).unwrap();
        let script_probe = obs("gmr.probe.v1", &[], &["pending"]);
        assert_eq!(unmet(&reads, &script_probe), vec!["sha"]);
    }

    #[test]
    fn contract_rides_ast_map() {
        for name in ["contract", "contract-dwell"] {
            let reads = reads_of_shape(get(name).unwrap());
            let ast_map = obs(
                COORD_SCHEMA,
                &[
                    "file", "kind", "form", "vis", "surface", "after", "name", "shape",
                ],
                &["body", "line"],
            );
            assert!(unmet(&reads, &ast_map).is_empty(), "{name}");

            let name_map = obs(COORD_SCHEMA, &["name", "scope"], &["occurrences"]);
            assert_eq!(
                unmet(&reads, &name_map),
                [
                    "at.after",
                    "at.file",
                    "at.form",
                    "at.shape",
                    "at.surface",
                    "facts.body"
                ],
                "{name}"
            );
        }
    }

    #[test]
    fn a_flat_probes_facts_are_known_at_the_top_level() {
        let transitions =
            crate::rules::transitions(&["obs.pending > 0 => { status: \"unapplied\" }".into()])
                .unwrap();
        let reads = reads_of(&transitions).unwrap();
        let script_probe = obs("gmr.probe.v1", &[], &["pending"]);
        assert!(unmet(&reads, &script_probe).is_empty());
    }
}
