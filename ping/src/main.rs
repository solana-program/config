mod cluster;

use {
    crate::cluster::Cluster,
    borsh::BorshDeserialize,
    clap::{Parser, Subcommand},
    kaigan::types::RemainderVec,
    solana_config_program_client::{
        instructions::{Store, StoreInstructionArgs},
        programs::SOLANA_CONFIG_ID,
        ShortVec,
    },
    solana_rpc_client::nonblocking::rpc_client::RpcClient,
    solana_sdk::{
        instruction::AccountMeta,
        signature::Keypair,
        signer::{EncodableKey, Signer},
        system_instruction,
        transaction::Transaction,
    },
};

// Make sure you give this baby some SOL to test.
const KEYPAIR_PATH: &str = "ping/key/payer.json";

#[derive(Subcommand)]
enum SubCommand {
    /// Ping the program with an instruction post-migration.
    Ping {
        /// The cluster on which to run the test.
        cluster: Cluster,
    },
}

#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    pub command: SubCommand,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        SubCommand::Ping { cluster } => {
            let rpc_client = RpcClient::new(cluster.url().to_string());
            let payer = Keypair::read_from_file(KEYPAIR_PATH)?;

            let config_keypair = Keypair::new();
            let random_signer_keypair = Keypair::new();

            let data: &[u8] = &[7u8; 12];

            let size = data.len() + 4 + 32 + 1; // vec len, pubkey, bool

            let recent_blockhash = rpc_client.get_latest_blockhash().await?;
            let minimum_balance = rpc_client
                .get_minimum_balance_for_rent_exemption(size)
                .await?;

            let transaction = Transaction::new_signed_with_payer(
                &[
                    system_instruction::create_account(
                        &payer.pubkey(),
                        &config_keypair.pubkey(),
                        minimum_balance,
                        size as u64,
                        &SOLANA_CONFIG_ID,
                    ),
                    Store {
                        config_account: (config_keypair.pubkey(), true),
                    }
                    .instruction_with_remaining_accounts(
                        StoreInstructionArgs {
                            keys: ShortVec(vec![(random_signer_keypair.pubkey(), true)]),
                            data: RemainderVec::<u8>::try_from_slice(data).unwrap(),
                        },
                        &[AccountMeta::new(random_signer_keypair.pubkey(), true)],
                    ),
                ],
                Some(&payer.pubkey()),
                &[&payer, &config_keypair, &random_signer_keypair],
                recent_blockhash,
            );

            rpc_client
                .send_and_confirm_transaction(&transaction)
                .await?;

            println!("Ping successful!");

            println!("Retrieving config account...");

            let config_account = rpc_client.get_account(&config_keypair.pubkey()).await?;
            println!("Dump of config account: {:?}", config_account);

            Ok(())
        }
    }
}
