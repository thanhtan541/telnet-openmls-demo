use std::sync::{Arc, Mutex};

use openmls::{
    group::{GroupId, MlsGroup, MlsGroupCreateConfig, StagedWelcome},
    prelude::{
        test_utils::new_credential, BasicCredential, Ciphersuite, CredentialWithKey, KeyPackage,
        MlsMessageIn, ProcessedMessageContent,
    },
};
use openmls_basic_credential::SignatureKeyPair;
use openmls_traits::OpenMlsProvider;
use uuid::Uuid;

use super::memory_provider::MemoryProvider;

pub const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_CHACHA20POLY1305_SHA256_Ed25519;
pub const MAX_PAST_EPOCHS: usize = 100;

#[derive(Debug)]
pub struct Member {
    pub provider: MemoryProvider,
    pub credential_with_key: CredentialWithKey,
    pub signer: SignatureKeyPair,
    pub group_id: GroupId,
    pub user_id: String,
}

impl Member {
    fn client_id(&self) -> &str {
        &self.user_id.as_str()
    }
}

impl Member {
    pub fn new(name: &str, user_id: String, group_id: GroupId) -> Member {
        let identity = format!("{}_{}", name, user_id);
        let creator_provider = MemoryProvider::default();
        // signature_key in credential_with_key is same as keypair.public
        // credential could be
        // Basic: Just unique identity with key_pair.public_key for signature's verification
        // X509: attach CA certificate for signature verification
        let (credential_with_key, creator_signer) = new_credential(
            &creator_provider,
            identity.as_bytes(),
            CIPHERSUITE.signature_algorithm(),
        );

        Self {
            provider: creator_provider,
            credential_with_key,
            signer: creator_signer,
            group_id,
            user_id,
        }
    }

    pub fn basic_credential(&self) -> BasicCredential {
        let client_id = &self.client_id().clone().as_bytes().to_vec();

        BasicCredential::new(client_id.to_owned())
    }
}

fn process_commit(
    group: &mut MlsGroup,
    provider: &MemoryProvider,
    commit: openmls::prelude::MlsMessageOut,
) {
    let processed_message = group
        .process_message(provider, commit.into_protocol_message().unwrap())
        .unwrap();

    if let ProcessedMessageContent::StagedCommitMessage(staged_commit) =
        processed_message.into_content()
    {
        group.merge_staged_commit(provider, *staged_commit).unwrap();
    } else {
        unreachable!("Expected a StagedCommit.");
    }
}

pub fn create_group(
    creator_name: String,
    group_name: &str,
    group_config: &MlsGroupCreateConfig,
) -> (MlsGroup, Member) {
    let group_id = GroupId::from_slice(group_name.as_bytes());
    let user_id = Uuid::new_v4().to_string();
    let creator_member = Member::new(&creator_name, user_id, group_id);

    let creator_group = MlsGroup::new_with_group_id(
        &creator_member.provider,
        &creator_member.signer,
        group_config,
        GroupId::from_slice(group_name.as_bytes()),
        creator_member.credential_with_key.clone(),
    )
    .expect("An unexpected error occurred.");

    (creator_group, creator_member)
}

fn new_member(
    name: &str,
) -> (
    MemoryProvider,
    SignatureKeyPair,
    CredentialWithKey,
    openmls::prelude::KeyPackageBundle,
) {
    let identity = format!("{}_{}", name, Uuid::new_v4());
    let member_provider = MemoryProvider::default();
    let (credential_with_key, signer) = new_credential(
        &member_provider,
        identity.as_bytes(),
        CIPHERSUITE.signature_algorithm(),
    );
    let key_package = KeyPackage::builder()
        .build(
            CIPHERSUITE,
            &member_provider,
            &signer,
            credential_with_key.clone(),
        )
        .expect("An unexpected error occurred.");
    (member_provider, signer, credential_with_key, key_package)
}

/// Convenience function that generates a Mls group based on
/// num: number of expected members in a group which
/// creator is always at the index 0 - creator = members[0]
/// The vec's element is a tuple of local_group and its local member
/// The steps is sequencial add one member after one.
///
/// Returns the [`MlsGroup`] and the [`Member`].
pub fn setup_group(group_name: &str, num: usize) -> Vec<(MlsGroup, Member)> {
    let mut members = Vec::new();
    let mls_group_create_config = create_group_config();
    let creator_name = "Member_0"; // Creator is always at index 0
    let (creator_group, creator_member) = create_group(
        creator_name.to_string(),
        group_name,
        &mls_group_create_config,
    );
    members.push((creator_group, creator_member));

    for member_i in 1..num {
        let (member_provider, signer, credential_with_key, key_package) =
            new_member(&format!("Member_{member_i}"));

        let creator = &mut members[0];
        let creator_group = &mut creator.0;
        let creator_provider = &creator.1.provider;
        let creator_signer = &creator.1.signer;

        let (commit, welcome, _) = creator_group
            .add_members(
                creator_provider,
                creator_signer,
                &[key_package.key_package().clone()],
            )
            .unwrap();

        creator_group
            .merge_pending_commit(creator_provider)
            .expect("error merging pending commit");

        let welcome: MlsMessageIn = welcome.into();
        let welcome = welcome
            .into_welcome()
            .expect("expected the message to be a welcome message");

        let member_i_group = StagedWelcome::new_from_welcome(
            &member_provider,
            mls_group_create_config.join_config(),
            welcome,
            Some(creator_group.export_ratchet_tree().into()),
        )
        .unwrap()
        .into_group(&member_provider)
        .unwrap();

        // Merge commit on all other members
        for (group, member) in members.iter_mut().skip(1) {
            process_commit(group, &member.provider, commit.clone());
        }

        let group_id = member_i_group.group_id().clone();
        // Add new member to list
        members.push((
            member_i_group,
            Member {
                // FIXME: each member is a user with one client
                user_id: Uuid::new_v4().to_string(),
                provider: member_provider,
                credential_with_key,
                signer,
                group_id,
            },
        ));
    }

    members
}

pub fn create_group_config() -> MlsGroupCreateConfig {
    MlsGroupCreateConfig::builder()
        .use_ratchet_tree_extension(true)
        .ciphersuite(CIPHERSUITE)
        .max_past_epochs(MAX_PAST_EPOCHS)
        .build()
}

pub fn show_key_package_details() {
    println!("[Client] KeyPackage has two properties: payload and signature");
    println!("[Client] Similar to Jwt");
    println!("[Client] Signature is the signature of the payload, provide the authenticity of the payload");
    println!("[Client] Caveat: Each Keypackage is used once to create a group");
    println!("[Client]      Client should publish multiple keypackages to create multiple groups");
    println!("[Client]      Delivery Service manages the pool of keypackages");
}

pub fn generate_key_package(
    provider: &impl OpenMlsProvider,
    identity: &str,
) -> openmls::prelude::KeyPackageBundle {
    let (credential_with_key, signer) = new_credential(
        provider,
        identity.as_bytes(),
        CIPHERSUITE.signature_algorithm(),
    );

    KeyPackage::builder()
        .build(CIPHERSUITE, provider, &signer, credential_with_key.clone())
        .expect("An unexpected error occurred.")
}

struct ProviderPool {
    providers: Vec<Arc<Mutex<MemoryProvider>>>,
}

impl ProviderPool {
    // Create a new pool with a specified number of providers
    fn new(size: usize) -> Self {
        let providers = Vec::with_capacity(size);
        ProviderPool { providers }
    }

    // Get a provider from the pool (round-robin style for simplicity)
    async fn get_provider(&self, index: usize) -> Arc<Mutex<MemoryProvider>> {
        self.providers[index].clone()
    }
}
