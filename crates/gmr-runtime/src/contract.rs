pub use gmr_core::{
    Binding, Claim, Derivation, Expr, FactAddress, LinkKind, Observes, Openness, Ref, SaidId,
    Source, Verifiability, Version,
};

pub use gmr_content::ContentErrorCode;

pub use crate::bind::Landed;
pub use crate::edges::{Edge, Edges, Raised};
pub use crate::link::{Inbound, Links, Reached};
pub use crate::open::{OpenRequest, Opened, Supersede};
pub use crate::read::{
    AnchorView, Anchored, Asked, Before, Blind, Depends, Evidence, Footing, Grounded, Grounding,
    Holding, Instructions, Knowledge, Linked, MemoryView, SaidView, Sample, Shown, Standing,
    Warrant,
};

pub const CONTRACT: &str = "gmr.contract.v13.0";

pub const SHAPE: &str = "sha256:6f5f2c46385b2ea8245e072ed875bec76dc4ab02616ef5f31833b22842fdc670";
