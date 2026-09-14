use std::sync::Arc;

use gmr_core::{
    AnchorKey, ContentHash, Expr, Footprint, Recorded, Rests, Retain, Rule, RunSettings, StatePath,
    Transitions,
};
use gmr_runtime::{Cited, OpenRequest, Runtime, Stands, Why};
use gmr_store::testkit::{MemoryBindings, MemoryJournal, MemoryQueue};
use gmr_transport::shell::Shell;

struct World {
    dir: tempfile::TempDir,
    rt: Runtime,
}

impl World {
    fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let bindings = Arc::new(MemoryBindings::default());
        let rt = Runtime::builder()
            .transport(Arc::new(Shell::new(dir.path(), dir.path().join(".probes"))))
            .store(Arc::new(MemoryJournal::default()))
            .bindings(bindings.clone())
            .sealer(bindings.clone())
            .links(bindings)
            .settings(Arc::new(MemoryQueue::default()))
            .sightings(Arc::new(MemoryQueue::default()))
            .build();
        Self { dir, rt }
    }

    fn write(&self, contents: &str) {
        std::fs::write(self.dir.path().join("world.json"), contents).unwrap();
    }

    fn blind(&self) {
        std::fs::remove_file(self.dir.path().join("world.json")).unwrap();
    }

    async fn open(&self) {
        let probe = gmr_transport::shell::testkit::install_script(
            self.dir.path().join(".probes"),
            "cat",
            "cat world.json",
        );
        self.rt
            .open(OpenRequest {
                key: key(),
                probe,
                transitions: Transitions(vec![Rule {
                    when: Expr::text("true"),
                    to: Expr::text("{ now: { sig: obs.sig, place: obs.place } }"),
                }]),
                terminal: Default::default(),
                initial: None,
                settings: RunSettings {
                    facts: Recorded::Plain,
                    budget_ms: None,
                    retain: Retain::Tick,
                    cadence_secs: None,
                },
                supersedes: None,
            })
            .await
            .unwrap();
    }

    async fn showing(&self) -> gmr_core::FactAddress {
        self.rt
            .read(&key())
            .await
            .unwrap()
            .fact_address
            .expect("an opened anchor is showing a reading")
    }

    async fn hash_at(&self, path: &str) -> ContentHash {
        self.rt
            .read(&key())
            .await
            .unwrap()
            .state
            .hash_at(&spelled(path))
            .expect("the shape puts a value on that path")
    }
}

fn key() -> AnchorKey {
    AnchorKey::new("a")
}

fn spelled(path: &str) -> StatePath {
    StatePath::try_new(path).unwrap()
}

fn resting(address: &gmr_core::FactAddress, paths: Vec<Footprint>) -> Rests {
    Rests {
        anchor: key(),
        address: address.clone(),
        paths,
    }
}

#[tokio::test]
async fn an_address_the_read_entry_issued_hands_back_the_value_and_the_instrument() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;

    let address = w.showing().await;
    let Cited {
        found,
        facts,
        looks,
        ..
    } = w.rt.reading(&address).await.unwrap();

    assert!(found);
    assert_eq!(
        facts.map(|f| f.as_value().clone()),
        Some(serde_json::json!({ "sig": "fn a()", "place": 10 })),
        "an address that cannot produce the value it names is half a citation"
    );
    assert_eq!(looks.len(), 1);
    assert_eq!(looks[0].anchor, key());
}

#[tokio::test]
async fn an_address_nobody_issued_is_refused_rather_than_answered_with_something_near_it() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;

    let never = gmr_core::FactAddress::try_new("c".repeat(64)).unwrap();
    let refused = w.rt.reading(&never).await.expect_err("nobody issued that");
    assert_eq!(refused.code(), "no_such_reading");
}

#[tokio::test]
async fn one_value_read_twice_is_one_reading_and_two_looks() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;
    let address = w.showing().await;

    w.rt.observe(&key()).await.unwrap();

    let cited = w.rt.reading(&address).await.unwrap();
    assert_eq!(
        cited.looks.len(),
        2,
        "the second read saw the same value, so it wrote no journal entry — but it \
         happened, and the record of a read that happened is the whole of traceability"
    );
    assert_eq!(cited.address, address);
}

#[tokio::test]
async fn a_path_that_did_not_move_still_stands_when_another_path_did() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;

    let address = w.showing().await;
    let sig = Footprint {
        path: spelled("now.sig"),
        hash: w.hash_at("now.sig").await,
    };
    let place = Footprint {
        path: spelled("now.place"),
        hash: w.hash_at("now.place").await,
    };

    w.write(r#"{"sig":"fn a()","place":44}"#);
    w.rt.observe(&key()).await.unwrap();

    let held =
        w.rt.stands(&[
            resting(&address, vec![sig]),
            resting(&address, vec![place]),
            resting(&address, vec![]),
        ])
        .await
        .unwrap();

    assert_eq!(
        held[0].stands,
        Stands::Holds,
        "an edit elsewhere in the file must not stale a memory about the signature"
    );
    match &held[1].stands {
        Stands::Moved { drifted } => {
            assert_eq!(drifted[0].path, spelled("now.place"));
            assert_eq!(drifted[0].why, Why::Value);
        }
        other => panic!("the path that moved must say so: {other:?}"),
    }
    assert!(
        matches!(held[2].stands, Stands::Superseded { .. }),
        "and a citation that named no path rests on the whole reading, so it moved"
    );
}

#[tokio::test]
async fn standing_is_answered_from_the_log_even_when_the_world_is_gone() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;

    let address = w.showing().await;
    let sig = Footprint {
        path: spelled("now.sig"),
        hash: w.hash_at("now.sig").await,
    };

    w.blind();

    let held = w.rt.stands(&[resting(&address, vec![sig])]).await.unwrap();
    assert_eq!(
        held[0].stands,
        Stands::Holds,
        "a probe run here would have failed outright; stands asks the log what it \
         recorded, not the world what it holds now, and that is why it costs nothing"
    );
}

#[tokio::test]
async fn a_path_the_shape_no_longer_carries_is_absent_rather_than_changed() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;
    let address = w.showing().await;

    let held =
        w.rt.stands(&[resting(
            &address,
            vec![Footprint {
                path: spelled("now.surface"),
                hash: w.hash_at("now.sig").await,
            }],
        )])
        .await
        .unwrap();

    match &held[0].stands {
        Stands::Moved { drifted } => assert_eq!(drifted[0].why, Why::Absent),
        other => panic!("a path that is not there is absent: {other:?}"),
    }
}

#[tokio::test]
async fn an_anchor_nobody_opened_says_so_instead_of_claiming_the_citation_holds() {
    let w = World::new();
    w.write(r#"{"sig":"fn a()","place":10}"#);
    w.open().await;
    let address = w.showing().await;

    let held =
        w.rt.stands(&[Rests {
            anchor: AnchorKey::new("never-opened"),
            address,
            paths: vec![],
        }])
        .await
        .unwrap();
    assert_eq!(held[0].stands, Stands::Unopened);
}
