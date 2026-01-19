use std::{path::Path, str::FromStr, time::Duration};

use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_config::CommitmentConfig,
    rpc_response::transaction::Transaction,
};
use solana_keypair::{read_keypair_file, Keypair, Signer};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

#[tokio::main]
async fn main() {
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

    println!("\nInitializing counter...");
    let counter_keypair = Keypair::new();
    let instruction_data = borsh::to_vec("Hello World!").expect("Failed to serialize instruction");

    let initialize_instruction = Instruction::new_with_bytes(
        program_id,
        &instruction_data,
        vec![
            AccountMeta::new(counter_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(solana_system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[initialize_instruction], Some(&payer.pubkey()));

    let blockhash = client
        .get_latest_blockhash()
        .await
        .expect("Failed to get blockhash");
    transaction.sign(&[&payer, &counter_keypair], blockhash);

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => {
            println!("Counter initialized!");
            println!("Transaction: {}", signature);
            println!("Counter address: {}", counter_keypair.pubkey());
        }
        Err(err) => {
            eprintln!("Failed to initial to initialize counter: {}", err);
            return;
        }
    }

    println!("\nIncrementing counter...");

    let increment_data = borsh::to_vec("Hello Solana").expect("Failed to serialize instruction");

    let increment_instruction = Instruction::new_with_bytes(
        program_id,
        &increment_data,
        vec![
            AccountMeta::new(counter_keypair.pubkey(), true),
            AccountMeta::new(payer.pubkey(), true),
            AccountMeta::new(solana_system_program::id(), false),
        ],
    );

    let mut transaction =
        Transaction::new_with_payer(&[increment_instruction], Some(&payer.pubkey()));

    // 重新获取最新的 Blockhash
    // 容错性：如果网络拥堵导致 Initialize 交易在链上排队了 30 秒，等你准备发第二笔交易时，最初那个 blockhash 的“生命值”已经过半，失败风险极高。
    // 避免重复交易检测：Solana 节点会通过 Blockhash 检查交易是否重复。如果你短时间内发送两笔 Blockhash 完全一样的交易，节点可能会误以为是重复提交而拒绝。
    let (recent_blockhash, _) = client
        .get_latest_blockhash_with_commitment(CommitmentConfig::confirmed())
        .await
        .expect("Failed to get a fresh blockhash for increment");
    transaction.sign(&[&payer, &counter_keypair], recent_blockhash);

    match client.send_and_confirm_transaction(&transaction).await {
        Ok(signature) => {
            println!("Counter incremented!");
            println!("Transaction: {}", signature);
        }
        Err(err) => {
            eprintln!("Failed to increment counter: {}", err);
            return;
        }
    }
}
