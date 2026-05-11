//! SolGrid demo client CLI.
//!
//! Wires the off-chain scheduler API together with the on-chain Anchor
//! programs so a presenter can run a complete demo from a terminal:
//!
//!   $ sol-client keygen              # create demo client wallet
//!   $ sol-client airdrop             # devnet SOL
//!   $ sol-client setup-mint          # create tSIMD mint + ATA
//!   $ sol-client mint-to 1000000     # mint 1M tSIMD to self
//!   $ sol-client create-job --budget 50000
//!
//! Each command prints the Solana Explorer link for any transaction it
//! submits so the audience can verify settlement on-chain.

use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    native_token::LAMPORTS_PER_SOL,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{read_keypair_file, write_keypair_file, Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{
    get_associated_token_address,
    instruction::create_associated_token_account_idempotent,
};
use spl_token::{instruction as token_ix, state::Mint};

use sol_common::{AppConfig, AttachEscrowTxRequest, CreateJobRequest};
use sol_onchain::{ids::ProgramIds, ix};

#[derive(Parser)]
#[command(name = "sol-client", about = "SolGrid demo client CLI")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Generate or print the demo client wallet.
    Keygen {
        /// Path to read/write the keypair. Defaults to ./data/client-keypair.json.
        #[arg(long)]
        path: Option<PathBuf>,
        /// Overwrite if a key already exists at `path`.
        #[arg(long)]
        force: bool,
    },
    /// Request a SOL airdrop on devnet.
    Airdrop {
        #[arg(long, default_value_t = 1.0)]
        amount_sol: f64,
    },
    /// Print balances for the client wallet (SOL + tSIMD).
    Balance,
    /// Create a fresh SPL mint to use as tSIMD. Saves the mint keypair to
    /// ./data/mint-keypair.json and prints the address to stdout.
    SetupMint {
        #[arg(long, default_value_t = 6)]
        decimals: u8,
    },
    /// Mint tokens of the configured mint to the client wallet.
    MintTo {
        /// Token amount in *base units* (not adjusted for decimals).
        amount: u64,
    },
    /// Create a new compute job: POST /jobs, then create_escrow on-chain.
    CreateJob {
        #[arg(long, default_value = "A100")]
        gpu_class: String,
        #[arg(long, default_value_t = 4)]
        gpu_count: u8,
        #[arg(long, default_value_t = 120)]
        max_duration_sec: u32,
        /// Escrow amount in base units of the mint.
        #[arg(long, default_value_t = 50_000)]
        budget_scu: u64,
    },
    /// Print the status of an existing job.
    Status { job_id: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("sol_client=info,warn")),
        )
        .init();

    let cli = Cli::parse();
    let cfg = AppConfig::from_env()?;

    match cli.cmd {
        Cmd::Keygen { path, force } => keygen(path, force),
        Cmd::Airdrop { amount_sol } => airdrop(&cfg, amount_sol).await,
        Cmd::Balance => balance(&cfg).await,
        Cmd::SetupMint { decimals } => setup_mint(&cfg, decimals).await,
        Cmd::MintTo { amount } => mint_to(&cfg, amount).await,
        Cmd::CreateJob {
            gpu_class,
            gpu_count,
            max_duration_sec,
            budget_scu,
        } => create_job(&cfg, &gpu_class, gpu_count, max_duration_sec, budget_scu).await,
        Cmd::Status { job_id } => status(&cfg, &job_id).await,
    }
}

// ───────────────────────── commands ───────────────────────────────────

fn keygen(path: Option<PathBuf>, force: bool) -> Result<()> {
    let path = path.unwrap_or_else(|| PathBuf::from("./data/client-keypair.json"));
    if path.exists() && !force {
        let kp = read_keypair_file(&path)
            .map_err(|e| anyhow!("read existing keypair {}: {}", path.display(), e))?;
        println!("client pubkey: {}", kp.pubkey());
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let kp = Keypair::new();
    write_keypair_file(&kp, &path)
        .map_err(|e| anyhow!("write keypair {}: {}", path.display(), e))?;
    println!("client pubkey: {}", kp.pubkey());
    println!("written to:    {}", path.display());
    Ok(())
}

async fn airdrop(cfg: &AppConfig, amount_sol: f64) -> Result<()> {
    let (rpc, client) = rpc_and_client(cfg)?;
    let lamports = (amount_sol * LAMPORTS_PER_SOL as f64) as u64;
    let sig = rpc.request_airdrop(&client.pubkey(), lamports).await?;
    println!("airdrop tx: {}", sig);
    println!("explorer:   {}", explorer_url(cfg, &sig.to_string()));
    Ok(())
}

async fn balance(cfg: &AppConfig) -> Result<()> {
    let (rpc, client) = rpc_and_client(cfg)?;
    let sol = rpc.get_balance(&client.pubkey()).await?;
    println!(
        "{}: {:.4} SOL ({} lamports)",
        client.pubkey(),
        sol as f64 / LAMPORTS_PER_SOL as f64,
        sol
    );

    if let Ok(mint) = Pubkey::from_str(&cfg.grid_token_mint) {
        let ata = get_associated_token_address(&client.pubkey(), &mint);
        match rpc.get_token_account_balance(&ata).await {
            Ok(b) => println!("tSIMD ({}): {} (raw: {})", ata, b.ui_amount_string, b.amount),
            Err(_) => println!("tSIMD ({}): no token account yet", ata),
        }
    }
    Ok(())
}

async fn setup_mint(cfg: &AppConfig, decimals: u8) -> Result<()> {
    let (rpc, client) = rpc_and_client(cfg)?;

    let mint_keypair = Keypair::new();
    let mint_pubkey = mint_keypair.pubkey();
    let mint_path = PathBuf::from("./data/mint-keypair.json");
    if let Some(parent) = mint_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    write_keypair_file(&mint_keypair, &mint_path)
        .map_err(|e| anyhow!("write mint keypair: {}", e))?;

    let rent = rpc
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .await?;

    let create_account_ix = system_instruction::create_account(
        &client.pubkey(),
        &mint_pubkey,
        rent,
        Mint::LEN as u64,
        &spl_token::ID,
    );
    let init_mint_ix = token_ix::initialize_mint2(
        &spl_token::ID,
        &mint_pubkey,
        &client.pubkey(),
        Some(&client.pubkey()),
        decimals,
    )?;

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_account_ix, init_mint_ix],
        Some(&client.pubkey()),
        &[&client, &mint_keypair],
        blockhash,
    );
    let sig = rpc.send_and_confirm_transaction(&tx).await?;

    println!("mint:        {}", mint_pubkey);
    println!("mint kp:     {}", mint_path.display());
    println!("decimals:    {}", decimals);
    println!("init tx:     {}", sig);
    println!("explorer:    {}", explorer_url(cfg, &sig.to_string()));
    println!();
    println!("Update your .env:");
    println!("  GRID_TOKEN_MINT={}", mint_pubkey);
    println!("  GRID_TOKEN_DECIMALS={}", decimals);
    Ok(())
}

async fn mint_to(cfg: &AppConfig, amount: u64) -> Result<()> {
    let (rpc, client) = rpc_and_client(cfg)?;
    let mint = Pubkey::from_str(&cfg.grid_token_mint)
        .context("GRID_TOKEN_MINT not a valid pubkey — run setup-mint first")?;

    let ata = get_associated_token_address(&client.pubkey(), &mint);

    let create_ata = create_associated_token_account_idempotent(
        &client.pubkey(),
        &client.pubkey(),
        &mint,
        &spl_token::ID,
    );
    let mint_ix = token_ix::mint_to(
        &spl_token::ID,
        &mint,
        &ata,
        &client.pubkey(),
        &[&client.pubkey()],
        amount,
    )?;

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[create_ata, mint_ix],
        Some(&client.pubkey()),
        &[&client],
        blockhash,
    );
    let sig = rpc.send_and_confirm_transaction(&tx).await?;

    println!("minted:   {} → {}", amount, ata);
    println!("tx:       {}", sig);
    println!("explorer: {}", explorer_url(cfg, &sig.to_string()));
    Ok(())
}

async fn create_job(
    cfg: &AppConfig,
    gpu_class: &str,
    gpu_count: u8,
    max_duration_sec: u32,
    budget_scu: u64,
) -> Result<()> {
    let (rpc, client) = rpc_and_client(cfg)?;
    let mint = Pubkey::from_str(&cfg.grid_token_mint)
        .context("GRID_TOKEN_MINT invalid — run setup-mint first")?;
    let ids = program_ids(cfg)?;
    let http = reqwest::Client::new();

    // 1. POST /jobs to scheduler to get a job_id.
    let req = CreateJobRequest {
        client_pubkey: client.pubkey().to_string(),
        gpu_class: gpu_class.to_string(),
        gpu_count: gpu_count as i16,
        max_duration_sec: max_duration_sec as i32,
        budget_scu: budget_scu as i64,
    };
    let resp = http
        .post(format!("{}/api/v1/jobs", cfg.scheduler_url))
        .json(&req)
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "POST /jobs failed ({}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }
    let job: serde_json::Value = resp.json().await?;
    let job_id_str = job["id"]
        .as_str()
        .ok_or_else(|| anyhow!("response missing id"))?;
    let job_uuid: uuid::Uuid = job_id_str.parse()?;
    println!("job created: {}", job_uuid);

    // 2. Build & submit create_escrow on-chain.
    let job_id_bytes = sol_onchain::pda::job_id_from_uuid_bytes(job_uuid.as_bytes());
    // The provider account is needed by the program but isn't a signer; on
    // create_escrow we don't know which provider will be assigned yet, so we
    // use the client's own pubkey as a placeholder — it's stored on-chain
    // but never re-checked against the receipt.
    let placeholder_provider = client.pubkey();

    let escrow_ix = ix::create_escrow_ix(
        &ids,
        &client.pubkey(),
        &placeholder_provider,
        &mint,
        job_id_bytes,
        budget_scu,
    )?;

    let blockhash = rpc.get_latest_blockhash().await?;
    let tx = Transaction::new_signed_with_payer(
        &[escrow_ix],
        Some(&client.pubkey()),
        &[&client],
        blockhash,
    );
    let sig = rpc.send_and_confirm_transaction(&tx).await?;
    println!("create_escrow tx: {}", sig);
    println!("explorer:         {}", explorer_url(cfg, &sig.to_string()));

    // 3. Attach the on-chain tx hash back to the job.
    let resp = http
        .post(format!(
            "{}/api/v1/jobs/{}/escrow",
            cfg.scheduler_url, job_uuid
        ))
        .json(&AttachEscrowTxRequest {
            escrow_tx: sig.to_string(),
        })
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!(
            "attaching escrow tx failed ({}): {}",
            resp.status(),
            resp.text().await.unwrap_or_default()
        ));
    }

    println!("\nJob {} funded and ready for assignment.", job_uuid);
    println!("Scheduler will pick it up within ~10s.");
    Ok(())
}

async fn status(cfg: &AppConfig, job_id: &str) -> Result<()> {
    let http = reqwest::Client::new();
    let resp = http
        .get(format!("{}/api/v1/jobs/{}", cfg.scheduler_url, job_id))
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(anyhow!("GET /jobs/{} returned {}", job_id, resp.status()));
    }
    let job: serde_json::Value = resp.json().await?;
    println!("{}", serde_json::to_string_pretty(&job)?);
    Ok(())
}

// ───────────────────────── helpers ────────────────────────────────────

fn rpc_and_client(cfg: &AppConfig) -> Result<(RpcClient, Keypair)> {
    let rpc = RpcClient::new_with_commitment(
        cfg.solana_rpc_url.clone(),
        CommitmentConfig::confirmed(),
    );
    let path = std::env::var("CLIENT_KEYPAIR_PATH")
        .unwrap_or_else(|_| "./data/client-keypair.json".to_string());
    let expanded = shellexpand::tilde(&path).into_owned();
    let kp = read_keypair_file(&expanded)
        .map_err(|e| anyhow!("read client keypair {}: {}", expanded, e))?;
    Ok((rpc, kp))
}

fn program_ids(cfg: &AppConfig) -> Result<ProgramIds> {
    Ok(ProgramIds {
        provider_registry: Pubkey::from_str(&cfg.provider_registry_program_id)?,
        job_escrow: Pubkey::from_str(&cfg.job_escrow_program_id)?,
        compute_receipts: Pubkey::from_str(&cfg.compute_receipts_program_id)?,
    })
}

fn explorer_url(cfg: &AppConfig, sig: &str) -> String {
    let cluster = if cfg.solana_rpc_url.contains("devnet") {
        "?cluster=devnet"
    } else if cfg.solana_rpc_url.contains("testnet") {
        "?cluster=testnet"
    } else if cfg.solana_rpc_url.contains("localhost") || cfg.solana_rpc_url.contains("127.0.0.1") {
        "?cluster=custom&customUrl=http://localhost:8899"
    } else {
        ""
    };
    format!("https://explorer.solana.com/tx/{}{}", sig, cluster)
}
