pub struct Creator();

// fn cal_interim_transcript_hash() {}

struct ProposalMessage;
struct CommitMessage {
    from_proposals: Vec<ProposalMessage>, // At least 1
}

fn cal_transcript_hash() {
    let proposal_one = ProposalMessage;
    let proposal_two = ProposalMessage;

    let commit = CommitMessage {
        from_proposals: vec![proposal_one, proposal_two],
    };
}

fn hkdf_expand() {}
fn hkdf_extract() {}

struct RatchetTree;
struct Extension;
struct IntiGroupState {
    group_id: u32,
    epoch: u32,
    ratchet_tree: RatchetTree,
    tree_hash: String,    //tls serialize of TreeHashInput to hash value
    epoch_secret: String, //Fix size: KDF.Nh
    extensions: Vec<Extension>,
}

#[derive(TlsSerialize, TlsSize)]
struct TreeHashInput {
    node_type: NodeType,
}

enum NodeType {
    #[tls_codec(discriminant = 1)]
    Leaf(LeafNodeHashInput<'a>),
    #[tls_codec(discriminant = 2)]
    Parent(ParentNodeHashInput<'a>),
}

#[derive(TlsSerialize, TlsSize)]
struct ParentNodeHashInput<'a> {
    parent_node: Option<&'a ParentNode>, // Hash of above node: aka parent node
    left_hash: VLByteSlice<'a>,          // Hash of children Node
    right_hash: VLByteSlice<'a>,         // Hash of children Node
}

// step 1: create group with one member
// step 2: send propose and commit message
// step 3: send welcome message to new members
// Welcome message include three parts:
// - Public part: group information and public part of ratchet tree
// - Secret part: joiner secret = current commit secret + previous init secret + allow new members to trigger key schedule.
// - Final part: seed to derive the secrets
//
