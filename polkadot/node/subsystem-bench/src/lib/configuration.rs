// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Test configuration definition and helpers.

use crate::keyring::Keyring;
use itertools::Itertools;
use polkadot_node_network_protocol::authority_discovery::AuthorityDiscovery;
use polkadot_primitives::{AssignmentId, AuthorityDiscoveryId, ValidatorId, ValidatorPair};
use rand::thread_rng;
use rand_distr::{Distribution, Normal, Uniform};
use sc_network::Multiaddr;
use sc_network_types::PeerId;
use serde::{Deserialize, Serialize};
use sp_consensus_babe::AuthorityId;
use sp_core::Pair;
use std::collections::{HashMap, HashSet};

/// Peer networking latency configuration.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PeerLatency {
	/// The mean latency(milliseconds) of the peers.
	pub mean_latency_ms: usize,
	/// The standard deviation
	pub std_dev: f64,
}

// Based on Kusama `max_validators`
fn default_n_validators() -> usize {
	300
}

// Based on Kusama cores
fn default_n_cores() -> usize {
	60
}

// Default PoV size in KiB.
fn default_pov_size() -> usize {
	5 * 1024
}

// Default bandwidth in bytes, based stats from Kusama validators
fn default_bandwidth() -> usize {
	42 * 1024 * 1024
}

// Default peer latency
fn default_peer_latency() -> Option<PeerLatency> {
	Some(PeerLatency { mean_latency_ms: 30, std_dev: 2.0 })
}

// Default connectivity percentage
fn default_connectivity() -> usize {
	90
}

// Default backing group size
fn default_backing_group_size() -> usize {
	5
}

// Default needed approvals
fn default_needed_approvals() -> usize {
	30
}

fn default_zeroth_delay_tranche_width() -> usize {
	0
}

fn default_relay_vrf_modulo_samples() -> usize {
	6
}

fn default_n_delay_tranches() -> usize {
	89
}
fn default_no_show_slots() -> usize {
	3
}
fn default_minimum_backing_votes() -> u32 {
	2
}
fn default_max_candidate_depth() -> u32 {
	3
}
fn default_allowed_ancestry_len() -> u32 {
	2
}

/// The test input parameters
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestConfiguration {
	/// Number of validators
	#[serde(default = "default_n_validators")]
	pub n_validators: usize,
	/// Number of cores
	#[serde(default = "default_n_cores")]
	pub n_cores: usize,
	/// The number of needed votes to approve a candidate.
	#[serde(default = "default_needed_approvals")]
	pub needed_approvals: usize,
	#[serde(default = "default_zeroth_delay_tranche_width")]
	pub zeroth_delay_tranche_width: usize,
	#[serde(default = "default_relay_vrf_modulo_samples")]
	pub relay_vrf_modulo_samples: usize,
	#[serde(default = "default_n_delay_tranches")]
	pub n_delay_tranches: usize,
	#[serde(default = "default_no_show_slots")]
	pub no_show_slots: usize,
	/// Maximum backing group size
	#[serde(default = "default_backing_group_size")]
	pub max_validators_per_core: usize,
	/// The min PoV size
	#[serde(default = "default_pov_size")]
	pub min_pov_size: usize,
	/// The max PoV size,
	#[serde(default = "default_pov_size")]
	pub max_pov_size: usize,
	/// Randomly sampled pov_sizes
	#[serde(skip)]
	pub pov_sizes: Vec<usize>,
	/// The amount of bandwidth remote validators have.
	#[serde(default = "default_bandwidth")]
	pub peer_bandwidth: usize,
	/// The amount of bandwidth our node has.
	#[serde(default = "default_bandwidth")]
	pub bandwidth: usize,
	/// Optional peer emulation latency (round trip time) wrt node under test
	#[serde(default = "default_peer_latency")]
	pub latency: Option<PeerLatency>,
	/// Connectivity ratio, the percentage of peers we are connected to, but as part of the
	/// topology.
	#[serde(default = "default_connectivity")]
	pub connectivity: usize,
	/// Number of blocks to run the test for
	pub num_blocks: usize,
	/// Number of minimum backing votes
	#[serde(default = "default_minimum_backing_votes")]
	pub minimum_backing_votes: u32,
	/// Async Backing max_candidate_depth
	#[serde(default = "default_max_candidate_depth")]
	pub max_candidate_depth: u32,
	/// Async Backing allowed_ancestry_len
	#[serde(default = "default_allowed_ancestry_len")]
	pub allowed_ancestry_len: u32,
}

impl Default for TestConfiguration {
	fn default() -> Self {
		Self {
			n_validators: default_n_validators(),
			n_cores: default_n_cores(),
			needed_approvals: default_needed_approvals(),
			zeroth_delay_tranche_width: default_zeroth_delay_tranche_width(),
			relay_vrf_modulo_samples: default_relay_vrf_modulo_samples(),
			n_delay_tranches: default_n_delay_tranches(),
			no_show_slots: default_no_show_slots(),
			max_validators_per_core: default_backing_group_size(),
			min_pov_size: default_pov_size(),
			max_pov_size: default_pov_size(),
			pov_sizes: Default::default(),
			peer_bandwidth: default_bandwidth(),
			bandwidth: default_bandwidth(),
			latency: default_peer_latency(),
			connectivity: default_connectivity(),
			num_blocks: Default::default(),
			minimum_backing_votes: default_minimum_backing_votes(),
			max_candidate_depth: default_max_candidate_depth(),
			allowed_ancestry_len: default_allowed_ancestry_len(),
		}
	}
}

impl TestConfiguration {
	pub fn generate_pov_sizes(&mut self) {
		self.pov_sizes = generate_pov_sizes(self.n_cores, self.min_pov_size, self.max_pov_size);
	}

	pub fn pov_sizes(&self) -> &[usize] {
		&self.pov_sizes
	}
	/// Return the number of peers connected to our node.
	pub fn connected_count(&self) -> usize {
		((self.n_validators - 1) as f64 / (100.0 / self.connectivity as f64)) as usize
	}

	/// Generates the authority keys we need for the network emulation.
	pub fn generate_authorities(&self) -> TestAuthorities {
		let keyring = Keyring::default();

		let key_seeds = (0..self.n_validators)
			.map(|peer_index| format!("//Node{peer_index}"))
			.collect_vec();

		let keys = key_seeds
			.iter()
			.map(|seed| keyring.sr25519_new(seed.as_str()))
			.collect::<Vec<_>>();

		// Generate keys and peers ids in each of the format needed by the tests.
		let validator_public: Vec<ValidatorId> =
			keys.iter().map(|key| (*key).into()).collect::<Vec<_>>();

		let validator_authority_id: Vec<AuthorityDiscoveryId> =
			keys.iter().map(|key| (*key).into()).collect::<Vec<_>>();

		let validator_babe_id: Vec<AuthorityId> =
			keys.iter().map(|key| (*key).into()).collect::<Vec<_>>();

		let validator_assignment_id: Vec<AssignmentId> =
			keys.iter().map(|key| (*key).into()).collect::<Vec<_>>();
		let peer_ids: Vec<PeerId> = keys.iter().map(|_| PeerId::random()).collect::<Vec<_>>();

		let peer_id_to_authority = peer_ids
			.iter()
			.zip(validator_authority_id.iter())
			.map(|(peer_id, authority_id)| (*peer_id, authority_id.clone()))
			.collect();

		let validator_pairs = key_seeds
			.iter()
			.map(|seed| ValidatorPair::from_string_with_seed(seed, None).unwrap().0)
			.collect();

		TestAuthorities {
			keyring,
			validator_public,
			validator_authority_id,
			peer_ids,
			validator_babe_id,
			validator_assignment_id,
			key_seeds,
			peer_id_to_authority,
			validator_pairs,
		}
	}
}

fn random_uniform_sample<T: Into<usize> + From<usize>>(min_value: T, max_value: T) -> T {
	Uniform::from(min_value.into()..=max_value.into())
		.sample(&mut thread_rng())
		.into()
}

fn random_pov_size(min_pov_size: usize, max_pov_size: usize) -> usize {
	random_uniform_sample(min_pov_size, max_pov_size)
}

fn generate_pov_sizes(count: usize, min_kib: usize, max_kib: usize) -> Vec<usize> {
	(0..count).map(|_| random_pov_size(min_kib * 1024, max_kib * 1024)).collect()
}

/// Helper struct for authority related state.
#[derive(Clone)]
pub struct TestAuthorities {
	pub keyring: Keyring,
	pub validator_public: Vec<ValidatorId>,
	pub validator_authority_id: Vec<AuthorityDiscoveryId>,
	pub validator_babe_id: Vec<AuthorityId>,
	pub validator_assignment_id: Vec<AssignmentId>,
	pub key_seeds: Vec<String>,
	pub peer_ids: Vec<PeerId>,
	pub peer_id_to_authority: HashMap<PeerId, AuthorityDiscoveryId>,
	pub validator_pairs: Vec<ValidatorPair>,
}

impl std::fmt::Debug for TestAuthorities {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "TestAuthorities")
	}
}

/// Sample latency (in milliseconds) from a normal distribution with parameters
/// specified in `maybe_peer_latency`.
pub fn random_latency(maybe_peer_latency: Option<&PeerLatency>) -> usize {
	maybe_peer_latency
		.map(|latency_config| {
			Normal::new(latency_config.mean_latency_ms as f64, latency_config.std_dev)
				.expect("normal distribution parameters are good")
				.sample(&mut thread_rng())
		})
		.unwrap_or(0.0) as usize
}

#[async_trait::async_trait]
impl AuthorityDiscovery for TestAuthorities {
	/// Get the addresses for the given [`AuthorityDiscoveryId`] from the local address cache.
	async fn get_addresses_by_authority_id(
		&mut self,
		_authority: AuthorityDiscoveryId,
	) -> Option<HashSet<Multiaddr>> {
		None
	}

	/// Get the [`AuthorityDiscoveryId`] for the given [`PeerId`] from the local address cache.
	async fn get_authority_ids_by_peer_id(
		&mut self,
		peer_id: PeerId,
	) -> Option<HashSet<AuthorityDiscoveryId>> {
		self.peer_id_to_authority.get(&peer_id).cloned().map(|id| HashSet::from([id]))
	}
}
