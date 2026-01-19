use std::{path::Path, str::FromStr, time::Duration};

use anyhow::anyhow;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig,
    rpc_response::transaction::Transaction,
};
use solana_keypair::Address;
use solana_keypair::{read_keypair_file, Keypair, Signer};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 请改成你的程序地址，不改也可以，默认使用我部署的程序
    let program_id = Pubkey::from_str("9bkM5WfTd7YbZouo9R19xXYa2q2hCjTiqoMtSan5963i")
        .expect("Invalid program ID");

    // 默认使用开发环境
    let rpc_url = String::from("https://api.devnet.solana.com");
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    // 获取本地的钱包，请改成你本地的钱包地址
    let path = Path::new("/home/codespace/.config/solana/id.json");
    let payer = read_keypair_file(path).expect(
        "Unable to read Keypair file. Please check that the path is correct and the file exists.",
    );
    println!("Wallet address loaded successfully: {}", payer.pubkey());

    init_wallet(&client, &payer).await;

    write_data(client, program_id, &payer, "Hello Solana".as_bytes()).await?;

    Ok(())
}

// 初始化钱包余额
// 如果余额小于 0.5 SOL时，才去空投
async fn init_wallet(client: &RpcClient, payer: &Keypair) {
    let balance = client
        .get_balance(&payer.pubkey())
        .await
        .expect("Failed to get balance");
    println!("balance: {}", balance);
    if balance < 500_000_000 {
        // 如果余额小于 0.5 SOL时，才去空投
        println!("Requesting airdrop...");
        let airdrop_signature = client
            .request_airdrop(&payer.pubkey(), 1_000_000_000)
            .await
            .expect("Failed to request airdrop");
        loop {
            if client
                .confirm_transaction(&airdrop_signature)
                .await
                .unwrap_or(false)
            {
                break;
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        println!("Airdrop confirmed");
    } else {
        println!("Balance sufficient: {} lamports", balance);
    }
}

async fn write_data(
    client: RpcClient,
    program_id: Address,
    payer: &Keypair,
    writed_data: &[u8],
) -> anyhow::Result<()> {
    let (pda_pubkey, _bump_seed) =
        Pubkey::find_program_address(&[payer.pubkey().as_ref()], &program_id);
    println!("Derived PDA: {}", pda_pubkey);

    let instruction = Instruction::new_with_bytes(
        program_id,
        writed_data,
        vec![
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(pda_pubkey, false),
            AccountMeta::new(solana_system_program::id(), false),
        ],
    );

    let recent_blockhash = client.get_latest_blockhash().await?;
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(sig) => {
            println!("Success! Transaction: {}", sig);
            println!("Data saved to PDA: {}", pda_pubkey);
        }
        Err(err) => {
            return Err(anyhow!("Failed to write data: {}", err));
        }
    }

    Ok(())
}
