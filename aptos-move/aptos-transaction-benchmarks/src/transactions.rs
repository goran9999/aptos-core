// Copyright © Aptos Foundation
// Parts of the project are originally copyright © Meta Platforms, Inc.
// SPDX-License-Identifier: Apache-2.0

use aptos_bitvec::BitVec;
use aptos_crypto::HashValue;
use aptos_language_e2e_tests::{
    account_universe::{AUTransactionGen, AccountUniverseGen},
    data_store::FakeDataStore,
    executor::FakeExecutor,
    gas_costs::TXN_RESERVED,
};
use aptos_types::{
    block_metadata::BlockMetadata,
    on_chain_config::{OnChainConfig, ValidatorSet},
    transaction::Transaction,
};
use aptos_vm::{data_cache::AsMoveResolver, sharded_block_executor::ShardedBlockExecutor};
use criterion::{measurement::Measurement, BatchSize, Bencher};
use proptest::{
    collection::vec,
    strategy::{Strategy, ValueTree},
    test_runner::TestRunner,
};
use std::{fmt::Debug, sync::Arc, time::Instant};

/// Benchmarking support for transactions.
#[derive(Clone)]
pub struct TransactionBencher<S> {
    num_accounts: usize,
    num_transactions: usize,
    strategy: S,
}

impl<S> Debug for TransactionBencher<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TransactionBencher: num_accounts {:?}, num_transactions {:?}",
            self.num_accounts, self.num_transactions
        )
    }
}

impl<S> TransactionBencher<S>
where
    S: Strategy,
    S::Value: AUTransactionGen,
{
    /// The number of accounts created by default.
    pub const DEFAULT_NUM_ACCOUNTS: usize = 100;
    /// The number of transactions created by default.
    pub const DEFAULT_NUM_TRANSACTIONS: usize = 1000;

    /// Creates a new transaction bencher with default settings.
    pub fn new(strategy: S) -> Self {
        Self {
            num_accounts: Self::DEFAULT_NUM_ACCOUNTS,
            num_transactions: Self::DEFAULT_NUM_TRANSACTIONS,
            strategy,
        }
    }

    /// Sets a custom number of accounts.
    pub fn num_accounts(&mut self, num_accounts: usize) -> &mut Self {
        self.num_accounts = num_accounts;
        self
    }

    /// Sets a custom number of transactions.
    pub fn num_transactions(&mut self, num_transactions: usize) -> &mut Self {
        self.num_transactions = num_transactions;
        self
    }

    /// Runs the bencher.
    pub fn bench<M: Measurement>(&self, b: &mut Bencher<M>) {
        b.iter_batched(
            || {
                TransactionBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                    1,
                    num_cpus::get(),
                )
            },
            |state| state.execute_sequential(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }

    /// Runs the bencher.
    pub fn bench_parallel<M: Measurement>(&self, b: &mut Bencher<M>) {
        b.iter_batched(
            || {
                TransactionBenchState::with_size(
                    &self.strategy,
                    self.num_accounts,
                    self.num_transactions,
                    1,
                    num_cpus::get(),
                )
            },
            |state| state.execute_parallel(),
            // The input here is the entire list of signed transactions, so it's pretty large.
            BatchSize::LargeInput,
        )
    }

    /// Runs the bencher.
    pub fn blockstm_benchmark(
        &self,
        num_accounts: usize,
        num_txn: usize,
        run_par: bool,
        run_seq: bool,
        num_warmups: usize,
        num_runs: usize,
        num_executor_shards: usize,
        concurrency_level_per_shard: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut par_tps = Vec::new();
        let mut seq_tps = Vec::new();

        let total_runs = num_warmups + num_runs;
        for i in 0..total_runs {
            let state = TransactionBenchState::with_size(
                &self.strategy,
                num_accounts,
                num_txn,
                num_executor_shards,
                concurrency_level_per_shard,
            );

            if i < num_warmups {
                println!("WARMUP - ignore results");
                state.execute_blockstm_benchmark(run_par, run_seq);
            } else {
                println!(
                    "RUN benchmark for: num_shards {},  concurrency_level_per_shard = {}, \
                        num_account = {}, \
                        block_size = {}",
                    num_executor_shards, concurrency_level_per_shard, num_accounts, num_txn,
                );
                let tps = state.execute_blockstm_benchmark(run_par, run_seq);
                par_tps.push(tps.0);
                seq_tps.push(tps.1);
            }
        }

        (par_tps, seq_tps)
    }
}

struct TransactionBenchState {
    transactions: Vec<Transaction>,
    parallel_block_executor: Arc<ShardedBlockExecutor<FakeDataStore>>,
    sequential_block_executor: Arc<ShardedBlockExecutor<FakeDataStore>>,
}

impl TransactionBenchState {
    /// Creates a new benchmark state with the given number of accounts and transactions.
    fn with_size<S>(
        strategy: S,
        num_accounts: usize,
        num_transactions: usize,
        num_executor_shards: usize,
        concurrency_level_per_shard: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        let mut state = Self::with_universe(
            strategy,
            universe_strategy(num_accounts, num_transactions),
            num_transactions,
            num_executor_shards,
            concurrency_level_per_shard,
        );

        // Insert a blockmetadata transaction at the beginning to better simulate the real life traffic.
        let validator_set = ValidatorSet::fetch_config(
            &FakeExecutor::from_head_genesis()
                .get_state_view()
                .as_move_resolver(),
        )
        .expect("Unable to retrieve the validator set from storage");

        let new_block = BlockMetadata::new(
            HashValue::zero(),
            0,
            0,
            *validator_set.payload().next().unwrap().account_address(),
            BitVec::with_num_bits(validator_set.num_validators() as u16).into(),
            vec![],
            1,
        );

        state
            .transactions
            .insert(0, Transaction::BlockMetadata(new_block));

        state
    }

    /// Creates a new benchmark state with the given account universe strategy and number of
    /// transactions.
    fn with_universe<S>(
        strategy: S,
        universe_strategy: impl Strategy<Value = AccountUniverseGen>,
        num_transactions: usize,
        num_executor_shards: usize,
        concurrency_level_per_shard: usize,
    ) -> Self
    where
        S: Strategy,
        S::Value: AUTransactionGen,
    {
        let mut runner = TestRunner::default();
        let universe = universe_strategy
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();

        let mut executor = FakeExecutor::from_head_genesis();
        // Run in gas-cost-stability mode for now -- this ensures that new accounts are ignored.
        // XXX We may want to include new accounts in case they have interesting performance
        // characteristics.
        let mut universe = universe.setup_gas_cost_stability(&mut executor);

        let transaction_gens = vec(strategy, num_transactions)
            .new_tree(&mut runner)
            .expect("creating a new value should succeed")
            .current();
        let transactions = transaction_gens
            .into_iter()
            .map(|txn_gen| Transaction::UserTransaction(txn_gen.apply(&mut universe).0))
            .collect();

        let state_view = Arc::new(executor.get_state_view().clone());
        let parallel_block_executor = Arc::new(ShardedBlockExecutor::new(
            num_executor_shards,
            Some(concurrency_level_per_shard),
            state_view.clone(),
        ));
        let sequential_block_executor =
            Arc::new(ShardedBlockExecutor::new(1, Some(1), state_view.clone()));

        Self {
            transactions,
            parallel_block_executor,
            sequential_block_executor,
        }
    }

    /// Executes this state in a single block.
    fn execute_sequential(self) {
        // The output is ignored here since we're just testing transaction performance, not trying
        // to assert correctness.
        self.sequential_block_executor
            .execute_block(self.transactions)
            .expect("VM should not fail to start");
    }

    /// Executes this state in a single block.
    fn execute_parallel(self) {
        // The output is ignored here since we're just testing transaction performance, not trying
        // to assert correctness.
        self.parallel_block_executor
            .execute_block(self.transactions)
            .expect("VM should not fail to start");
    }

    fn execute_benchmark(
        &self,
        transactions: Vec<Transaction>,
        block_executor: Arc<ShardedBlockExecutor<FakeDataStore>>,
    ) -> usize {
        let block_size = transactions.len();
        let timer = Instant::now();
        block_executor
            .execute_block(transactions)
            .expect("VM should not fail to start");
        let exec_time = timer.elapsed().as_millis();

        block_size * 1000 / exec_time as usize
    }

    fn execute_blockstm_benchmark(self, run_par: bool, run_seq: bool) -> (usize, usize) {
        let par_tps = if run_par {
            println!("Parallel execution starts...");
            let tps = self.execute_benchmark(
                self.transactions.clone(),
                self.parallel_block_executor.clone(),
            );
            println!("Parallel execution finishes, TPS = {}", tps);
            tps
        } else {
            0
        };
        let seq_tps = if run_seq {
            println!("Sequential execution starts...");
            let tps = self.execute_benchmark(
                self.transactions.clone(),
                self.sequential_block_executor.clone(),
            );
            println!("Sequential execution finishes, TPS = {}", tps);
            tps
        } else {
            0
        };
        (par_tps, seq_tps)
    }
}

/// Returns a strategy for the account universe customized for benchmarks, i.e. having
/// sufficiently large balance for gas.
fn universe_strategy(
    num_accounts: usize,
    num_transactions: usize,
) -> impl Strategy<Value = AccountUniverseGen> {
    let balance = TXN_RESERVED * num_transactions as u64 * 5;
    AccountUniverseGen::strategy(num_accounts, balance..(balance + 1))
}
